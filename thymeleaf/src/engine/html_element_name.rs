use crate::util::JavaString;
use crate::util::java_string_case_utils::to_lower_case_default;

use super::{ElementName, ElementNameError, ElementNameKind};

/// HTML 模式的大小写不敏感元素名称。
///
/// 对应 Java: `org.thymeleaf.engine.HTMLElementName`。
pub struct HTMLElementName {
    element_name: ElementName,
    complete_namespaced_element_name: JavaString,
    complete_html5_element_name: JavaString,
}

impl HTMLElementName {
    /// 对应 Java: `HTMLElementName#forName()`。
    pub(super) fn for_name(
        prefix: Option<JavaString>,
        element_name: Option<JavaString>,
    ) -> Result<Self, ElementNameError> {
        let normalized_element = element_name.filter(|value| !value.is_empty()).map_or_else(
            || JavaString::from_utf16(Vec::new()),
            |value| to_lower_case_default(&value),
        );
        let normalized_prefix = prefix
            .as_ref()
            .filter(|value| !value.is_empty())
            .map(to_lower_case_default);
        let (namespaced, html5, complete_names) = if let Some(prefix) = normalized_prefix.as_ref() {
            let namespaced = join(prefix, b':', &normalized_element, None);
            let html5 = join(prefix, b'-', &normalized_element, None);
            (
                namespaced.clone(),
                html5.clone(),
                vec![Some(namespaced), Some(html5)],
            )
        } else {
            (
                normalized_element.clone(),
                normalized_element.clone(),
                vec![Some(normalized_element.clone())],
            )
        };
        let base = ElementName::new(
            ElementNameKind::Html,
            normalized_prefix,
            Some(normalized_element),
            complete_names,
        )?;
        Ok(Self {
            element_name: base,
            complete_namespaced_element_name: namespaced,
            complete_html5_element_name: html5,
        })
    }

    /// 返回基础 `ElementName` 视图。
    #[must_use]
    pub const fn as_element_name(&self) -> &ElementName {
        &self.element_name
    }

    /// 返回 `prefix:name` 形式的完整名称。
    #[must_use]
    pub const fn get_complete_namespaced_element_name(&self) -> &JavaString {
        &self.complete_namespaced_element_name
    }

    /// 返回 `prefix-name` 形式的 HTML5 完整名称。
    #[must_use]
    pub const fn get_complete_html5_element_name(&self) -> &JavaString {
        &self.complete_html5_element_name
    }
}

fn join(
    prefix: &JavaString,
    separator: u8,
    name: &JavaString,
    leading: Option<&str>,
) -> JavaString {
    let mut result = leading.map_or_else(Vec::new, |value| value.encode_utf16().collect());
    result.extend_from_slice(prefix.as_utf16());
    result.push(u16::from(separator));
    result.extend_from_slice(name.as_utf16());
    JavaString::from_utf16(result)
}
