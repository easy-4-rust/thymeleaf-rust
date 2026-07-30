use std::sync::Arc;

use crate::TemplateResolutionAttributes;
use crate::cache::{AlwaysValidCacheEntryValidity, ICacheEntryValidity};
use crate::util::JavaString;
use crate::{
    IEngineConfiguration, ITemplateResource, StringTemplateResource, TemplateMode,
    TemplateModeParseError,
};

use super::{AbstractTemplateResolver, ITemplateResolver, TemplateResolution};

/// 无论输入模板名为何都返回同一段配置文本的解析器。
///
/// 对应 Java: `org.thymeleaf.templateresolver.DefaultTemplateResolver`。
pub struct DefaultTemplateResolver {
    resolver: AbstractTemplateResolver,
    template_mode: TemplateMode,
    template: Option<JavaString>,
}

impl DefaultTemplateResolver {
    /// 默认 HTML 模式。
    pub const DEFAULT_TEMPLATE_MODE: TemplateMode = TemplateMode::HTML;

    /// 创建正文为空字符串的默认解析器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            resolver: AbstractTemplateResolver::new(
                "org.thymeleaf.templateresolver.DefaultTemplateResolver",
            ),
            template_mode: Self::DEFAULT_TEMPLATE_MODE,
            template: Some(JavaString::from_rust_str("")),
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

    /// 使用文本设置模板模式。
    pub fn set_template_mode_name(
        &mut self,
        template_mode: Option<&str>,
    ) -> Result<(), TemplateModeParseError> {
        self.template_mode = TemplateMode::parse(template_mode)?;
        Ok(())
    }

    /// 返回固定模板正文；`None` 保留 Java setter 接受 null 的行为。
    #[must_use]
    pub fn get_template(&self) -> Option<&JavaString> {
        self.template.as_ref()
    }

    /// 设置固定模板正文。
    pub fn set_template(&mut self, template: Option<JavaString>) {
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
        self.resolver.resolve_template(
            template,
            || {
                let text = self.template.as_ref().map(JavaString::to_string_lossy);
                StringTemplateResource::new(text.as_deref())
                    .ok()
                    .map(|resource| Arc::new(resource) as Arc<dyn ITemplateResource>)
            },
            || self.template_mode,
            || Arc::new(AlwaysValidCacheEntryValidity::new()) as Arc<dyn ICacheEntryValidity>,
        )
    }
}
