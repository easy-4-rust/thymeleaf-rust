use std::fmt::{Display, Formatter};
use std::ptr;
use std::sync::Arc;

use num_bigint::BigInt;
use num_traits::Zero;
use thiserror::Error;

use crate::expression::{JavaObjectArray, LiteralValue};

use super::{JavaBigDecimal, JavaNumber, JavaString, ValidateError};

const JAVA_BMP_DECIMAL_ZEROES: &[u16] = &[
    0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6,
    0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80,
    0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0,
    0xFF10,
];

/// `EvaluationUtils` 标量求值时可观察的 Java 运行时类型。
///
/// 对应 Java: `java.lang.Object` 在
/// `org.thymeleaf.util.EvaluationUtils#evaluateAsBoolean` 与
/// `EvaluationUtils#evaluateAsNumber` 中的 `instanceof` 分派。
#[derive(Clone, Debug, PartialEq)]
pub enum JavaEvaluationValue {
    /// Java null。
    Null,
    /// `java.lang.Boolean`。
    Boolean(bool),
    /// 任意 `java.lang.Number`。
    Number(JavaNumber),
    /// `java.lang.Character` 的 UTF-16 代码单元。
    Character(u16),
    /// `java.lang.String`。
    String(JavaString),
    /// `LiteralValue` 对象；`Arc` 保留 Java 引用身份及共享语义。
    LiteralValue(Arc<LiteralValue>),
    /// 其他 Java 对象及其运行时类名。
    Other(String),
}

/// `evaluateAsNumber` 返回的借用或新建 `BigDecimal`。
///
/// 对应 Java 方法对 `BigDecimal` 输入返回同一实例，而对其他支持的数字类型创建
/// 新对象的身份语义。
#[derive(Debug)]
pub enum JavaBigDecimalResult<'a> {
    /// 原 `BigDecimal` 的同一引用。
    Borrowed(&'a JavaBigDecimal),
    /// 新创建的 `BigDecimal`。
    Owned(JavaBigDecimal),
}

impl<'a> JavaBigDecimalResult<'a> {
    /// 返回统一的 `BigDecimal` 只读引用。
    ///
    /// # 返回
    /// 借用或拥有分支中的十进制值。
    /// 对应 Java 语义：`EvaluationUtils` 的 `as_decimal` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn as_decimal(&self) -> &JavaBigDecimal {
        match self {
            Self::Borrowed(value) => value,
            Self::Owned(value) => value,
        }
    }

    /// 判断结果是否为指定输入的同一实例。
    ///
    /// # 参数
    /// - `source`：待比较的原始 `BigDecimal`。
    ///
    /// # 返回
    /// 结果直接借用该实例时返回 `true`。
    /// 对应 Java 语义：`EvaluationUtils` 的 `is_borrowed_from` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn is_borrowed_from(&self, source: &JavaBigDecimal) -> bool {
        matches!(self, Self::Borrowed(value) if ptr::eq(*value, source))
    }
}

/// Java 对象的 `hashCode()` 适配契约。
///
/// 对应 Java: `java.lang.Object#hashCode()`；用于精确复现嵌套
/// `EvaluationUtils.MapEntry#hashCode()` 的 31 倍组合算法。
pub trait JavaHashCode {
    /// 返回 Java 有符号 32 位哈希。
    ///
    /// # 返回
    /// 与对象 `hashCode()` 相同、按二进制补码溢出的值。
    fn java_hash_code(&self) -> i32;
}

impl JavaHashCode for i32 {
    fn java_hash_code(&self) -> i32 {
        *self
    }
}

impl JavaHashCode for String {
    fn java_hash_code(&self) -> i32 {
        self.encode_utf16().fold(0_i32, |hash, unit| {
            hash.wrapping_mul(31).wrapping_add(i32::from(unit))
        })
    }
}

impl JavaHashCode for JavaString {
    fn java_hash_code(&self) -> i32 {
        self.as_utf16().iter().fold(0_i32, |hash, unit| {
            hash.wrapping_mul(31).wrapping_add(i32::from(*unit))
        })
    }
}

/// Map 遍历列表中使用的不可变条目快照。
///
/// 对应 Java: `org.thymeleaf.util.EvaluationUtils.MapEntry`。该嵌套对象避免
/// `EnumMap` 迭代器复用条目对象，并保留非标准的 31 倍哈希算法。
#[derive(Clone, Debug)]
pub struct JavaMapEntry<T> {
    entry_key: Option<T>,
    entry_value: Option<T>,
    class_name: String,
}

impl<T: PartialEq> PartialEq for JavaMapEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.entry_key == other.entry_key && self.entry_value == other.entry_value
    }
}

impl<T> JavaMapEntry<T> {
    /// 从可空键和值创建不可变条目。
    ///
    /// 对应 Java: `EvaluationUtils.MapEntry#MapEntry(Object,Object)`。
    ///
    /// # 参数
    /// - `key`：Map 键，`None` 对应 null；
    /// - `value`：Map 值，`None` 对应 null。
    ///
    /// # 返回
    /// 保存键和值的新条目。
    #[must_use]
    pub fn new(key: Option<T>, value: Option<T>) -> Self {
        Self {
            entry_key: key,
            entry_value: value,
            class_name: "org.thymeleaf.util.EvaluationUtils$MapEntry".to_owned(),
        }
    }

    /// 创建保留 Map 实现运行时条目类的原始条目。
    ///
    /// 对应 Java: `Map#entrySet()` 迭代器返回的实现类对象；该对象在
    /// `evaluateAsArray` 中原样保留，在 `evaluateAsList` 中被快照替换。
    ///
    /// # 参数
    /// - `class_name`：原条目的 `Class#getName()`；
    /// - `key`：可空键；
    /// - `value`：可空值。
    ///
    /// # 返回
    /// 带原始运行时类元数据的条目。
    #[must_use]
    pub fn raw(class_name: impl Into<String>, key: Option<T>, value: Option<T>) -> Self {
        Self {
            entry_key: key,
            entry_value: value,
            class_name: class_name.into(),
        }
    }

    /// 返回条目键。对应 Java: `MapEntry#getKey()`。
    ///
    /// # 返回
    /// 可空键的只读引用。
    #[must_use]
    pub fn get_key(&self) -> Option<&T> {
        self.entry_key.as_ref()
    }

    /// 返回条目值。对应 Java: `MapEntry#getValue()`。
    ///
    /// # 返回
    /// 可空值的只读引用。
    #[must_use]
    pub fn get_value(&self) -> Option<&T> {
        self.entry_value.as_ref()
    }

    /// 返回条目的 Java 运行时类名。
    ///
    /// # 返回
    /// 原 Map 条目实现类或 `EvaluationUtils$MapEntry`。
    /// 对应 Java 语义：`EvaluationUtils` 的 `java_class_name` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn java_class_name(&self) -> &str {
        &self.class_name
    }

    /// 拒绝修改不可变条目。对应 Java: `MapEntry#setValue(Object)`。
    ///
    /// # 参数
    /// - `value`：Java 调用方尝试写入的值。
    ///
    /// # 错误
    /// 始终返回 `UnsupportedOperationException` 等价错误。
    pub fn set_value(&mut self, _value: Option<T>) -> Result<Option<T>, EvaluationError> {
        Err(EvaluationError::UnsupportedOperation)
    }
}

impl<T: JavaHashCode> JavaMapEntry<T> {
    /// 返回上游条目的非标准哈希。对应 Java: `MapEntry#hashCode()`。
    ///
    /// # 返回
    /// `31 * keyHash + valueHash`，按 Java `int` 规则溢出。
    #[must_use]
    pub fn java_hash_code(&self) -> i32 {
        let key_hash = self
            .entry_key
            .as_ref()
            .map_or(0, JavaHashCode::java_hash_code);
        let value_hash = self
            .entry_value
            .as_ref()
            .map_or(0, JavaHashCode::java_hash_code);
        key_hash.wrapping_mul(31).wrapping_add(value_hash)
    }
}

impl<T: Display> Display for JavaMapEntry<T> {
    /// 输出 Java `key=value` 文本，其中 null 使用字面量 `null`。
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.entry_key {
            Some(key) => write!(formatter, "{key}")?,
            None => formatter.write_str("null")?,
        }
        formatter.write_str("=")?;
        match &self.entry_value {
            Some(value) => write!(formatter, "{value}"),
            None => formatter.write_str("null"),
        }
    }
}

/// `evaluateAsList` 新建列表中的元素类型。
///
/// 对应 Java primitive array 的装箱结果、普通对象引用及 Map 条目。
#[derive(Clone, Debug, PartialEq)]
pub enum JavaEvaluationElement<T> {
    /// 普通 Java 引用对象。
    Object(T),
    /// `java.lang.Byte`。
    Byte(i8),
    /// `java.lang.Short`。
    Short(i16),
    /// `java.lang.Integer`。
    Integer(i32),
    /// `java.lang.Long`。
    Long(i64),
    /// `java.lang.Float`。
    Float(f32),
    /// `java.lang.Double`。
    Double(f64),
    /// `java.lang.Boolean`。
    Boolean(bool),
    /// `java.lang.Character`。
    Character(u16),
    /// Java `Map.Entry`。
    MapEntry(Arc<JavaMapEntry<T>>),
}

/// `evaluateAsList` 输入的 Java 运行时分类。
///
/// 对应 Java 的 `Iterable`、`Map`、八种 primitive array、引用数组和标量分支。
pub enum JavaEvaluationTarget<'a, T> {
    /// Java `Iterable<?>`，按迭代顺序保存可空对象。
    Iterable(&'a [Option<T>]),
    /// Java `Map<?,?>` 的稳定迭代条目；`Arc` 保存原条目身份。
    Map(&'a [Arc<JavaMapEntry<T>>]),
    /// `byte[]`。
    Bytes(&'a [i8]),
    /// `short[]`。
    Shorts(&'a [i16]),
    /// `int[]`。
    Integers(&'a [i32]),
    /// `long[]`。
    Longs(&'a [i64]),
    /// `float[]`。
    Floats(&'a [f32]),
    /// `double[]`。
    Doubles(&'a [f64]),
    /// `boolean[]`。
    Booleans(&'a [bool]),
    /// `char[]`。
    Characters(&'a [u16]),
    /// Java 引用数组。
    ReferenceArray(&'a JavaObjectArray<T>),
    /// 其他标量对象。
    Other(&'a T),
}

/// `evaluateAsList` 返回的具体 Java 列表类别。
/// 对应 Java 语义：`EvaluationUtils` 的 Rust 侧类型 `JavaEvaluationListType`。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JavaEvaluationListType {
    /// null 输入返回的 `java.util.Collections$EmptyList`。
    EmptyList,
    /// 非 null 输入返回的 `java.util.Collections$UnmodifiableRandomAccessList`。
    UnmodifiableRandomAccessList,
}

/// Java 不可修改求值列表。
///
/// 对应 Java: `Collections.emptyList()` 或
/// `Collections.unmodifiableList(ArrayList)`。
#[derive(Clone, Debug, PartialEq)]
pub struct JavaEvaluationList<T> {
    list_type: JavaEvaluationListType,
    elements: Vec<Option<JavaEvaluationElement<T>>>,
}

impl<T> JavaEvaluationList<T> {
    /// 返回具体 Java 列表类别。
    ///
    /// # 返回
    /// null 与非 null 输入对应的运行时包装类型。
    #[must_use]
    pub const fn list_type(&self) -> JavaEvaluationListType {
        self.list_type
    }

    /// 返回按 Java 迭代顺序排列的元素。
    ///
    /// # 返回
    /// 只读元素切片；外层 `None` 对应 Java null。
    #[must_use]
    pub fn as_slice(&self) -> &[Option<JavaEvaluationElement<T>>] {
        &self.elements
    }

    /// 返回列表元素数量。
    ///
    /// # 返回
    /// Java `List#size()` 等价值。
    /// 对应 Java 语义：`EvaluationUtils` 的 `len` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// 判断列表是否为空。
    ///
    /// # 返回
    /// `size() == 0` 时返回 `true`。
    /// 对应 Java 语义：Java 接口/超类方法 `isEmpty()` 的 Rust 移植（`EvaluationUtils` 继承路径）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

/// `evaluateAsArray` 的借用或新建数组结果。
///
/// 对应 Java 对引用数组原样返回同一实例，其他受支持输入新建 `Object[]`。
#[derive(Debug)]
pub enum JavaEvaluationArray<'a, T> {
    /// 原引用数组的同一实例。
    Borrowed(&'a JavaObjectArray<T>),
    /// 新建的 `java.lang.Object[]`。
    Owned(JavaObjectArray<JavaEvaluationElement<T>>),
}

impl<'a, T> JavaEvaluationArray<'a, T> {
    /// 判断结果是否为指定引用数组的同一实例。
    ///
    /// # 参数
    /// - `source`：待比较的原数组。
    ///
    /// # 返回
    /// 借用分支指向该数组时返回 `true`。
    /// 对应 Java 语义：`EvaluationUtils` 的 `is_borrowed_from` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn is_borrowed_from(&self, source: &JavaObjectArray<T>) -> bool {
        matches!(self, Self::Borrowed(value) if ptr::eq(*value, source))
    }

    /// 返回新建 `Object[]`；借用原类型数组时返回 `None`。
    ///
    /// # 返回
    /// 仅 Owned 分支中的数组引用。
    /// 对应 Java 语义：`EvaluationUtils` 的 `as_owned_array` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn as_owned_array(&self) -> Option<&JavaObjectArray<JavaEvaluationElement<T>>> {
        match self {
            Self::Borrowed(_) => None,
            Self::Owned(value) => Some(value),
        }
    }
}

/// `EvaluationUtils` 可观察的 Java 异常。
/// 对应 Java 语义：`EvaluationUtils` 的 Rust 侧类型 `EvaluationError`。
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum EvaluationError {
    /// `Validate.notNull` 的参数错误。
    #[error(transparent)]
    Validation(#[from] ValidateError),
    /// `LiteralValue(null)` 求值时调用 `trim()`。
    #[error(
        "Cannot invoke \"String.trim()\" because the return value of \"LiteralValue.getValue()\" is null"
    )]
    NullPointer,
    /// 非有限浮点值传给 `BigDecimal(double)`。
    #[error("Infinite or NaN")]
    NumberFormat,
    /// primitive array 被强制转换为 `Object[]`。
    #[error("class {array_class_name} cannot be cast to class [Ljava.lang.Object;")]
    ClassCast {
        /// JVM primitive array 类名，例如 `[I`。
        array_class_name: &'static str,
    },
    /// 不可变 Map 条目拒绝 `setValue`。
    #[error("")]
    UnsupportedOperation,
}

impl EvaluationError {
    /// 返回对应 Java 异常类名。
    ///
    /// # 返回
    /// 原实现会抛出的 JVM 异常类名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::Validation(error) => error.java_class_name(),
            Self::NullPointer => "java.lang.NullPointerException",
            Self::NumberFormat => "java.lang.NumberFormatException",
            Self::ClassCast { .. } => "java.lang.ClassCastException",
            Self::UnsupportedOperation => "java.lang.UnsupportedOperationException",
        }
    }
}

/// Thymeleaf 表达式求值转换工具。
///
/// 对应 Java: `org.thymeleaf.util.EvaluationUtils`。本对象保留 JVM 运行时类型
/// 分派、`BigDecimal(double)` 精确构造、primitive array 装箱、Map 条目身份、
/// 不可修改列表以及引用数组原样返回等语义。
pub struct EvaluationUtils;

impl EvaluationUtils {
    /// 按 Thymeleaf 规则求布尔值。
    ///
    /// 对应 Java: `EvaluationUtils#evaluateAsBoolean(Object)`。
    ///
    /// # 参数
    /// - `condition`：显式保存 Java 运行时类型的条件值。
    ///
    /// # 返回
    /// null、数值零、字符零及裁剪后为 false/off/no 的字符串返回 `false`。
    ///
    /// # 错误
    /// `LiteralValue` 内部为 null 时返回 Java `NullPointerException` 等价错误。
    pub fn evaluate_as_boolean(condition: &JavaEvaluationValue) -> Result<bool, EvaluationError> {
        match condition {
            JavaEvaluationValue::Null => Ok(false),
            JavaEvaluationValue::Boolean(value) => Ok(*value),
            JavaEvaluationValue::Number(number) => Ok(number_is_non_zero(number)),
            JavaEvaluationValue::Character(value) => Ok(*value != 0),
            JavaEvaluationValue::String(value) => Ok(string_is_true(value)),
            JavaEvaluationValue::LiteralValue(value) => value
                .get_value()
                .map(string_is_true)
                .ok_or(EvaluationError::NullPointer),
            JavaEvaluationValue::Other(_) => Ok(true),
        }
    }

    /// 按 Java 运行时类型转换为 `BigDecimal`。
    ///
    /// 对应 Java: `EvaluationUtils#evaluateAsNumber(Object)`。
    ///
    /// # 参数
    /// - `object`：待转换标量。
    ///
    /// # 返回
    /// 支持的数字或字符串返回十进制值；不支持或非法字符串返回 `None`；
    /// `BigDecimal` 输入通过 Borrowed 保留同一实例。
    ///
    /// # 错误
    /// Float/Double 为 NaN 或无穷时返回 `NumberFormatException` 等价错误。
    pub fn evaluate_as_number(
        object: &JavaEvaluationValue,
    ) -> Result<Option<JavaBigDecimalResult<'_>>, EvaluationError> {
        match object {
            JavaEvaluationValue::Number(number) => number_as_decimal(number),
            JavaEvaluationValue::String(value) if !value.is_empty() => {
                Ok(parse_java_big_decimal(value).map(JavaBigDecimalResult::Owned))
            }
            _ => Ok(None),
        }
    }

    /// 将 Java 值转换成不可修改列表。
    ///
    /// 对应 Java: `EvaluationUtils#evaluateAsList(Object)`。
    ///
    /// # 参数
    /// - `value`：`None` 对应 Java null；其他分支显式描述运行时类型。
    ///
    /// # 返回
    /// null 返回共享空列表语义；Map 使用新条目快照；primitive array 逐项装箱。
    #[must_use]
    pub fn evaluate_as_list<T: Clone>(
        value: Option<JavaEvaluationTarget<'_, T>>,
    ) -> JavaEvaluationList<T> {
        let Some(value) = value else {
            return JavaEvaluationList {
                list_type: JavaEvaluationListType::EmptyList,
                elements: Vec::new(),
            };
        };
        let elements = match value {
            JavaEvaluationTarget::Iterable(values) => values
                .iter()
                .map(|value| value.clone().map(JavaEvaluationElement::Object))
                .collect(),
            JavaEvaluationTarget::Map(entries) => entries
                .iter()
                .map(|entry| {
                    Some(JavaEvaluationElement::MapEntry(Arc::new(
                        JavaMapEntry::new(entry.entry_key.clone(), entry.entry_value.clone()),
                    )))
                })
                .collect(),
            JavaEvaluationTarget::Bytes(values) => boxed(values, JavaEvaluationElement::Byte),
            JavaEvaluationTarget::Shorts(values) => boxed(values, JavaEvaluationElement::Short),
            JavaEvaluationTarget::Integers(values) => boxed(values, JavaEvaluationElement::Integer),
            JavaEvaluationTarget::Longs(values) => boxed(values, JavaEvaluationElement::Long),
            JavaEvaluationTarget::Floats(values) => boxed(values, JavaEvaluationElement::Float),
            JavaEvaluationTarget::Doubles(values) => boxed(values, JavaEvaluationElement::Double),
            JavaEvaluationTarget::Booleans(values) => boxed(values, JavaEvaluationElement::Boolean),
            JavaEvaluationTarget::Characters(values) => {
                boxed(values, JavaEvaluationElement::Character)
            }
            JavaEvaluationTarget::ReferenceArray(values) => values
                .as_slice()
                .iter()
                .map(|value| value.clone().map(JavaEvaluationElement::Object))
                .collect(),
            JavaEvaluationTarget::Other(value) => {
                vec![Some(JavaEvaluationElement::Object(value.clone()))]
            }
        };
        JavaEvaluationList {
            list_type: JavaEvaluationListType::UnmodifiableRandomAccessList,
            elements,
        }
    }

    /// 将 Java 值转换成对象数组。
    ///
    /// 对应 Java: `EvaluationUtils#evaluateAsArray(Object)`。
    ///
    /// # 参数
    /// - `value`：`None` 对应 Java null。
    ///
    /// # 返回
    /// 引用数组返回同一实例；null、Iterable、Map 与标量返回新 `Object[]`。
    ///
    /// # 错误
    /// 任意 primitive array 都按 Java 强制转换规则返回 `ClassCastException`。
    pub fn evaluate_as_array<T: Clone>(
        value: Option<JavaEvaluationTarget<'_, T>>,
    ) -> Result<JavaEvaluationArray<'_, T>, EvaluationError> {
        let Some(value) = value else {
            return Ok(JavaEvaluationArray::Owned(JavaObjectArray::object(vec![
                None,
            ])));
        };
        match value {
            JavaEvaluationTarget::ReferenceArray(values) => {
                Ok(JavaEvaluationArray::Borrowed(values))
            }
            JavaEvaluationTarget::Bytes(_) => Err(class_cast("[B")),
            JavaEvaluationTarget::Shorts(_) => Err(class_cast("[S")),
            JavaEvaluationTarget::Integers(_) => Err(class_cast("[I")),
            JavaEvaluationTarget::Longs(_) => Err(class_cast("[J")),
            JavaEvaluationTarget::Floats(_) => Err(class_cast("[F")),
            JavaEvaluationTarget::Doubles(_) => Err(class_cast("[D")),
            JavaEvaluationTarget::Booleans(_) => Err(class_cast("[Z")),
            JavaEvaluationTarget::Characters(_) => Err(class_cast("[C")),
            JavaEvaluationTarget::Iterable(values) => {
                Ok(JavaEvaluationArray::Owned(JavaObjectArray::object(
                    values
                        .iter()
                        .map(|value| value.clone().map(JavaEvaluationElement::Object))
                        .collect(),
                )))
            }
            JavaEvaluationTarget::Map(entries) => {
                Ok(JavaEvaluationArray::Owned(JavaObjectArray::object(
                    entries
                        .iter()
                        .map(|entry| Some(JavaEvaluationElement::MapEntry(Arc::clone(entry))))
                        .collect(),
                )))
            }
            JavaEvaluationTarget::Other(value) => {
                Ok(JavaEvaluationArray::Owned(JavaObjectArray::object(vec![
                    Some(JavaEvaluationElement::Object(value.clone())),
                ])))
            }
        }
    }
}

fn number_is_non_zero(number: &JavaNumber) -> bool {
    match number {
        JavaNumber::BigDecimal(value) => !value.unscaled_value().is_zero(),
        JavaNumber::BigInteger(value) => !value.is_zero(),
        JavaNumber::Byte(value) => *value != 0,
        JavaNumber::Short(value) => *value != 0,
        JavaNumber::Integer(value) => *value != 0,
        JavaNumber::Long(value) => *value != 0,
        JavaNumber::Float(value) => f64::from(*value) != 0.0,
        JavaNumber::Double(value) => *value != 0.0,
        JavaNumber::Other { double_value, .. } => *double_value != 0.0,
    }
}

fn string_is_true(value: &JavaString) -> bool {
    let trimmed = java_trim(value.as_utf16());
    !equals_ascii_ignore_case(trimmed, b"false")
        && !equals_ascii_ignore_case(trimmed, b"off")
        && !equals_ascii_ignore_case(trimmed, b"no")
}

fn equals_ascii_ignore_case(value: &[u16], expected: &[u8]) -> bool {
    if value.len() != expected.len() {
        return false;
    }
    for (actual, expected) in value.iter().zip(expected) {
        let Ok(actual) = u8::try_from(*actual) else {
            return false;
        };
        if !actual.eq_ignore_ascii_case(expected) {
            return false;
        }
    }
    true
}

fn number_as_decimal(
    number: &JavaNumber,
) -> Result<Option<JavaBigDecimalResult<'_>>, EvaluationError> {
    let result = match number {
        JavaNumber::BigDecimal(value) => return Ok(Some(JavaBigDecimalResult::Borrowed(value))),
        JavaNumber::BigInteger(value) => JavaBigDecimal::from_unscaled(value.clone(), 0),
        JavaNumber::Byte(value) => JavaBigDecimal::from_unscaled(BigInt::from(*value), 0),
        JavaNumber::Short(value) => JavaBigDecimal::from_unscaled(BigInt::from(*value), 0),
        JavaNumber::Integer(value) => JavaBigDecimal::from_unscaled(BigInt::from(*value), 0),
        JavaNumber::Long(value) => JavaBigDecimal::from_unscaled(BigInt::from(*value), 0),
        JavaNumber::Float(value) => JavaBigDecimal::from_f64_exact(f64::from(*value))
            .ok_or(EvaluationError::NumberFormat)?,
        JavaNumber::Double(value) => {
            JavaBigDecimal::from_f64_exact(*value).ok_or(EvaluationError::NumberFormat)?
        }
        JavaNumber::Other { .. } => return Ok(None),
    };
    Ok(Some(JavaBigDecimalResult::Owned(result)))
}

fn parse_java_big_decimal(value: &JavaString) -> Option<JavaBigDecimal> {
    let units = value.as_utf16();
    // 公共入口已经按 Java `String#length() > 0` 完成该前置检查。
    let first = units[0];
    if !((u16::from(b'0')..=u16::from(b'9')).contains(&first)
        || first == u16::from(b'+')
        || first == u16::from(b'-'))
    {
        return None;
    }
    let trimmed = java_trim(units);
    let mut ascii = String::with_capacity(trimmed.len());
    for unit in trimmed {
        if matches!(*unit, value if value == u16::from(b'+') || value == u16::from(b'-')
            || value == u16::from(b'.') || value == u16::from(b'e') || value == u16::from(b'E'))
        {
            ascii.push(u8::try_from(*unit).expect("matched ASCII syntax unit") as char);
        } else {
            ascii.push(char::from(b'0' + java_decimal_digit(*unit)?));
        }
    }
    JavaBigDecimal::parse(&ascii).ok()
}

fn java_trim(units: &[u16]) -> &[u16] {
    let mut start = 0;
    while start < units.len() && units[start] <= 0x20 {
        start += 1;
    }
    let mut end = units.len();
    while end > start && units[end - 1] <= 0x20 {
        end -= 1;
    }
    &units[start..end]
}

fn java_decimal_digit(unit: u16) -> Option<u8> {
    for zero in JAVA_BMP_DECIMAL_ZEROES {
        if let Some(offset) = unit.checked_sub(*zero)
            && offset <= 9
        {
            return Some(offset as u8);
        }
    }
    None
}

fn boxed<T: Copy, U>(
    values: &[T],
    constructor: impl Fn(T) -> JavaEvaluationElement<U>,
) -> Vec<Option<JavaEvaluationElement<U>>> {
    values
        .iter()
        .copied()
        .map(|value| Some(constructor(value)))
        .collect()
}

fn class_cast(array_class_name: &'static str) -> EvaluationError {
    EvaluationError::ClassCast { array_class_name }
}
