use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::util::{JavaHashCode, Utf16String};

/// `AttributeName` 的具体 Java 子类标识。
///
/// Java `AttributeName#equals` 要求两个对象运行时类完全一致，因此组合式 Rust
/// 迁移显式保存该标识。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 对应 Java 语义：`AttributeName` 的 Rust 侧类型 `AttributeNameKind`。
pub enum AttributeNameKind {
    /// `HTMLAttributeName`。
    Html,
    /// `XMLAttributeName`。
    Xml,
    /// `TextAttributeName`。
    Text,
}

/// 属性名构造、比较或字符串化时的 Java 运行时错误。
///
/// 对应 Java: `org.thymeleaf.engine.AttributeName`。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttributeNameError {
    /// 属性名为 null、空串或只包含 Java trim 空白。
    InvalidAttributeName,
    /// complete names 数组被外部清空后访问首元素。
    EmptyCompleteAttributeNames,
    /// complete names 首元素被外部替换为 null 后调用 `equals`。
    NullFirstCompleteAttributeName,
}

impl AttributeNameError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::InvalidAttributeName => "java.lang.IllegalArgumentException",
            Self::EmptyCompleteAttributeNames => "java.lang.ArrayIndexOutOfBoundsException",
            Self::NullFirstCompleteAttributeName => "java.lang.NullPointerException",
        }
    }
}

impl Display for AttributeNameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAttributeName => {
                formatter.write_str("Attribute name cannot be null or empty")
            }
            Self::EmptyCompleteAttributeNames => {
                formatter.write_str("Index 0 out of bounds for length 0")
            }
            Self::NullFirstCompleteAttributeName => {
                formatter.write_str("Cannot invoke \"String.equals(Object)\" because the first complete attribute name is null")
            }
        }
    }
}

impl Error for AttributeNameError {}

/// 模板属性名称的基础值对象。
///
/// 对应 Java: `org.thymeleaf.engine.AttributeName`。
///
/// 该对象不保存 TemplateMode。complete names getter 返回内部数组的共享可变句柄，
/// 而哈希只在构造时计算，完整保留 Java 暴露数组后可能出现的缓存不一致。
pub struct AttributeName {
    kind: AttributeNameKind,
    prefix: Option<Utf16String>,
    attribute_name: Utf16String,
    complete_attribute_names: Arc<RwLock<Vec<Option<Utf16String>>>>,
    hash_code: i32,
}

impl AttributeName {
    /// 对应 Java 语义：`AttributeName` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(super) fn new(
        kind: AttributeNameKind,
        prefix: Option<Utf16String>,
        attribute_name: Option<Utf16String>,
        complete_attribute_names: Vec<Option<Utf16String>>,
    ) -> Result<Self, AttributeNameError> {
        let attribute_name = attribute_name.ok_or(AttributeNameError::InvalidAttributeName)?;
        if attribute_name.is_empty()
            || attribute_name
                .as_utf16()
                .iter()
                .all(|character| *character <= 0x20)
        {
            return Err(AttributeNameError::InvalidAttributeName);
        }
        let hash_code = arrays_hash_code(&complete_attribute_names);
        Ok(Self {
            kind,
            prefix,
            attribute_name,
            complete_attribute_names: Arc::new(RwLock::new(complete_attribute_names)),
            hash_code,
        })
    }

    /// 返回具体 Java 子类标识。
    #[must_use]
    pub const fn kind(&self) -> AttributeNameKind {
        self.kind
    }

    /// 返回不带命名空间的属性名。
    #[must_use]
    pub const fn get_attribute_name(&self) -> &Utf16String {
        &self.attribute_name
    }

    /// 判断构造时 prefix 是否非 null；空 prefix 仍视为 prefixed。
    #[must_use]
    pub const fn is_prefixed(&self) -> bool {
        self.prefix.is_some()
    }

    /// 返回可空命名空间 prefix。
    #[must_use]
    pub const fn get_prefix(&self) -> Option<&Utf16String> {
        self.prefix.as_ref()
    }

    /// 返回内部 complete names 数组的共享可变句柄。
    #[must_use]
    /// 对应 Java: `AttributeName#getCompleteAttributeNames()`。
    pub fn get_complete_attribute_names(&self) -> Arc<RwLock<Vec<Option<Utf16String>>>> {
        Arc::clone(&self.complete_attribute_names)
    }

    /// 返回构造时按 Java `Arrays.hashCode` 计算的缓存哈希。
    #[must_use]
    pub const fn hash_code(&self) -> i32 {
        self.hash_code
    }

    /// 按 Java `equals` 的运行时类、缓存哈希和首 complete name 规则比较。
    ///
    /// # 错误
    ///
    /// 外部通过数组 getter 清空数组或把首元素设为 null 时，返回对应 JVM 异常。
    /// 对应 Java 语义：`AttributeName` 的 `equals_java` 行为（Rust 侧辅助/私有路径）。
    pub fn equals_java(&self, other: &Self) -> Result<bool, AttributeNameError> {
        if std::ptr::eq(self, other) {
            return Ok(true);
        }
        if self.kind != other.kind || self.hash_code != other.hash_code {
            return Ok(false);
        }
        let own_names = read_recovering_poison(&self.complete_attribute_names);
        let other_names = read_recovering_poison(&other.complete_attribute_names);
        let own_first = own_names
            .first()
            .ok_or(AttributeNameError::EmptyCompleteAttributeNames)?
            .as_ref()
            .ok_or(AttributeNameError::NullFirstCompleteAttributeName)?;
        let other_first = other_names
            .first()
            .ok_or(AttributeNameError::EmptyCompleteAttributeNames)?;
        Ok(other_first.as_ref().is_some_and(|value| value == own_first))
    }

    /// 按 `{name1,name2}` 格式返回 complete names。
    ///
    /// # 错误
    ///
    /// complete names 数组为空时返回 Java 数组越界对应错误。
    /// 对应 Java 语义：`AttributeName` 的 `to_utf16_string` 行为（Rust 侧辅助/私有路径）。
    pub fn to_utf16_string(&self) -> Result<Utf16String, AttributeNameError> {
        let names = read_recovering_poison(&self.complete_attribute_names);
        let Some(first) = names.first() else {
            return Err(AttributeNameError::EmptyCompleteAttributeNames);
        };
        let mut result = vec![u16::from(b'{')];
        append_nullable(&mut result, first.as_ref());
        for name in names.iter().skip(1) {
            result.push(u16::from(b','));
            append_nullable(&mut result, name.as_ref());
        }
        result.push(u16::from(b'}'));
        Ok(Utf16String::from_utf16(result))
    }
}

fn arrays_hash_code(names: &[Option<Utf16String>]) -> i32 {
    names.iter().fold(1_i32, |result, name| {
        result
            .wrapping_mul(31)
            .wrapping_add(name.as_ref().map_or(0, JavaHashCode::java_hash_code))
    })
}

fn append_nullable(target: &mut Vec<u16>, value: Option<&Utf16String>) {
    match value {
        Some(value) => target.extend_from_slice(value.as_utf16()),
        None => target.extend("null".encode_utf16()),
    }
}

fn read_recovering_poison<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
