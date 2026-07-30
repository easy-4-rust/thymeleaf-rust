use std::sync::Arc;

use crate::TemplateResolutionAttributes;
use crate::util::JavaString;
use crate::{FileTemplateResource, IEngineConfiguration, ITemplateResource};

use super::{AbstractConfigurableTemplateResolver, ITemplateResolver, TemplateResolution};

/// 从文件系统解析模板资源的可配置解析器。
///
/// 对应 Java: `org.thymeleaf.templateresolver.FileTemplateResolver`。
pub struct FileTemplateResolver {
    resolver: AbstractConfigurableTemplateResolver,
}

impl FileTemplateResolver {
    /// 创建使用标准可配置解析器默认值的文件解析器。
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
                FileTemplateResource::new(
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
            || self.resolver.compute_validity(template),
        )
    }
}
