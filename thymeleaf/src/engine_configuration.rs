use std::any::TypeId;
use std::sync::{Arc, OnceLock, Weak};

use indexmap::IndexMap;

use crate::cache::ICacheManager;
use crate::cdatasection::ICDATASectionProcessor;
use crate::comment::ICommentProcessor;
use crate::context::IEngineContextFactory;
use crate::decoupled::IDecoupledTemplateLogicResolver;
use crate::dialect::{
    IDialect, IExecutionAttributeDialect, IExpressionObjectDialect, IPostProcessorDialect,
    IPreProcessorDialect, IProcessorDialect,
};
use crate::doctype::IDocTypeProcessor;
use crate::element::IElementProcessor;
use crate::engine::{
    AttributeDefinitions, ElementDefinitions, ITemplateManager, StandardModelFactory,
    TemplateManager,
};
use crate::expression::IExpressionObjectFactory;
use crate::linkbuilder::ILinkBuilder;
use crate::messageresolver::IMessageResolver;
use crate::model::IModelFactory;
use crate::postprocessor::IPostProcessor;
use crate::preprocessor::IPreProcessor;
use crate::processinginstruction::IProcessingInstructionProcessor;
use crate::templateboundaries::ITemplateBoundariesProcessor;
use crate::templateresolver::ITemplateResolver;
use crate::text::ITextProcessor;
use crate::util::JavaString;
use crate::xmldeclaration::IXMLDeclarationProcessor;
use crate::{
    DialectConfiguration, DialectSetConfiguration, ExecutionAttributeValue, IEngineConfiguration,
    TemplateMode,
};

/// 默认不可变引擎配置，聚合 Resolver、Dialect、Cache 和运行时工厂。
///
/// 对应 Java: `org.thymeleaf.EngineConfiguration`。
///
/// 用户通常应通过 [`crate::TemplateEngine`] 配置并取得本对象的
/// [`IEngineConfiguration`] 接口，而不是直接构造。发布后的 Resolver、方言及运行时
/// 工厂快照不可修改，并可在线程间安全共享。
///
/// # 起始版本
///
/// 上游自 Thymeleaf 3.0.0 起提供该实现。
pub struct EngineConfiguration {
    template_resolvers: Vec<Arc<dyn ITemplateResolver>>,
    message_resolvers: Vec<Arc<dyn IMessageResolver>>,
    link_builders: Vec<Arc<dyn ILinkBuilder>>,
    cache_manager: Option<Arc<dyn ICacheManager>>,
    engine_context_factory: Arc<dyn IEngineContextFactory>,
    decoupled_template_logic_resolver: Arc<dyn IDecoupledTemplateLogicResolver>,
    dialect_set_configuration: DialectSetConfiguration,
    self_weak: OnceLock<Weak<EngineConfiguration>>,
    template_manager: OnceLock<TemplateManager>,
    html_model_factory: OnceLock<StandardModelFactory>,
    xml_model_factory: OnceLock<StandardModelFactory>,
    text_model_factory: OnceLock<StandardModelFactory>,
    javascript_model_factory: OnceLock<StandardModelFactory>,
    css_model_factory: OnceLock<StandardModelFactory>,
    raw_model_factory: OnceLock<StandardModelFactory>,
}

impl EngineConfiguration {
    /// 构建配置、稳定排序扩展链，并在自引用建立后初始化 TemplateManager。
    ///
    /// 对应 Java: package 构造器及 `EngineConfiguration#initialize()`。
    ///
    /// # 参数
    ///
    /// - `template_resolvers`：至少包含一个模板 Resolver；按可空 order 稳定排序；
    /// - `message_resolvers`：消息 Resolver 快照；按可空 order 稳定排序；
    /// - `link_builders`：链接构建器快照；按可空 order 稳定排序；
    /// - `dialect_configurations`：完整方言配置快照；
    /// - `cache_manager`：可选缓存管理器，`None` 表示禁用全部缓存；
    /// - `engine_context_factory`：创建引擎上下文的线程安全工厂；
    /// - `decoupled_template_logic_resolver`：解析解耦模板逻辑的线程安全组件。
    ///
    /// # 返回值
    ///
    /// 返回已经完成二阶段初始化、可立即发布的共享配置。
    ///
    /// # 错误
    ///
    /// 模板 Resolver 为空、方言聚合失败或 TemplateManager 重复初始化时返回
    /// [`crate::exceptions::ConfigurationException`]。
    pub fn new(
        mut template_resolvers: Vec<Arc<dyn ITemplateResolver>>,
        mut message_resolvers: Vec<Arc<dyn IMessageResolver>>,
        mut link_builders: Vec<Arc<dyn ILinkBuilder>>,
        dialect_configurations: Vec<DialectConfiguration>,
        cache_manager: Option<Arc<dyn ICacheManager>>,
        engine_context_factory: Arc<dyn IEngineContextFactory>,
        decoupled_template_logic_resolver: Arc<dyn IDecoupledTemplateLogicResolver>,
    ) -> Result<Arc<Self>, crate::exceptions::ConfigurationException> {
        if template_resolvers.is_empty() {
            return Err(crate::exceptions::ConfigurationException::new(Some(
                "Template Resolver set cannot be empty".to_owned(),
            )));
        }
        template_resolvers.sort_by(TemplateResolverComparator::compare);
        message_resolvers.sort_by(MessageResolverComparator::compare);
        link_builders.sort_by(LinkBuilderComparator::compare);
        let dialect_set_configuration =
            DialectSetConfiguration::build(Some(dialect_configurations))
                .map_err(crate::DialectSetConfigurationError::into_configuration_exception)?;

        let configuration = Arc::new(Self {
            template_resolvers,
            message_resolvers,
            link_builders,
            cache_manager,
            engine_context_factory,
            decoupled_template_logic_resolver,
            dialect_set_configuration,
            self_weak: OnceLock::new(),
            template_manager: OnceLock::new(),
            html_model_factory: OnceLock::new(),
            xml_model_factory: OnceLock::new(),
            text_model_factory: OnceLock::new(),
            javascript_model_factory: OnceLock::new(),
            css_model_factory: OnceLock::new(),
            raw_model_factory: OnceLock::new(),
        });
        // Java 构造器不能在 TemplateManager 中安全发布尚未完成的 this，因此把该
        // 自依赖推迟到 initialize；Rust 先建立 Weak 自引用，再完成同一二阶段流程。
        configuration
            .self_weak
            .set(Arc::downgrade(&configuration))
            .expect("self weak reference is initialized exactly once");
        let shared: Arc<dyn IEngineConfiguration> = configuration.clone();
        configuration
            .template_manager
            .set(TemplateManager::new(shared))
            .map_err(|_| {
                crate::exceptions::ConfigurationException::new(Some(
                    "Template Manager has already been initialized".to_owned(),
                ))
            })?;
        Ok(configuration)
    }

    /// 升级构造阶段登记的弱自引用，供延迟模型工厂共享同一配置身份。
    fn shared_configuration(&self) -> Arc<dyn IEngineConfiguration> {
        self.self_weak
            .get()
            .and_then(Weak::upgrade)
            .expect("published EngineConfiguration retains a strong owner")
    }

    /// 返回指定模板模式唯一的并发初始化槽位。
    fn model_factory_slot(&self, mode: TemplateMode) -> &OnceLock<StandardModelFactory> {
        match mode {
            TemplateMode::HTML => &self.html_model_factory,
            TemplateMode::XML => &self.xml_model_factory,
            TemplateMode::TEXT => &self.text_model_factory,
            TemplateMode::JAVASCRIPT => &self.javascript_model_factory,
            TemplateMode::CSS => &self.css_model_factory,
            TemplateMode::RAW => &self.raw_model_factory,
        }
    }

    /// 返回初始化时已经按 Java 比较器稳定排序的模板解析器共享快照。
    ///
    /// 仅供 `TemplateEngine` 在冻结后实现 Java `getTemplateResolvers()` 的可观察顺序。
    /// 对应 Java: `EngineConfiguration#getTemplateResolvers()`。
    pub(crate) fn template_resolver_arcs(&self) -> Vec<Arc<dyn ITemplateResolver>> {
        self.template_resolvers.clone()
    }

    /// 返回初始化时已经按 Java 比较器稳定排序的消息解析器共享快照。
    ///
    /// 对应 Java: `EngineConfiguration#getMessageResolvers()`。
    pub(crate) fn message_resolver_arcs(&self) -> Vec<Arc<dyn IMessageResolver>> {
        self.message_resolvers.clone()
    }

    /// 返回初始化时已经按 Java 比较器稳定排序的链接构建器共享快照。
    ///
    /// 对应 Java: `EngineConfiguration#getLinkBuilders()`。
    pub(crate) fn link_builder_arcs(&self) -> Vec<Arc<dyn ILinkBuilder>> {
        self.link_builders.clone()
    }

    /// 判断指定模式的已解析模型能否安全执行结构重塑优化。
    ///
    /// 对应 Java: `EngineConfiguration#isModelReshapeable(TemplateMode)`。
    #[must_use]
    pub fn is_model_reshapeable(&self, template_mode: TemplateMode) -> bool {
        if !self.dialect_set_configuration.is_standard_dialect_present()
            || self
                .dialect_set_configuration
                .get_text_processors(Some(template_mode))
                .expect("template mode is non-null")
                .len()
                > 1
        {
            return false;
        }
        if template_mode.is_markup() {
            let allowed_comment_processors = if template_mode == TemplateMode::HTML {
                2
            } else {
                1
            };
            if self
                .dialect_set_configuration
                .get_comment_processors(Some(template_mode))
                .expect("template mode is non-null")
                .len()
                > allowed_comment_processors
                || self
                    .dialect_set_configuration
                    .get_cdata_section_processors(Some(template_mode))
                    .expect("template mode is non-null")
                    .len()
                    > 1
            {
                return false;
            }
        }
        self.dialect_set_configuration
            .get_pre_processors(Some(template_mode))
            .expect("template mode is non-null")
            .is_empty()
            && self
                .dialect_set_configuration
                .get_post_processors(Some(template_mode))
                .expect("template mode is non-null")
                .is_empty()
    }
}

impl IEngineConfiguration for EngineConfiguration {
    fn get_template_resolvers(&self) -> Vec<&dyn ITemplateResolver> {
        self.template_resolvers.iter().map(Arc::as_ref).collect()
    }
    fn get_message_resolvers(&self) -> Vec<&dyn IMessageResolver> {
        self.message_resolvers.iter().map(Arc::as_ref).collect()
    }
    fn get_link_builders(&self) -> Vec<&dyn ILinkBuilder> {
        self.link_builders.iter().map(Arc::as_ref).collect()
    }
    fn get_cache_manager(&self) -> Option<&dyn ICacheManager> {
        self.cache_manager.as_deref()
    }
    fn get_engine_context_factory(&self) -> &dyn IEngineContextFactory {
        self.engine_context_factory.as_ref()
    }
    fn get_decoupled_template_logic_resolver(&self) -> &dyn IDecoupledTemplateLogicResolver {
        self.decoupled_template_logic_resolver.as_ref()
    }
    fn get_dialect_configurations(&self) -> Vec<&DialectConfiguration> {
        self.dialect_set_configuration
            .get_dialect_configurations()
            .iter()
            .collect()
    }
    fn get_dialects(&self) -> Vec<&dyn crate::IDialect> {
        self.dialect_set_configuration
            .get_dialects()
            .iter()
            .map(Arc::as_ref)
            .collect()
    }
    fn get_dialects_of_type(&self, type_id: TypeId) -> Vec<&dyn crate::IDialect> {
        self.get_dialects()
            .into_iter()
            .filter(|dialect| dialect_matches_type(*dialect, type_id))
            .collect()
    }
    fn is_standard_dialect_present(&self) -> bool {
        self.dialect_set_configuration.is_standard_dialect_present()
    }
    fn get_standard_dialect_prefix(&self) -> Option<&JavaString> {
        self.dialect_set_configuration.get_standard_dialect_prefix()
    }
    fn get_element_definitions(&self) -> &ElementDefinitions {
        self.dialect_set_configuration.get_element_definitions()
    }
    fn get_attribute_definitions(&self) -> &AttributeDefinitions {
        self.dialect_set_configuration.get_attribute_definitions()
    }
    fn get_template_boundaries_processors(
        &self,
        mode: TemplateMode,
    ) -> Vec<&dyn ITemplateBoundariesProcessor> {
        self.dialect_set_configuration
            .get_template_boundaries_processors(Some(mode))
            .expect("EngineConfiguration always supplies a non-null template mode")
    }
    fn get_cdata_section_processors(&self, mode: TemplateMode) -> Vec<&dyn ICDATASectionProcessor> {
        self.dialect_set_configuration
            .get_cdata_section_processors(Some(mode))
            .expect("EngineConfiguration always supplies a non-null template mode")
    }
    fn get_comment_processors(&self, mode: TemplateMode) -> Vec<&dyn ICommentProcessor> {
        self.dialect_set_configuration
            .get_comment_processors(Some(mode))
            .expect("EngineConfiguration always supplies a non-null template mode")
    }
    fn get_doc_type_processors(&self, mode: TemplateMode) -> Vec<&dyn IDocTypeProcessor> {
        self.dialect_set_configuration
            .get_doc_type_processors(Some(mode))
            .expect("EngineConfiguration always supplies a non-null template mode")
    }
    fn get_element_processors(&self, mode: TemplateMode) -> Vec<&dyn IElementProcessor> {
        self.dialect_set_configuration
            .get_element_processors(Some(mode))
            .expect("EngineConfiguration always supplies a non-null template mode")
    }
    fn get_text_processors(&self, mode: TemplateMode) -> Vec<&dyn ITextProcessor> {
        self.dialect_set_configuration
            .get_text_processors(Some(mode))
            .expect("EngineConfiguration always supplies a non-null template mode")
    }
    fn get_processing_instruction_processors(
        &self,
        mode: TemplateMode,
    ) -> Vec<&dyn IProcessingInstructionProcessor> {
        self.dialect_set_configuration
            .get_processing_instruction_processors(Some(mode))
            .expect("EngineConfiguration always supplies a non-null template mode")
    }
    fn get_xml_declaration_processors(
        &self,
        mode: TemplateMode,
    ) -> Vec<&dyn IXMLDeclarationProcessor> {
        self.dialect_set_configuration
            .get_xml_declaration_processors(Some(mode))
            .expect("EngineConfiguration always supplies a non-null template mode")
    }
    fn get_pre_processors(&self, mode: TemplateMode) -> Vec<&dyn IPreProcessor> {
        self.dialect_set_configuration
            .get_pre_processors(Some(mode))
            .expect("EngineConfiguration always supplies a non-null template mode")
    }
    fn get_post_processors(&self, mode: TemplateMode) -> Vec<&dyn IPostProcessor> {
        self.dialect_set_configuration
            .get_post_processors(Some(mode))
            .expect("EngineConfiguration always supplies a non-null template mode")
    }
    fn get_execution_attributes(
        &self,
    ) -> &IndexMap<Option<JavaString>, Option<Arc<ExecutionAttributeValue>>> {
        self.dialect_set_configuration.get_execution_attributes()
    }
    fn get_expression_object_factory(&self) -> Arc<dyn IExpressionObjectFactory> {
        self.dialect_set_configuration
            .get_expression_object_factory()
    }
    fn get_template_manager(&self) -> &dyn ITemplateManager {
        self.template_manager
            .get()
            .expect("EngineConfiguration is initialized before publication")
    }
    fn get_model_factory(&self, mode: TemplateMode) -> &dyn IModelFactory {
        self.model_factory_slot(mode).get_or_init(|| {
            StandardModelFactory::new(
                self.shared_configuration(),
                mode,
                self.dialect_set_configuration.get_element_definitions_arc(),
                self.dialect_set_configuration
                    .get_attribute_definitions_arc(),
            )
        })
    }
}

/// 比较两个可空顺序值。
///
/// Java `null` 排在所有显式顺序之后；两边都是 `null` 时相等。
///
/// # 参数
///
/// - `left`：左侧顺序值；
/// - `right`：右侧顺序值。
///
/// # 返回值
///
/// 返回左值相对右值的稳定排序关系。
fn compare_optional_order(left: Option<i32>, right: Option<i32>) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

/// 按 Java `Class#isInstance` 语义判断方言是否匹配具体类或已知方言接口。
fn dialect_matches_type(dialect: &dyn IDialect, type_id: TypeId) -> bool {
    if type_id == TypeId::of::<dyn IDialect>() {
        return true;
    }
    if type_id == TypeId::of::<dyn IProcessorDialect>() {
        return dialect.as_processor_dialect().is_some();
    }
    if type_id == TypeId::of::<dyn IExecutionAttributeDialect>() {
        return dialect.as_execution_attribute_dialect().is_some();
    }
    if type_id == TypeId::of::<dyn IExpressionObjectDialect>() {
        return dialect.as_expression_object_dialect().is_some();
    }
    if type_id == TypeId::of::<dyn IPreProcessorDialect>() {
        return dialect.as_pre_processor_dialect().is_some();
    }
    if type_id == TypeId::of::<dyn IPostProcessorDialect>() {
        return dialect.as_post_processor_dialect().is_some();
    }
    dialect.dialect_type_id() == type_id
}

/// 模板解析器的 Java 空值安全顺序比较器。
///
/// 对应 Java: `EngineConfiguration.TemplateResolverComparator`。
struct TemplateResolverComparator;

impl TemplateResolverComparator {
    fn compare(
        left: &Arc<dyn ITemplateResolver>,
        right: &Arc<dyn ITemplateResolver>,
    ) -> std::cmp::Ordering {
        compare_optional_order(left.get_order(), right.get_order())
    }
}

/// 消息解析器的 Java 空值安全顺序比较器。
///
/// 对应 Java: `EngineConfiguration.MessageResolverComparator`。
struct MessageResolverComparator;

impl MessageResolverComparator {
    fn compare(
        left: &Arc<dyn IMessageResolver>,
        right: &Arc<dyn IMessageResolver>,
    ) -> std::cmp::Ordering {
        compare_optional_order(left.get_order(), right.get_order())
    }
}

/// 链接构建器的 Java 空值安全顺序比较器。
///
/// 对应 Java: `EngineConfiguration.LinkBuilderComparator`。
struct LinkBuilderComparator;

impl LinkBuilderComparator {
    fn compare(left: &Arc<dyn ILinkBuilder>, right: &Arc<dyn ILinkBuilder>) -> std::cmp::Ordering {
        compare_optional_order(left.get_order(), right.get_order())
    }
}
