use std::sync::Arc;

use crate::TemplateResolutionAttributes;
use crate::cache::{AlwaysValidCacheEntryValidity, ICacheEntryValidity};
use crate::util::Utf16String;
use crate::{
    IEngineConfiguration, ITemplateResource, StringTemplateResource, TemplateMode,
    TemplateModeParseError,
};

use super::{
    AbstractTemplateResolver, ITemplateResolver, TemplateResolution, TemplateResolverError,
};

/// 无论输入模板名为何都返回同一段配置文本的解析器。
///
/// 对应 Java: `org.thymeleaf.templateresolver.DefaultTemplateResolver`。
pub struct DefaultTemplateResolver {
    resolver: AbstractTemplateResolver,
    template_mode: TemplateMode,
    template: Option<Utf16String>,
}

impl DefaultTemplateResolver {
    /// 默认 HTML 模式。
    pub const DEFAULT_TEMPLATE_MODE: TemplateMode = TemplateMode::HTML;

    /// 创建正文为空字符串的默认解析器。
    ///
    /// 对应 Java: `DefaultTemplateResolver#DefaultTemplateResolver()`。
    #[must_use]
    pub fn new() -> Self {
        Self {
            resolver: AbstractTemplateResolver::new(
                "org.thymeleaf.templateresolver.DefaultTemplateResolver",
            ),
            template_mode: Self::DEFAULT_TEMPLATE_MODE,
            template: Some(Utf16String::from_rust_str("")),
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
    /// 对应 Java: `DefaultTemplateResolver#setTemplateMode(TemplateMode)`。
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

    /// 使用文本设置模板模式。
    ///
    /// 对应 Java: `DefaultTemplateResolver#setTemplateMode(String)`。
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

    /// 返回固定模板正文；`None` 保留 Java setter 接受 null 的行为。
    ///
    /// 对应 Java: `DefaultTemplateResolver#getTemplate()`。
    #[must_use]
    pub fn get_template(&self) -> Option<&Utf16String> {
        self.template.as_ref()
    }

    /// 设置固定模板正文。
    ///
    /// 对应 Java: `DefaultTemplateResolver#setTemplate(String)`。
    pub fn set_template(&mut self, template: Option<Utf16String>) {
        self.template = template;
    }
}

impl Default for DefaultTemplateResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for DefaultTemplateResolver {
    type Target = AbstractTemplateResolver;
    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl std::ops::DerefMut for DefaultTemplateResolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}

impl ITemplateResolver for DefaultTemplateResolver {
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
        self.resolver.resolve_template(
            template,
            || {
                let text = self.template.as_ref().map(Utf16String::to_string_lossy);
                StringTemplateResource::new(text.as_deref())
                    .map(|resource| Some(Arc::new(resource) as Arc<dyn ITemplateResource>))
                    .map_err(TemplateResolverError::from)
            },
            || self.template_mode,
            || Arc::new(AlwaysValidCacheEntryValidity::new()) as Arc<dyn ICacheEntryValidity>,
        )
    }
}
