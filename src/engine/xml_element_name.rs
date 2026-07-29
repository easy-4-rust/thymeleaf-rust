use crate::util::JavaString;

use super::{ElementName, ElementNameError, ElementNameKind};

/// XML 模式使用的大小写敏感元素名称。
///
/// 对应 Java: `org.thymeleaf.engine.XMLElementName`。
pub struct XMLElementName {
    element_name: ElementName,
    complete_namespaced_element_name: JavaString,
}

impl XMLElementName {
    pub(super) fn for_name(
        prefix: Option<JavaString>,
        element_name: Option<JavaString>,
    ) -> Result<Self, ElementNameError> {
        let raw_element_name = element_name
            .as_ref()
            .ok_or(ElementNameError::InvalidElementName)?;
        let has_prefix = prefix.as_ref().is_some_and(|value| !value.is_empty());
        let complete_namespaced_element_name = if has_prefix {
            let prefix_value = prefix.as_ref().expect("checked non-null prefix");
            let mut complete = prefix_value.as_utf16().to_vec();
            complete.push(u16::from(b':'));
            complete.extend_from_slice(raw_element_name.as_utf16());
            JavaString::from_utf16(complete)
        } else {
            raw_element_name.clone()
        };
        let base = ElementName::new(
            ElementNameKind::Xml,
            prefix,
            element_name,
            vec![Some(complete_namespaced_element_name.clone())],
        )?;
        Ok(Self {
            element_name: base,
            complete_namespaced_element_name,
        })
    }

    /// 返回基础 `ElementName` 视图。
    #[must_use]
    pub const fn as_element_name(&self) -> &ElementName {
        &self.element_name
    }

    /// 返回 `prefix:name` 或无 prefix 的原始 XML 元素名。
    #[must_use]
    pub const fn get_complete_namespaced_element_name(&self) -> &JavaString {
        &self.complete_namespaced_element_name
    }
}
