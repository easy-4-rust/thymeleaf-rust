use std::any::Any;
use std::sync::Arc;

use crate::cache::ICacheEntryValidity;
use crate::expression::{TemplateObject, TemplateObjectPropertyError, TemplateValue};
use crate::templatemode::TemplateMode;
use crate::templateresource::ITemplateResource;
use crate::util::JavaString;

/// 当前处理模板的名称、选择器、资源、模式和缓存有效性元数据。
///
/// 对应 Java: `org.thymeleaf.engine.TemplateData`。
///
/// 构造器与上游一致，不执行任何校验或转换。
#[derive(Clone)]
pub struct TemplateData {
    template: Option<JavaString>,
    template_selectors: Option<Vec<JavaString>>,
    template_resource: Option<Arc<dyn ITemplateResource>>,
    template_mode: Option<TemplateMode>,
    cache_validity: Option<Arc<dyn ICacheEntryValidity>>,
}

impl TemplateData {
    /// 原样保存五个构造参数。
    #[must_use]
    pub fn new(
        template: Option<JavaString>,
        template_selectors: Option<Vec<JavaString>>,
        template_resource: Option<Arc<dyn ITemplateResource>>,
        template_mode: Option<TemplateMode>,
        cache_validity: Option<Arc<dyn ICacheEntryValidity>>,
    ) -> Self {
        Self {
            template,
            template_selectors,
            template_resource,
            template_mode,
            cache_validity,
        }
    }

    /// 返回可空模板名。
    #[must_use]
    pub const fn get_template(&self) -> Option<&JavaString> {
        self.template.as_ref()
    }

    /// 仅根据选择器集合是否为 null 判断。
    #[must_use]
    pub const fn has_template_selectors(&self) -> bool {
        self.template_selectors.is_some()
    }

    /// 返回可空、保持原顺序的模板选择器。
    #[must_use]
    pub fn get_template_selectors(&self) -> Option<&[JavaString]> {
        self.template_selectors.as_deref()
    }

    /// 返回可空模板资源。
    #[must_use]
    pub fn get_template_resource(&self) -> Option<&dyn ITemplateResource> {
        self.template_resource.as_deref()
    }

    /// 返回可共享的模板资源身份。
    ///
    /// 对应 Java: `TemplateData#getTemplateResource()` 返回同一对象引用。
    #[must_use]
    pub(crate) fn get_template_resource_arc(&self) -> Option<Arc<dyn ITemplateResource>> {
        self.template_resource.clone()
    }

    /// 返回可空模板模式。
    #[must_use]
    pub const fn get_template_mode(&self) -> Option<TemplateMode> {
        self.template_mode
    }

    /// 返回可空缓存有效性对象。
    #[must_use]
    pub fn get_validity(&self) -> Option<&dyn ICacheEntryValidity> {
        self.cache_validity.as_deref()
    }

    /// 返回可共享的缓存有效性身份。
    ///
    /// 对应 Java: `TemplateData#getValidity()` 返回同一对象引用。
    #[must_use]
    #[expect(dead_code, reason = "保留 Java 缓存有效性对象的共享身份入口")]
    pub(crate) fn get_validity_arc(&self) -> Option<Arc<dyn ICacheEntryValidity>> {
        self.cache_validity.clone()
    }
}

impl TemplateObject for TemplateData {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.engine.TemplateData"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str("org.thymeleaf.engine.TemplateData")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        let value = match property_name.to_string_lossy().as_str() {
            "template" => self
                .get_template()
                .cloned()
                .map(TemplateValue::string)
                .map(Arc::new),
            "templateSelectors" => self.get_template_selectors().map(|selectors| {
                Arc::new(TemplateValue::List(Arc::new(
                    selectors
                        .iter()
                        .cloned()
                        .map(TemplateValue::string)
                        .map(Arc::new)
                        .collect(),
                )))
            }),
            "templateMode" => self
                .get_template_mode()
                .map(|mode| Arc::new(TemplateValue::Object(Arc::new(mode)))),
            _ => return None,
        };
        Some(Ok(value))
    }
}
