use std::error::Error;
use std::fmt::{Display, Formatter};

use super::{JavaString, JavaWriter};

const NULL_TEXT: &str = "null";
const NULL_CHAR_ARRAY_MESSAGE: &str =
    "Cannot read the array length because \"<parameter1>\" is null";

/// `FastStringWriter` 操作失败。
///
/// 对应 Java: `org.thymeleaf.util.FastStringWriter` 调用
/// `StringBuilder`、数组访问和 `Writer#append` 时抛出的运行时异常。本类型保留 Java
/// 异常类名、可空消息及范围整数溢出后的文本，以便上层按原语义诊断。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FastStringWriterError {
    /// 初始缓冲区大小为负数，对应 `IllegalArgumentException`。
    NegativeBufferSize,
    /// `write(String,int,int)` 的 UTF-16 范围非法。
    StringRange {
        /// 起始 UTF-16 下标。
        start: i32,
        /// 按 Java `int` 回绕计算的结束下标。
        end: i32,
        /// 源字符串的 UTF-16 长度。
        length: usize,
    },
    /// 继承的 `Writer#append(CharSequence,int,int)` 范围非法。
    StringSubsequenceRange {
        /// 起始 UTF-16 下标。
        start: i32,
        /// 结束 UTF-16 下标。
        end: i32,
        /// 源字符序列的 UTF-16 长度。
        length: usize,
    },
    /// `write(char[],int,int)` 范围非法；Java 异常消息为 null。
    CharArrayRange,
    /// `char[]` 参数为 null，对应 Java 增强空指针消息。
    NullCharArray,
}

impl FastStringWriterError {
    /// 返回对应的 Java 异常全限定名。
    ///
    /// # 返回
    /// Java Golden 中 `Throwable#getClass().getName()` 的精确结果。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::NegativeBufferSize => "java.lang.IllegalArgumentException",
            Self::StringRange { .. } | Self::CharArrayRange => {
                "java.lang.IndexOutOfBoundsException"
            }
            Self::StringSubsequenceRange { .. } => "java.lang.StringIndexOutOfBoundsException",
            Self::NullCharArray => "java.lang.NullPointerException",
        }
    }

    /// 返回对应 Java 异常的可空消息。
    ///
    /// # 返回
    /// 有消息时返回独立 UTF-16 Java 字符串；`CharArrayRange` 返回 `None`，对应
    /// `Throwable#getMessage()` 为 null。
    #[must_use]
    pub fn message(&self) -> Option<JavaString> {
        match self {
            Self::NegativeBufferSize => Some(JavaString::from_rust_str("Negative buffer size")),
            Self::StringRange { start, end, length }
            | Self::StringSubsequenceRange { start, end, length } => {
                Some(JavaString::from_rust_str(&format!(
                    "Range [{start}, {end}) out of bounds for length {length}"
                )))
            }
            Self::CharArrayRange => None,
            Self::NullCharArray => Some(JavaString::from_rust_str(NULL_CHAR_ARRAY_MESSAGE)),
        }
    }
}

impl Display for FastStringWriterError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self.message() {
            Some(message) => formatter.write_str(&message.to_string_lossy()),
            None => formatter.write_str(NULL_TEXT),
        }
    }
}

impl Error for FastStringWriterError {}

/// 基于 UTF-16 缓冲区的高效字符串写入器。
///
/// 对应 Java: `org.thymeleaf.util.FastStringWriter`。
///
/// 本对象完整保留 Java `Writer`/`StringBuilder` 的可观察语义：写入 `int` 时只取低
/// 16 位；允许孤立代理项；null 字符串写入 `"null"`；字符串与字符数组重载采用各自
/// 的异常类型和检查顺序；`flush`/`close` 均不关闭缓冲区；每次 `toString` 都创建
/// 独立快照。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FastStringWriter {
    builder: Vec<u16>,
}

impl FastStringWriter {
    /// 使用 Java `StringBuilder()` 的默认容量语义创建写入器。
    ///
    /// 对应 Java: `FastStringWriter#FastStringWriter()`。
    ///
    /// # 返回
    /// 内容为空、可立即写入的独立写入器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            builder: Vec::with_capacity(16),
        }
    }

    /// 使用指定初始容量创建写入器。
    ///
    /// 对应 Java: `FastStringWriter#FastStringWriter(int)`。
    ///
    /// # 参数
    /// - `initial_size`：初始 UTF-16 代码单元容量。
    ///
    /// # 错误
    /// 负数返回与 Java `StringBuilder(int)` 一致的 `IllegalArgumentException`。
    pub fn with_initial_size(initial_size: i32) -> Result<Self, FastStringWriterError> {
        if initial_size < 0 {
            return Err(FastStringWriterError::NegativeBufferSize);
        }
        Ok(Self {
            builder: Vec::with_capacity(initial_size as usize),
        })
    }

    /// 写入 Java `Writer#write(int)` 的低 16 位。
    ///
    /// 对应 Java: `FastStringWriter#write(int)`。
    ///
    /// # 参数
    /// - `character`：Java `int`；转换为 `char` 时高位被截断。
    pub fn write_char(&mut self, character: i32) {
        self.builder.push(character as u16);
    }

    /// 写入完整 Java 字符串。
    ///
    /// 对应 Java: `FastStringWriter#write(String)`。
    ///
    /// # 参数
    /// - `string`：UTF-16 字符串；`None` 按 `StringBuilder#append(String)` 写入
    ///   `"null"`。
    pub fn write_string(&mut self, string: Option<&JavaString>) {
        self.builder.extend(java_string_or_null(string));
    }

    /// 写入 Java 字符串的 UTF-16 子范围。
    ///
    /// 对应 Java: `FastStringWriter#write(String,int,int)`。
    ///
    /// # 参数
    /// - `string`：源字符串；`None` 视为 `"null"`。
    /// - `offset`：起始 UTF-16 下标。
    /// - `length`：写入的 UTF-16 代码单元数。
    ///
    /// # 错误
    /// 范围非法时返回 `IndexOutOfBoundsException` 对应错误，结束下标使用 Java
    /// `int` 回绕加法。
    pub fn write_string_range(
        &mut self,
        string: Option<&JavaString>,
        offset: i32,
        length: i32,
    ) -> Result<(), FastStringWriterError> {
        let source = java_string_or_null(string);
        let end = offset.wrapping_add(length);
        let source_length = source.len();
        let invalid = offset < 0
            || offset > end
            || usize::try_from(end).map_or(true, |end| end > source_length);
        if invalid {
            return Err(FastStringWriterError::StringRange {
                start: offset,
                end,
                length: source_length,
            });
        }
        self.builder
            .extend_from_slice(&source[offset as usize..end as usize]);
        Ok(())
    }

    /// 写入完整 Java `char[]`。
    ///
    /// 对应 Java: `FastStringWriter#write(char[])`。
    ///
    /// # 参数
    /// - `characters`：UTF-16 代码单元数组；`None` 对应 Java null。
    ///
    /// # 错误
    /// null 数组返回与 Java 数组长度读取一致的 `NullPointerException`。
    pub fn write_chars(&mut self, characters: Option<&[u16]>) -> Result<(), FastStringWriterError> {
        let characters = characters.ok_or(FastStringWriterError::NullCharArray)?;
        self.builder.extend_from_slice(characters);
        Ok(())
    }

    /// 写入 Java `char[]` 的子范围。
    ///
    /// 对应 Java: `FastStringWriter#write(char[],int,int)`。
    ///
    /// # 参数
    /// - `characters`：源 UTF-16 数组；`None` 对应 Java null。
    /// - `offset`：起始数组下标。
    /// - `length`：写入代码单元数。
    ///
    /// # 错误
    /// 严格按 Java 条件短路顺序区分无消息的 `IndexOutOfBoundsException` 与读取
    /// null 数组长度产生的 `NullPointerException`。
    pub fn write_chars_range(
        &mut self,
        characters: Option<&[u16]>,
        offset: i32,
        length: i32,
    ) -> Result<(), FastStringWriterError> {
        if offset < 0 {
            return Err(FastStringWriterError::CharArrayRange);
        }
        let characters = characters.ok_or(FastStringWriterError::NullCharArray)?;
        let source_length = characters.len();
        let end = offset.wrapping_add(length);
        if offset as usize > source_length
            || length < 0
            || usize::try_from(end).map_or(true, |end| end > source_length)
            || end < 0
        {
            return Err(FastStringWriterError::CharArrayRange);
        }
        if length != 0 {
            self.builder
                .extend_from_slice(&characters[offset as usize..end as usize]);
        }
        Ok(())
    }

    /// 刷新写入器。
    ///
    /// 对应 Java: `FastStringWriter#flush()`。此内存写入器无需刷新，因此保持原样（no-op）。
    pub const fn flush(&mut self) {}

    /// 关闭写入器。
    ///
    /// 对应 Java: `FastStringWriter#close()`。Java 实现为空操作（no-op），关闭后仍可继续写入。
    pub const fn close(&mut self) {}

    /// 返回当前内容的独立 Java 字符串快照。
    ///
    /// 对应 Java: `FastStringWriter#toString()`。
    ///
    /// # 返回
    /// 包含当前全部 UTF-16 代码单元的新对象，后续写入不会改变该快照。
    #[must_use]
    pub fn to_string(&self) -> JavaString {
        JavaString::from_utf16(self.builder.clone())
    }

    /// 追加完整 Java `CharSequence` 并返回同一写入器。
    ///
    /// 对应 Java: 继承的 `Writer#append(CharSequence)`。
    ///
    /// # 参数
    /// - `sequence`：字符序列；`None` 按 Java 约定追加 `"null"`。
    ///
    /// # 返回
    /// `self` 的同一可变引用，保留链式调用身份。
    pub fn append_sequence(&mut self, sequence: Option<&JavaString>) -> &mut Self {
        self.write_string(sequence);
        self
    }

    /// 追加 Java `CharSequence` 的 UTF-16 子序列并返回同一写入器。
    ///
    /// 对应 Java: 继承的 `Writer#append(CharSequence,int,int)`。
    ///
    /// # 参数
    /// - `sequence`：字符序列；`None` 视为 `"null"`。
    /// - `start`：包含的起始 UTF-16 下标。
    /// - `end`：不包含的结束 UTF-16 下标。
    ///
    /// # 错误
    /// 非法范围返回 `StringIndexOutOfBoundsException` 对应错误。
    pub fn append_sequence_range(
        &mut self,
        sequence: Option<&JavaString>,
        start: i32,
        end: i32,
    ) -> Result<&mut Self, FastStringWriterError> {
        let source = java_string_or_null(sequence);
        let source_length = source.len();
        let invalid = start < 0
            || start > end
            || usize::try_from(end).map_or(true, |end| end > source_length);
        if invalid {
            return Err(FastStringWriterError::StringSubsequenceRange {
                start,
                end,
                length: source_length,
            });
        }
        self.builder
            .extend_from_slice(&source[start as usize..end as usize]);
        Ok(self)
    }

    /// 追加单个 Java UTF-16 `char` 并返回同一写入器。
    ///
    /// 对应 Java: 继承的 `Writer#append(char)`。
    ///
    /// # 参数
    /// - `character`：可为孤立代理项的 UTF-16 代码单元。
    ///
    /// # 返回
    /// `self` 的同一可变引用。
    pub fn append_char(&mut self, character: u16) -> &mut Self {
        self.builder.push(character);
        self
    }
}

impl Default for FastStringWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaWriter for FastStringWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> std::io::Result<()> {
        // 非 null 的完整切片不可能触发 Java 数组范围异常。
        self.write_chars(Some(characters))
            .map_err(std::io::Error::other)
    }
}

fn java_string_or_null(string: Option<&JavaString>) -> &[u16] {
    static NULL_UTF16: [u16; 4] = [b'n' as u16, b'u' as u16, b'l' as u16, b'l' as u16];
    string.map_or(&NULL_UTF16, JavaString::as_utf16)
}

#[cfg(test)]
mod tests {
    use super::{FastStringWriter, FastStringWriterError, JavaString};

    #[test]
    fn preserves_utf16_null_snapshot_and_lifecycle_semantics() {
        let mut writer = FastStringWriter::new();
        writer.write_char(i32::from(b'A'));
        writer.write_char(-1);
        writer.write_char(0x1_0000);
        writer.write_string(None);
        writer.append_char(0xD800);
        let snapshot = writer.to_string();
        writer.close();
        writer.flush();
        writer.write_string(Some(&JavaString::from_rust_str("tail")));

        assert_eq!(
            snapshot.as_utf16(),
            &[
                0x0041, 0xFFFF, 0x0000, 0x006E, 0x0075, 0x006C, 0x006C, 0xD800
            ]
        );
        assert_eq!(
            &writer.to_string().as_utf16()[snapshot.len()..],
            "tail".encode_utf16().collect::<Vec<_>>()
        );
    }

    #[test]
    fn preserves_overload_specific_error_class_message_and_short_circuit() {
        assert_eq!(
            FastStringWriter::with_initial_size(-1),
            Err(FastStringWriterError::NegativeBufferSize)
        );

        let mut writer = FastStringWriter::new();
        let string_error = writer
            .write_string_range(Some(&JavaString::from_rust_str("abc")), i32::MAX, 1)
            .expect_err("overflowing Java range");
        assert_eq!(
            string_error.java_class_name(),
            "java.lang.IndexOutOfBoundsException"
        );
        assert_eq!(
            string_error.message().expect("message").to_string_lossy(),
            "Range [2147483647, -2147483648) out of bounds for length 3"
        );

        assert_eq!(
            writer.write_chars_range(None, -1, 0),
            Err(FastStringWriterError::CharArrayRange)
        );
        assert_eq!(
            writer.write_chars_range(None, 0, -1),
            Err(FastStringWriterError::NullCharArray)
        );
        assert!(FastStringWriterError::CharArrayRange.message().is_none());
    }

    #[test]
    fn exposes_default_and_display_for_message_and_null_message_errors() {
        assert!(FastStringWriter::default().to_string().is_empty());
        assert_eq!(
            FastStringWriterError::NegativeBufferSize.to_string(),
            "Negative buffer size"
        );
        assert_eq!(FastStringWriterError::CharArrayRange.to_string(), "null");
    }
}
