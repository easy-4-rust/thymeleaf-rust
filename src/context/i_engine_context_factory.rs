use std::sync::Arc;

use indexmap::IndexMap;

use crate::IEngineConfiguration;
use crate::engine::TemplateData;
use crate::expression::TemplateValue;
use crate::util::JavaString;

use super::{IContext, IEngineContext};

/// 从用户上下文创建引擎内部上下文的工厂合同。
///
/// 对应 Java: `org.thymeleaf.context.IEngineContextFactory`。
pub trait IEngineContextFactory {
    /// 创建或复用本次模板执行的 EngineContext。
    fn create_engine_context(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        template_data: TemplateData,
        template_resolution_attributes: Option<
            &IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>,
        >,
        context: &dyn IContext,
    ) -> Box<dyn IEngineContext>;
}
