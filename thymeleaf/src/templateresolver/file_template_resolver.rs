use std::sync::Arc;

use crate::TemplateResolutionAttributes;
use crate::util::Utf16String;
use crate::{FileTemplateResource, IEngineConfiguration, ITemplateResource};

use super::{
    AbstractConfigurableTemplateResolver, ITemplateResolver, TemplateResolution,
    TemplateResolverError,
};

/// 从文件系统解析模板资源的可配置解析器。
///
/// 对应 Java: `org.thymeleaf.templateresolver.FileTemplateResolver`。
pub struct FileTemplateResolver {
    resolver: AbstractConfigurableTemplateResolver,
}

impl FileTemplateResolver {
    /// 创建使用标准可配置解析器默认值的文件解析器。
    ///
    /// 对应 Java: `FileTemplateResolver#FileTemplateResolver()`。
    #[must_use]
    pub fn new() -> Self {
        Self {
            resolver: AbstractConfigurableTemplateResolver::new(
                "org.thymeleaf.templateresolver.FileTemplateResolver",
            ),
        }
    }
}

impl Default for FileTemplateResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for FileTemplateResolver {
    type Target = AbstractConfigurableTemplateResolver;
    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl std::ops::DerefMut for FileTemplateResolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}

impl ITemplateResolver for FileTemplateResolver {
    fn get_name(&self) -> Option<&Utf16String> {
        self.resolver.get_name()
    }

    fn get_order(&self) -> Option<i32> {
        self.resolver.get_order()
    }

    fn resolve_template(
        &self,
        _configuration: &dyn IEngineConfiguration,
        _owner_template: Option<&Utf16String>,
        template: &Utf16String,
        _template_resolution_attributes: Option<&TemplateResolutionAttributes>,
    ) -> Result<Option<TemplateResolution>, TemplateResolverError> {
        self.resolver.resolver().resolve_template(
            template,
            || {
                let resource_name = self.resolver.compute_resource_name(template);
                FileTemplateResource::new(
                    Some(&resource_name.to_string_lossy()),
                    self.resolver
                        .get_character_encoding()
                        .map(Utf16String::to_string_lossy)
                        .as_deref(),
                )
                .map(|resource| Some(Arc::new(resource) as Arc<dyn ITemplateResource>))
                .map_err(TemplateResolverError::from)
            },
            || self.resolver.compute_template_mode(template),
            || self.resolver.compute_validity(template),
        )
    }
}
