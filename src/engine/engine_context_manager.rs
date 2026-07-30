use std::sync::Arc;

use crate::context::{IContext, IEngineContext};
use crate::{IEngineConfiguration, TemplateResolutionAttributes};

use super::TemplateData;

/// 创建、复用并按嵌套模板边界管理 EngineContext。
///
/// 对应 Java: `org.thymeleaf.engine.EngineContextManager`。
pub(crate) struct EngineContextManager;

impl EngineContextManager {
    /// 准备模板执行上下文并增加一级。
    ///
    /// 若输入已经是 EngineContext，则复用同一对象并设置新模板数据；否则委托配置
    /// 中的工厂创建。对应 Java: `EngineContextManager#prepareEngineContext`。
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

    /// 结束嵌套模板执行并恢复上一上下文层。
    ///
    /// 对应 Java: `EngineContextManager#disposeEngineContext`。
    pub(crate) fn dispose_engine_context(engine_context: &dyn IEngineContext) {
        engine_context.decrease_level();
    }
}
