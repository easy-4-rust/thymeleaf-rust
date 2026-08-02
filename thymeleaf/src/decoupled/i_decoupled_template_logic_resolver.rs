use std::sync::Arc;

use crate::templateresource::{ITemplateResource, TemplateResourceError};
use crate::util::JavaString;
use crate::{IEngineConfiguration, TemplateMode};

/// 解析与主模板资源配套的解耦逻辑资源。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.markup.decoupled.IDecoupledTemplateLogicResolver`。
pub trait IDecoupledTemplateLogicResolver: Send + Sync {
    /// 返回解耦逻辑资源；Java 实现返回 null 时为 `Ok(None)`。
    ///
    /// 相对资源构造失败时保留底层 `TemplateResourceError`，不得把 Java 原本会传播
    /// 的异常静默改写成“没有解耦逻辑”。
    fn resolve_decoupled_template_logic(
        &self,
        configuration: &dyn IEngineConfiguration,
        owner_template: Option<&JavaString>,
        template: &JavaString,
        template_selectors: Option<&[JavaString]>,
        resource: &dyn ITemplateResource,
        template_mode: TemplateMode,
    ) -> Result<Option<Arc<dyn ITemplateResource>>, TemplateResourceError>;
}
