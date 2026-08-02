use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::element::ElementProcessorSet;

use super::{
    AttributeDefinition, AttributeDefinitionError, AttributeDefinitionKind, AttributeNameValue,
    HTMLAttributeName,
};

/// HTML 属性定义，额外记录该属性是否属于布尔属性。
///
/// 对应 Java: `org.thymeleaf.engine.HTMLAttributeDefinition`。
pub struct HTMLAttributeDefinition {
    attribute_definition: AttributeDefinition,
    boolean_attribute: bool,
}

impl HTMLAttributeDefinition {
    /// 对应 Java 语义：`HTMLAttributeDefinition` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(
        name: Arc<HTMLAttributeName>,
        boolean_attribute: bool,
        associated_processors: Arc<RwLock<ElementProcessorSet>>,
    ) -> Result<Self, AttributeDefinitionError> {
        Ok(Self {
            attribute_definition: AttributeDefinition::new(
                AttributeDefinitionKind::Html,
                Some(AttributeNameValue::Html(name)),
                Some(associated_processors),
            )?,
            boolean_attribute,
        })
    }

    /// 判断该 HTML 属性是否采用布尔属性语义。
    ///
    /// 对应 Java: `HTMLAttributeDefinition#isBooleanAttribute()`。
    #[must_use]
    pub const fn is_boolean_attribute(&self) -> bool {
        self.boolean_attribute
    }

    /// 返回公共属性定义基类视图。
    #[must_use]
    pub const fn as_attribute_definition(&self) -> &AttributeDefinition {
        &self.attribute_definition
    }
}

impl Deref for HTMLAttributeDefinition {
    type Target = AttributeDefinition;

    fn deref(&self) -> &Self::Target {
        &self.attribute_definition
    }
}
