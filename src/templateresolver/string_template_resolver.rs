use std::sync::Arc;

use crate::TemplateResolutionAttributes;
use crate::cache::{
    AlwaysValidCacheEntryValidity, ICacheEntryValidity, NonCacheableCacheEntryValidity,
    TTLCacheEntryValidity,
};
use crate::exceptions::ConfigurationException;
use crate::util::JavaString;
use crate::{
    IEngineConfiguration, ITemplateResource, StringTemplateResource, TemplateMode,
    TemplateModeParseError,
};

use super::{
    AbstractTemplateResolver, ITemplateResolver, TemplateResolution, TemplateResolverError,
};

/// 把待解析模板名本身当作模板正文的解析器。
///
/// 对应 Java: `org.thymeleaf.templateresolver.StringTemplateResolver`。
pub struct StringTemplateResolver {
    resolver: AbstractTemplateResolver,
    template_mode: TemplateMode,
    cacheable: bool,
    cache_ttl_ms: Option<i64>,
}

impl StringTemplateResolver {
    /// 默认 HTML 模式。
    pub const DEFAULT_TEMPLATE_MODE: TemplateMode = TemplateMode::HTML;
    /// 字符串模板默认不缓存。
    pub const DEFAULT_CACHEABLE: bool = false;
    /// 默认缓存 TTL；`None` 表示由缓存容量淘汰。
    pub const DEFAULT_CACHE_TTL_MS: Option<i64> = None;

    /// 创建字符串模板解析器。
    ///
    /// 对应 Java: `StringTemplateResolver#StringTemplateResolver()`。
    #[must_use]
    pub fn new() -> Self {
        Self {
            resolver: AbstractTemplateResolver::new(
                "org.thymeleaf.templateresolver.StringTemplateResolver",
            ),
            template_mode: Self::DEFAULT_TEMPLATE_MODE,
            cacheable: Self::DEFAULT_CACHEABLE,
            cache_ttl_ms: Self::DEFAULT_CACHE_TTL_MS,
        }
    }

    /// 返回模板模式。
    #[must_use]
    pub const fn get_template_mode(&self) -> TemplateMode {
        self.template_mode
    }

    /// 设置模板模式。
    pub const fn set_template_mode(&mut self, template_mode: TemplateMode) {
        self.template_mode = template_mode;
    }

    /// 使用可空枚举设置模板模式。
    ///
    /// 对应 Java: `StringTemplateResolver#setTemplateMode(TemplateMode)`。
    ///
    /// # 错误
    /// `template_mode` 缺失时返回 Java setter 的精确参数错误。
    pub fn set_template_mode_nullable(
        &mut self,
        template_mode: Option<TemplateMode>,
    ) -> Result<(), TemplateResolverError> {
        self.template_mode = template_mode.ok_or_else(|| {
            TemplateResolverError::InvalidArgument(
                "Cannot set a null template mode value".to_owned(),
            )
        })?;
        Ok(())
    }

    /// 使用 Java 兼容文本设置模板模式。
    ///
    /// 对应 Java: `StringTemplateResolver#setTemplateMode(String)`。
    pub fn set_template_mode_name(
        &mut self,
        template_mode: Option<&str>,
    ) -> Result<(), TemplateResolverError> {
        let template_mode = template_mode.ok_or_else(|| {
            TemplateResolverError::InvalidArgument(
                "Cannot set a null template mode value".to_owned(),
            )
        })?;
        self.template_mode =
            TemplateMode::parse(Some(template_mode)).map_err(|error: TemplateModeParseError| {
                TemplateResolverError::InvalidArgument(error.to_string())
            })?;
        Ok(())
    }

    /// 返回是否缓存解析结果。
    #[must_use]
    pub const fn is_cacheable(&self) -> bool {
        self.cacheable
    }

    /// 设置是否缓存解析结果。
    pub const fn set_cacheable(&mut self, cacheable: bool) {
        self.cacheable = cacheable;
    }

    /// 返回缓存 TTL。
    #[must_use]
    pub const fn get_cache_ttl_ms(&self) -> Option<i64> {
        self.cache_ttl_ms
    }

    /// 设置缓存 TTL。
    pub const fn set_cache_ttl_ms(&mut self, cache_ttl_ms: Option<i64>) {
        self.cache_ttl_ms = cache_ttl_ms;
    }

    /// 字符串资源不支持解耦逻辑；启用时返回配置异常。
    ///
    /// 对应 Java: `StringTemplateResolver#setUseDecoupledLogic(boolean)`。
    pub fn set_use_decoupled_logic(
        &mut self,
        use_decoupled_logic: bool,
    ) -> Result<(), ConfigurationException> {
        if use_decoupled_logic {
            return Err(ConfigurationException::new(Some(
                "The 'useDecoupledLogic' flag is not allowed for String template resolution"
                    .to_owned(),
            )));
        }
        self.resolver.set_use_decoupled_logic(false);
        Ok(())
    }

    fn compute_validity(&self) -> Arc<dyn ICacheEntryValidity> {
        if !self.cacheable {
            return Arc::new(NonCacheableCacheEntryValidity::new());
        }
        self.cache_ttl_ms.map_or_else(
            || Arc::new(AlwaysValidCacheEntryValidity::new()) as Arc<dyn ICacheEntryValidity>,
            |ttl| Arc::new(TTLCacheEntryValidity::new(ttl)),
        )
    }
}

impl Default for StringTemplateResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for StringTemplateResolver {
    type Target = AbstractTemplateResolver;

    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl std::ops::DerefMut for StringTemplateResolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}

impl ITemplateResolver for StringTemplateResolver {
    fn get_name(&self) -> Option<&JavaString> {
        self.resolver.get_name()
    }

    fn get_order(&self) -> Option<i32> {
        self.resolver.get_order()
    }

    fn resolve_template(
        &self,
        _configuration: &dyn IEngineConfiguration,
        _owner_template: Option<&JavaString>,
        template: &JavaString,
        _template_resolution_attributes: Option<&TemplateResolutionAttributes>,
    ) -> Result<Option<TemplateResolution>, TemplateResolverError> {
        self.resolver.resolve_template(
            template,
            || {
                StringTemplateResource::new(Some(&template.to_string_lossy()))
                    .map(|resource| Some(Arc::new(resource) as Arc<dyn ITemplateResource>))
                    .map_err(TemplateResolverError::from)
            },
            || self.template_mode,
            || self.compute_validity(),
        )
    }
}
