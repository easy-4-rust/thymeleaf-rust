use std::io;
use std::sync::RwLock;

use super::{
    CharSequenceValue, HashCodeValue, IWritableCharSequence, TemplateWriter, TextUtilsError,
    Utf16String,
};

/// `AbstractLazyCharSequence` 子类提供的延迟解析与未解析写出行为。
/// 对应 Java 语义：`AbstractLazyCharSequence` 的 Rust 侧类型 `LazyCharSequenceResolver`。
pub trait LazyCharSequenceResolver: Send + Sync {
    /// 返回 Java 具体子类全限定名，供基类 `equals` 执行精确类判断。
    fn class_name(&self) -> &str;

    /// 对应子类 `resolveText()`；允许返回 Java null。
    fn resolve_text(&self) -> Option<Utf16String>;

    /// 对应子类 `writeUnresolved(Writer)`。
    fn write_unresolved(&self, writer: &mut dyn TemplateWriter) -> io::Result<()>;
}

/// 可延迟解析完整文本、并在未解析时直接写入输出的字符序列基类。
///
/// 对应 Java: `org.thymeleaf.util.AbstractLazyCharSequence`。
///
/// Rust 使用锁安全表达 Java 的惰性缓存，同时不改变“写出未解析内容不会填充
/// resolvedText”这一关键语义；该能力使解析事件可以进入跨线程模板缓存。
pub struct AbstractLazyCharSequence<R: LazyCharSequenceResolver> {
    resolver: R,
    resolved_text: RwLock<Option<Utf16String>>,
}

impl<R: LazyCharSequenceResolver> AbstractLazyCharSequence<R> {
    /// 创建尚未解析文本的基类实例。
    #[must_use]
    pub const fn new(resolver: R) -> Self {
        Self {
            resolver,
            resolved_text: RwLock::new(None),
        }
    }

    /// 返回缓存文本；未解析时调用一次 `resolveText()`。
    ///
    /// 若 resolver 返回 null，缓存仍保持 null，后续访问会再次求值。
    /// 对应 Java: `AbstractLazyCharSequence#getText()`。
    pub fn get_text(&self) -> Option<Utf16String> {
        if let Some(value) = read_lock(&self.resolved_text).as_ref() {
            return Some(value.clone());
        }
        let value = self.resolver.resolve_text();
        *write_lock(&self.resolved_text) = value.clone();
        value
    }

    /// 按 Java 精确具体类与解析后字符串内容判断相等。
    /// 对应 Java 语义：`AbstractLazyCharSequence` 的 `equals_java` 行为（Rust 侧辅助/私有路径）。
    pub fn equals_java(&self, other: &Self) -> Result<bool, TextUtilsError> {
        if std::ptr::eq(self, other) {
            return Ok(true);
        }
        if self.resolver.class_name() != other.resolver.class_name() {
            return Ok(false);
        }
        let left = self.get_text().ok_or(TextUtilsError::NullPointer)?;
        let right = other.get_text().ok_or(TextUtilsError::NullPointer)?;
        Ok(left == right)
    }

    /// 返回解析字符串的 Java `String#hashCode()`。
    /// 对应 Java: `AbstractLazyCharSequence#hashCode()`。
    pub fn hash_code(&self) -> Result<i32, TextUtilsError> {
        self.get_text()
            .ok_or(TextUtilsError::NullPointer)
            .map(|value| value.java_hash_code())
    }

    /// 返回解析后的 Java String。
    /// 对应 Java 语义：`AbstractLazyCharSequence` 的 `to_utf16_string` 行为（Rust 侧辅助/私有路径）。
    pub fn to_utf16_string(&self) -> Result<Utf16String, TextUtilsError> {
        self.get_text().ok_or(TextUtilsError::NullPointer)
    }
}

impl<R: LazyCharSequenceResolver> CharSequenceValue for AbstractLazyCharSequence<R> {
    fn java_length(&self) -> Result<i32, TextUtilsError> {
        Ok(self.get_text().ok_or(TextUtilsError::NullPointer)?.len() as i32)
    }

    fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
        self.get_text()
            .ok_or(TextUtilsError::NullPointer)?
            .java_char_at(index)
    }

    fn as_utf16_string(&self) -> Option<&Utf16String> {
        None
    }

    fn java_to_string(&self) -> Result<Utf16String, TextUtilsError> {
        self.to_utf16_string()
    }

    fn java_sub_sequence(&self, start: i32, end: i32) -> Result<Utf16String, TextUtilsError> {
        self.get_text()
            .ok_or(TextUtilsError::NullPointer)?
            .java_sub_sequence(start, end)
    }

    fn write_direct(&self, writer: &mut dyn TemplateWriter) -> Option<io::Result<()>> {
        Some(IWritableCharSequence::write(self, writer))
    }
}

impl<R: LazyCharSequenceResolver> IWritableCharSequence for AbstractLazyCharSequence<R> {
    fn write(&self, writer: &mut dyn TemplateWriter) -> io::Result<()> {
        if let Some(value) = read_lock(&self.resolved_text).as_ref() {
            writer.write_utf16(value.as_utf16())
        } else {
            self.resolver.write_unresolved(writer)
        }
    }
}

fn read_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
