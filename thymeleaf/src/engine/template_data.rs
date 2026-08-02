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
    /// 对应 Java 语义：`TemplateData` 的 `new` 行为（Rust 侧辅助/私有路径）。
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
    /// 对应 Java: `TemplateData#getTemplateSelectors()`。
    pub fn get_template_selectors(&self) -> Option<&[JavaString]> {
        self.template_selectors.as_deref()
    }

    /// 返回可空模板资源。
    #[must_use]
    /// 对应 Java: `TemplateData#getTemplateResource()`。
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
    /// 对应 Java: `TemplateData#getValidity()`。
    pub fn get_validity(&self) -> Option<&dyn ICacheEntryValidity> {
        self.cache_validity.as_deref()
    }

    /// 返回可共享的缓存有效性身份。
    ///
    /// 对应 Java: `TemplateData#getValidity()` 返回同一对象引用。
    #[must_use]
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
            "templateResource" => self.get_template_resource_arc().map(|resource| {
                Arc::new(TemplateValue::Object(Arc::new(
                    TemplateResourceExpressionObject { resource },
                )))
            }),
            "validity" => self.get_validity_arc().map(|validity| {
                Arc::new(TemplateValue::Object(Arc::new(
                    CacheEntryValidityExpressionObject { validity },
                )))
            }),
            _ => return None,
        };
        Some(Ok(value))
    }
}

/// 将模板资源 trait object 适配为表达式中的 JavaBean 对象。
///
/// Java 的 `TemplateData#getTemplateResource()` 会保留接口对象，随后 OGNL 能继续读取
/// `description`、`baseName` 与 `exists`。Rust trait object 不能直接装入
/// `TemplateValue::Object`，因此该私有适配对象只转发 `ITemplateResource` 的公共合同。
struct TemplateResourceExpressionObject {
    resource: Arc<dyn ITemplateResource>,
}

impl TemplateObject for TemplateResourceExpressionObject {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.templateresource.ITemplateResource"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str(&self.resource.get_description())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        let value = match property_name.to_string_lossy().as_str() {
            "description" => Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
                &self.resource.get_description(),
            )))),
            "baseName" => self.resource.get_base_name().map(|base_name| {
                Arc::new(TemplateValue::string(JavaString::from_rust_str(&base_name)))
            }),
            "exists" => Some(Arc::new(TemplateValue::Boolean(self.resource.exists()))),
            _ => return None,
        };
        Some(Ok(value))
    }
}

/// 将缓存有效性 trait object 适配为表达式中的 JavaBean 对象。
///
/// 对应 Java `ICacheEntryValidity` 的两个 boolean getter，保证通过
/// `TemplateData#validity` 取得的对象仍可被嵌套表达式读取。
struct CacheEntryValidityExpressionObject {
    validity: Arc<dyn ICacheEntryValidity>,
}

impl TemplateObject for CacheEntryValidityExpressionObject {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.cache.ICacheEntryValidity"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str("org.thymeleaf.cache.ICacheEntryValidity")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        let value = match property_name.to_string_lossy().as_str() {
            "cacheable" => Some(Arc::new(TemplateValue::Boolean(
                self.validity.is_cacheable(),
            ))),
            "cacheStillValid" => Some(Arc::new(TemplateValue::Boolean(
                self.validity.is_cache_still_valid(),
            ))),
            _ => return None,
        };
        Some(Ok(value))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::TemplateData;
    use crate::TemplateMode;
    use crate::cache::AlwaysValidCacheEntryValidity;
    use crate::expression::{TemplateObject, TemplateValue};
    use crate::templateresource::{ITemplateResource, StringTemplateResource};
    use crate::util::JavaString;

    #[test]
    fn constructor_preserves_nullable_metadata_like_java() {
        let data = TemplateData::new(
            Some(JavaString::from_rust_str("template")),
            None,
            None,
            Some(TemplateMode::HTML),
            None,
        );
        let nullable = |value: Option<String>| value.unwrap_or_else(|| "null".to_owned());
        let output = format!(
            "{},{},{},{},{},{}",
            data.get_template()
                .map(JavaString::to_string_lossy)
                .expect("template"),
            data.has_template_selectors(),
            nullable(
                data.get_template_selectors()
                    .map(|value| format!("{value:?}"))
            ),
            nullable(
                data.get_template_resource()
                    .map(|value| value.get_description())
            ),
            data.get_template_mode().expect("mode"),
            nullable(
                data.get_validity()
                    .map(|value| value.is_cacheable().to_string())
            ),
        );
        let expected = include_str!("../../tests/fixtures/model_golden.txt")
            .lines()
            .find_map(|line| line.strip_prefix("templateData="))
            .expect("Java Golden record");
        assert_eq!(output, expected);

        let resource: Arc<dyn ITemplateResource> =
            Arc::new(StringTemplateResource::new(Some("contents")).expect("resource"));
        let validity: Arc<dyn crate::cache::ICacheEntryValidity> =
            Arc::new(AlwaysValidCacheEntryValidity::new());
        let full = TemplateData::new(
            Some(JavaString::from_rust_str("full")),
            Some(vec![
                JavaString::from_rust_str("second"),
                JavaString::from_rust_str("first"),
            ]),
            Some(Arc::clone(&resource)),
            Some(TemplateMode::XML),
            Some(Arc::clone(&validity)),
        );
        let full_output = format!(
            "{},[{}, {}],{},{},{},{}",
            full.get_template().expect("template").to_string_lossy(),
            full.get_template_selectors().expect("selectors")[0].to_string_lossy(),
            full.get_template_selectors().expect("selectors")[1].to_string_lossy(),
            std::ptr::eq(
                full.get_template_resource().expect("resource"),
                resource.as_ref()
            ),
            full.get_template_resource()
                .expect("resource")
                .get_description(),
            full.get_template_mode().expect("mode"),
            std::ptr::eq(full.get_validity().expect("validity"), validity.as_ref()),
        );
        let full_expected = include_str!("../../tests/fixtures/model_golden.txt")
            .lines()
            .find_map(|line| line.strip_prefix("templateDataFull="))
            .expect("Java Golden record");
        assert_eq!(full_output, full_expected);

        assert_property(&full, "template", Some("full"));
        assert_property(&full, "templateSelectors", Some("[second, first]"));
        assert_property(&full, "templateMode", Some("XML"));
        let template_resource = object_property(&full, "templateResource");
        assert_object_property(template_resource.as_ref(), "description", Some("contents"));
        assert_object_property(template_resource.as_ref(), "baseName", None);
        assert_object_property(template_resource.as_ref(), "exists", Some("true"));
        let validity = object_property(&full, "validity");
        assert_object_property(validity.as_ref(), "cacheable", Some("true"));
        assert_object_property(validity.as_ref(), "cacheStillValid", Some("true"));
    }

    fn assert_property(data: &TemplateData, property: &str, expected: Option<&str>) {
        let actual = data
            .java_get_property(&JavaString::from_rust_str(property))
            .expect("TemplateData JavaBean property")
            .expect("TemplateData getter must not fail")
            .and_then(|value| value.to_java_string())
            .map(|value| value.to_string_lossy());
        assert_eq!(actual.as_deref(), expected, "property {property}");
    }

    fn object_property(data: &TemplateData, property: &str) -> Arc<dyn TemplateObject> {
        let value = data
            .java_get_property(&JavaString::from_rust_str(property))
            .expect("TemplateData JavaBean property")
            .expect("TemplateData getter must not fail")
            .expect("non-null Java object");
        let TemplateValue::Object(value) = value.as_ref() else {
            panic!("property {property} must preserve a Java object");
        };
        Arc::clone(value)
    }

    fn assert_object_property(object: &dyn TemplateObject, property: &str, expected: Option<&str>) {
        let actual = object
            .java_get_property(&JavaString::from_rust_str(property))
            .expect("nested JavaBean property")
            .expect("nested getter must not fail")
            .and_then(|value| value.to_java_string())
            .map(|value| value.to_string_lossy());
        assert_eq!(actual.as_deref(), expected, "nested property {property}");
    }
}
