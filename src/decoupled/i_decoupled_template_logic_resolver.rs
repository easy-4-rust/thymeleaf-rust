use std::sync::Arc;

use crate::templateresource::ITemplateResource;
use crate::util::JavaString;
use crate::{IEngineConfiguration, TemplateMode};

/// 解析与主模板资源配套的解耦逻辑资源。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.markup.decoupled.IDecoupledTemplateLogicResolver`。
pub trait IDecoupledTemplateLogicResolver: Send + Sync {
    /// 返回解耦逻辑资源；不存在时返回 `None`。
    fn resolve_decoupled_template_logic(
        &self,
        configuration: &dyn IEngineConfiguration,
        owner_template: Option<&JavaString>,
        template: &JavaString,
        template_selectors: Option<&[JavaString]>,
        resource: &dyn ITemplateResource,
        template_mode: TemplateMode,
    ) -> Option<Arc<dyn ITemplateResource>>;
}
