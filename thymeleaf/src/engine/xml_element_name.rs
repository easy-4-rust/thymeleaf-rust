use crate::util::Utf16String;

use super::{ElementName, ElementNameError, ElementNameKind};

/// XML 模式使用的大小写敏感元素名称。
///
/// 对应 Java: `org.thymeleaf.engine.XMLElementName`。
pub struct XMLElementName {
    element_name: ElementName,
    complete_namespaced_element_name: Utf16String,
}

impl XMLElementName {
    /// 对应 Java: `XMLElementName#forName()`。
    pub(super) fn for_name(
        prefix: Option<Utf16String>,
        element_name: Option<Utf16String>,
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
            Utf16String::from_utf16(complete)
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
    pub const fn get_complete_namespaced_element_name(&self) -> &Utf16String {
        &self.complete_namespaced_element_name
    }
}

#[cfg(test)]
mod tests {
    use crate::util::Utf16String;

    use super::super::element_names::ElementNames;

    #[test]
    fn xml_name_contract_matches_java_golden() {
        let prefixed = ElementNames::for_xml_name(Some(&Utf16String::from_rust_str("p:Code")))
            .expect("valid prefixed XML name");
        let same = ElementNames::for_xml_name(Some(&Utf16String::from_rust_str("p:Code")))
            .expect("valid same XML name");
        let different_case =
            ElementNames::for_xml_name(Some(&Utf16String::from_rust_str("p:code")))
                .expect("valid differently cased XML name");
        let bare = ElementNames::for_xml_name(Some(&Utf16String::from_rust_str("Code")))
            .expect("valid bare XML name");
        let actual = format!(
            "prefixed={}\nbare={}\nequalsSame={}\nequalsDifferentCase={}\nhashSame={}\n",
            describe(&prefixed),
            describe(&bare),
            prefixed.as_element_name() == same.as_element_name(),
            prefixed.as_element_name() == different_case.as_element_name(),
            prefixed.as_element_name().hash_code() == same.as_element_name().hash_code(),
        );
        assert_eq!(
            actual,
            include_str!("../../tests/fixtures/xml_element_name_golden.txt")
        );
    }

    fn describe(name: &super::XMLElementName) -> String {
        let base = name.as_element_name();
        let complete_names = base.get_complete_element_names();
        let complete = complete_names.read().expect("complete names lock");
        let complete = complete
            .iter()
            .map(|value| {
                value
                    .as_ref()
                    .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy)
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{},{},{},[{}],{},{}",
            base.get_element_name().to_string_lossy(),
            base.is_prefixed(),
            base.get_prefix()
                .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy),
            complete,
            base.to_utf16_string().expect("valid").to_string_lossy(),
            name.get_complete_namespaced_element_name()
                .to_string_lossy(),
        )
    }
}
