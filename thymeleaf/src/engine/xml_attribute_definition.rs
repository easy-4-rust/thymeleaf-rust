use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::element::ElementProcessorSet;

use super::{
    AttributeDefinition, AttributeDefinitionError, AttributeDefinitionKind, AttributeNameValue,
    XMLAttributeName,
};

/// XML 模式使用的大小写敏感属性定义。
///
/// 对应 Java: `org.thymeleaf.engine.XMLAttributeDefinition`。
pub struct XMLAttributeDefinition {
    attribute_definition: AttributeDefinition,
}

impl XMLAttributeDefinition {
    pub(crate) fn new(
        name: Arc<XMLAttributeName>,
        associated_processors: Arc<RwLock<ElementProcessorSet>>,
    ) -> Result<Self, AttributeDefinitionError> {
        Ok(Self {
            attribute_definition: AttributeDefinition::new(
                AttributeDefinitionKind::Xml,
                Some(AttributeNameValue::Xml(name)),
                Some(associated_processors),
            )?,
        })
    }

    /// 返回公共属性定义基类视图。
    #[must_use]
    pub const fn as_attribute_definition(&self) -> &AttributeDefinition {
        &self.attribute_definition
    }
}

impl Deref for XMLAttributeDefinition {
    type Target = AttributeDefinition;

    fn deref(&self) -> &Self::Target {
        &self.attribute_definition
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use crate::element::ElementProcessorSet;
    use crate::util::JavaString;

    use super::super::xml_attribute_name::XMLAttributeName;
    use super::XMLAttributeDefinition;

    #[test]
    fn inherited_attribute_definition_contract_matches_java_golden() {
        let first = definition(Some("p"), "code");
        let same = definition(Some("p"), "code");
        let different = definition(None, "code");

        let name = first
            .get_attribute_name()
            .as_attribute_name()
            .to_java_string()
            .expect("valid XML attribute name")
            .to_string_lossy();
        let actual = format!(
            "name={name}\nhasProcessors={}\nprocessorCount={}\nstring={name}\nequalsSelf={}\nequalsSame={}\nequalsDifferent={}\nhashSame={}\nhashDifferent={}\n",
            first.has_associated_processors(),
            first.get_associated_processors().len(),
            first.equals_java(&first).expect("self equality"),
            first.equals_java(&same).expect("same equality"),
            first.equals_java(&different).expect("different equality"),
            first.hash_code() == same.hash_code(),
            first.hash_code() == different.hash_code(),
        );
        assert_eq!(
            actual,
            include_str!("../../tests/fixtures/xml_attribute_definition_golden.txt")
        );
    }

    fn definition(prefix: Option<&str>, name: &str) -> XMLAttributeDefinition {
        let name = XMLAttributeName::for_name(
            prefix.map(JavaString::from_rust_str),
            Some(JavaString::from_rust_str(name)),
        )
        .expect("valid XML attribute name");
        XMLAttributeDefinition::new(
            Arc::new(name),
            Arc::new(RwLock::new(ElementProcessorSet::new())),
        )
        .expect("empty processor set is valid")
    }
}
