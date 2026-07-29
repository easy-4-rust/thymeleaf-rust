use std::io;
use std::sync::RwLock;

use super::{
    IWritableCharSequence, JavaCharSequence, JavaHashCode, JavaString, JavaWriter, TextUtilsError,
};

/// `AbstractLazyCharSequence` 子类提供的延迟解析与未解析写出行为。
pub trait LazyCharSequenceResolver {
    /// 返回 Java 具体子类全限定名，供基类 `equals` 执行精确类判断。
    fn java_class_name(&self) -> &str;

    /// 对应子类 `resolveText()`；允许返回 Java null。
    fn resolve_text(&self) -> Option<JavaString>;

    /// 对应子类 `writeUnresolved(Writer)`。
    fn write_unresolved(&self, writer: &mut dyn JavaWriter) -> io::Result<()>;
}

/// 可延迟解析完整文本、并在未解析时直接写入输出的字符序列基类。
///
/// 对应 Java: `org.thymeleaf.util.AbstractLazyCharSequence`。
///
/// 与上游一样，本对象不承诺线程安全；Rust 使用锁只为安全表达内部缓存，不改变
/// “写出未解析内容不会填充 resolvedText”这一关键语义。
pub struct AbstractLazyCharSequence<R: LazyCharSequenceResolver> {
    resolver: R,
    resolved_text: RwLock<Option<JavaString>>,
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
    pub fn get_text(&self) -> Option<JavaString> {
        if let Some(value) = read_lock(&self.resolved_text).as_ref() {
            return Some(value.clone());
        }
        let value = self.resolver.resolve_text();
        *write_lock(&self.resolved_text) = value.clone();
        value
    }

    /// 按 Java 精确具体类与解析后字符串内容判断相等。
    pub fn equals_java(&self, other: &Self) -> Result<bool, TextUtilsError> {
        if std::ptr::eq(self, other) {
            return Ok(true);
        }
        if self.resolver.java_class_name() != other.resolver.java_class_name() {
            return Ok(false);
        }
        let left = self.get_text().ok_or(TextUtilsError::NullPointer)?;
        let right = other.get_text().ok_or(TextUtilsError::NullPointer)?;
        Ok(left == right)
    }

    /// 返回解析字符串的 Java `String#hashCode()`。
    pub fn hash_code(&self) -> Result<i32, TextUtilsError> {
        self.get_text()
            .ok_or(TextUtilsError::NullPointer)
            .map(|value| value.java_hash_code())
    }

    /// 返回解析后的 Java String。
    pub fn to_java_string(&self) -> Result<JavaString, TextUtilsError> {
        self.get_text().ok_or(TextUtilsError::NullPointer)
    }
}

impl<R: LazyCharSequenceResolver> JavaCharSequence for AbstractLazyCharSequence<R> {
    fn java_length(&self) -> Result<i32, TextUtilsError> {
        Ok(self.get_text().ok_or(TextUtilsError::NullPointer)?.len() as i32)
    }

    fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
        self.get_text()
            .ok_or(TextUtilsError::NullPointer)?
            .java_char_at(index)
    }

    fn as_java_string(&self) -> Option<&JavaString> {
        None
    }

    fn java_to_string(&self) -> Result<JavaString, TextUtilsError> {
        self.to_java_string()
    }

    fn java_sub_sequence(&self, start: i32, end: i32) -> Result<JavaString, TextUtilsError> {
        self.get_text()
            .ok_or(TextUtilsError::NullPointer)?
            .java_sub_sequence(start, end)
    }

    fn write_direct(&self, writer: &mut dyn JavaWriter) -> Option<io::Result<()>> {
        Some(IWritableCharSequence::write(self, writer))
    }
}

impl<R: LazyCharSequenceResolver> IWritableCharSequence for AbstractLazyCharSequence<R> {
    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
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
