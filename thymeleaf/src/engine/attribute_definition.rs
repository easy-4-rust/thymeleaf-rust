use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};

use crate::element::{ElementProcessorSet, IElementProcessor, UnmodifiableElementProcessorSet};
use crate::util::JavaString;

use super::{AttributeNameError, AttributeNameValue};

/// `AttributeDefinition` 的具体 Java 子类标识。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributeDefinitionKind {
    /// `HTMLAttributeDefinition`。
    Html,
    /// `XMLAttributeDefinition`。
    Xml,
    /// `TextAttributeDefinition`。
    Text,
}

/// 属性定义构造、比较和显示错误。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttributeDefinitionError {
    /// 属性名为 null。
    NullAttributeName,
    /// Processor Set 为 null。
    NullAssociatedProcessors,
    /// Processor Set 包含 null，排序比较器解引用失败。
    NullAssociatedProcessor,
    /// 属性名的内部 complete names 数组被外部破坏。
    AttributeName(AttributeNameError),
}

impl AttributeDefinitionError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::NullAttributeName | Self::NullAssociatedProcessors => {
                "java.lang.IllegalArgumentException"
            }
            Self::NullAssociatedProcessor => "java.lang.NullPointerException",
            Self::AttributeName(error) => error.java_class_name(),
        }
    }
}

impl Display for AttributeDefinitionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullAttributeName => formatter.write_str("Attribute name cannot be null"),
            Self::NullAssociatedProcessors => {
                formatter.write_str("Associated processors cannot be null")
            }
            Self::NullAssociatedProcessor => {
                formatter.write_str("processor comparator received null")
            }
            Self::AttributeName(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for AttributeDefinitionError {}

/// 属性名及其关联元素 Processor 的不可变运行时定义。
///
/// 对应 Java: `org.thymeleaf.engine.AttributeDefinition`。
pub struct AttributeDefinition {
    kind: AttributeDefinitionKind,
    attribute_name: AttributeNameValue,
    associated_processors_set: UnmodifiableElementProcessorSet,
    associated_processors: Vec<Arc<dyn IElementProcessor>>,
    has_associated_processors: bool,
}

impl AttributeDefinition {
    pub(super) fn new(
        kind: AttributeDefinitionKind,
        attribute_name: Option<AttributeNameValue>,
        associated_processors: Option<Arc<RwLock<ElementProcessorSet>>>,
    ) -> Result<Self, AttributeDefinitionError> {
        let attribute_name = attribute_name.ok_or(AttributeDefinitionError::NullAttributeName)?;
        let associated_processors =
            associated_processors.ok_or(AttributeDefinitionError::NullAssociatedProcessors)?;
        let mut sorted = crate::element::read_set(&associated_processors)
            .iter()
            .map(|value| {
                value
                    .cloned()
                    .ok_or(AttributeDefinitionError::NullAssociatedProcessor)
            })
            .collect::<Result<Vec<_>, _>>()?;
        sorted.sort_by(compare_processors);
        let has_associated_processors = !sorted.is_empty();
        Ok(Self {
            kind,
            attribute_name,
            associated_processors_set: UnmodifiableElementProcessorSet::new(associated_processors),
            associated_processors: sorted,
            has_associated_processors,
        })
    }

    /// 返回定义对应的属性名。
    #[must_use]
    pub const fn get_attribute_name(&self) -> &AttributeNameValue {
        &self.attribute_name
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

    /// 返回引擎内部按 Processor comparator 排序的固定快照。
    #[must_use]
    pub fn sorted_associated_processors(&self) -> &[Arc<dyn IElementProcessor>] {
        &self.associated_processors
    }

    /// 判断两个定义是否属于同一具体类且属性名相等。
    ///
    /// # 错误
    ///
    /// 属性名称数组被外部破坏时传播对应错误。
    pub fn equals_java(&self, other: &Self) -> Result<bool, AttributeDefinitionError> {
        if std::ptr::eq(self, other) {
            return Ok(true);
        }
        if self.kind != other.kind {
            return Ok(false);
        }
        self.attribute_name
            .as_attribute_name()
            .equals_java(other.attribute_name.as_attribute_name())
            .map_err(AttributeDefinitionError::AttributeName)
    }

    /// 返回属性名的缓存哈希。
    #[must_use]
    pub fn hash_code(&self) -> i32 {
        self.attribute_name.as_attribute_name().hash_code()
    }

    /// 返回属性名字符串表示。
    ///
    /// # 错误
    ///
    /// 属性名 complete names 数组为空时传播对应错误。
    pub fn to_java_string(&self) -> Result<JavaString, AttributeDefinitionError> {
        self.attribute_name
            .as_attribute_name()
            .to_java_string()
            .map_err(AttributeDefinitionError::AttributeName)
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
