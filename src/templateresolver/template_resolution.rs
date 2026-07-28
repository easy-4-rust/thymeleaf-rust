use std::rc::Rc;
use std::sync::Arc;

use thiserror::Error;

use crate::cache::ICacheEntryValidity;
use crate::{ITemplateResource, TemplateMode};

/// 模板解析器成功解析模板后的完整结果。
///
/// 对应 Java: `org.thymeleaf.templateresolver.TemplateResolution`。
///
/// 本对象由 `ITemplateResolver` 实现创建，聚合已解析模板资源、建议模板模式、是否已
/// 验证资源存在、是否检查解耦逻辑，以及缓存有效性策略。返回解析结果并不等于底层
/// 资源一定存在：只有 `is_template_resource_existence_verified()` 返回 `true` 时，
/// 才表示 Resolver 已经调用过 `ITemplateResource#exists()` 并确认存在。
///
/// 上游明确声明该对象不应视为线程安全。Rust 因而使用 `Rc` 保存未被
/// `Send + Sync` 约束的模板资源，并使用 `Arc` 保存可跨线程共享的缓存有效性策略。
pub struct TemplateResolution {
    template_resource: Rc<dyn ITemplateResource>,
    template_resource_existence_verified: bool,
    template_mode: TemplateMode,
    use_decoupled_logic: bool,
    validity: Arc<dyn ICacheEntryValidity>,
}

impl TemplateResolution {
    /// 使用上游三参数构造器的默认标志创建模板解析结果。
    ///
    /// 对应 Java:
    /// `TemplateResolution#TemplateResolution(ITemplateResource,TemplateMode,ICacheEntryValidity)`。
    ///
    /// # 参数
    /// - `template_resource`：Resolver 创建的模板资源；`None` 对应 Java `null`。
    /// - `template_mode`：Resolver 建议的模板模式；`None` 对应 Java `null`。
    /// - `validity`：解析结果的缓存有效性；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// 创建成功时，资源存在性已验证和解耦逻辑标志均为 `false`。
    ///
    /// # 错误
    /// 任一必填对象缺失时，按 Java 构造器的校验顺序返回精确参数错误。
    pub fn new(
        template_resource: Option<Rc<dyn ITemplateResource>>,
        template_mode: Option<TemplateMode>,
        validity: Option<Arc<dyn ICacheEntryValidity>>,
    ) -> Result<Self, TemplateResolutionError> {
        Self::with_options(template_resource, false, template_mode, false, validity)
    }

    /// 使用完整五参数构造器创建模板解析结果。
    ///
    /// 对应 Java:
    /// `TemplateResolution#TemplateResolution(ITemplateResource,boolean,TemplateMode,boolean,ICacheEntryValidity)`。
    ///
    /// # 参数
    /// - `template_resource`：Resolver 创建的模板资源；`None` 对应 Java `null`。
    /// - `template_resource_existence_verified`：Resolver 是否已确认资源存在。
    /// - `template_mode`：Resolver 建议的模板模式；`None` 对应 Java `null`。
    /// - `use_decoupled_logic`：解析期间是否应查找可选解耦逻辑资源。
    /// - `validity`：解析结果的缓存有效性；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// 保留全部对象身份与标志值的新解析结果。
    ///
    /// # 错误
    /// 按资源、模板模式、有效性的顺序校验必填对象，并返回 Java 精确错误消息。
    pub fn with_options(
        template_resource: Option<Rc<dyn ITemplateResource>>,
        template_resource_existence_verified: bool,
        template_mode: Option<TemplateMode>,
        use_decoupled_logic: bool,
        validity: Option<Arc<dyn ICacheEntryValidity>>,
    ) -> Result<Self, TemplateResolutionError> {
        let template_resource = template_resource.ok_or(
            TemplateResolutionError::InvalidArgument("Template Resource cannot be null"),
        )?;
        let template_mode = template_mode.ok_or(TemplateResolutionError::InvalidArgument(
            "Template mode cannot be null",
        ))?;
        let validity = validity.ok_or(TemplateResolutionError::InvalidArgument(
            "Validity cannot be null",
        ))?;
        Ok(Self {
            template_resource,
            template_resource_existence_verified,
            template_mode,
            use_decoupled_logic,
            validity,
        })
    }

    /// 返回 Resolver 创建的同一模板资源实例。
    ///
    /// 对应 Java: `TemplateResolution#getTemplateResource()`。
    ///
    /// 资源对象本身恒非空，但除非存在性已验证标志为 `true`，否则不能仅凭该对象存在
    /// 就断言底层资源存在。
    ///
    /// # 返回
    /// 构造时传入的模板资源动态实例。
    #[must_use]
    pub fn get_template_resource(&self) -> &dyn ITemplateResource {
        self.template_resource.as_ref()
    }

    /// 返回 Resolver 建议的模板模式。
    ///
    /// 对应 Java: `TemplateResolution#getTemplateMode()`。
    ///
    /// 引擎可在调用端用显式配置的模板模式覆盖此建议。
    ///
    /// # 返回
    /// 构造时指定的模板模式。
    #[must_use]
    pub const fn get_template_mode(&self) -> TemplateMode {
        self.template_mode
    }

    /// 返回 Resolver 是否已确认模板资源真实存在。
    ///
    /// 对应 Java: `TemplateResolution#isTemplateResourceExistenceVerified()`。
    ///
    /// `false` 只表示尚未检查，不能解释为资源不存在；该标志用于避免重复执行成本可能
    /// 很高的 `ITemplateResource#exists()`。
    ///
    /// # 返回
    /// 已在解析阶段确认存在时返回 `true`。
    #[must_use]
    pub const fn is_template_resource_existence_verified(&self) -> bool {
        self.template_resource_existence_verified
    }

    /// 返回解析时是否应检查解耦逻辑资源。
    ///
    /// 对应 Java: `TemplateResolution#getUseDecoupledLogic()`。
    ///
    /// `true` 只要求检查并在存在时使用，不表示解耦逻辑资源必然存在。
    ///
    /// # 返回
    /// 应检查解耦逻辑时返回 `true`。
    #[must_use]
    pub const fn get_use_decoupled_logic(&self) -> bool {
        self.use_decoupled_logic
    }

    /// 返回构造时传入的同一缓存有效性实例。
    ///
    /// 对应 Java: `TemplateResolution#getValidity()`。
    ///
    /// 有效性决定解析结果能否进入模板缓存，以及缓存项何时应被丢弃并重新解析。
    ///
    /// # 返回
    /// 构造时传入的缓存有效性动态实例。
    #[must_use]
    pub fn get_validity(&self) -> &dyn ICacheEntryValidity {
        self.validity.as_ref()
    }
}

/// 创建 `TemplateResolution` 时的参数校验错误。
///
/// 对应 Java: `org.thymeleaf.util.Validate` 在
/// `org.thymeleaf.templateresolver.TemplateResolution` 构造器中抛出的
/// `IllegalArgumentException`。
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TemplateResolutionError {
    /// 必填参数为 Java `null`。
    #[error("{0}")]
    InvalidArgument(&'static str),
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::sync::Arc;

    use super::{TemplateResolution, TemplateResolutionError};
    use crate::cache::{
        AlwaysValidCacheEntryValidity, ICacheEntryValidity, NonCacheableCacheEntryValidity,
    };
    use crate::{ITemplateResource, StringTemplateResource, TemplateMode};

    #[test]
    fn validates_required_objects_in_java_order_with_exact_messages() {
        assert_eq!(
            TemplateResolution::with_options(None, true, None, true, None)
                .err()
                .expect("null resource"),
            TemplateResolutionError::InvalidArgument("Template Resource cannot be null")
        );

        let resource: Rc<dyn ITemplateResource> =
            Rc::new(StringTemplateResource::new(Some("template")).expect("string resource"));
        assert_eq!(
            TemplateResolution::new(Some(Rc::clone(&resource)), None, None)
                .err()
                .expect("null mode"),
            TemplateResolutionError::InvalidArgument("Template mode cannot be null")
        );
        assert_eq!(
            TemplateResolution::new(Some(resource), Some(TemplateMode::HTML), None)
                .err()
                .expect("null validity"),
            TemplateResolutionError::InvalidArgument("Validity cannot be null")
        );
    }

    #[test]
    fn three_argument_constructor_preserves_identity_and_default_flags() {
        let resource: Rc<dyn ITemplateResource> =
            Rc::new(StringTemplateResource::new(Some("body")).expect("string resource"));
        let validity: Arc<dyn ICacheEntryValidity> = Arc::new(AlwaysValidCacheEntryValidity::new());
        let resolution = TemplateResolution::new(
            Some(Rc::clone(&resource)),
            Some(TemplateMode::HTML),
            Some(Arc::clone(&validity)),
        )
        .expect("template resolution");

        assert!(std::ptr::eq(
            resolution.get_template_resource(),
            resource.as_ref()
        ));
        assert!(std::ptr::eq(resolution.get_validity(), validity.as_ref()));
        assert_eq!(resolution.get_template_mode(), TemplateMode::HTML);
        assert!(!resolution.is_template_resource_existence_verified());
        assert!(!resolution.get_use_decoupled_logic());
        assert!(resolution.get_template_resource().exists());
        assert!(resolution.get_validity().is_cacheable());
    }

    #[test]
    fn full_constructor_preserves_independent_flags_and_dynamic_contracts() {
        let resource: Rc<dyn ITemplateResource> =
            Rc::new(StringTemplateResource::new(Some("")).expect("empty resource"));
        let validity: Arc<dyn ICacheEntryValidity> =
            Arc::new(NonCacheableCacheEntryValidity::new());
        let resolution = TemplateResolution::with_options(
            Some(resource),
            true,
            Some(TemplateMode::RAW),
            true,
            Some(validity),
        )
        .expect("full template resolution");

        assert!(resolution.is_template_resource_existence_verified());
        assert!(resolution.get_use_decoupled_logic());
        assert_eq!(resolution.get_template_mode(), TemplateMode::RAW);
        assert!(!resolution.get_validity().is_cacheable());
        assert!(!resolution.get_validity().is_cache_still_valid());
    }
}
