use std::path::PathBuf;
use std::sync::Arc;

use crate::TemplateResolutionAttributes;
use crate::util::JavaString;
use crate::{EmbeddedTemplateResource, IEngineConfiguration, ITemplateResource};

use super::{AbstractConfigurableTemplateResolver, ITemplateResolver, TemplateResolution};

/// 从 Rust 应用的嵌入式资源搜索路径解析模板。
///
/// 对应 Java: `org.thymeleaf.templateresolver.ClassLoaderTemplateResolver`。
pub struct EmbeddedTemplateResolver {
    resolver: AbstractConfigurableTemplateResolver,
    search_roots: Option<Vec<PathBuf>>,
}

impl EmbeddedTemplateResolver {
    /// 使用默认应用资源搜索顺序创建解析器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            resolver: AbstractConfigurableTemplateResolver::new(
                "org.thymeleaf.templateresolver.ClassLoaderTemplateResolver",
            ),
            search_roots: None,
        }
    }

    /// 使用显式有序搜索根目录创建解析器。
    #[must_use]
    pub fn with_search_roots(search_roots: Vec<PathBuf>) -> Self {
        Self {
            search_roots: Some(search_roots),
            ..Self::new()
        }
    }
}

impl Default for EmbeddedTemplateResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for EmbeddedTemplateResolver {
    type Target = AbstractConfigurableTemplateResolver;
    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl std::ops::DerefMut for EmbeddedTemplateResolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}

impl ITemplateResolver for EmbeddedTemplateResolver {
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
                let encoding = self
                    .resolver
                    .get_character_encoding()
                    .map(JavaString::to_string_lossy);
                let resource = match &self.search_roots {
                    Some(search_roots) => EmbeddedTemplateResource::with_search_roots(
                        search_roots.clone(),
                        Some(&resource_name.to_string_lossy()),
                        encoding.as_deref(),
                    ),
                    None => EmbeddedTemplateResource::new(
                        Some(&resource_name.to_string_lossy()),
                        encoding.as_deref(),
                    ),
                };
                resource
                    .ok()
                    .map(|resource| Arc::new(resource) as Arc<dyn ITemplateResource>)
            },
            || self.resolver.compute_template_mode(template),
            || self.resolver.compute_validity(template),
        )
    }
}
