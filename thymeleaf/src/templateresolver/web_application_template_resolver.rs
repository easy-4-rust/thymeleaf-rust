use std::sync::Arc;

use crate::TemplateResolutionAttributes;
use crate::util::JavaString;
use crate::web::IWebApplication;
use crate::{IEngineConfiguration, ITemplateResource, WebApplicationTemplateResource};

use super::{
    AbstractConfigurableTemplateResolver, ITemplateResolver, TemplateResolution,
    TemplateResolverError,
};

/// 从 Web 应用根目录解析模板的可配置 Resolver。
///
/// 对应 Java: `org.thymeleaf.templateresolver.WebApplicationTemplateResolver`。
pub struct WebApplicationTemplateResolver {
    resolver: AbstractConfigurableTemplateResolver,
    web_application: Arc<dyn IWebApplication>,
}

impl WebApplicationTemplateResolver {
    /// 创建绑定到指定 Web 应用的 Resolver。
    ///
    /// 对应 Java:
    /// `WebApplicationTemplateResolver#WebApplicationTemplateResolver(IWebApplication)`。
    #[must_use]
    pub fn new(web_application: Arc<dyn IWebApplication>) -> Self {
        Self {
            resolver: AbstractConfigurableTemplateResolver::new(
                "org.thymeleaf.templateresolver.WebApplicationTemplateResolver",
            ),
            web_application,
        }
    }

    /// 使用可空宿主应用创建 Resolver。
    ///
    /// 该入口用于保留 Java 构造器的空值校验顺序；正常 Rust 调用可使用 [`Self::new`]。
    ///
    /// # 参数
    /// - `web_application`：Web 应用对象；`None` 对应 Java `null`。
    ///
    /// # 返回值
    /// 返回绑定非空 Web 应用的 Resolver。
    ///
    /// # 错误
    /// 应用缺失时返回与 Java 构造器一致的参数错误。
    pub fn try_new(
        web_application: Option<Arc<dyn IWebApplication>>,
    ) -> Result<Self, TemplateResolverError> {
        web_application.map(Self::new).ok_or_else(|| {
            TemplateResolverError::InvalidArgument(
                "Web Application object cannot be null".to_owned(),
            )
        })
    }
}

impl std::ops::Deref for WebApplicationTemplateResolver {
    type Target = AbstractConfigurableTemplateResolver;

    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl std::ops::DerefMut for WebApplicationTemplateResolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}

impl ITemplateResolver for WebApplicationTemplateResolver {
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
        self.resolver.resolver().resolve_template(
            template,
            || {
                let resource_name = self.resolver.compute_resource_name(template);
                WebApplicationTemplateResource::new(
                    Some(Arc::clone(&self.web_application)),
                    Some(&resource_name.to_string_lossy()),
                    self.resolver
                        .get_character_encoding()
                        .map(JavaString::to_string_lossy)
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
