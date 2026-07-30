use crate::engine::TemplateModel;

use super::{ICacheEntryValidityChecker, TemplateCacheKey};

/// 根据解析模板自身携带的有效性策略检查缓存条目。
///
/// 缓存键和条目创建时间不参与判断；资源是否变化、TTL 是否到期等语义由
/// `TemplateData` 内的 `ICacheEntryValidity` 实现负责。
///
/// 对应 Java: `org.thymeleaf.cache.StandardParsedTemplateEntryValidator`。
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardParsedTemplateEntryValidator;

impl StandardParsedTemplateEntryValidator {
    /// 创建无状态的标准模板缓存有效性检查器。
    ///
    /// 对应 Java:
    /// `StandardParsedTemplateEntryValidator#StandardParsedTemplateEntryValidator()`。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ICacheEntryValidityChecker<TemplateCacheKey, TemplateModel>
    for StandardParsedTemplateEntryValidator
{
    fn check_is_value_still_valid(
        &self,
        _key: &TemplateCacheKey,
        value: &TemplateModel,
        _entry_creation_timestamp: i64,
    ) -> bool {
        value
            .get_template_data()
            .get_validity()
            .expect("TemplateModel cache validity cannot be null")
            .is_cache_still_valid()
    }
}
