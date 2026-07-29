use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::element::ElementProcessorSet;

use super::{
    ElementDefinition, ElementDefinitionError, ElementDefinitionKind, ElementNameValue,
    XMLElementName,
};

/// XML 模式使用的大小写敏感元素定义。
///
/// 对应 Java: `org.thymeleaf.engine.XMLElementDefinition`。
pub struct XMLElementDefinition {
    element_definition: ElementDefinition,
}

impl XMLElementDefinition {
    pub(crate) fn new(
        name: Arc<XMLElementName>,
        associated_processors: Arc<RwLock<ElementProcessorSet>>,
    ) -> Result<Self, ElementDefinitionError> {
        Ok(Self {
            element_definition: ElementDefinition::new(
                ElementDefinitionKind::Xml,
                Some(ElementNameValue::Xml(name)),
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

impl Deref for XMLElementDefinition {
    type Target = ElementDefinition;

    fn deref(&self) -> &Self::Target {
        &self.element_definition
    }
}
