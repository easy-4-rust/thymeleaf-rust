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
