use crate::util::Utf16String;

use super::{AttributeName, AttributeNameError, AttributeNameKind};

/// TEXT、JAVASCRIPT 与 CSS 模式共用的属性名称。
///
/// 对应 Java: `org.thymeleaf.engine.TextAttributeName`。
pub struct TextAttributeName {
    attribute_name: AttributeName,
    complete_namespaced_attribute_name: Utf16String,
}

impl TextAttributeName {
    /// 对应 Java: `TextAttributeName#forName()`。
    pub(super) fn for_name(
        prefix: Option<Utf16String>,
        attribute_name: Option<Utf16String>,
    ) -> Result<Self, AttributeNameError> {
        let raw_name = attribute_name
            .as_ref()
            .ok_or(AttributeNameError::InvalidAttributeName)?;
        let complete = complete_namespaced(prefix.as_ref(), raw_name);
        let base = AttributeName::new(
            AttributeNameKind::Text,
            prefix,
            attribute_name,
            vec![Some(complete.clone())],
        )?;
        Ok(Self {
            attribute_name: base,
            complete_namespaced_attribute_name: complete,
        })
    }

    /// 返回基础 `AttributeName` 视图。
    #[must_use]
    pub const fn as_attribute_name(&self) -> &AttributeName {
        &self.attribute_name
    }

    /// 返回 `prefix:name` 或无 prefix 的原始属性名。
    #[must_use]
    pub const fn get_complete_namespaced_attribute_name(&self) -> &Utf16String {
        &self.complete_namespaced_attribute_name
    }
}

fn complete_namespaced(prefix: Option<&Utf16String>, attribute_name: &Utf16String) -> Utf16String {
    let Some(prefix) = prefix.filter(|value| !value.is_empty()) else {
        return attribute_name.clone();
    };
    let mut result = prefix.as_utf16().to_vec();
    result.push(u16::from(b':'));
    result.extend_from_slice(attribute_name.as_utf16());
    Utf16String::from_utf16(result)
}
