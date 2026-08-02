use crate::util::JavaString;
use crate::util::java_string_case_utils::to_lower_case_default;

use super::{AttributeName, AttributeNameError, AttributeNameKind};

/// HTML 模式的大小写不敏感属性名称。
///
/// 对应 Java: `org.thymeleaf.engine.HTMLAttributeName`。
pub struct HTMLAttributeName {
    attribute_name: AttributeName,
    complete_namespaced_attribute_name: JavaString,
    complete_html5_attribute_name: JavaString,
}

impl HTMLAttributeName {
    pub(super) fn for_name(
        prefix: Option<JavaString>,
        attribute_name: Option<JavaString>,
    ) -> Result<Self, AttributeNameError> {
        let normalized_attribute = attribute_name
            .filter(|value| !value.is_empty())
            .map(|value| to_lower_case_default(&value));
        let normalized_prefix = prefix
            .as_ref()
            .filter(|value| !value.is_empty())
            .map(to_lower_case_default);
        let raw_attribute = normalized_attribute
            .as_ref()
            .ok_or(AttributeNameError::InvalidAttributeName)?;
        let (namespaced, html5, complete_names) = if let Some(prefix) = normalized_prefix.as_ref() {
            let namespaced = join(prefix, b':', raw_attribute, None);
            let html5 = join(prefix, b'-', raw_attribute, Some("data-"));
            (
                namespaced.clone(),
                html5.clone(),
                vec![Some(namespaced), Some(html5)],
            )
        } else {
            (
                raw_attribute.clone(),
                raw_attribute.clone(),
                vec![Some(raw_attribute.clone())],
            )
        };
        let base = AttributeName::new(
            AttributeNameKind::Html,
            normalized_prefix,
            normalized_attribute,
            complete_names,
        )?;
        Ok(Self {
            attribute_name: base,
            complete_namespaced_attribute_name: namespaced,
            complete_html5_attribute_name: html5,
        })
    }

    /// 返回基础 `AttributeName` 视图。
    #[must_use]
    pub const fn as_attribute_name(&self) -> &AttributeName {
        &self.attribute_name
    }

    /// 返回 `prefix:name` 形式的完整属性名。
    #[must_use]
    pub const fn get_complete_namespaced_attribute_name(&self) -> &JavaString {
        &self.complete_namespaced_attribute_name
    }

    /// 返回 `data-prefix-name` 形式的 HTML5 属性名。
    #[must_use]
    pub const fn get_complete_html5_attribute_name(&self) -> &JavaString {
        &self.complete_html5_attribute_name
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
