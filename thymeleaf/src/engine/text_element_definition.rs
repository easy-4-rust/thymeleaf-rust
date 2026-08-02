use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::element::ElementProcessorSet;

use super::{
    ElementDefinition, ElementDefinitionError, ElementDefinitionKind, ElementNameValue,
    TextElementName,
};

/// TEXT、JAVASCRIPT 与 CSS 模式使用的元素定义。
///
/// 对应 Java: `org.thymeleaf.engine.TextElementDefinition`。
pub struct TextElementDefinition {
    element_definition: ElementDefinition,
}

impl TextElementDefinition {
    /// 对应 Java 语义：`TextElementDefinition` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(
        name: Arc<TextElementName>,
        associated_processors: Arc<RwLock<ElementProcessorSet>>,
    ) -> Result<Self, ElementDefinitionError> {
        Ok(Self {
            element_definition: ElementDefinition::new(
                ElementDefinitionKind::Text,
                Some(ElementNameValue::Text(name)),
                Some(associated_processors),
            )?,
        })
    }

    /// 返回公共元素定义基类视图。
    #[must_use]
    pub const fn as_element_definition(&self) -> &ElementDefinition {
        &self.element_definition
    }
}

impl Deref for TextElementDefinition {
    type Target = ElementDefinition;

    fn deref(&self) -> &Self::Target {
        &self.element_definition
    }
}
