use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use indexmap::IndexMap;

use crate::cache::{ICacheManager, StandardCacheManager};
use crate::context::{IContext, IEngineContextFactory, StandardEngineContextFactory};
use crate::decoupled::{IDecoupledTemplateLogicResolver, StandardDecoupledTemplateLogicResolver};
use crate::dialect::IDialect;
use crate::exceptions::{
    AlreadyInitializedException, ConfigurationException, TemplateEngineException,
    TemplateOutputException,
};
use crate::linkbuilder::{ILinkBuilder, StandardLinkBuilder};
use crate::messageresolver::{IMessageResolver, StandardMessageResolver};
use crate::standard::StandardDialect;
use crate::templateresolver::{ITemplateResolver, StringTemplateResolver};
use crate::util::{TemplateWriter, Utf16String};
use crate::{
    ConfigurationPrinterHelper, DialectConfiguration, EngineConfiguration, IEngineConfiguration,
    ITemplateEngine, IThrottledTemplateProcessor, TemplateEngineResult, TemplateSpec,
};

const ALREADY_INITIALIZED_MESSAGE: &str = "Template engine has already been initialized (probably \
because it has already been executed or a fully-built Configuration object has been requested \
from it. At this state, no modifications on its configuration are allowed.";

type InitializationHook =
    dyn Fn(&TemplateEngine) -> TemplateEngineResult<()> + Send + Sync + 'static;

struct TemplateEngineState {
    dialect_configurations: Vec<DialectConfiguration>,
    template_resolvers: Vec<Arc<dyn ITemplateResolver>>,
    message_resolvers: Vec<Arc<dyn IMessageResolver>>,
    link_builders: Vec<Arc<dyn ILinkBuilder>>,
    cache_manager: Option<Arc<dyn ICacheManager>>,
    engine_context_factory: Arc<dyn IEngineContextFactory>,
    decoupled_template_logic_resolver: Arc<dyn IDecoupledTemplateLogicResolver>,
}

/// Thymeleaf 默认模板引擎实现。
///
/// 对应 Java: `org.thymeleaf.TemplateEngine`。
pub struct TemplateEngine {
    initialized: AtomicBool,
    initialization_lock: Mutex<()>,
    initialization_hook: Mutex<Option<Arc<InitializationHook>>>,
    state: Mutex<TemplateEngineState>,
    configuration: OnceLock<Arc<EngineConfiguration>>,
}

impl TemplateEngine {
    /// 模板计时 logger 的 Java 名称。
    pub const TIMER_LOGGER_NAME: &'static str = "org.thymeleaf.TemplateEngine.TIMER";

    /// 创建带标准缓存、上下文工厂、消息解析器、链接构建器和 Standard Dialect 的引擎。
    #[must_use]
    /// 对应 Java 语义：`TemplateEngine` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new() -> Self {
        let standard_dialect: Arc<dyn IDialect> = Arc::new(StandardDialect::new());
        Self {
            initialized: AtomicBool::new(false),
            initialization_lock: Mutex::new(()),
            initialization_hook: Mutex::new(None),
            state: Mutex::new(TemplateEngineState {
                dialect_configurations: vec![
                    DialectConfiguration::new(Some(standard_dialect))
                        .expect("standard dialect is non-null"),
                ],
                template_resolvers: Vec::new(),
                message_resolvers: vec![Arc::new(StandardMessageResolver::new())],
                link_builders: vec![Arc::new(StandardLinkBuilder::new())],
                cache_manager: Some(Arc::new(StandardCacheManager::new())),
                engine_context_factory: Arc::new(StandardEngineContextFactory::new()),
                decoupled_template_logic_resolver: Arc::new(
                    StandardDecoupledTemplateLogicResolver::new(),
                ),
            }),
            configuration: OnceLock::new(),
        }
    }

    /// 判断首次配置读取或模板处理是否已完成初始化。
    #[must_use]
    /// 对应 Java: `TemplateEngine#isInitialized()`。
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }

    /// 设置首次初始化前执行的宿主扩展回调。
    ///
    /// 对应 Java 子类覆盖的 `TemplateEngine#initializeSpecific()`。Rust 不使用继承，
    /// 因而把同一扩展点表示为一次性配置回调；回调在默认解析器补齐和
    /// `EngineConfiguration` 构建之前执行，可以安全调用本对象的配置修改方法。
    ///
    /// # 参数
    /// - `initialization_hook`：初始化扩展逻辑。
    ///
    /// # 错误
    /// 引擎已经初始化时返回 `AlreadyInitializedException`。
    pub fn set_initialization_hook(
        &self,
        initialization_hook: Arc<InitializationHook>,
    ) -> Result<(), AlreadyInitializedException> {
        self.check_not_initialized()?;
        let mut hook = lock(&self.initialization_hook);
        self.check_not_initialized()?;
        *hook = Some(initialization_hook);
        Ok(())
    }

    /// 返回按配置顺序去重的方言快照。
    /// 对应 Java: `TemplateEngine#getDialects()`。
    pub fn get_dialects(&self) -> Vec<Arc<dyn IDialect>> {
        let state = lock(&self.state);
        let mut dialects = Vec::new();
        for configuration in &state.dialect_configurations {
            let dialect = configuration.get_dialect_arc();
            if !dialects
                .iter()
                .any(|current| Arc::ptr_eq(current, &dialect))
            {
                dialects.push(dialect);
            }
        }
        dialects
    }

    /// 返回按显式或默认前缀分组的方言快照。
    /// 对应 Java: `TemplateEngine#getDialectsByPrefix()`。
    pub fn get_dialects_by_prefix(&self) -> IndexMap<Option<Utf16String>, Vec<Arc<dyn IDialect>>> {
        let state = lock(&self.state);
        let mut result = IndexMap::<Option<Utf16String>, Vec<Arc<dyn IDialect>>>::new();
        for configuration in &state.dialect_configurations {
            let key = configuration.get_prefix().map(Utf16String::from_rust_str);
            let dialect = configuration.get_dialect_arc();
            let values = result.entry(key).or_default();
            if !values.iter().any(|current| Arc::ptr_eq(current, &dialect)) {
                values.push(dialect);
            }
        }
        result
    }

    /// 设置唯一方言并清除先前方言配置。
    /// 对应 Java: `TemplateEngine#setDialect()`。
    pub fn set_dialect(
        &self,
        dialect: Arc<dyn IDialect>,
    ) -> Result<(), AlreadyInitializedException> {
        self.check_not_initialized()?;
        let mut state = lock(&self.state);
        self.check_not_initialized()?;
        state.dialect_configurations.clear();
        state.dialect_configurations.push(
            DialectConfiguration::new(Some(dialect)).expect("Rust Arc excludes a null dialect"),
        );
        Ok(())
    }

    /// 使用显式前缀增加一个方言。
    /// 对应 Java 语义：`TemplateEngine` 的 `add_dialect_with_prefix` 行为（Rust 侧辅助/私有路径）。
    pub fn add_dialect_with_prefix(
        &self,
        prefix: Option<&str>,
        dialect: Arc<dyn IDialect>,
    ) -> Result<(), AlreadyInitializedException> {
        self.check_not_initialized()?;
        let mut state = lock(&self.state);
        self.check_not_initialized()?;
        if !contains_dialect_configuration(&state.dialect_configurations, true, prefix, &dialect) {
            state.dialect_configurations.push(
                DialectConfiguration::with_prefix(prefix, Some(dialect))
                    .expect("Rust Arc excludes a null dialect"),
            );
        }
        Ok(())
    }

    /// 使用方言默认前缀增加一个方言。
    /// 对应 Java: `TemplateEngine#addDialect()`。
    pub fn add_dialect(
        &self,
        dialect: Arc<dyn IDialect>,
    ) -> Result<(), AlreadyInitializedException> {
        self.check_not_initialized()?;
        let mut state = lock(&self.state);
        self.check_not_initialized()?;
        if !contains_dialect_configuration(&state.dialect_configurations, false, None, &dialect) {
            state.dialect_configurations.push(
                DialectConfiguration::new(Some(dialect)).expect("Rust Arc excludes a null dialect"),
            );
        }
        Ok(())
    }

    /// 用全部采用默认前缀的方言替换现有方言集合。
    /// 对应 Java: `TemplateEngine#setDialects()`。
    pub fn set_dialects(
        &self,
        dialects: Vec<Arc<dyn IDialect>>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            state.dialect_configurations.clear();
            for dialect in dialects {
                if !contains_dialect_configuration(
                    &state.dialect_configurations,
                    false,
                    None,
                    &dialect,
                ) {
                    state.dialect_configurations.push(
                        DialectConfiguration::new(Some(dialect))
                            .expect("Rust Arc excludes a null dialect"),
                    );
                }
            }
        })
    }

    /// 用调用方给定的前缀/方言映射替换现有方言集合。
    /// 对应 Java: `TemplateEngine#setDialectsByPrefix()`。
    pub fn set_dialects_by_prefix(
        &self,
        dialects: Vec<(Option<Utf16String>, Arc<dyn IDialect>)>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            state.dialect_configurations.clear();
            for (prefix, dialect) in dialects {
                let prefix_text = prefix.as_ref().map(Utf16String::to_string_lossy);
                if !contains_dialect_configuration(
                    &state.dialect_configurations,
                    true,
                    prefix_text.as_deref(),
                    &dialect,
                ) {
                    state.dialect_configurations.push(
                        DialectConfiguration::with_prefix(prefix_text.as_deref(), Some(dialect))
                            .expect("Rust Arc excludes a null dialect"),
                    );
                }
            }
        })
    }

    /// 追加一组采用默认前缀的方言。
    /// 对应 Java: `TemplateEngine#setAdditionalDialects()`。
    pub fn set_additional_dialects(
        &self,
        dialects: Vec<Arc<dyn IDialect>>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            for dialect in dialects {
                if !contains_dialect_configuration(
                    &state.dialect_configurations,
                    false,
                    None,
                    &dialect,
                ) {
                    state.dialect_configurations.push(
                        DialectConfiguration::new(Some(dialect))
                            .expect("Rust Arc excludes a null dialect"),
                    );
                }
            }
        })
    }

    /// 删除全部方言配置。
    /// 对应 Java: `TemplateEngine#clearDialects()`。
    pub fn clear_dialects(&self) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| state.dialect_configurations.clear())
    }

    /// 设置唯一模板解析器。
    /// 对应 Java: `TemplateEngine#setTemplateResolver()`。
    pub fn set_template_resolver(
        &self,
        template_resolver: Arc<dyn ITemplateResolver>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            state.template_resolvers.clear();
            state.template_resolvers.push(template_resolver);
        })
    }

    /// 返回模板解析器配置快照。
    /// 对应 Java: `TemplateEngine#getTemplateResolvers()`。
    pub fn get_template_resolvers(&self) -> Vec<Arc<dyn ITemplateResolver>> {
        self.configuration.get().map_or_else(
            || lock(&self.state).template_resolvers.clone(),
            |configuration| configuration.template_resolver_arcs(),
        )
    }

    /// 替换全部模板解析器，并按对象身份去重。
    /// 对应 Java: `TemplateEngine#setTemplateResolvers()`。
    pub fn set_template_resolvers(
        &self,
        template_resolvers: Vec<Arc<dyn ITemplateResolver>>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            state.template_resolvers.clear();
            for resolver in template_resolvers {
                if !state
                    .template_resolvers
                    .iter()
                    .any(|current| Arc::ptr_eq(current, &resolver))
                {
                    state.template_resolvers.push(resolver);
                }
            }
        })
    }

    /// 按插入顺序增加模板解析器，并按对象身份去重。
    /// 对应 Java: `TemplateEngine#addTemplateResolver()`。
    pub fn add_template_resolver(
        &self,
        template_resolver: Arc<dyn ITemplateResolver>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            if !state
                .template_resolvers
                .iter()
                .any(|current| Arc::ptr_eq(current, &template_resolver))
            {
                state.template_resolvers.push(template_resolver);
            }
        })
    }

    /// 设置可空缓存管理器；`None` 禁用全部引擎缓存。
    /// 对应 Java: `TemplateEngine#setCacheManager()`。
    pub fn set_cache_manager(
        &self,
        cache_manager: Option<Arc<dyn ICacheManager>>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| state.cache_manager = cache_manager)
    }

    /// 返回当前缓存管理器快照。
    /// 对应 Java: `TemplateEngine#getCacheManager()`。
    pub fn get_cache_manager(&self) -> Option<Arc<dyn ICacheManager>> {
        lock(&self.state).cache_manager.clone()
    }

    /// 设置引擎上下文工厂。
    /// 对应 Java: `TemplateEngine#setEngineContextFactory()`。
    pub fn set_engine_context_factory(
        &self,
        engine_context_factory: Arc<dyn IEngineContextFactory>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            state.engine_context_factory = engine_context_factory;
        })
    }

    /// 返回当前引擎上下文工厂。
    /// 对应 Java: `TemplateEngine#getEngineContextFactory()`。
    pub fn get_engine_context_factory(&self) -> Arc<dyn IEngineContextFactory> {
        Arc::clone(&lock(&self.state).engine_context_factory)
    }

    /// 设置解耦模板逻辑解析器。
    /// 对应 Java: `TemplateEngine#setDecoupledTemplateLogicResolver()`。
    pub fn set_decoupled_template_logic_resolver(
        &self,
        resolver: Arc<dyn IDecoupledTemplateLogicResolver>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            state.decoupled_template_logic_resolver = resolver;
        })
    }

    /// 返回当前解耦模板逻辑解析器。
    /// 对应 Java: `TemplateEngine#getDecoupledTemplateLogicResolver()`。
    pub fn get_decoupled_template_logic_resolver(
        &self,
    ) -> Arc<dyn IDecoupledTemplateLogicResolver> {
        Arc::clone(&lock(&self.state).decoupled_template_logic_resolver)
    }

    /// 设置唯一消息解析器。
    /// 对应 Java: `TemplateEngine#setMessageResolver()`。
    pub fn set_message_resolver(
        &self,
        message_resolver: Arc<dyn IMessageResolver>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            state.message_resolvers.clear();
            state.message_resolvers.push(message_resolver);
        })
    }

    /// 返回消息解析器配置快照。
    /// 对应 Java: `TemplateEngine#getMessageResolvers()`。
    pub fn get_message_resolvers(&self) -> Vec<Arc<dyn IMessageResolver>> {
        self.configuration.get().map_or_else(
            || lock(&self.state).message_resolvers.clone(),
            |configuration| configuration.message_resolver_arcs(),
        )
    }

    /// 替换全部消息解析器，并按对象身份去重。
    /// 对应 Java: `TemplateEngine#setMessageResolvers()`。
    pub fn set_message_resolvers(
        &self,
        message_resolvers: Vec<Arc<dyn IMessageResolver>>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            state.message_resolvers.clear();
            for resolver in message_resolvers {
                if !state
                    .message_resolvers
                    .iter()
                    .any(|current| Arc::ptr_eq(current, &resolver))
                {
                    state.message_resolvers.push(resolver);
                }
            }
        })
    }

    /// 增加消息解析器，并按对象身份去重。
    /// 对应 Java: `TemplateEngine#addMessageResolver()`。
    pub fn add_message_resolver(
        &self,
        message_resolver: Arc<dyn IMessageResolver>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            if !state
                .message_resolvers
                .iter()
                .any(|current| Arc::ptr_eq(current, &message_resolver))
            {
                state.message_resolvers.push(message_resolver);
            }
        })
    }

    /// 设置唯一链接构建器。
    /// 对应 Java: `TemplateEngine#setLinkBuilder()`。
    pub fn set_link_builder(
        &self,
        link_builder: Arc<dyn ILinkBuilder>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            state.link_builders.clear();
            state.link_builders.push(link_builder);
        })
    }

    /// 返回链接构建器配置快照。
    /// 对应 Java: `TemplateEngine#getLinkBuilders()`。
    pub fn get_link_builders(&self) -> Vec<Arc<dyn ILinkBuilder>> {
        self.configuration.get().map_or_else(
            || lock(&self.state).link_builders.clone(),
            |configuration| configuration.link_builder_arcs(),
        )
    }

    /// 替换全部链接构建器，并按对象身份去重。
    /// 对应 Java: `TemplateEngine#setLinkBuilders()`。
    pub fn set_link_builders(
        &self,
        link_builders: Vec<Arc<dyn ILinkBuilder>>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            state.link_builders.clear();
            for builder in link_builders {
                if !state
                    .link_builders
                    .iter()
                    .any(|current| Arc::ptr_eq(current, &builder))
                {
                    state.link_builders.push(builder);
                }
            }
        })
    }

    /// 增加链接构建器，并按对象身份去重。
    /// 对应 Java: `TemplateEngine#addLinkBuilder()`。
    pub fn add_link_builder(
        &self,
        link_builder: Arc<dyn ILinkBuilder>,
    ) -> Result<(), AlreadyInitializedException> {
        self.mutate_before_initialization(|state| {
            if !state
                .link_builders
                .iter()
                .any(|current| Arc::ptr_eq(current, &link_builder))
            {
                state.link_builders.push(link_builder);
            }
        })
    }

    /// 清空全部模板缓存；调用会触发引擎初始化。
    /// 对应 Java: `TemplateEngine#clearTemplateCache()`。
    pub fn clear_template_cache(&self) -> TemplateEngineResult<()> {
        self.initialize()?.get_template_manager().clear_caches();
        Ok(())
    }

    /// 清空指定模板名称的缓存；调用会触发引擎初始化。
    /// 对应 Java: `TemplateEngine#clearTemplateCacheFor()`。
    pub fn clear_template_cache_for(
        &self,
        template_name: &Utf16String,
    ) -> TemplateEngineResult<()> {
        self.initialize()?
            .get_template_manager()
            .clear_caches_for(template_name);
        Ok(())
    }

    /// 使用模板文本或名称创建 TemplateSpec 并返回完整输出。
    /// 对应 Java 语义：`TemplateEngine` 的 `process_template` 行为（Rust 侧辅助/私有路径）。
    pub fn process_template(
        &self,
        template: &str,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Utf16String> {
        let template_spec = template_spec(template, None)?;
        self.process(&template_spec, context)
    }

    /// 使用模板名称及片段选择器返回完整输出。
    ///
    /// 对应 Java: `TemplateEngine#process(String, Set<String>, IContext)`。
    pub fn process_template_with_selectors(
        &self,
        template: &str,
        template_selectors: &crate::TemplateSelectorSet,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Utf16String> {
        let template_spec = template_spec(template, Some(template_selectors))?;
        self.process(&template_spec, context)
    }

    /// 使用模板名称把完整输出写入调用方 Writer。
    ///
    /// 对应 Java: `TemplateEngine#process(String, IContext, Writer)`。
    pub fn process_template_to_writer(
        &self,
        template: &str,
        context: &dyn IContext,
        writer: Box<dyn TemplateWriter>,
    ) -> TemplateEngineResult<()> {
        let template_spec = template_spec(template, None)?;
        self.process_to_writer(&template_spec, context, writer)
    }

    /// 使用模板名称及片段选择器把完整输出写入调用方 Writer。
    ///
    /// 对应 Java: `TemplateEngine#process(String, Set<String>, IContext, Writer)`。
    pub fn process_template_with_selectors_to_writer(
        &self,
        template: &str,
        template_selectors: &crate::TemplateSelectorSet,
        context: &dyn IContext,
        writer: Box<dyn TemplateWriter>,
    ) -> TemplateEngineResult<()> {
        let template_spec = template_spec(template, Some(template_selectors))?;
        self.process_to_writer(&template_spec, context, writer)
    }

    /// 使用模板名称创建节流处理器。
    ///
    /// 对应 Java: `TemplateEngine#processThrottled(String, IContext)`。
    pub fn process_throttled_template(
        &self,
        template: &str,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Box<dyn IThrottledTemplateProcessor>> {
        let template_spec = template_spec(template, None)?;
        self.process_throttled(&template_spec, context)
    }

    /// 使用模板名称及片段选择器创建节流处理器。
    ///
    /// 对应 Java:
    /// `TemplateEngine#processThrottled(String, Set<String>, IContext)`。
    pub fn process_throttled_template_with_selectors(
        &self,
        template: &str,
        template_selectors: &crate::TemplateSelectorSet,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Box<dyn IThrottledTemplateProcessor>> {
        let template_spec = template_spec(template, Some(template_selectors))?;
        self.process_throttled(&template_spec, context)
    }

    /// 返回当前执行线程名称。
    #[must_use]
    /// 对应 Java: `TemplateEngine#threadIndex()`。
    pub fn thread_index() -> String {
        std::thread::current()
            .name()
            .unwrap_or("unnamed")
            .to_owned()
    }

    fn initialize(&self) -> TemplateEngineResult<Arc<EngineConfiguration>> {
        if let Some(configuration) = self.configuration.get() {
            return Ok(Arc::clone(configuration));
        }
        let _initialization = lock(&self.initialization_lock);
        if let Some(configuration) = self.configuration.get() {
            return Ok(Arc::clone(configuration));
        }

        self.initialize_specific()?;
        let mut state = lock(&self.state);
        if state.template_resolvers.is_empty() {
            state
                .template_resolvers
                .push(Arc::new(StringTemplateResolver::new()));
        }
        let template_resolvers = state.template_resolvers.clone();
        let dialect_configurations = state
            .dialect_configurations
            .iter()
            .map(clone_dialect_configuration)
            .collect::<Result<Vec<_>, ConfigurationException>>()
            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException + Send + Sync>)?;
        let configuration = EngineConfiguration::new(
            template_resolvers,
            state.message_resolvers.clone(),
            state.link_builders.clone(),
            dialect_configurations,
            state.cache_manager.clone(),
            Arc::clone(&state.engine_context_factory),
            Arc::clone(&state.decoupled_template_logic_resolver),
        )
        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException + Send + Sync>)?;
        drop(state);

        ConfigurationPrinterHelper::print_configuration(configuration.as_ref());
        self.configuration
            .set(Arc::clone(&configuration))
            .map_err(|_| {
                Box::new(ConfigurationException::new(Some(
                    "Template engine configuration was initialized concurrently".to_owned(),
                ))) as Box<dyn TemplateEngineException + Send + Sync>
            })?;
        self.initialized.store(true, Ordering::Release);
        Ok(configuration)
    }

    /// 执行 Rust 回调形式的专用初始化扩展点。
    ///
    /// 对应 Java: `TemplateEngine#initializeSpecific()`。
    fn initialize_specific(&self) -> TemplateEngineResult<()> {
        let hook = lock(&self.initialization_hook).clone();
        hook.map_or(Ok(()), |hook| hook(self))
    }

    fn check_not_initialized(&self) -> Result<(), AlreadyInitializedException> {
        if self.is_initialized() || self.configuration.get().is_some() {
            return Err(AlreadyInitializedException::new(Some(
                ALREADY_INITIALIZED_MESSAGE.to_owned(),
            )));
        }
        Ok(())
    }

    fn mutate_before_initialization(
        &self,
        mutation: impl FnOnce(&mut TemplateEngineState),
    ) -> Result<(), AlreadyInitializedException> {
        self.check_not_initialized()?;
        let mut state = lock(&self.state);
        self.check_not_initialized()?;
        mutation(&mut state);
        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ITemplateEngine for TemplateEngine {
    fn get_configuration(&self) -> TemplateEngineResult<Arc<dyn IEngineConfiguration>> {
        self.initialize()
            .map(|configuration| configuration as Arc<dyn IEngineConfiguration>)
    }

    fn process(
        &self,
        template_spec: &TemplateSpec,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Utf16String> {
        let buffer = Arc::new(Mutex::new(Vec::with_capacity(100)));
        let writer = SharedStringWriter {
            buffer: Arc::clone(&buffer),
        };
        self.process_to_writer(template_spec, context, Box::new(writer))?;
        let units = Arc::try_unwrap(buffer)
            .expect("the processing writer has been dropped")
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(Utf16String::from_utf16(units))
    }

    fn process_to_writer(
        &self,
        template_spec: &TemplateSpec,
        context: &dyn IContext,
        writer: Box<dyn TemplateWriter>,
    ) -> TemplateEngineResult<()> {
        let configuration = self.initialize()?;
        let mut shared_writer = SharedTemplateWriter::new(writer);
        configuration
            .get_template_manager()
            .parse_and_process(template_spec, context, Box::new(shared_writer.clone()))
            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException + Send + Sync>)?;
        shared_writer.flush().map_err(|error| {
            Box::new(TemplateOutputException::new(
                Some("An error happened while flushing output writer".to_owned()),
                Some(template_spec.get_template().to_owned()),
                -1,
                -1,
                error,
            )) as Box<dyn TemplateEngineException + Send + Sync>
        })
    }

    fn process_throttled(
        &self,
        template_spec: &TemplateSpec,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Box<dyn IThrottledTemplateProcessor>> {
        self.initialize()?
            .get_template_manager()
            .parse_and_process_throttled(template_spec, context)
            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException + Send + Sync>)
    }
}

fn contains_dialect_configuration(
    configurations: &[DialectConfiguration],
    prefix_specified: bool,
    prefix: Option<&str>,
    dialect: &Arc<dyn IDialect>,
) -> bool {
    configurations.iter().any(|configuration| {
        configuration.is_prefix_specified() == prefix_specified
            && configuration.get_prefix() == prefix
            && Arc::ptr_eq(&configuration.get_dialect_arc(), dialect)
    })
}

fn clone_dialect_configuration(
    configuration: &DialectConfiguration,
) -> Result<DialectConfiguration, ConfigurationException> {
    let cloned = if configuration.is_prefix_specified() {
        DialectConfiguration::with_prefix(
            configuration.get_prefix(),
            Some(configuration.get_dialect_arc()),
        )
    } else {
        DialectConfiguration::new(Some(configuration.get_dialect_arc()))
    };
    cloned.map_err(|error| ConfigurationException::with_cause(Some(error.to_string()), error))
}

fn template_spec(
    template: &str,
    template_selectors: Option<&crate::TemplateSelectorSet>,
) -> TemplateEngineResult<TemplateSpec> {
    TemplateSpec::with_selectors_and_template_mode(Some(template), template_selectors, None, None)
        .map_err(|error| {
            Box::new(crate::TemplateProcessingException::with_cause(
                Some("Invalid template specification".to_owned()),
                error,
            )) as Box<dyn TemplateEngineException + Send + Sync>
        })
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct SharedStringWriter {
    buffer: Arc<Mutex<Vec<u16>>>,
}

impl TemplateWriter for SharedStringWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> std::io::Result<()> {
        lock(&self.buffer).extend_from_slice(characters);
        Ok(())
    }
}

#[derive(Clone)]
struct SharedTemplateWriter {
    writer: Arc<Mutex<Box<dyn TemplateWriter>>>,
}

impl SharedTemplateWriter {
    fn new(writer: Box<dyn TemplateWriter>) -> Self {
        Self {
            writer: Arc::new(Mutex::new(writer)),
        }
    }
}

impl TemplateWriter for SharedTemplateWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> std::io::Result<()> {
        lock(&self.writer).write_utf16(characters)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        lock(&self.writer).flush()
    }

    fn close(&mut self) -> std::io::Result<()> {
        lock(&self.writer).close()
    }
}
