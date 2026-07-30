use crate::util::JavaString;
use crate::{IEngineConfiguration, TemplateResolutionAttributes};

use super::TemplateResolution;

/// 把模板标识解析为资源、模式和缓存策略的合同。
///
/// 对应 Java: `org.thymeleaf.templateresolver.ITemplateResolver`。
pub trait ITemplateResolver: Send + Sync {
    /// 返回可空 Resolver 名称。
    fn get_name(&self) -> Option<&JavaString>;
    /// 返回可空执行顺序；未设置顺序的 Resolver 最后执行。
    fn get_order(&self) -> Option<i32>;
    /// 尝试解析模板；不适用时返回 `None`。
    fn resolve_template(
        &self,
        configuration: &dyn IEngineConfiguration,
        owner_template: Option<&JavaString>,
        template: &JavaString,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
    ) -> Option<TemplateResolution>;
}
