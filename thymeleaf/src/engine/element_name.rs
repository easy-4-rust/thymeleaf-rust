use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::util::{HashCodeValue, Utf16String};

/// `ElementName` 的具体 Java 子类标识。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 对应 Java 语义：`ElementName` 的 Rust 侧类型 `ElementNameKind`。
pub enum ElementNameKind {
    /// `HTMLElementName`。
    Html,
    /// `XMLElementName`。
    Xml,
    /// `TextElementName`。
    Text,
}

/// `ElementName` 构造或字符串化时的 Java 运行时错误。
///
/// 对应 Java: `org.thymeleaf.engine.ElementName` 的参数校验和数组访问。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ElementNameError {
    /// 元素名为 null 或只包含 Java `String#trim()` 会移除的字符。
    InvalidElementName,
    /// 子类传入空的 complete names 数组后调用 `toString()`。
    EmptyCompleteElementNames,
}

impl ElementNameError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::InvalidElementName => "java.lang.IllegalArgumentException",
            Self::EmptyCompleteElementNames => "java.lang.ArrayIndexOutOfBoundsException",
        }
    }
}

impl Display for ElementNameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidElementName => formatter.write_str("Element name cannot be null"),
            Self::EmptyCompleteElementNames => {
                formatter.write_str("Index 0 out of bounds for length 0")
            }
        }
    }
}

impl Error for ElementNameError {}

/// 模板元素名称的基础值对象。
///
/// 对应 Java: `org.thymeleaf.engine.ElementName`。
///
/// 该对象刻意不保存 `TemplateMode`，因为 `TextElementName` 同时服务 TEXT、
/// JAVASCRIPT 与 CSS。complete names 使用共享可变数组，复现 Java getter 返回
/// 内部 `String[]` 的别名行为；哈希仅在构造时计算，数组后续修改不会刷新缓存。
pub struct ElementName {
    kind: ElementNameKind,
    prefix: Option<Utf16String>,
    element_name: Utf16String,
    complete_element_names: Arc<RwLock<Vec<Option<Utf16String>>>>,
    hash_code: i32,
}

impl ElementName {
    /// 对应 Java 语义：`ElementName` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(super) fn new(
        kind: ElementNameKind,
        prefix: Option<Utf16String>,
        element_name: Option<Utf16String>,
        complete_element_names: Vec<Option<Utf16String>>,
    ) -> Result<Self, ElementNameError> {
        let element_name = element_name.ok_or(ElementNameError::InvalidElementName)?;
        if !element_name.is_empty()
            && element_name
                .as_utf16()
                .iter()
                .all(|character| *character <= 0x20)
        {
            return Err(ElementNameError::InvalidElementName);
        }
        let hash_code = arrays_hash_code(&complete_element_names);
        Ok(Self {
            kind,
            prefix,
            element_name,
            complete_element_names: Arc::new(RwLock::new(complete_element_names)),
            hash_code,
        })
    }

    /// 返回具体 Java 子类标识。
    #[must_use]
    pub const fn kind(&self) -> ElementNameKind {
        self.kind
    }

    /// 返回不带命名空间前缀的元素名。
    #[must_use]
    pub const fn get_element_name(&self) -> &Utf16String {
        &self.element_name
    }

    /// 判断构造时的 prefix 是否非 null。
    ///
    /// 空字符串 prefix 仍返回 `true`，与 Java 实现一致。
    #[must_use]
    pub const fn is_prefixed(&self) -> bool {
        self.prefix.is_some()
    }

    /// 返回可空命名空间前缀。
    #[must_use]
    pub const fn get_prefix(&self) -> Option<&Utf16String> {
        self.prefix.as_ref()
    }

    /// 返回内部 complete element names 数组的共享句柄。
    ///
    /// # 返回
    ///
    /// 所有调用共享同一个可写数组，等价于 Java 直接返回字段引用。
    #[must_use]
    /// 对应 Java: `ElementName#getCompleteElementNames()`。
    pub fn get_complete_element_names(&self) -> Arc<RwLock<Vec<Option<Utf16String>>>> {
        Arc::clone(&self.complete_element_names)
    }

    /// 返回构造时按 `Arrays.hashCode(String[])` 计算的缓存哈希。
    #[must_use]
    pub const fn hash_code(&self) -> i32 {
        self.hash_code
    }

    /// 按 `{name1,name2}` 格式返回完整名称集合。
    ///
    /// # 错误
    ///
    /// complete names 数组为空时保留 Java 首元素访问越界异常。
    /// 对应 Java 语义：`ElementName` 的 `to_utf16_string` 行为（Rust 侧辅助/私有路径）。
    pub fn to_utf16_string(&self) -> Result<Utf16String, ElementNameError> {
        let names = read_recovering_poison(&self.complete_element_names);
        let Some(first) = names.first() else {
            return Err(ElementNameError::EmptyCompleteElementNames);
        };
        let mut result = Vec::new();
        result.push(u16::from(b'{'));
        append_nullable(&mut result, first.as_ref());
        for name in names.iter().skip(1) {
            result.push(u16::from(b','));
            append_nullable(&mut result, name.as_ref());
        }
        result.push(u16::from(b'}'));
        Ok(Utf16String::from_utf16(result))
    }
}

impl PartialEq for ElementName {
    fn eq(&self, other: &Self) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }
        if self.hash_code != other.hash_code {
            return false;
        }
        let own_names = read_recovering_poison(&self.complete_element_names);
        let other_names = read_recovering_poison(&other.complete_element_names);
        *own_names == *other_names
    }
}

impl Eq for ElementName {}

fn arrays_hash_code(names: &[Option<Utf16String>]) -> i32 {
    names.iter().fold(1_i32, |result, name| {
        result
            .wrapping_mul(31)
            .wrapping_add(name.as_ref().map_or(0, HashCodeValue::java_hash_code))
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
