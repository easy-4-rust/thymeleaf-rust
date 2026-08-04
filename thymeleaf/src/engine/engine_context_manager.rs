use std::sync::Arc;

use crate::context::{IContext, IEngineContext};
use crate::{IEngineConfiguration, TemplateResolutionAttributes};

use super::TemplateData;

/// 在模板执行边界创建或复用 Engine Context 的静态管理器。
///
/// 管理器确保每次 `prepare_engine_context` 都有一次配对的层级增加，使后续
/// `dispose_engine_context` 可以无条件降低层级。原始 Context 已经是 Engine Context
/// 时保留对象身份，并把嵌套 TemplateData 写入新层；否则只调用配置中的工厂一次，
/// 根 TemplateData 已由工厂构造过程写入，不重复设置。
///
/// 对应 Java: `org.thymeleaf.engine.EngineContextManager`。
pub(crate) struct EngineContextManager;

impl EngineContextManager {
    /// 创建或复用模板执行上下文，并无条件增加一级。
    ///
    /// # 参数
    ///
    /// - `configuration`：当前冻结引擎配置。
    /// - `template_data`：根模板或当前嵌套模板的数据。
    /// - `template_resolution_attributes`：可空解析属性，仅创建新上下文时交给工厂。
    /// - `context`：调用方 Context 或上一层已经存在的 Engine Context。
    ///
    /// # 返回值
    ///
    /// 新 Context 或复用的同一 Context，且返回时层级已经增加一次。复用分支先增加
    /// 层级，再把 TemplateData 设置到该层，与 Java 的可观察模板栈顺序一致。
    ///
    /// 对应 Java: `EngineContextManager#prepareEngineContext`。
    pub(crate) fn prepare_engine_context(
        configuration: Arc<dyn IEngineConfiguration>,
        template_data: TemplateData,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
        context: &dyn IContext,
    ) -> Arc<dyn IEngineContext> {
        if let Some(engine_context) = context.get_engine_context_arc() {
            engine_context.increase_level();
            engine_context.set_template_data(Arc::new(template_data));
            return engine_context;
        }
        let engine_context = configuration
            .get_engine_context_factory()
            .create_engine_context(
                Arc::clone(&configuration),
                template_data,
                template_resolution_attributes,
                context,
            );
        engine_context.increase_level();
        engine_context
    }

    /// 结束一次模板处理边界并恢复上一上下文层。
    ///
    /// # 参数
    ///
    /// - `engine_context`：此前由 `prepare_engine_context` 返回的上下文。
    ///
    /// 方法只委托 `decrease_level`；层级状态、局部变量、selection、inliner、
    /// TemplateData 和元素栈的恢复由具体 Engine Context 负责。
    ///
    /// 对应 Java: `EngineContextManager#disposeEngineContext`。
    pub(crate) fn dispose_engine_context(engine_context: &dyn IEngineContext) {
        engine_context.decrease_level();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use indexmap::IndexMap;

    use super::{EngineContextManager, TemplateData};
    use crate::cache::AlwaysValidCacheEntryValidity;
    use crate::context::{Context, EngineContext, IContext, IEngineContext, IEngineContextFactory};
    use crate::expression::TemplateValue;
    use crate::templateresource::StringTemplateResource;
    use crate::util::{JavaLocale, Utf16String};
    use crate::{
        IEngineConfiguration, ITemplateEngine, TemplateEngine, TemplateMode,
        TemplateResolutionAttributeValue, TemplateResolutionAttributes,
    };

    fn golden() -> BTreeMap<String, String> {
        include_str!("../../tests/fixtures/engine_context_factory_golden.txt")
            .lines()
            .map(|line| {
                let (key, value) = line.split_once('=').expect("golden key/value");
                (key.to_owned(), value.to_owned())
            })
            .collect()
    }

    fn utf16_string(value: &str) -> Utf16String {
        Utf16String::from_rust_str(value)
    }

    fn locale(language_tag: &str, country: &str) -> JavaLocale {
        JavaLocale::new(utf16_string(language_tag), utf16_string(country))
    }

    fn template_data(name: &str) -> TemplateData {
        TemplateData::new(
            Some(utf16_string(name)),
            None,
            Some(Arc::new(
                StringTemplateResource::new(Some(name)).expect("string resource"),
            )),
            Some(TemplateMode::HTML),
            Some(Arc::new(AlwaysValidCacheEntryValidity::new())),
        )
    }

    fn resolution_attributes() -> TemplateResolutionAttributes {
        let mut attributes = TemplateResolutionAttributes::new();
        attributes.insert(
            Some("second".to_owned()),
            TemplateResolutionAttributeValue::new(2_i32),
        );
        attributes.insert(
            Some("first".to_owned()),
            TemplateResolutionAttributeValue::new("one".to_owned()),
        );
        attributes
    }

    fn stack(context: &dyn IEngineContext) -> String {
        context
            .get_template_stack()
            .into_iter()
            .map(|template_data| {
                template_data
                    .get_template()
                    .expect("template")
                    .to_string_lossy()
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn manager_create_reuse_and_dispose_lifecycle_matches_java_golden() {
        let fixture = golden();
        assert!(
            fixture["shape.manager"]
                .contains("static org.thymeleaf.context.IEngineContext prepareEngineContext(")
        );
        assert!(
            fixture["shape.manager"]
                .contains("static void disposeEngineContext(org.thymeleaf.context.IEngineContext)")
        );
        assert!(fixture["shape.manager"].contains(
            "private static org.thymeleaf.context.IEngineContext createEngineContextIfNeeded("
        ));
        assert!(fixture["shape.manager"].contains("private <init>()"));

        let counting_factory = Arc::new(CountingFactory::default());
        let engine = TemplateEngine::new();
        let configured_factory: Arc<dyn IEngineContextFactory> = counting_factory.clone();
        engine
            .set_engine_context_factory(configured_factory)
            .expect("factory before initialization");
        let configuration = engine.get_configuration().expect("engine configuration");
        let attributes = resolution_attributes();

        let original_variables = vec![(
            Some(utf16_string("root")),
            Some(Arc::new(TemplateValue::string(utf16_string("value")))),
        )];
        let original = Context::with_locale_and_variables(
            Some(locale("en-US", "US")),
            Some(original_variables.as_slice()),
        );
        let created = EngineContextManager::prepare_engine_context(
            Arc::clone(&configuration),
            template_data("root"),
            Some(&attributes),
            &original,
        );
        assert_eq!(
            counting_factory.calls.load(Ordering::SeqCst).to_string(),
            fixture["manager.created.factory.calls"]
        );
        assert!(created.as_any().is::<EngineContext>());
        assert_eq!(
            fixture["manager.created.class"],
            "org.thymeleaf.context.EngineContext"
        );
        assert_eq!(
            created.level().to_string(),
            fixture["manager.created.level"]
        );
        assert_eq!(
            created
                .get_template_data()
                .get_template()
                .expect("template")
                .to_string_lossy(),
            fixture["manager.created.template"]
        );
        assert_eq!(stack(created.as_ref()), fixture["manager.created.stack"]);
        EngineContextManager::dispose_engine_context(created.as_ref());
        assert_eq!(
            created.level().to_string(),
            fixture["manager.created.disposed.level"]
        );
        assert_eq!(
            stack(created.as_ref()),
            fixture["manager.created.disposed.stack"]
        );

        let mut existing_variables = IndexMap::new();
        existing_variables.insert(
            Some(utf16_string("existing")),
            Some(Arc::new(TemplateValue::string(utf16_string("yes")))),
        );
        let existing = EngineContext::new(
            Arc::clone(&configuration),
            template_data("existing-root"),
            Some(&attributes),
            locale("en-GB", "GB"),
            Some(&existing_variables),
        );
        let existing_dynamic: Arc<dyn IEngineContext> = existing.clone();
        let reused = EngineContextManager::prepare_engine_context(
            Arc::clone(&configuration),
            template_data("nested"),
            None,
            existing.as_ref(),
        );
        assert_eq!(
            Arc::ptr_eq(&reused, &existing_dynamic).to_string(),
            fixture["manager.reused.same"]
        );
        assert_eq!(
            counting_factory.calls.load(Ordering::SeqCst).to_string(),
            fixture["manager.reused.factory.calls"]
        );
        assert_eq!(reused.level().to_string(), fixture["manager.reused.level"]);
        assert_eq!(
            reused
                .get_template_data()
                .get_template()
                .expect("template")
                .to_string_lossy(),
            fixture["manager.reused.template"]
        );
        assert_eq!(stack(reused.as_ref()), fixture["manager.reused.stack"]);
        EngineContextManager::dispose_engine_context(reused.as_ref());
        assert_eq!(
            reused.level().to_string(),
            fixture["manager.reused.disposed.level"]
        );
        assert_eq!(
            reused
                .get_template_data()
                .get_template()
                .expect("template")
                .to_string_lossy(),
            fixture["manager.reused.disposed.template"]
        );
        assert_eq!(
            stack(reused.as_ref()),
            fixture["manager.reused.disposed.stack"]
        );
    }

    #[derive(Default)]
    struct CountingFactory {
        calls: AtomicUsize,
    }

    impl IEngineContextFactory for CountingFactory {
        fn create_engine_context(
            &self,
            configuration: Arc<dyn IEngineConfiguration>,
            template_data: TemplateData,
            template_resolution_attributes: Option<&TemplateResolutionAttributes>,
            context: &dyn IContext,
        ) -> Arc<dyn IEngineContext> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let names = context.get_variable_names().snapshot();
            let mut variables = IndexMap::with_capacity(names.len());
            for name in names {
                variables.insert(name.clone(), context.get_variable(name.as_ref()));
            }
            EngineContext::new(
                configuration,
                template_data,
                template_resolution_attributes,
                context.get_locale(),
                Some(&variables),
            )
        }
    }
}
