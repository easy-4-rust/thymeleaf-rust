use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::element::ElementProcessorSet;

use super::{
    AttributeDefinition, AttributeDefinitionError, AttributeDefinitionKind, AttributeNameValue,
    TextAttributeName,
};

/// TEXT、JAVASCRIPT 与 CSS 模式使用的属性定义。
///
/// 对应 Java: `org.thymeleaf.engine.TextAttributeDefinition`。
pub struct TextAttributeDefinition {
    attribute_definition: AttributeDefinition,
}

impl TextAttributeDefinition {
    /// 对应 Java 语义：`TextAttributeDefinition` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(
        name: Arc<TextAttributeName>,
        associated_processors: Arc<RwLock<ElementProcessorSet>>,
    ) -> Result<Self, AttributeDefinitionError> {
        Ok(Self {
            attribute_definition: AttributeDefinition::new(
                AttributeDefinitionKind::Text,
                Some(AttributeNameValue::Text(name)),
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

impl Deref for TextAttributeDefinition {
    type Target = AttributeDefinition;

    fn deref(&self) -> &Self::Target {
        &self.attribute_definition
    }
}
