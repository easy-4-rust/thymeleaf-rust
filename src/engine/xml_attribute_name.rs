use crate::util::JavaString;

use super::{AttributeName, AttributeNameError, AttributeNameKind};

/// XML 模式使用的大小写敏感属性名称。
///
/// 对应 Java: `org.thymeleaf.engine.XMLAttributeName`。
pub struct XMLAttributeName {
    attribute_name: AttributeName,
    complete_namespaced_attribute_name: JavaString,
}

impl XMLAttributeName {
    pub(super) fn for_name(
        prefix: Option<JavaString>,
        attribute_name: Option<JavaString>,
    ) -> Result<Self, AttributeNameError> {
        let raw_name = attribute_name
            .as_ref()
            .ok_or(AttributeNameError::InvalidAttributeName)?;
        let complete = complete_namespaced(prefix.as_ref(), raw_name);
        let base = AttributeName::new(
            AttributeNameKind::Xml,
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

    /// 返回 `prefix:name` 或无 prefix 的原始 XML 属性名。
    #[must_use]
    pub const fn get_complete_namespaced_attribute_name(&self) -> &JavaString {
        &self.complete_namespaced_attribute_name
    }
}

fn complete_namespaced(prefix: Option<&JavaString>, attribute_name: &JavaString) -> JavaString {
    let Some(prefix) = prefix.filter(|value| !value.is_empty()) else {
        return attribute_name.clone();
    };
    let mut result = prefix.as_utf16().to_vec();
    result.push(u16::from(b':'));
    result.extend_from_slice(attribute_name.as_utf16());
    JavaString::from_utf16(result)
}
