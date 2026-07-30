use std::sync::Arc;

use crate::TemplateResolutionAttributes;
use crate::cache::{ICacheEntryValidity, NonCacheableCacheEntryValidity};
use crate::util::JavaString;
use crate::{IEngineConfiguration, ITemplateResource, UrlTemplateResource};

use super::{AbstractConfigurableTemplateResolver, ITemplateResolver, TemplateResolution};

/// 从 URL 解析模板资源的可配置解析器。
///
/// 非法 URL 表示当前解析器不适用而返回 `None`；包含 `;jsessionid` 的模板名强制
/// 不缓存，避免同一模板因不同会话标识占据多个缓存条目。
///
/// 对应 Java: `org.thymeleaf.templateresolver.UrlTemplateResolver`。
pub struct UrlTemplateResolver {
    resolver: AbstractConfigurableTemplateResolver,
}

impl UrlTemplateResolver {
    /// 创建使用标准可配置默认值的 URL 解析器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            resolver: AbstractConfigurableTemplateResolver::new(
                "org.thymeleaf.templateresolver.UrlTemplateResolver",
            ),
        }
    }

    fn compute_validity(&self, template: &JavaString) -> Arc<dyn ICacheEntryValidity> {
        if template
            .to_string_lossy()
            .to_lowercase()
            .contains(";jsessionid")
        {
            return Arc::new(NonCacheableCacheEntryValidity::new());
        }
        self.resolver.compute_validity(template)
    }
}

impl Default for UrlTemplateResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for UrlTemplateResolver {
    type Target = AbstractConfigurableTemplateResolver;
    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl std::ops::DerefMut for UrlTemplateResolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}

impl ITemplateResolver for UrlTemplateResolver {
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
    ) -> Option<TemplateResolution> {
        self.resolver.resolver().resolve_template(
            template,
            || {
                let resource_name = self.resolver.compute_resource_name(template);
                UrlTemplateResource::new(
                    Some(&resource_name.to_string_lossy()),
                    self.resolver
                        .get_character_encoding()
                        .map(JavaString::to_string_lossy)
                        .as_deref(),
                )
                .ok()
                .map(|resource| Arc::new(resource) as Arc<dyn ITemplateResource>)
            },
            || self.resolver.compute_template_mode(template),
            || self.compute_validity(template),
        )
    }
}
