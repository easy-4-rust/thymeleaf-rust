use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};

use crate::element::{ElementProcessorSet, IElementProcessor, UnmodifiableElementProcessorSet};
use crate::util::JavaString;

use super::{ElementNameError, ElementNameValue};

/// `ElementDefinition` 的具体 Java 子类标识。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 对应 Java 语义：`ElementDefinition` 的 Rust 侧类型 `ElementDefinitionKind`。
pub enum ElementDefinitionKind {
    /// `HTMLElementDefinition`。
    Html,
    /// `XMLElementDefinition`。
    Xml,
    /// `TextElementDefinition`。
    Text,
}

/// 元素定义构造和显示错误。
#[derive(Clone, Debug, Eq, PartialEq)]
/// 对应 Java 语义：`ElementDefinition` 的 Rust 侧类型 `ElementDefinitionError`。
pub enum ElementDefinitionError {
    /// 元素名为 null。
    NullElementName,
    /// Processor Set 为 null。
    NullAssociatedProcessors,
    /// Processor Set 包含 null。
    NullAssociatedProcessor,
    /// 元素名的 complete names 数组被外部破坏。
    ElementName(ElementNameError),
}

impl ElementDefinitionError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::NullElementName | Self::NullAssociatedProcessors => {
                "java.lang.IllegalArgumentException"
            }
            Self::NullAssociatedProcessor => "java.lang.NullPointerException",
            Self::ElementName(error) => error.java_class_name(),
        }
    }
}

impl Display for ElementDefinitionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullElementName => formatter.write_str("Element name cannot be null"),
            Self::NullAssociatedProcessors => {
                formatter.write_str("Associated processors cannot be null")
            }
            Self::NullAssociatedProcessor => {
                formatter.write_str("processor comparator received null")
            }
            Self::ElementName(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for ElementDefinitionError {}

/// 元素名及其关联元素 Processor 的不可变运行时定义。
///
/// 对应 Java: `org.thymeleaf.engine.ElementDefinition`。
pub struct ElementDefinition {
    kind: ElementDefinitionKind,
    element_name: ElementNameValue,
    associated_processors_set: UnmodifiableElementProcessorSet,
    associated_processors: Vec<Arc<dyn IElementProcessor>>,
    has_associated_processors: bool,
}

impl ElementDefinition {
    /// 对应 Java 语义：`ElementDefinition` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(super) fn new(
        kind: ElementDefinitionKind,
        element_name: Option<ElementNameValue>,
        associated_processors: Option<Arc<RwLock<ElementProcessorSet>>>,
    ) -> Result<Self, ElementDefinitionError> {
        let element_name = element_name.ok_or(ElementDefinitionError::NullElementName)?;
        let associated_processors =
            associated_processors.ok_or(ElementDefinitionError::NullAssociatedProcessors)?;
        let mut sorted = crate::element::read_set(&associated_processors)
            .iter()
            .map(|value| {
                value
                    .cloned()
                    .ok_or(ElementDefinitionError::NullAssociatedProcessor)
            })
            .collect::<Result<Vec<_>, _>>()?;
        sorted.sort_by(compare_processors);
        let has_associated_processors = !sorted.is_empty();
        Ok(Self {
            kind,
            element_name,
            associated_processors_set: UnmodifiableElementProcessorSet::new(associated_processors),
            associated_processors: sorted,
            has_associated_processors,
        })
    }

    /// 返回定义对应的元素名。
    #[must_use]
    pub const fn get_element_name(&self) -> &ElementNameValue {
        &self.element_name
    }

    /// 判断构造时排序快照是否含 Processor。
    #[must_use]
    pub const fn has_associated_processors(&self) -> bool {
        self.has_associated_processors
    }

    /// 返回对原 Processor Set 的实时不可修改视图。
    #[must_use]
    pub const fn get_associated_processors(&self) -> &UnmodifiableElementProcessorSet {
        &self.associated_processors_set
    }

    /// 返回引擎内部排序的固定 Processor 快照。
    #[must_use]
    /// 对应 Java 语义：`ElementDefinition` 的 `sorted_associated_processors` 行为（Rust 侧辅助/私有路径）。
    pub fn sorted_associated_processors(&self) -> &[Arc<dyn IElementProcessor>] {
        &self.associated_processors
    }

    /// 判断两个定义是否属于同一具体类且元素名相等。
    #[must_use]
    /// 对应 Java 语义：`ElementDefinition` 的 `equals_java` 行为（Rust 侧辅助/私有路径）。
    pub fn equals_java(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
            || (self.kind == other.kind
                && self.element_name.as_element_name() == other.element_name.as_element_name())
    }

    /// 返回元素名缓存哈希。
    #[must_use]
    /// 对应 Java: `ElementDefinition#hashCode()`。
    pub fn hash_code(&self) -> i32 {
        self.element_name.as_element_name().hash_code()
    }

    /// 返回元素名字符串表示。
    ///
    /// # 错误
    ///
    /// complete names 数组为空时传播对应错误。
    /// 对应 Java 语义：`ElementDefinition` 的 `to_java_string` 行为（Rust 侧辅助/私有路径）。
    pub fn to_java_string(&self) -> Result<JavaString, ElementDefinitionError> {
        self.element_name
            .as_element_name()
            .to_java_string()
            .map_err(ElementDefinitionError::ElementName)
    }
}

fn compare_processors(
    left: &Arc<dyn IElementProcessor>,
    right: &Arc<dyn IElementProcessor>,
) -> Ordering {
    if Arc::ptr_eq(left, right) {
        return Ordering::Equal;
    }
    crate::util::ProcessorComparators::compare_processors(left.as_ref(), right.as_ref())
}
