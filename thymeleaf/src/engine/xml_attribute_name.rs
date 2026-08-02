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
    /// 对应 Java: `XMLAttributeName#forName()`。
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

#[cfg(test)]
mod tests {
    use crate::util::JavaString;

    use super::super::attribute_names::AttributeNames;

    #[test]
    fn xml_name_contract_matches_java_golden() {
        let prefixed = AttributeNames::for_xml_name(Some(&JavaString::from_rust_str("p:Code")))
            .expect("valid prefixed XML name");
        let same = AttributeNames::for_xml_name(Some(&JavaString::from_rust_str("p:Code")))
            .expect("valid same XML name");
        let different_case =
            AttributeNames::for_xml_name(Some(&JavaString::from_rust_str("p:code")))
                .expect("valid differently cased XML name");
        let bare = AttributeNames::for_xml_name(Some(&JavaString::from_rust_str("Code")))
            .expect("valid bare XML name");

        let actual = format!(
            "prefixed={}\nbare={}\nequalsSame={}\nequalsDifferentCase={}\nhashSame={}\n",
            describe(&prefixed),
            describe(&bare),
            prefixed
                .as_attribute_name()
                .equals_java(same.as_attribute_name())
                .expect("same equality"),
            prefixed
                .as_attribute_name()
                .equals_java(different_case.as_attribute_name())
                .expect("different case equality"),
            prefixed.as_attribute_name().hash_code() == same.as_attribute_name().hash_code(),
        );
        assert_eq!(
            actual,
            include_str!("../../tests/fixtures/xml_attribute_name_golden.txt")
        );
    }

    fn describe(name: &super::XMLAttributeName) -> String {
        let base = name.as_attribute_name();
        let complete_names = base.get_complete_attribute_names();
        let complete = complete_names
            .read()
            .expect("complete attribute names lock");
        let complete = complete
            .iter()
            .map(|value| {
                value
                    .as_ref()
                    .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy)
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{},{},{},[{}],{},{}",
            base.get_attribute_name().to_string_lossy(),
            base.is_prefixed(),
            base.get_prefix()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
            complete,
            base.to_java_string()
                .expect("valid XML name")
                .to_string_lossy(),
            name.get_complete_namespaced_attribute_name()
                .to_string_lossy(),
        )
    }
}
