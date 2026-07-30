use std::sync::Arc;

use crate::engine::TemplateData;
use crate::{IEngineConfiguration, TemplateResolutionAttributes};

use super::{IContext, IEngineContext};

/// 从用户上下文创建引擎内部上下文的工厂合同。
///
/// 对应 Java: `org.thymeleaf.context.IEngineContextFactory`。
pub trait IEngineContextFactory: Send + Sync {
    /// 创建或复用本次模板执行的 EngineContext。
    fn create_engine_context(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        template_data: TemplateData,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
        context: &dyn IContext,
    ) -> Arc<dyn IEngineContext>;
}
