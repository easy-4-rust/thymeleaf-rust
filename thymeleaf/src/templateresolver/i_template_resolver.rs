use crate::util::Utf16String;
use crate::{IEngineConfiguration, TemplateResolutionAttributes};

use super::{TemplateResolution, TemplateResolverError};

/// 把模板标识解析为资源、模式和缓存策略的合同。
///
/// 对应 Java: `org.thymeleaf.templateresolver.ITemplateResolver`。
///
/// Resolver 必须可在线程间安全共享。多个 Resolver 按 `get_order` 排列；返回
/// `Ok(None)` 时继续询问链中的下一个 Resolver。解析结果包含真实资源、模板模式、
/// 缓存有效性以及是否启用解耦逻辑。资源对象存在并不保证底层资源存在，除非 Resolver
/// 明确启用了存在性检查。
pub trait ITemplateResolver: Send + Sync {
    /// 返回用于日志和配置诊断的可空 Resolver 名称。
    fn get_name(&self) -> Option<&Utf16String>;

    /// 返回可空执行顺序。
    ///
    /// 未设置顺序的 Resolver 在已设置顺序的 Resolver 之后执行。
    fn get_order(&self) -> Option<i32>;

    /// 尝试解析指定模板。
    ///
    /// 模板选择器不会传入 Resolver，因为选择操作属于 Parser；`owner_template` 和
    /// `template_resolution_attributes` 均可缺失。
    ///
    /// # 参数
    /// - `configuration`：当前引擎配置。
    /// - `owner_template`：插入当前模板片段的上层模板；缺失表示顶层解析。
    /// - `template`：待解析的模板名或字符串模板正文。
    /// - `template_resolution_attributes`：调用方附加的解析属性。
    ///
    /// # 返回值
    /// 成功解析时返回 `Ok(Some(...))`；当前 Resolver 不适用、资源无法按其协议解析
    /// 或启用存在性检查后资源不存在时返回 `Ok(None)`。
    ///
    /// # 错误
    /// 资源构造、配置或解析结果违反 Java 前置条件时返回类型化错误；错误不能伪装成
    /// “当前 Resolver 不适用”。
    fn resolve_template(
        &self,
        configuration: &dyn IEngineConfiguration,
        owner_template: Option<&Utf16String>,
        template: &Utf16String,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
    ) -> Result<Option<TemplateResolution>, TemplateResolverError>;

    /// 使用 Java 可空参数边界尝试解析模板。
    ///
    /// 正常 Rust 调用应使用 [`Self::resolve_template`]，由类型系统保证配置和模板非空；
    /// 兼容层、反射式配置和双语测试可通过本入口观察 Java 的校验顺序。
    ///
    /// # 参数
    /// - `configuration`：可空引擎配置。
    /// - `owner_template`：可空上层模板。
    /// - `template`：可空模板名。
    /// - `template_resolution_attributes`：可空解析属性。
    ///
    /// # 返回值
    /// 参数有效时与 [`Self::resolve_template`] 完全一致。
    ///
    /// # 错误
    /// 先拒绝空引擎配置，再拒绝空模板名，消息与 Java
    /// `AbstractTemplateResolver#resolveTemplate` 一致。
    fn resolve_template_nullable(
        &self,
        configuration: Option<&dyn IEngineConfiguration>,
        owner_template: Option<&Utf16String>,
        template: Option<&Utf16String>,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
    ) -> Result<Option<TemplateResolution>, TemplateResolverError> {
        let configuration = configuration.ok_or_else(|| {
            TemplateResolverError::InvalidArgument("Engine Configuration cannot be null".to_owned())
        })?;
        let template = template.ok_or_else(|| {
            TemplateResolverError::InvalidArgument("Template Name cannot be null".to_owned())
        })?;
        self.resolve_template(
            configuration,
            owner_template,
            template,
            template_resolution_attributes,
        )
    }
}
