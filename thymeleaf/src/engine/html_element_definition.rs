use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::element::ElementProcessorSet;

use super::{
    ElementDefinition, ElementDefinitionError, ElementDefinitionKind, ElementNameValue,
    HTMLElementName, HTMLElementType,
};

/// HTML 元素定义，包含解析器所需的元素类别。
///
/// 对应 Java: `org.thymeleaf.engine.HTMLElementDefinition`。
pub struct HTMLElementDefinition {
    element_definition: ElementDefinition,
    element_type: HTMLElementType,
}

impl HTMLElementDefinition {
    pub(crate) fn new(
        name: Arc<HTMLElementName>,
        element_type: HTMLElementType,
        associated_processors: Arc<RwLock<ElementProcessorSet>>,
    ) -> Result<Self, ElementDefinitionError> {
        Ok(Self {
            element_definition: ElementDefinition::new(
                ElementDefinitionKind::Html,
                Some(ElementNameValue::Html(name)),
                Some(associated_processors),
            )?,
            element_type,
        })
    }

    /// 返回 HTML 元素类别。
    ///
    /// 对应 Java: `HTMLElementDefinition#getType()`。
    #[must_use]
    pub const fn get_type(&self) -> HTMLElementType {
        self.element_type
    }

    /// 返回公共元素定义基类视图。
    #[must_use]
    pub const fn as_element_definition(&self) -> &ElementDefinition {
        &self.element_definition
    }
}

impl Deref for HTMLElementDefinition {
    type Target = ElementDefinition;

    fn deref(&self) -> &Self::Target {
        &self.element_definition
    }
}
