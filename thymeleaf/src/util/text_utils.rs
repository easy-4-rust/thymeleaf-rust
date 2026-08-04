use std::error::Error;
use std::fmt::{Display, Formatter};

use super::{CharArrayWrapperSequence, HashCodeValue, TemplateWriter, Utf16String};

const CASE_MAP: &[u8] = include_bytes!("text_utils_case_map.bin");

const FIRST_TEXT_NULL: &str = "First text being compared cannot be null";
const SECOND_TEXT_NULL: &str = "Second text being compared cannot be null";
const FIRST_BUFFER_NULL: &str = "First text buffer being compared cannot be null";
const SECOND_BUFFER_NULL: &str = "Second text buffer being compared cannot be null";
const TEXT_NULL: &str = "Text cannot be null";
const PREFIX_NULL: &str = "Prefix cannot be null";
const SUFFIX_NULL: &str = "Suffix cannot be null";
const FRAGMENT_NULL: &str = "Fragment cannot be null";
const VALUES_NULL: &str = "Values array cannot be null";

/// `TextUtils` 执行失败时对应的 Java 运行时异常。
///
/// 对应 Java: `org.thymeleaf.util.TextUtils` 的显式参数异常、UTF-16 数组访问、
/// `String#charAt` 以及任意 `CharSequence` 实现传播的运行时异常。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextUtilsError {
    /// 上游显式抛出的 `IllegalArgumentException`。
    IllegalArgument {
        /// 精确 Java detail message。
        message: &'static str,
    },
    /// 对 null 数组或 `CharSequence` 隐式解引用产生的异常。
    NullPointer,
    /// Java `char[]` 或外层数组访问越界。
    ArrayIndexOutOfBounds {
        /// 被访问的 Java `int` 下标。
        index: i32,
        /// 数组长度。
        length: usize,
    },
    /// Java `String#charAt` 访问越界。
    StringIndexOutOfBounds {
        /// 被访问的 Java `int` 下标。
        index: i32,
        /// UTF-16 代码单元长度。
        length: usize,
    },
    /// 自定义 `CharSequence#length/charAt` 原样传播的运行时异常。
    SequenceAccess {
        /// Java 异常全限定名。
        class_name: String,
        /// Java detail message；null 映射为 `None`。
        message: Option<Utf16String>,
    },
}

impl TextUtilsError {
    /// 返回 Java 异常全限定名。
    ///
    /// # 返回
    /// 与上游调用路径对应的 `Throwable#getClass().getName()`。
    /// 对应 Java 语义：`TextUtils` 的 `class_name` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn class_name(&self) -> &str {
        match self {
            Self::IllegalArgument { .. } => "java.lang.IllegalArgumentException",
            Self::NullPointer => "java.lang.NullPointerException",
            Self::ArrayIndexOutOfBounds { .. } => "java.lang.ArrayIndexOutOfBoundsException",
            Self::StringIndexOutOfBounds { .. } => "java.lang.StringIndexOutOfBoundsException",
            Self::SequenceAccess { class_name, .. } => class_name,
        }
    }

    /// 返回 Java detail message。
    ///
    /// # 返回
    /// 显式或运行时消息的 UTF-16 副本；隐式 null 解引用返回 `None`。
    /// 对应 Java 语义：Java 接口/超类方法 `message()` 的 Rust 移植（`TextUtils` 继承路径）。
    #[must_use]
    pub fn message(&self) -> Option<Utf16String> {
        match self {
            Self::IllegalArgument { message } => Some(Utf16String::from_rust_str(message)),
            Self::NullPointer => None,
            Self::ArrayIndexOutOfBounds { index, length }
            | Self::StringIndexOutOfBounds { index, length } => Some(Utf16String::from_rust_str(
                &format!("Index {index} out of bounds for length {length}"),
            )),
            Self::SequenceAccess { message, .. } => message.clone(),
        }
    }
}

impl Display for TextUtilsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self.message() {
            Some(message) => formatter.write_str(&message.to_string_lossy()),
            None => Ok(()),
        }
    }
}

impl Error for TextUtilsError {}

/// Java `CharSequence` 的可失败 UTF-16 动态访问合同。
///
/// 对应 Java: `java.lang.CharSequence`。该 Rust 扩展接口保留 `length()` 与
/// `charAt(int)` 的调用次数、顺序、可变底层数据和运行时异常，避免将所有实现
/// 提前复制成不可变字符串。
pub trait CharSequenceValue: Send + Sync {
    /// 返回 Java 运行时类名；自定义适配器应覆盖为原对象的全限定类名。
    fn java_sequence_class_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// 调用 Java `CharSequence#length()`。
    ///
    /// # 返回
    /// Java `int` 长度，或实现抛出的运行时异常。
    fn java_length(&self) -> Result<i32, TextUtilsError>;

    /// 调用 Java `CharSequence#charAt(int)`。
    ///
    /// # 参数
    /// - `index`：UTF-16 代码单元下标。
    ///
    /// # 返回
    /// 单个 Java `char`，或实现抛出的运行时异常。
    fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError>;

    /// 暴露真实 Java `String` 快路径。
    ///
    /// # 返回
    /// 仅当实现语义就是 `java.lang.String` 时返回该字符串。
    fn as_utf16_string(&self) -> Option<&Utf16String>;

    /// 调用 Java `Object#toString()` 得到序列的字符串表示。
    ///
    /// 默认实现按当前 `length/charAt` 复制；自定义 Java `CharSequence` 若覆盖
    /// `toString()`，对应 Rust 适配器也必须覆盖此方法。
    fn java_to_string(&self) -> Result<Utf16String, TextUtilsError> {
        let length = self.java_length()?;
        self.java_sub_sequence(0, length)
    }

    /// 在对象同时对应 Java `IWritableCharSequence` 时直接写出。
    ///
    /// `None` 表示不具备该接口；`Some` 中的结果对应其 `write(Writer)` 调用。
    fn write_direct(&self, _writer: &mut dyn TemplateWriter) -> Option<std::io::Result<()>> {
        None
    }

    /// 返回 Java `Object#hashCode()`；覆盖过 hashCode 的适配器必须同步覆盖。
    fn java_sequence_hash_code(&self) -> Result<i32, TextUtilsError> {
        let address = self as *const Self as *const () as usize;
        Ok((address as u64 ^ ((address as u64) >> 32)) as i32)
    }

    /// 执行 Java `equals(Object)`；默认保留引用身份语义。
    fn java_sequence_equals(&self, other: &dyn CharSequenceValue) -> Result<bool, TextUtilsError> {
        Ok(std::ptr::eq(
            self as *const Self as *const (),
            other as *const dyn CharSequenceValue as *const (),
        ))
    }

    /// 调用 Java `CharSequence#subSequence(int, int)`。
    ///
    /// 默认实现逐 UTF-16 代码单元复制；具有不同异常或视图语义的自定义序列可以覆盖。
    fn java_sub_sequence(&self, start: i32, end: i32) -> Result<Utf16String, TextUtilsError> {
        let length = self.java_length()?;
        if start < 0 || end < start || end > length {
            return Err(TextUtilsError::StringIndexOutOfBounds {
                index: if start < 0 || start > length {
                    start
                } else {
                    end
                },
                length: usize::try_from(length).unwrap_or_default(),
            });
        }
        let mut value = Vec::with_capacity((end - start) as usize);
        for index in start..end {
            value.push(self.java_char_at(index)?);
        }
        Ok(Utf16String::from_utf16(value))
    }
}

impl CharSequenceValue for Utf16String {
    fn java_sequence_class_name(&self) -> &str {
        "java.lang.String"
    }

    fn java_length(&self) -> Result<i32, TextUtilsError> {
        Ok(self.len() as i32)
    }

    fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
        let index_usize =
            usize::try_from(index).map_err(|_| TextUtilsError::StringIndexOutOfBounds {
                index,
                length: self.len(),
            })?;
        self.as_utf16()
            .get(index_usize)
            .copied()
            .ok_or(TextUtilsError::StringIndexOutOfBounds {
                index,
                length: self.len(),
            })
    }

    fn as_utf16_string(&self) -> Option<&Utf16String> {
        Some(self)
    }

    fn java_to_string(&self) -> Result<Utf16String, TextUtilsError> {
        Ok(self.clone())
    }

    fn java_sequence_hash_code(&self) -> Result<i32, TextUtilsError> {
        Ok(self.java_hash_code())
    }

    fn java_sequence_equals(&self, other: &dyn CharSequenceValue) -> Result<bool, TextUtilsError> {
        Ok(other.as_utf16_string().is_some_and(|value| value == self))
    }

    fn java_sub_sequence(&self, start: i32, end: i32) -> Result<Utf16String, TextUtilsError> {
        let start = usize::try_from(start).map_err(|_| TextUtilsError::StringIndexOutOfBounds {
            index: start,
            length: self.len(),
        })?;
        let end = usize::try_from(end).map_err(|_| TextUtilsError::StringIndexOutOfBounds {
            index: end,
            length: self.len(),
        })?;
        if start > end || end > self.len() {
            return Err(TextUtilsError::StringIndexOutOfBounds {
                index: if start > self.len() {
                    start as i32
                } else {
                    end as i32
                },
                length: self.len(),
            });
        }
        Ok(Utf16String::from_utf16(
            self.as_utf16()[start..end].to_vec(),
        ))
    }
}

impl CharSequenceValue for CharArrayWrapperSequence {
    fn java_length(&self) -> Result<i32, TextUtilsError> {
        Ok(self.length())
    }

    fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
        self.char_at(index)
            .map_err(|error| TextUtilsError::SequenceAccess {
                class_name: error.class_name().to_owned(),
                message: Some(error.message()),
            })
    }

    fn as_utf16_string(&self) -> Option<&Utf16String> {
        None
    }
}

/// 不分配字符串即可比较、搜索和散列 Java UTF-16 文本的工具对象。
///
/// 对应 Java: `org.thymeleaf.util.TextUtils`。
///
/// 所有算法保留 Java `int` 回绕、UTF-16 `char` 顺序、重载的参数求值顺序、
/// JDK 21 `Character` 单码元大小写映射和动态 `CharSequence` 调用。
pub struct TextUtils {
    _private: (),
}

impl TextUtils {
    /// 比较两个完整 `CharSequence`。
    ///
    /// # 参数
    /// `case_sensitive` 控制大小写；两个文本的 `None` 对应 Java null。
    /// # 返回
    /// 完整 UTF-16 内容是否相等。
    pub fn equals_sequences(
        case_sensitive: bool,
        text1: Option<&dyn CharSequenceValue>,
        text2: Option<&dyn CharSequenceValue>,
    ) -> Result<bool, TextUtilsError> {
        let text1 = require_sequence(text1, FIRST_TEXT_NULL)?;
        let text2 = require_sequence(text2, SECOND_TEXT_NULL)?;
        if case_sensitive
            && let (Some(left), Some(right)) = (text1.as_utf16_string(), text2.as_utf16_string())
        {
            return Ok(left.as_utf16() == right.as_utf16());
        }
        let text1_len = text1.java_length()?;
        let text2_len = text2.java_length()?;
        Self::equals_sequences_range(
            case_sensitive,
            Some(text1),
            0,
            text1_len,
            Some(text2),
            0,
            text2_len,
        )
    }

    /// 比较完整 `CharSequence` 与完整 `char[]`。
    ///
    /// # 参数
    /// 参数与 Java 同名重载一致。
    /// # 返回
    /// 两段 UTF-16 内容是否相等。
    /// 对应 Java 语义：`TextUtils` 的 `equals_sequence_and_chars` 行为（Rust 侧辅助/私有路径）。
    pub fn equals_sequence_and_chars(
        case_sensitive: bool,
        text1: Option<&dyn CharSequenceValue>,
        text2: Option<&[u16]>,
    ) -> Result<bool, TextUtilsError> {
        let text1_len = implicit_sequence_length(text1)?;
        let text2_len = implicit_array_length(text2)?;
        Self::equals_sequence_and_chars_range(
            case_sensitive,
            text1,
            0,
            text1_len,
            text2,
            0,
            text2_len,
        )
    }

    /// 比较两个完整 `char[]`。
    ///
    /// # 参数
    /// `None` 保留 Java 数组长度读取产生的 NPE 顺序。
    /// # 返回
    /// 两个数组内容是否相等。
    /// 对应 Java 语义：`TextUtils` 的 `equals_chars` 行为（Rust 侧辅助/私有路径）。
    pub fn equals_chars(
        case_sensitive: bool,
        text1: Option<&[u16]>,
        text2: Option<&[u16]>,
    ) -> Result<bool, TextUtilsError> {
        let text1_len = implicit_array_length(text1)?;
        let text2_len = implicit_array_length(text2)?;
        Self::equals_chars_range(case_sensitive, text1, 0, text1_len, text2, 0, text2_len)
    }

    /// 比较两个 `char[]` 范围。
    ///
    /// # 参数
    /// offset/len 均按 Java `int` 和 UTF-16 代码单元解释。
    /// # 返回
    /// 两个范围是否相等。
    /// 对应 Java 语义：`TextUtils` 的 `equals_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn equals_chars_range(
        case_sensitive: bool,
        text1: Option<&[u16]>,
        text1_offset: i32,
        text1_len: i32,
        text2: Option<&[u16]>,
        text2_offset: i32,
        text2_len: i32,
    ) -> Result<bool, TextUtilsError> {
        let text1 = require_chars(text1, FIRST_BUFFER_NULL)?;
        let text2 = require_chars(text2, SECOND_BUFFER_NULL)?;
        if text1_len != text2_len {
            return Ok(false);
        }
        if std::ptr::eq(text1, text2) && text1_offset == text2_offset && text1_len == text2_len {
            return Ok(true);
        }
        equals_core(
            case_sensitive,
            TextRef::Chars(text1),
            text1_offset,
            TextRef::Chars(text2),
            text2_offset,
            text1_len,
        )
    }

    /// 比较 `CharSequence` 与 `char[]` 范围。
    ///
    /// # 参数
    /// offset/len 保留 Java 边界和访问顺序。
    /// # 返回
    /// 两个范围是否相等。
    /// 对应 Java 语义：`TextUtils` 的 `equals_sequence_and_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn equals_sequence_and_chars_range(
        case_sensitive: bool,
        text1: Option<&dyn CharSequenceValue>,
        text1_offset: i32,
        text1_len: i32,
        text2: Option<&[u16]>,
        text2_offset: i32,
        text2_len: i32,
    ) -> Result<bool, TextUtilsError> {
        let text1 = require_sequence(text1, FIRST_TEXT_NULL)?;
        let text2 = require_chars(text2, SECOND_BUFFER_NULL)?;
        if text1_len != text2_len {
            return Ok(false);
        }
        equals_core(
            case_sensitive,
            TextRef::Sequence(text1),
            text1_offset,
            TextRef::Chars(text2),
            text2_offset,
            text1_len,
        )
    }

    /// 比较两个 `CharSequence` 范围。
    ///
    /// # 参数
    /// offset/len 按上游原样执行，不预先归一化。
    /// # 返回
    /// 两个范围是否相等。
    /// 对应 Java 语义：`TextUtils` 的 `equals_sequences_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn equals_sequences_range(
        case_sensitive: bool,
        text1: Option<&dyn CharSequenceValue>,
        text1_offset: i32,
        text1_len: i32,
        text2: Option<&dyn CharSequenceValue>,
        text2_offset: i32,
        text2_len: i32,
    ) -> Result<bool, TextUtilsError> {
        let text1 = require_sequence(text1, FIRST_TEXT_NULL)?;
        let text2 = require_sequence(text2, SECOND_TEXT_NULL)?;
        if text1_len != text2_len {
            return Ok(false);
        }
        if std::ptr::eq(text1, text2) && text1_offset == text2_offset && text1_len == text2_len {
            return Ok(true);
        }
        equals_core(
            case_sensitive,
            TextRef::Sequence(text1),
            text1_offset,
            TextRef::Sequence(text2),
            text2_offset,
            text1_len,
        )
    }

    /// 判断完整 `CharSequence` 是否以另一序列开头。
    ///
    /// # 参数
    /// `case_sensitive` 与两个可空序列对应 Java 参数。
    /// # 返回
    /// 是否匹配指定前缀。
    pub fn starts_with_sequences(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        prefix: Option<&dyn CharSequenceValue>,
    ) -> Result<bool, TextUtilsError> {
        let text = require_sequence(text, TEXT_NULL)?;
        let prefix = require_sequence(prefix, PREFIX_NULL)?;
        if case_sensitive
            && let (Some(text), Some(prefix)) = (text.as_utf16_string(), prefix.as_utf16_string())
        {
            return Ok(text.as_utf16().starts_with(prefix.as_utf16()));
        }
        let text_len = text.java_length()?;
        let prefix_len = prefix.java_length()?;
        Self::starts_with_sequences_range(
            case_sensitive,
            Some(text),
            0,
            text_len,
            Some(prefix),
            0,
            prefix_len,
        )
    }

    /// 判断完整序列是否以完整 `char[]` 开头。
    ///
    /// # 参数
    /// 参数求值顺序与 Java 委托重载一致。
    /// # 返回
    /// 是否匹配指定前缀。
    /// 对应 Java 语义：`TextUtils` 的 `starts_with_sequence_and_chars` 行为（Rust 侧辅助/私有路径）。
    pub fn starts_with_sequence_and_chars(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        prefix: Option<&[u16]>,
    ) -> Result<bool, TextUtilsError> {
        let text_len = implicit_sequence_length(text)?;
        let prefix_len = implicit_array_length(prefix)?;
        Self::starts_with_sequence_and_chars_range(
            case_sensitive,
            text,
            0,
            text_len,
            prefix,
            0,
            prefix_len,
        )
    }

    /// 判断完整 `char[]` 是否以另一数组开头。
    ///
    /// # 参数
    /// 两个数组均可用 `None` 表达 Java null。
    /// # 返回
    /// 是否匹配指定前缀。
    /// 对应 Java 语义：`TextUtils` 的 `starts_with_chars` 行为（Rust 侧辅助/私有路径）。
    pub fn starts_with_chars(
        case_sensitive: bool,
        text: Option<&[u16]>,
        prefix: Option<&[u16]>,
    ) -> Result<bool, TextUtilsError> {
        let text_len = implicit_array_length(text)?;
        let prefix_len = implicit_array_length(prefix)?;
        Self::starts_with_chars_range(case_sensitive, text, 0, text_len, prefix, 0, prefix_len)
    }

    /// 判断两个 `char[]` 范围的前缀关系。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否以 `prefix` 范围开头。
    /// 对应 Java 语义：`TextUtils` 的 `starts_with_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn starts_with_chars_range(
        case_sensitive: bool,
        text: Option<&[u16]>,
        text_offset: i32,
        text_len: i32,
        prefix: Option<&[u16]>,
        prefix_offset: i32,
        prefix_len: i32,
    ) -> Result<bool, TextUtilsError> {
        starts_with_validated(
            case_sensitive,
            TextRef::Chars(require_chars(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Chars(require_chars(prefix, PREFIX_NULL)?),
            prefix_offset,
            prefix_len,
        )
    }

    /// 判断序列与数组范围的前缀关系。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否以 `prefix` 范围开头。
    /// 对应 Java 语义：`TextUtils` 的 `starts_with_sequence_and_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn starts_with_sequence_and_chars_range(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        text_offset: i32,
        text_len: i32,
        prefix: Option<&[u16]>,
        prefix_offset: i32,
        prefix_len: i32,
    ) -> Result<bool, TextUtilsError> {
        starts_with_validated(
            case_sensitive,
            TextRef::Sequence(require_sequence(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Chars(require_chars(prefix, PREFIX_NULL)?),
            prefix_offset,
            prefix_len,
        )
    }

    /// 判断数组与序列范围的前缀关系。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否以 `prefix` 范围开头。
    /// 对应 Java 语义：`TextUtils` 的 `starts_with_chars_and_sequence_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn starts_with_chars_and_sequence_range(
        case_sensitive: bool,
        text: Option<&[u16]>,
        text_offset: i32,
        text_len: i32,
        prefix: Option<&dyn CharSequenceValue>,
        prefix_offset: i32,
        prefix_len: i32,
    ) -> Result<bool, TextUtilsError> {
        starts_with_validated(
            case_sensitive,
            TextRef::Chars(require_chars(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Sequence(require_sequence(prefix, PREFIX_NULL)?),
            prefix_offset,
            prefix_len,
        )
    }

    /// 判断两个序列范围的前缀关系。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否以 `prefix` 范围开头。
    /// 对应 Java 语义：`TextUtils` 的 `starts_with_sequences_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn starts_with_sequences_range(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        text_offset: i32,
        text_len: i32,
        prefix: Option<&dyn CharSequenceValue>,
        prefix_offset: i32,
        prefix_len: i32,
    ) -> Result<bool, TextUtilsError> {
        starts_with_validated(
            case_sensitive,
            TextRef::Sequence(require_sequence(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Sequence(require_sequence(prefix, PREFIX_NULL)?),
            prefix_offset,
            prefix_len,
        )
    }
}

impl TextUtils {
    /// 判断完整 `CharSequence` 是否以另一序列结尾。
    ///
    /// # 参数
    /// `case_sensitive` 与两个可空序列对应 Java 参数。
    /// # 返回
    /// 是否匹配指定后缀。
    pub fn ends_with_sequences(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        suffix: Option<&dyn CharSequenceValue>,
    ) -> Result<bool, TextUtilsError> {
        let text = require_sequence(text, TEXT_NULL)?;
        let suffix = require_sequence(suffix, SUFFIX_NULL)?;
        if case_sensitive
            && let (Some(text), Some(suffix)) = (text.as_utf16_string(), suffix.as_utf16_string())
        {
            return Ok(text.as_utf16().ends_with(suffix.as_utf16()));
        }
        let text_len = text.java_length()?;
        let suffix_len = suffix.java_length()?;
        Self::ends_with_sequences_range(
            case_sensitive,
            Some(text),
            0,
            text_len,
            Some(suffix),
            0,
            suffix_len,
        )
    }

    /// 判断完整序列是否以完整 `char[]` 结尾。
    ///
    /// # 参数
    /// 参数求值顺序与 Java 委托重载一致。
    /// # 返回
    /// 是否匹配指定后缀。
    /// 对应 Java 语义：`TextUtils` 的 `ends_with_sequence_and_chars` 行为（Rust 侧辅助/私有路径）。
    pub fn ends_with_sequence_and_chars(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        suffix: Option<&[u16]>,
    ) -> Result<bool, TextUtilsError> {
        let text_len = implicit_sequence_length(text)?;
        let suffix_len = implicit_array_length(suffix)?;
        Self::ends_with_sequence_and_chars_range(
            case_sensitive,
            text,
            0,
            text_len,
            suffix,
            0,
            suffix_len,
        )
    }

    /// 判断完整 `char[]` 是否以另一数组结尾。
    ///
    /// # 参数
    /// 两个数组均可用 `None` 表达 Java null。
    /// # 返回
    /// 是否匹配指定后缀。
    /// 对应 Java 语义：`TextUtils` 的 `ends_with_chars` 行为（Rust 侧辅助/私有路径）。
    pub fn ends_with_chars(
        case_sensitive: bool,
        text: Option<&[u16]>,
        suffix: Option<&[u16]>,
    ) -> Result<bool, TextUtilsError> {
        let text_len = implicit_array_length(text)?;
        let suffix_len = implicit_array_length(suffix)?;
        Self::ends_with_chars_range(case_sensitive, text, 0, text_len, suffix, 0, suffix_len)
    }

    /// 判断两个 `char[]` 范围的后缀关系。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否以 `suffix` 范围结尾。
    /// 对应 Java 语义：`TextUtils` 的 `ends_with_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn ends_with_chars_range(
        case_sensitive: bool,
        text: Option<&[u16]>,
        text_offset: i32,
        text_len: i32,
        suffix: Option<&[u16]>,
        suffix_offset: i32,
        suffix_len: i32,
    ) -> Result<bool, TextUtilsError> {
        ends_with_validated(
            case_sensitive,
            TextRef::Chars(require_chars(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Chars(require_chars(suffix, SUFFIX_NULL)?),
            suffix_offset,
            suffix_len,
        )
    }

    /// 判断序列与数组范围的后缀关系。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否以 `suffix` 范围结尾。
    /// 对应 Java 语义：`TextUtils` 的 `ends_with_sequence_and_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn ends_with_sequence_and_chars_range(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        text_offset: i32,
        text_len: i32,
        suffix: Option<&[u16]>,
        suffix_offset: i32,
        suffix_len: i32,
    ) -> Result<bool, TextUtilsError> {
        ends_with_validated(
            case_sensitive,
            TextRef::Sequence(require_sequence(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Chars(require_chars(suffix, SUFFIX_NULL)?),
            suffix_offset,
            suffix_len,
        )
    }

    /// 判断数组与序列范围的后缀关系。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否以 `suffix` 范围结尾。
    /// 对应 Java 语义：`TextUtils` 的 `ends_with_chars_and_sequence_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn ends_with_chars_and_sequence_range(
        case_sensitive: bool,
        text: Option<&[u16]>,
        text_offset: i32,
        text_len: i32,
        suffix: Option<&dyn CharSequenceValue>,
        suffix_offset: i32,
        suffix_len: i32,
    ) -> Result<bool, TextUtilsError> {
        ends_with_validated(
            case_sensitive,
            TextRef::Chars(require_chars(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Sequence(require_sequence(suffix, SUFFIX_NULL)?),
            suffix_offset,
            suffix_len,
        )
    }

    /// 判断两个序列范围的后缀关系。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否以 `suffix` 范围结尾。
    /// 对应 Java 语义：`TextUtils` 的 `ends_with_sequences_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn ends_with_sequences_range(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        text_offset: i32,
        text_len: i32,
        suffix: Option<&dyn CharSequenceValue>,
        suffix_offset: i32,
        suffix_len: i32,
    ) -> Result<bool, TextUtilsError> {
        ends_with_validated(
            case_sensitive,
            TextRef::Sequence(require_sequence(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Sequence(require_sequence(suffix, SUFFIX_NULL)?),
            suffix_offset,
            suffix_len,
        )
    }

    /// 判断完整 `CharSequence` 是否包含另一序列。
    ///
    /// # 参数
    /// `case_sensitive` 与两个可空序列对应 Java 参数。
    /// # 返回
    /// 是否包含指定片段。
    pub fn contains_sequences(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        fragment: Option<&dyn CharSequenceValue>,
    ) -> Result<bool, TextUtilsError> {
        let text = require_sequence(text, TEXT_NULL)?;
        let fragment = require_sequence(fragment, FRAGMENT_NULL)?;
        if case_sensitive
            && let (Some(text), Some(fragment)) =
                (text.as_utf16_string(), fragment.as_utf16_string())
        {
            return Ok(slice_contains(text.as_utf16(), fragment.as_utf16()));
        }
        let text_len = text.java_length()?;
        let fragment_len = fragment.java_length()?;
        Self::contains_sequences_range(
            case_sensitive,
            Some(text),
            0,
            text_len,
            Some(fragment),
            0,
            fragment_len,
        )
    }

    /// 判断完整序列是否包含完整 `char[]`。
    ///
    /// # 参数
    /// 参数求值顺序与 Java 委托重载一致。
    /// # 返回
    /// 是否包含指定片段。
    /// 对应 Java 语义：`TextUtils` 的 `contains_sequence_and_chars` 行为（Rust 侧辅助/私有路径）。
    pub fn contains_sequence_and_chars(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        fragment: Option<&[u16]>,
    ) -> Result<bool, TextUtilsError> {
        let text_len = implicit_sequence_length(text)?;
        let fragment_len = implicit_array_length(fragment)?;
        Self::contains_sequence_and_chars_range(
            case_sensitive,
            text,
            0,
            text_len,
            fragment,
            0,
            fragment_len,
        )
    }

    /// 判断完整 `char[]` 是否包含另一数组。
    ///
    /// # 参数
    /// 两个数组均可用 `None` 表达 Java null。
    /// # 返回
    /// 是否包含指定片段。
    /// 对应 Java 语义：`TextUtils` 的 `contains_chars` 行为（Rust 侧辅助/私有路径）。
    pub fn contains_chars(
        case_sensitive: bool,
        text: Option<&[u16]>,
        fragment: Option<&[u16]>,
    ) -> Result<bool, TextUtilsError> {
        let text_len = implicit_array_length(text)?;
        let fragment_len = implicit_array_length(fragment)?;
        Self::contains_chars_range(case_sensitive, text, 0, text_len, fragment, 0, fragment_len)
    }

    /// 在两个 `char[]` 范围间执行包含搜索。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否包含 `fragment` 范围。
    /// 对应 Java 语义：`TextUtils` 的 `contains_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn contains_chars_range(
        case_sensitive: bool,
        text: Option<&[u16]>,
        text_offset: i32,
        text_len: i32,
        fragment: Option<&[u16]>,
        fragment_offset: i32,
        fragment_len: i32,
    ) -> Result<bool, TextUtilsError> {
        contains_validated(
            case_sensitive,
            TextRef::Chars(require_chars(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Chars(require_chars(fragment, FRAGMENT_NULL)?),
            fragment_offset,
            fragment_len,
        )
    }

    /// 在序列与数组范围间执行包含搜索。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否包含 `fragment` 范围。
    /// 对应 Java 语义：`TextUtils` 的 `contains_sequence_and_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn contains_sequence_and_chars_range(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        text_offset: i32,
        text_len: i32,
        fragment: Option<&[u16]>,
        fragment_offset: i32,
        fragment_len: i32,
    ) -> Result<bool, TextUtilsError> {
        contains_validated(
            case_sensitive,
            TextRef::Sequence(require_sequence(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Chars(require_chars(fragment, FRAGMENT_NULL)?),
            fragment_offset,
            fragment_len,
        )
    }

    /// 在数组与序列范围间执行包含搜索。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否包含 `fragment` 范围。
    /// 对应 Java 语义：`TextUtils` 的 `contains_chars_and_sequence_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn contains_chars_and_sequence_range(
        case_sensitive: bool,
        text: Option<&[u16]>,
        text_offset: i32,
        text_len: i32,
        fragment: Option<&dyn CharSequenceValue>,
        fragment_offset: i32,
        fragment_len: i32,
    ) -> Result<bool, TextUtilsError> {
        contains_validated(
            case_sensitive,
            TextRef::Chars(require_chars(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Sequence(require_sequence(fragment, FRAGMENT_NULL)?),
            fragment_offset,
            fragment_len,
        )
    }

    /// 在两个序列范围间执行包含搜索。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// `text` 范围是否包含 `fragment` 范围。
    /// 对应 Java 语义：`TextUtils` 的 `contains_sequences_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn contains_sequences_range(
        case_sensitive: bool,
        text: Option<&dyn CharSequenceValue>,
        text_offset: i32,
        text_len: i32,
        fragment: Option<&dyn CharSequenceValue>,
        fragment_offset: i32,
        fragment_len: i32,
    ) -> Result<bool, TextUtilsError> {
        contains_validated(
            case_sensitive,
            TextRef::Sequence(require_sequence(text, TEXT_NULL)?),
            text_offset,
            text_len,
            TextRef::Sequence(require_sequence(fragment, FRAGMENT_NULL)?),
            fragment_offset,
            fragment_len,
        )
    }

    /// 按 Java 字符顺序比较两个完整序列。
    ///
    /// # 参数
    /// `case_sensitive` 与两个可空序列对应 Java 参数。
    /// # 返回
    /// 精确字符差值或长度差值。
    pub fn compare_sequences(
        case_sensitive: bool,
        text1: Option<&dyn CharSequenceValue>,
        text2: Option<&dyn CharSequenceValue>,
    ) -> Result<i32, TextUtilsError> {
        let text1 = require_sequence(text1, FIRST_TEXT_NULL)?;
        let text2 = require_sequence(text2, SECOND_TEXT_NULL)?;
        let text1_len = text1.java_length()?;
        let text2_len = text2.java_length()?;
        Self::compare_sequences_range(
            case_sensitive,
            Some(text1),
            0,
            text1_len,
            Some(text2),
            0,
            text2_len,
        )
    }

    /// 比较完整序列与完整 `char[]`。
    ///
    /// # 参数
    /// 参数求值顺序与 Java 委托重载一致。
    /// # 返回
    /// 精确字符差值或长度差值。
    /// 对应 Java 语义：`TextUtils` 的 `compare_sequence_and_chars` 行为（Rust 侧辅助/私有路径）。
    pub fn compare_sequence_and_chars(
        case_sensitive: bool,
        text1: Option<&dyn CharSequenceValue>,
        text2: Option<&[u16]>,
    ) -> Result<i32, TextUtilsError> {
        let text1_len = implicit_sequence_length(text1)?;
        let text2_len = implicit_array_length(text2)?;
        Self::compare_sequence_and_chars_range(
            case_sensitive,
            text1,
            0,
            text1_len,
            text2,
            0,
            text2_len,
        )
    }

    /// 比较两个完整 `char[]`。
    ///
    /// # 参数
    /// 两个数组均可用 `None` 表达 Java null。
    /// # 返回
    /// 精确字符差值或长度差值。
    /// 对应 Java 语义：`TextUtils` 的 `compare_chars` 行为（Rust 侧辅助/私有路径）。
    pub fn compare_chars(
        case_sensitive: bool,
        text1: Option<&[u16]>,
        text2: Option<&[u16]>,
    ) -> Result<i32, TextUtilsError> {
        let text1_len = implicit_array_length(text1)?;
        let text2_len = implicit_array_length(text2)?;
        Self::compare_chars_range(case_sensitive, text1, 0, text1_len, text2, 0, text2_len)
    }

    /// 比较两个 `char[]` 范围。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// 精确字符差值或长度差值。
    /// 对应 Java 语义：`TextUtils` 的 `compare_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn compare_chars_range(
        case_sensitive: bool,
        text1: Option<&[u16]>,
        text1_offset: i32,
        text1_len: i32,
        text2: Option<&[u16]>,
        text2_offset: i32,
        text2_len: i32,
    ) -> Result<i32, TextUtilsError> {
        let text1 = require_chars(text1, FIRST_BUFFER_NULL)?;
        let text2 = require_chars(text2, SECOND_BUFFER_NULL)?;
        if std::ptr::eq(text1, text2) && text1_offset == text2_offset && text1_len == text2_len {
            return Ok(0);
        }
        compare_core(
            case_sensitive,
            TextRef::Chars(text1),
            text1_offset,
            text1_len,
            TextRef::Chars(text2),
            text2_offset,
            text2_len,
        )
    }

    /// 比较序列与数组范围。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// 精确字符差值或长度差值。
    /// 对应 Java 语义：`TextUtils` 的 `compare_sequence_and_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn compare_sequence_and_chars_range(
        case_sensitive: bool,
        text1: Option<&dyn CharSequenceValue>,
        text1_offset: i32,
        text1_len: i32,
        text2: Option<&[u16]>,
        text2_offset: i32,
        text2_len: i32,
    ) -> Result<i32, TextUtilsError> {
        compare_core(
            case_sensitive,
            TextRef::Sequence(require_sequence(text1, FIRST_TEXT_NULL)?),
            text1_offset,
            text1_len,
            TextRef::Chars(require_chars(text2, SECOND_BUFFER_NULL)?),
            text2_offset,
            text2_len,
        )
    }

    /// 比较两个序列范围。
    ///
    /// # 参数
    /// offset/len 使用 Java `int`。
    /// # 返回
    /// 精确字符差值或长度差值。
    /// 对应 Java 语义：`TextUtils` 的 `compare_sequences_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn compare_sequences_range(
        case_sensitive: bool,
        text1: Option<&dyn CharSequenceValue>,
        text1_offset: i32,
        text1_len: i32,
        text2: Option<&dyn CharSequenceValue>,
        text2_offset: i32,
        text2_len: i32,
    ) -> Result<i32, TextUtilsError> {
        let text1 = require_sequence(text1, FIRST_TEXT_NULL)?;
        let text2 = require_sequence(text2, SECOND_TEXT_NULL)?;
        if std::ptr::eq(text1, text2) && text1_offset == text2_offset && text1_len == text2_len {
            return Ok(0);
        }
        compare_core(
            case_sensitive,
            TextRef::Sequence(text1),
            text1_offset,
            text1_len,
            TextRef::Sequence(text2),
            text2_offset,
            text2_len,
        )
    }
}

impl TextUtils {
    /// 在有序 `char[][]` 中搜索 `char[]` 范围。
    ///
    /// # 参数
    /// `values` 是完整有序数组；text offset/len 指定搜索键。
    /// # 返回
    /// 命中下标或 Java `-(insertion point)-1`。
    /// 对应 Java 语义：`TextUtils` 的 `binary_search_chars_values_and_chars` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn binary_search_chars_values_and_chars(
        case_sensitive: bool,
        values: Option<&[Option<&[u16]>]>,
        text: Option<&[u16]>,
        text_offset: i32,
        text_len: i32,
    ) -> Result<i32, TextUtilsError> {
        let values = require_char_values(values)?;
        Self::binary_search_chars_values_and_chars_range(
            case_sensitive,
            Some(values),
            0,
            values.len() as i32,
            text,
            text_offset,
            text_len,
        )
    }

    /// 在有序 `char[][]` 中搜索 `CharSequence` 范围。
    ///
    /// # 参数
    /// `values` 是完整有序数组；text offset/len 指定搜索键。
    /// # 返回
    /// 命中下标或 Java `-(insertion point)-1`。
    /// 对应 Java 语义：`TextUtils` 的 `binary_search_chars_values_and_sequence` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn binary_search_chars_values_and_sequence(
        case_sensitive: bool,
        values: Option<&[Option<&[u16]>]>,
        text: Option<&dyn CharSequenceValue>,
        text_offset: i32,
        text_len: i32,
    ) -> Result<i32, TextUtilsError> {
        let values = require_char_values(values)?;
        Self::binary_search_chars_values_and_sequence_range(
            case_sensitive,
            Some(values),
            0,
            values.len() as i32,
            text,
            text_offset,
            text_len,
        )
    }

    /// 在有序 `CharSequence[]` 中搜索 `char[]` 范围。
    ///
    /// # 参数
    /// `values` 是完整有序数组；text offset/len 指定搜索键。
    /// # 返回
    /// 命中下标或 Java `-(insertion point)-1`。
    /// 对应 Java 语义：`TextUtils` 的 `binary_search_sequence_values_and_chars` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn binary_search_sequence_values_and_chars(
        case_sensitive: bool,
        values: Option<&[Option<&dyn CharSequenceValue>]>,
        text: Option<&[u16]>,
        text_offset: i32,
        text_len: i32,
    ) -> Result<i32, TextUtilsError> {
        let values = require_sequence_values(values)?;
        Self::binary_search_sequence_values_and_chars_range(
            case_sensitive,
            Some(values),
            0,
            values.len() as i32,
            text,
            text_offset,
            text_len,
        )
    }

    /// 在有序 `CharSequence[]` 中搜索另一序列范围。
    ///
    /// # 参数
    /// `values` 是完整有序数组；text offset/len 指定搜索键。
    /// # 返回
    /// 命中下标或 Java `-(insertion point)-1`。
    /// 对应 Java 语义：`TextUtils` 的 `binary_search_sequence_values_and_sequence` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn binary_search_sequence_values_and_sequence(
        case_sensitive: bool,
        values: Option<&[Option<&dyn CharSequenceValue>]>,
        text: Option<&dyn CharSequenceValue>,
        text_offset: i32,
        text_len: i32,
    ) -> Result<i32, TextUtilsError> {
        let values = require_sequence_values(values)?;
        Self::binary_search_sequence_values_and_sequence_range(
            case_sensitive,
            Some(values),
            0,
            values.len() as i32,
            text,
            text_offset,
            text_len,
        )
    }

    /// 在 `char[][]` 指定范围中搜索 `char[]` 范围。
    ///
    /// # 参数
    /// 两组 offset/len 均按 Java `int` 回绕。
    /// # 返回
    /// 命中下标或精确插入点编码。
    /// 对应 Java 语义：`TextUtils` 的 `binary_search_chars_values_and_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn binary_search_chars_values_and_chars_range(
        case_sensitive: bool,
        values: Option<&[Option<&[u16]>]>,
        values_offset: i32,
        values_len: i32,
        text: Option<&[u16]>,
        text_offset: i32,
        text_len: i32,
    ) -> Result<i32, TextUtilsError> {
        let values = require_char_values(values)?;
        let text = require_chars(text, TEXT_NULL)?;
        let mut low = values_offset;
        let mut high = values_offset.wrapping_add(values_len).wrapping_sub(1);
        while low <= high {
            let mid = unsigned_midpoint(low, high);
            let mid_value = char_value_at(values, mid)?;
            let mid_value = mid_value.ok_or(TextUtilsError::NullPointer)?;
            let comparison = Self::compare_chars_range(
                case_sensitive,
                Some(mid_value),
                0,
                mid_value.len() as i32,
                Some(text),
                text_offset,
                text_len,
            )?;
            if comparison < 0 {
                low = mid.wrapping_add(1);
            } else if comparison > 0 {
                high = mid.wrapping_sub(1);
            } else {
                return Ok(mid);
            }
        }
        Ok(low.wrapping_add(1).wrapping_neg())
    }

    /// 在 `char[][]` 指定范围中搜索序列范围。
    ///
    /// # 参数
    /// 两组 offset/len 均按 Java `int` 回绕。
    /// # 返回
    /// 命中下标或精确插入点编码。
    /// 对应 Java 语义：`TextUtils` 的 `binary_search_chars_values_and_sequence_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn binary_search_chars_values_and_sequence_range(
        case_sensitive: bool,
        values: Option<&[Option<&[u16]>]>,
        values_offset: i32,
        values_len: i32,
        text: Option<&dyn CharSequenceValue>,
        text_offset: i32,
        text_len: i32,
    ) -> Result<i32, TextUtilsError> {
        let values = require_char_values(values)?;
        let text = require_sequence(text, TEXT_NULL)?;
        let mut low = values_offset;
        let mut high = values_offset.wrapping_add(values_len).wrapping_sub(1);
        while low <= high {
            let mid = unsigned_midpoint(low, high);
            let mid_value = char_value_at(values, mid)?;
            let mid_value = mid_value.ok_or(TextUtilsError::NullPointer)?;
            let comparison = Self::compare_sequence_and_chars_range(
                case_sensitive,
                Some(text),
                text_offset,
                text_len,
                Some(mid_value),
                0,
                mid_value.len() as i32,
            )?;
            if comparison > 0 {
                low = mid.wrapping_add(1);
            } else if comparison < 0 {
                high = mid.wrapping_sub(1);
            } else {
                return Ok(mid);
            }
        }
        Ok(low.wrapping_add(1).wrapping_neg())
    }

    /// 在 `CharSequence[]` 指定范围中搜索 `char[]` 范围。
    ///
    /// # 参数
    /// 两组 offset/len 均按 Java `int` 回绕。
    /// # 返回
    /// 命中下标或精确插入点编码。
    /// 对应 Java 语义：`TextUtils` 的 `binary_search_sequence_values_and_chars_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn binary_search_sequence_values_and_chars_range(
        case_sensitive: bool,
        values: Option<&[Option<&dyn CharSequenceValue>]>,
        values_offset: i32,
        values_len: i32,
        text: Option<&[u16]>,
        text_offset: i32,
        text_len: i32,
    ) -> Result<i32, TextUtilsError> {
        let values = require_sequence_values(values)?;
        let text = require_chars(text, TEXT_NULL)?;
        let mut low = values_offset;
        let mut high = values_offset.wrapping_add(values_len).wrapping_sub(1);
        while low <= high {
            let mid = unsigned_midpoint(low, high);
            let mid_value = sequence_value_at(values, mid)?;
            let mid_value = mid_value.ok_or(TextUtilsError::NullPointer)?;
            let mid_len = mid_value.java_length()?;
            let comparison = Self::compare_sequence_and_chars_range(
                case_sensitive,
                Some(mid_value),
                0,
                mid_len,
                Some(text),
                text_offset,
                text_len,
            )?;
            if comparison < 0 {
                low = mid.wrapping_add(1);
            } else if comparison > 0 {
                high = mid.wrapping_sub(1);
            } else {
                return Ok(mid);
            }
        }
        Ok(low.wrapping_add(1).wrapping_neg())
    }

    /// 在 `CharSequence[]` 指定范围中搜索另一序列范围。
    ///
    /// # 参数
    /// 两组 offset/len 均按 Java `int` 回绕。
    /// # 返回
    /// 命中下标或精确插入点编码。
    /// 对应 Java 语义：`TextUtils` 的 `binary_search_sequence_values_and_sequence_range` 行为（Rust 侧辅助/私有路径）。
    #[allow(clippy::too_many_arguments)]
    pub fn binary_search_sequence_values_and_sequence_range(
        case_sensitive: bool,
        values: Option<&[Option<&dyn CharSequenceValue>]>,
        values_offset: i32,
        values_len: i32,
        text: Option<&dyn CharSequenceValue>,
        text_offset: i32,
        text_len: i32,
    ) -> Result<i32, TextUtilsError> {
        let values = require_sequence_values(values)?;
        let text = require_sequence(text, TEXT_NULL)?;
        let mut low = values_offset;
        let mut high = values_offset.wrapping_add(values_len).wrapping_sub(1);
        while low <= high {
            let mid = unsigned_midpoint(low, high);
            let mid_value = sequence_value_at(values, mid)?;
            let mid_value = mid_value.ok_or(TextUtilsError::NullPointer)?;
            let mid_len = mid_value.java_length()?;
            let comparison = Self::compare_sequences_range(
                case_sensitive,
                Some(text),
                text_offset,
                text_len,
                Some(mid_value),
                0,
                mid_len,
            )?;
            if comparison > 0 {
                low = mid.wrapping_add(1);
            } else if comparison < 0 {
                high = mid.wrapping_sub(1);
            } else {
                return Ok(mid);
            }
        }
        Ok(low.wrapping_add(1).wrapping_neg())
    }

    /// 计算 `char[]` 范围的 Java 字符串哈希。
    ///
    /// # 参数
    /// `text_offset`/`text_len` 按 Java 循环原样执行。
    /// # 返回
    /// 31 倍累乘并按 `int` 回绕的哈希。
    /// 对应 Java 语义：`TextUtils` 的 `hash_chars_range` 行为（Rust 侧辅助/私有路径）。
    pub fn hash_chars_range(
        text: Option<&[u16]>,
        text_offset: i32,
        text_len: i32,
    ) -> Result<i32, TextUtilsError> {
        let mut hash = 0_i32;
        let mut offset = text_offset;
        let mut index = 0_i32;
        while index < text_len {
            let chars = text.ok_or(TextUtilsError::NullPointer)?;
            hash = hash
                .wrapping_mul(31)
                .wrapping_add(i32::from(array_char_at(chars, offset)?));
            offset = offset.wrapping_add(1);
            index = index.wrapping_add(1);
        }
        Ok(hash)
    }

    /// 计算完整 `CharSequence` 的 Java 字符串哈希。
    ///
    /// # 参数
    /// `text` 可用 `None` 表达 Java null。
    /// # 返回
    /// 完整 UTF-16 内容哈希。
    /// 对应 Java 语义：`TextUtils` 的 `hash_sequence` 行为（Rust 侧辅助/私有路径）。
    pub fn hash_sequence(text: Option<&dyn CharSequenceValue>) -> Result<i32, TextUtilsError> {
        hash_part_whole(0, text)
    }

    /// 计算 `CharSequence` 的 `[begin_index,end_index)` 哈希。
    ///
    /// # 参数
    /// 范围端点按 Java `int` 原样执行。
    /// # 返回
    /// 指定 UTF-16 范围哈希。
    /// 对应 Java 语义：`TextUtils` 的 `hash_sequence_range` 行为（Rust 侧辅助/私有路径）。
    pub fn hash_sequence_range(
        text: Option<&dyn CharSequenceValue>,
        begin_index: i32,
        end_index: i32,
    ) -> Result<i32, TextUtilsError> {
        hash_part_range(0, text, begin_index, end_index)
    }

    /// 计算两个序列拼接后的 Java 字符串哈希。
    ///
    /// # 参数
    /// `text0`、`text1` 按从左到右顺序读取。
    /// # 返回
    /// 不分配拼接字符串的等价哈希。
    /// 对应 Java 语义：`TextUtils` 的 `hash_pair` 行为（Rust 侧辅助/私有路径）。
    pub fn hash_pair(
        text0: Option<&dyn CharSequenceValue>,
        text1: Option<&dyn CharSequenceValue>,
    ) -> Result<i32, TextUtilsError> {
        hash_part_whole(hash_part_whole(0, text0)?, text1)
    }

    /// 计算三个序列拼接后的 Java 字符串哈希。
    ///
    /// # 参数
    /// 三个序列按从左到右顺序读取。
    /// # 返回
    /// 不分配拼接字符串的等价哈希。
    /// 对应 Java 语义：`TextUtils` 的 `hash_triple` 行为（Rust 侧辅助/私有路径）。
    pub fn hash_triple(
        text0: Option<&dyn CharSequenceValue>,
        text1: Option<&dyn CharSequenceValue>,
        text2: Option<&dyn CharSequenceValue>,
    ) -> Result<i32, TextUtilsError> {
        hash_part_whole(hash_part_whole(hash_part_whole(0, text0)?, text1)?, text2)
    }

    /// 计算四个序列拼接后的 Java 字符串哈希。
    ///
    /// # 参数
    /// 四个序列按从左到右顺序读取。
    /// # 返回
    /// 不分配拼接字符串的等价哈希。
    /// 对应 Java 语义：`TextUtils` 的 `hash_quadruple` 行为（Rust 侧辅助/私有路径）。
    pub fn hash_quadruple(
        text0: Option<&dyn CharSequenceValue>,
        text1: Option<&dyn CharSequenceValue>,
        text2: Option<&dyn CharSequenceValue>,
        text3: Option<&dyn CharSequenceValue>,
    ) -> Result<i32, TextUtilsError> {
        hash_part_whole(
            hash_part_whole(hash_part_whole(hash_part_whole(0, text0)?, text1)?, text2)?,
            text3,
        )
    }

    /// 计算五个序列拼接后的 Java 字符串哈希。
    ///
    /// # 参数
    /// 五个序列按从左到右顺序读取。
    /// # 返回
    /// 不分配拼接字符串的等价哈希。
    /// 对应 Java 语义：`TextUtils` 的 `hash_quintuple` 行为（Rust 侧辅助/私有路径）。
    pub fn hash_quintuple(
        text0: Option<&dyn CharSequenceValue>,
        text1: Option<&dyn CharSequenceValue>,
        text2: Option<&dyn CharSequenceValue>,
        text3: Option<&dyn CharSequenceValue>,
        text4: Option<&dyn CharSequenceValue>,
    ) -> Result<i32, TextUtilsError> {
        hash_part_whole(
            hash_part_whole(
                hash_part_whole(hash_part_whole(hash_part_whole(0, text0)?, text1)?, text2)?,
                text3,
            )?,
            text4,
        )
    }
}

#[derive(Clone, Copy)]
enum TextRef<'a> {
    Sequence(&'a dyn CharSequenceValue),
    Chars(&'a [u16]),
}

impl TextRef<'_> {
    fn char_at(self, index: i32) -> Result<u16, TextUtilsError> {
        match self {
            Self::Sequence(sequence) => sequence.java_char_at(index),
            Self::Chars(chars) => array_char_at(chars, index),
        }
    }
}

fn require_sequence<'a>(
    sequence: Option<&'a dyn CharSequenceValue>,
    message: &'static str,
) -> Result<&'a dyn CharSequenceValue, TextUtilsError> {
    sequence.ok_or(TextUtilsError::IllegalArgument { message })
}

fn require_chars<'a>(
    chars: Option<&'a [u16]>,
    message: &'static str,
) -> Result<&'a [u16], TextUtilsError> {
    chars.ok_or(TextUtilsError::IllegalArgument { message })
}

fn implicit_sequence_length(
    sequence: Option<&dyn CharSequenceValue>,
) -> Result<i32, TextUtilsError> {
    sequence.ok_or(TextUtilsError::NullPointer)?.java_length()
}

fn implicit_array_length(chars: Option<&[u16]>) -> Result<i32, TextUtilsError> {
    chars
        .map(|chars| chars.len() as i32)
        .ok_or(TextUtilsError::NullPointer)
}

fn require_char_values<'a>(
    values: Option<&'a [Option<&'a [u16]>]>,
) -> Result<&'a [Option<&'a [u16]>], TextUtilsError> {
    values.ok_or(TextUtilsError::IllegalArgument {
        message: VALUES_NULL,
    })
}

fn require_sequence_values<'a>(
    values: Option<&'a [Option<&'a dyn CharSequenceValue>]>,
) -> Result<&'a [Option<&'a dyn CharSequenceValue>], TextUtilsError> {
    values.ok_or(TextUtilsError::IllegalArgument {
        message: VALUES_NULL,
    })
}

fn array_char_at(chars: &[u16], index: i32) -> Result<u16, TextUtilsError> {
    let index_usize =
        usize::try_from(index).map_err(|_| TextUtilsError::ArrayIndexOutOfBounds {
            index,
            length: chars.len(),
        })?;
    chars
        .get(index_usize)
        .copied()
        .ok_or(TextUtilsError::ArrayIndexOutOfBounds {
            index,
            length: chars.len(),
        })
}

fn char_value_at<'a>(
    values: &'a [Option<&'a [u16]>],
    index: i32,
) -> Result<Option<&'a [u16]>, TextUtilsError> {
    let index_usize = match usize::try_from(index) {
        Ok(index_usize) => index_usize,
        Err(_) => {
            return Err(TextUtilsError::ArrayIndexOutOfBounds {
                index,
                length: values.len(),
            });
        }
    };
    match values.get(index_usize) {
        Some(value) => Ok(*value),
        None => Err(TextUtilsError::ArrayIndexOutOfBounds {
            index,
            length: values.len(),
        }),
    }
}

fn sequence_value_at<'a>(
    values: &'a [Option<&'a dyn CharSequenceValue>],
    index: i32,
) -> Result<Option<&'a dyn CharSequenceValue>, TextUtilsError> {
    let index_usize = match usize::try_from(index) {
        Ok(index_usize) => index_usize,
        Err(_) => {
            return Err(TextUtilsError::ArrayIndexOutOfBounds {
                index,
                length: values.len(),
            });
        }
    };
    match values.get(index_usize) {
        Some(value) => Ok(*value),
        None => Err(TextUtilsError::ArrayIndexOutOfBounds {
            index,
            length: values.len(),
        }),
    }
}

fn unsigned_midpoint(low: i32, high: i32) -> i32 {
    (low.wrapping_add(high) as u32 >> 1) as i32
}

fn equals_core(
    case_sensitive: bool,
    text1: TextRef<'_>,
    text1_offset: i32,
    text2: TextRef<'_>,
    text2_offset: i32,
    text_len: i32,
) -> Result<bool, TextUtilsError> {
    let mut remaining = text_len;
    let mut index = 0_i32;
    while remaining != 0 {
        remaining = remaining.wrapping_sub(1);
        let c1 = text1.char_at(text1_offset.wrapping_add(index))?;
        let c2 = text2.char_at(text2_offset.wrapping_add(index))?;
        if !chars_equal(case_sensitive, c1, c2) {
            return Ok(false);
        }
        index = index.wrapping_add(1);
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn starts_with_validated(
    case_sensitive: bool,
    text: TextRef<'_>,
    text_offset: i32,
    text_len: i32,
    prefix: TextRef<'_>,
    prefix_offset: i32,
    prefix_len: i32,
) -> Result<bool, TextUtilsError> {
    if text_len < prefix_len {
        return Ok(false);
    }
    equals_core(
        case_sensitive,
        text,
        text_offset,
        prefix,
        prefix_offset,
        prefix_len,
    )
}

#[allow(clippy::too_many_arguments)]
fn ends_with_validated(
    case_sensitive: bool,
    text: TextRef<'_>,
    text_offset: i32,
    text_len: i32,
    suffix: TextRef<'_>,
    suffix_offset: i32,
    suffix_len: i32,
) -> Result<bool, TextUtilsError> {
    if text_len < suffix_len {
        return Ok(false);
    }
    let text_reverse_offset = text_offset.wrapping_add(text_len).wrapping_sub(1);
    let suffix_reverse_offset = suffix_offset.wrapping_add(suffix_len).wrapping_sub(1);
    let mut remaining = suffix_len;
    let mut index = 0_i32;
    while remaining != 0 {
        remaining = remaining.wrapping_sub(1);
        let c1 = text.char_at(text_reverse_offset.wrapping_sub(index))?;
        let c2 = suffix.char_at(suffix_reverse_offset.wrapping_sub(index))?;
        if !chars_equal(case_sensitive, c1, c2) {
            return Ok(false);
        }
        index = index.wrapping_add(1);
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn contains_validated(
    case_sensitive: bool,
    text: TextRef<'_>,
    text_offset: i32,
    text_len: i32,
    fragment: TextRef<'_>,
    fragment_offset: i32,
    fragment_len: i32,
) -> Result<bool, TextUtilsError> {
    if text_len < fragment_len {
        return Ok(false);
    }
    if fragment_len == 0 {
        return Ok(true);
    }

    // 严格保留上游朴素搜索的回退方式：部分匹配失败时回到“起点 + 1”。
    let mut index = 0_i32;
    let mut fragment_index = 0_i32;
    while index < text_len {
        let c1 = text.char_at(text_offset.wrapping_add(index))?;
        let c2 = fragment.char_at(fragment_offset.wrapping_add(fragment_index))?;
        if chars_equal(case_sensitive, c1, c2) {
            fragment_index = fragment_index.wrapping_add(1);
            if fragment_index == fragment_len {
                return Ok(true);
            }
        } else {
            if fragment_index > 0 {
                index = index.wrapping_sub(fragment_index);
            }
            fragment_index = 0;
        }
        index = index.wrapping_add(1);
    }
    Ok(false)
}

fn slice_contains(text: &[u16], fragment: &[u16]) -> bool {
    fragment.is_empty()
        || (fragment.len() <= text.len()
            && text
                .windows(fragment.len())
                .any(|window| window == fragment))
}

#[allow(clippy::too_many_arguments)]
fn compare_core(
    case_sensitive: bool,
    text1: TextRef<'_>,
    text1_offset: i32,
    text1_len: i32,
    text2: TextRef<'_>,
    text2_offset: i32,
    text2_len: i32,
) -> Result<i32, TextUtilsError> {
    let mut remaining = text1_len.min(text2_len);
    let mut index = 0_i32;
    while remaining != 0 {
        remaining = remaining.wrapping_sub(1);
        let mut c1 = text1.char_at(text1_offset.wrapping_add(index))?;
        let mut c2 = text2.char_at(text2_offset.wrapping_add(index))?;
        if c1 != c2 {
            if case_sensitive {
                return Ok(i32::from(c1) - i32::from(c2));
            }
            c1 = java_upper(c1);
            c2 = java_upper(c2);
            if c1 != c2 {
                c1 = java_lower(c1);
                c2 = java_lower(c2);
                if c1 != c2 {
                    return Ok(i32::from(c1) - i32::from(c2));
                }
            }
        }
        index = index.wrapping_add(1);
    }
    Ok(text1_len.wrapping_sub(text2_len))
}

fn hash_part_whole(hash: i32, text: Option<&dyn CharSequenceValue>) -> Result<i32, TextUtilsError> {
    let length = text.ok_or(TextUtilsError::NullPointer)?.java_length()?;
    hash_part_range(hash, text, 0, length)
}

fn hash_part_range(
    mut hash: i32,
    text: Option<&dyn CharSequenceValue>,
    begin_index: i32,
    end_index: i32,
) -> Result<i32, TextUtilsError> {
    // Java 的 && 短路决定 null 是否在 String 快路径中提前触发 length()。
    if hash == 0 && begin_index == 0 {
        let sequence = text.ok_or(TextUtilsError::NullPointer)?;
        if end_index == sequence.java_length()?
            && let Some(string) = sequence.as_utf16_string()
        {
            let mut string_hash = 0_i32;
            for &unit in string.as_utf16() {
                string_hash = string_hash.wrapping_mul(31).wrapping_add(i32::from(unit));
            }
            return Ok(string_hash);
        }
    }
    let mut index = begin_index;
    while index < end_index {
        let sequence = text.ok_or(TextUtilsError::NullPointer)?;
        hash = hash
            .wrapping_mul(31)
            .wrapping_add(i32::from(sequence.java_char_at(index)?));
        index = index.wrapping_add(1);
    }
    Ok(hash)
}

fn chars_equal(case_sensitive: bool, mut c1: u16, mut c2: u16) -> bool {
    if c1 == c2 {
        return true;
    }
    if case_sensitive {
        return false;
    }
    c1 = java_upper(c1);
    c2 = java_upper(c2);
    c1 == c2 || java_lower(c1) == java_lower(c2)
}

fn java_upper(value: u16) -> u16 {
    case_map(value, true)
}

/// 按 JDK 21 `Character.toLowerCase(char)` 对单个 UTF-16 code unit 做简单小写映射。
///
/// 返回值仍是单个 code unit，不使用可能扩张为多个字符的字符串级 lowercase。
/// 对应 Java 语义：`TextUtils` 的 `java_lower` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn java_lower(value: u16) -> u16 {
    case_map(value, false)
}

fn case_map(value: u16, upper: bool) -> u16 {
    debug_assert_eq!(&CASE_MAP[0..4], b"TUTM");
    debug_assert_eq!(read_u16(CASE_MAP, 4), 1);
    let upper_count = usize::from(read_u16(CASE_MAP, 6));
    let upper_start = 8;
    let lower_count_offset = upper_start + upper_count * 4;
    let (start, count) = if upper {
        (upper_start, upper_count)
    } else {
        (
            lower_count_offset + 2,
            usize::from(read_u16(CASE_MAP, lower_count_offset)),
        )
    };
    let mut low = 0_usize;
    let mut high = count;
    while low < high {
        let mid = low + (high - low) / 2;
        let source = read_u16(CASE_MAP, start + mid * 4);
        if source < value {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    if low < count && read_u16(CASE_MAP, start + low * 4) == value {
        read_u16(CASE_MAP, start + low * 4 + 2)
    } else {
        value
    }
}

/// 对应 Java 语义：`TextUtils` 的 `java_case_fold_unit` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn java_case_fold_unit(value: u16) -> u16 {
    java_lower(java_upper(value))
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use super::{
        CharSequenceValue, TextRef, TextUtils, TextUtilsError, Utf16String, char_value_at,
        compare_core, contains_validated, ends_with_validated, equals_core, hash_part_range,
        hash_part_whole, sequence_value_at,
    };
    use crate::util::CharArrayWrapperSequence;

    struct PlainSequence(Utf16String);

    impl PlainSequence {
        fn new(value: &str) -> Self {
            Self(Utf16String::from_rust_str(value))
        }
    }

    impl CharSequenceValue for PlainSequence {
        fn java_length(&self) -> Result<i32, TextUtilsError> {
            Ok(self.0.len() as i32)
        }

        fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
            self.0.java_char_at(index)
        }

        fn as_utf16_string(&self) -> Option<&Utf16String> {
            None
        }
    }

    struct LengthFailure;

    impl CharSequenceValue for LengthFailure {
        fn java_length(&self) -> Result<i32, TextUtilsError> {
            Err(dynamic_error())
        }

        fn java_char_at(&self, _index: i32) -> Result<u16, TextUtilsError> {
            Err(dynamic_error())
        }

        fn as_utf16_string(&self) -> Option<&Utf16String> {
            None
        }
    }

    struct CharFailure;

    impl CharSequenceValue for CharFailure {
        fn java_length(&self) -> Result<i32, TextUtilsError> {
            Ok(1)
        }

        fn java_char_at(&self, _index: i32) -> Result<u16, TextUtilsError> {
            Err(dynamic_error())
        }

        fn as_utf16_string(&self) -> Option<&Utf16String> {
            None
        }
    }

    fn dynamic_error() -> TextUtilsError {
        TextUtilsError::SequenceAccess {
            class_name: "example.SequenceFailure".to_owned(),
            message: None,
        }
    }

    fn java(value: &str) -> Utf16String {
        Utf16String::from_rust_str(value)
    }

    #[test]
    fn exposes_every_error_and_dynamic_sequence_adapter_path() {
        let string_error = java("x").java_char_at(-1).unwrap_err();
        assert_eq!(
            string_error.class_name(),
            "java.lang.StringIndexOutOfBoundsException"
        );
        assert_eq!(
            string_error.message().unwrap().to_string_lossy(),
            "Index -1 out of bounds for length 1"
        );
        assert_eq!(
            string_error.to_string(),
            "Index -1 out of bounds for length 1"
        );
        assert!(java("x").java_char_at(1).is_err());

        let null = TextUtilsError::NullPointer;
        assert_eq!(null.class_name(), "java.lang.NullPointerException");
        assert_eq!(null.message(), None);
        assert_eq!(null.to_string(), "");

        let dynamic_without_message = TextUtilsError::SequenceAccess {
            class_name: "example.Runtime".to_owned(),
            message: None,
        };
        assert_eq!(dynamic_without_message.class_name(), "example.Runtime");
        assert_eq!(dynamic_without_message.message(), None);
        assert_eq!(dynamic_without_message.to_string(), "");

        let shared = Arc::new(RwLock::new(vec!['a' as u16]));
        let wrapper = CharArrayWrapperSequence::with_range(Some(shared), 0, 1).unwrap();
        assert_eq!(wrapper.java_length().unwrap(), 1);
        assert_eq!(wrapper.java_char_at(0).unwrap(), 'a' as u16);
        assert!(wrapper.as_utf16_string().is_none());
        let dynamic = wrapper.java_char_at(1).unwrap_err();
        assert_eq!(
            dynamic.class_name(),
            "java.lang.ArrayIndexOutOfBoundsException"
        );
        assert_eq!(
            dynamic.message().unwrap().to_string_lossy(),
            "Array index out of range: 1"
        );
    }

    #[test]
    fn covers_fast_paths_identity_length_and_search_mismatch_branches() {
        let abc = java("abc");
        let abd = java("abd");
        let plain_abc = PlainSequence::new("abc");
        let plain_ab = PlainSequence::new("ab");
        assert_eq!(
            TextUtils::equals_sequences(true, Some(&abc), Some(&abc)),
            Ok(true)
        );
        assert_eq!(
            TextUtils::equals_sequences(true, Some(&abc), Some(&abd)),
            Ok(false)
        );
        assert_eq!(
            TextUtils::equals_sequences(true, Some(&plain_abc), Some(&plain_abc)),
            Ok(true)
        );
        assert_eq!(
            TextUtils::equals_chars_range(true, Some(&[1]), 0, 1, Some(&[1, 2]), 0, 2),
            Ok(false)
        );
        let same = [1_u16, 2];
        assert_eq!(
            TextUtils::equals_chars_range(true, Some(&same), 1, 1, Some(&same), 1, 1),
            Ok(true)
        );
        assert_eq!(
            TextUtils::equals_sequence_and_chars_range(true, Some(&abc), 0, 1, Some(&[1, 2]), 0, 2),
            Ok(false)
        );
        assert_eq!(
            TextUtils::equals_sequences_range(true, Some(&abc), 0, 1, Some(&abd), 0, 2),
            Ok(false)
        );
        assert_eq!(
            TextUtils::equals_sequences_range(true, Some(&abc), 0, 3, Some(&abc), 0, 3),
            Ok(true)
        );
        assert_eq!(
            TextUtils::equals_sequences_range(true, Some(&abc), 0, 2, Some(&abd), 0, 2),
            Ok(true)
        );
        assert_eq!(
            TextUtils::equals_chars(true, Some(&['a' as u16]), Some(&['b' as u16])),
            Ok(false)
        );
        assert_eq!(
            TextUtils::equals_chars(false, Some(&[0x0130]), Some(&['i' as u16])),
            Ok(true)
        );

        assert_eq!(
            TextUtils::starts_with_sequences(true, Some(&abc), Some(&java("ab"))),
            Ok(true)
        );
        assert_eq!(
            TextUtils::starts_with_sequences(true, Some(&abc), Some(&abd)),
            Ok(false)
        );
        assert_eq!(
            TextUtils::starts_with_sequences(true, Some(&plain_abc), Some(&plain_ab)),
            Ok(true)
        );
        assert_eq!(
            TextUtils::starts_with_chars_range(true, Some(&[1]), 0, 1, Some(&[1, 2]), 0, 2),
            Ok(false)
        );
        assert_eq!(
            TextUtils::starts_with_chars_range(
                true,
                Some(&['a' as u16]),
                0,
                1,
                Some(&['b' as u16]),
                0,
                1
            ),
            Ok(false)
        );

        assert_eq!(
            TextUtils::ends_with_sequences(true, Some(&abc), Some(&java("bc"))),
            Ok(true)
        );
        assert_eq!(
            TextUtils::ends_with_sequences(true, Some(&abc), Some(&abd)),
            Ok(false)
        );
        assert_eq!(
            TextUtils::ends_with_sequences(true, Some(&plain_abc), Some(&plain_ab)),
            Ok(false)
        );
        assert_eq!(
            TextUtils::ends_with_chars_range(true, Some(&[1]), 0, 1, Some(&[1, 2]), 0, 2),
            Ok(false)
        );
        assert_eq!(
            TextUtils::ends_with_chars_range(
                true,
                Some(&['a' as u16]),
                0,
                1,
                Some(&['b' as u16]),
                0,
                1
            ),
            Ok(false)
        );

        assert_eq!(
            TextUtils::contains_sequences(true, Some(&abc), Some(&java("b"))),
            Ok(true)
        );
        assert_eq!(
            TextUtils::contains_sequences(true, Some(&abc), Some(&java("z"))),
            Ok(false)
        );
        assert_eq!(
            TextUtils::contains_sequences(true, Some(&abc), Some(&java(""))),
            Ok(true)
        );
        assert_eq!(
            TextUtils::contains_sequences(true, Some(&abc), Some(&java("abcd"))),
            Ok(false)
        );
        assert_eq!(
            TextUtils::contains_sequences(true, Some(&plain_abc), Some(&plain_ab)),
            Ok(true)
        );
        assert_eq!(
            TextUtils::contains_sequences_range(
                true,
                Some(&plain_abc),
                0,
                0,
                Some(&plain_ab),
                0,
                1
            ),
            Ok(false)
        );
        assert_eq!(
            TextUtils::contains_sequences_range(
                true,
                Some(&plain_abc),
                0,
                1,
                Some(&plain_ab),
                0,
                0
            ),
            Ok(true)
        );
        assert!(
            TextUtils::contains_sequences_range(
                true,
                Some(&CharFailure),
                0,
                1,
                Some(&plain_ab),
                0,
                1
            )
            .is_err()
        );
        assert_eq!(
            TextUtils::contains_chars(
                true,
                Some(&['a' as u16, 'a' as u16, 'b' as u16]),
                Some(&['a' as u16, 'b' as u16])
            ),
            Ok(true)
        );
        assert_eq!(
            TextUtils::contains_chars(
                true,
                Some(&['a' as u16, 'b' as u16, 'c' as u16]),
                Some(&['z' as u16])
            ),
            Ok(false)
        );
    }

    #[test]
    fn covers_compare_and_all_binary_search_directions_and_failures() {
        assert!(PlainSequence::new("plain").as_utf16_string().is_none());

        let same = [1_u16];
        assert_eq!(
            TextUtils::compare_chars_range(true, Some(&same), 0, 1, Some(&same), 0, 1),
            Ok(0)
        );
        let same_sequence = java("x");
        assert_eq!(
            TextUtils::compare_sequences_range(
                true,
                Some(&same_sequence),
                0,
                1,
                Some(&same_sequence),
                0,
                1
            ),
            Ok(0)
        );
        assert_eq!(
            TextUtils::compare_chars(true, Some(&['a' as u16]), Some(&['z' as u16])),
            Ok(-25)
        );
        assert!(
            TextUtils::compare_chars(false, Some(&['a' as u16]), Some(&['β' as u16])).unwrap() < 0
        );
        assert_eq!(
            TextUtils::compare_chars(true, Some(&['a' as u16]), Some(&['a' as u16, 'a' as u16])),
            Ok(-1)
        );

        let a = java("a");
        let c = java("c");
        let e = java("e");
        let char_values = [Some(a.as_utf16()), Some(c.as_utf16()), Some(e.as_utf16())];
        let sequence_values: [Option<&dyn CharSequenceValue>; 3] = [Some(&a), Some(&c), Some(&e)];
        for key in ["0", "b", "c", "d", "z"] {
            let key = java(key);
            let char_result = TextUtils::binary_search_chars_values_and_chars(
                true,
                Some(&char_values),
                Some(key.as_utf16()),
                0,
                key.len() as i32,
            )
            .unwrap();
            let char_sequence_result = TextUtils::binary_search_chars_values_and_sequence(
                true,
                Some(&char_values),
                Some(&key),
                0,
                key.len() as i32,
            )
            .unwrap();
            let sequence_char_result = TextUtils::binary_search_sequence_values_and_chars(
                true,
                Some(&sequence_values),
                Some(key.as_utf16()),
                0,
                key.len() as i32,
            )
            .unwrap();
            let sequence_result = TextUtils::binary_search_sequence_values_and_sequence(
                true,
                Some(&sequence_values),
                Some(&key),
                0,
                key.len() as i32,
            )
            .unwrap();
            assert_eq!(char_result, char_sequence_result);
            assert_eq!(char_result, sequence_char_result);
            assert_eq!(char_result, sequence_result);
        }

        let empty_chars: [Option<&[u16]>; 0] = [];
        assert!(
            TextUtils::binary_search_chars_values_and_chars_range(
                true,
                Some(&empty_chars),
                -1,
                1,
                Some(&[]),
                0,
                0
            )
            .is_err()
        );
        let empty_sequences: [Option<&dyn CharSequenceValue>; 0] = [];
        assert!(
            TextUtils::binary_search_sequence_values_and_sequence_range(
                true,
                Some(&empty_sequences),
                -1,
                1,
                Some(&a),
                0,
                1
            )
            .is_err()
        );
        let null_sequence = [None];
        assert!(
            TextUtils::binary_search_sequence_values_and_chars(
                true,
                Some(&null_sequence),
                Some(&[]),
                0,
                0
            )
            .is_err()
        );

        assert!(char_value_at(&[], -1).is_err());
        assert!(char_value_at(&[], 0).is_err());
        assert!(sequence_value_at(&empty_sequences, -1).is_err());
        assert!(sequence_value_at(&empty_sequences, 0).is_err());

        let one_char_value = [Some(a.as_utf16())];
        assert!(
            TextUtils::binary_search_chars_values_and_chars_range(
                true,
                Some(&one_char_value),
                0,
                1,
                Some(&[]),
                1,
                1
            )
            .is_err()
        );
        assert!(
            TextUtils::binary_search_chars_values_and_sequence_range(
                true,
                Some(&one_char_value),
                0,
                1,
                Some(&a),
                1,
                1
            )
            .is_err()
        );
        let one_sequence_value: [Option<&dyn CharSequenceValue>; 1] = [Some(&a)];
        assert!(
            TextUtils::binary_search_sequence_values_and_chars_range(
                true,
                Some(&one_sequence_value),
                0,
                1,
                Some(&[]),
                1,
                1
            )
            .is_err()
        );
        assert!(
            TextUtils::binary_search_sequence_values_and_sequence_range(
                true,
                Some(&one_sequence_value),
                0,
                1,
                Some(&a),
                1,
                1
            )
            .is_err()
        );
    }

    #[test]
    fn covers_non_string_hash_paths_short_circuits_and_argument_order() {
        let plain = PlainSequence::new("abc");
        assert_eq!(TextUtils::hash_sequence(Some(&plain)), Ok(96_354));
        assert_eq!(
            TextUtils::hash_sequence_range(Some(&plain), 0, 2),
            Ok(3_105)
        );
        assert_eq!(TextUtils::hash_sequence_range(Some(&plain), 1, 1), Ok(0));
        assert!(TextUtils::hash_sequence_range(Some(&plain), -1, 1).is_err());
        assert!(TextUtils::hash_chars_range(Some(&[]), -1, 1).is_err());

        let a = java("a");
        assert!(TextUtils::hash_pair(Some(&a), None).is_err());
        assert!(TextUtils::hash_triple(Some(&a), Some(&a), None).is_err());
        assert!(TextUtils::hash_quadruple(Some(&a), Some(&a), Some(&a), None).is_err());
        assert!(TextUtils::hash_quintuple(Some(&a), Some(&a), Some(&a), None, Some(&a)).is_err());
        assert!(TextUtils::hash_quintuple(Some(&a), Some(&a), Some(&a), Some(&a), None).is_err());
    }

    #[test]
    fn covers_every_public_delegating_success_path_in_the_unit_build() {
        let sequence = java("a");
        let chars = ['a' as u16];
        assert_eq!(
            TextUtils::equals_sequence_and_chars(true, Some(&sequence), Some(&chars)),
            Ok(true)
        );
        assert_eq!(
            TextUtils::starts_with_sequence_and_chars(true, Some(&sequence), Some(&chars)),
            Ok(true)
        );
        assert_eq!(
            TextUtils::starts_with_chars(true, Some(&chars), Some(&chars)),
            Ok(true)
        );
        assert_eq!(
            TextUtils::starts_with_sequence_and_chars_range(
                true,
                Some(&sequence),
                0,
                1,
                Some(&chars),
                0,
                1
            ),
            Ok(true)
        );
        assert_eq!(
            TextUtils::starts_with_chars_and_sequence_range(
                true,
                Some(&chars),
                0,
                1,
                Some(&sequence),
                0,
                1
            ),
            Ok(true)
        );
        assert_eq!(
            TextUtils::starts_with_sequences_range(
                true,
                Some(&sequence),
                0,
                1,
                Some(&sequence),
                0,
                1
            ),
            Ok(true)
        );
        assert_eq!(
            TextUtils::ends_with_sequence_and_chars(true, Some(&sequence), Some(&chars)),
            Ok(true)
        );
        assert_eq!(
            TextUtils::ends_with_chars(true, Some(&chars), Some(&chars)),
            Ok(true)
        );
        assert_eq!(
            TextUtils::ends_with_sequence_and_chars_range(
                true,
                Some(&sequence),
                0,
                1,
                Some(&chars),
                0,
                1
            ),
            Ok(true)
        );
        assert_eq!(
            TextUtils::ends_with_chars_and_sequence_range(
                true,
                Some(&chars),
                0,
                1,
                Some(&sequence),
                0,
                1
            ),
            Ok(true)
        );
        assert_eq!(
            TextUtils::ends_with_sequences_range(
                true,
                Some(&sequence),
                0,
                1,
                Some(&sequence),
                0,
                1
            ),
            Ok(true)
        );
        assert_eq!(
            TextUtils::contains_sequence_and_chars(true, Some(&sequence), Some(&chars)),
            Ok(true)
        );
        assert_eq!(
            TextUtils::contains_sequence_and_chars_range(
                true,
                Some(&sequence),
                0,
                1,
                Some(&chars),
                0,
                1
            ),
            Ok(true)
        );
        assert_eq!(
            TextUtils::contains_chars_and_sequence_range(
                true,
                Some(&chars),
                0,
                1,
                Some(&sequence),
                0,
                1
            ),
            Ok(true)
        );
        assert_eq!(
            TextUtils::contains_sequences_range(true, Some(&sequence), 0, 1, Some(&sequence), 0, 1),
            Ok(true)
        );
        assert_eq!(
            TextUtils::compare_sequences(true, Some(&sequence), Some(&sequence)),
            Ok(0)
        );
        assert_eq!(
            TextUtils::compare_sequence_and_chars(true, Some(&sequence), Some(&chars)),
            Ok(0)
        );
        assert_eq!(TextUtils::hash_chars_range(Some(&chars), 0, 1), Ok(97));

        let illegal = TextUtils::equals_chars_range(true, None, 0, 0, Some(&[]), 0, 0).unwrap_err();
        assert_eq!(illegal.class_name(), "java.lang.IllegalArgumentException");
        assert!(!illegal.to_string().is_empty());
        let array = TextUtils::hash_chars_range(Some(&[]), 0, 1).unwrap_err();
        assert_eq!(
            array.class_name(),
            "java.lang.ArrayIndexOutOfBoundsException"
        );
        assert!(array.message().is_some());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn exhausts_validation_and_dynamic_failure_residuals() {
        let good = java("a");
        let chars = ['a' as u16];
        let plain = PlainSequence::new("aa");
        let length_failure = LengthFailure;
        let char_failure = CharFailure;
        assert!(length_failure.java_char_at(0).is_err());
        assert_eq!(char_failure.java_length(), Ok(1));
        assert!(length_failure.as_utf16_string().is_none());
        assert!(char_failure.as_utf16_string().is_none());

        assert!(TextUtils::equals_sequences(false, None, Some(&good)).is_err());
        assert!(TextUtils::equals_sequences(false, Some(&good), None).is_err());
        assert!(TextUtils::equals_sequences(false, Some(&length_failure), Some(&good)).is_err());
        assert!(TextUtils::equals_sequences(false, Some(&good), Some(&length_failure)).is_err());
        assert!(TextUtils::equals_sequence_and_chars(false, Some(&good), None).is_err());
        assert!(TextUtils::equals_chars(false, None, Some(&chars)).is_err());
        assert!(TextUtils::equals_chars(false, Some(&chars), None).is_err());
        assert!(TextUtils::equals_chars_range(false, None, 0, 1, Some(&chars), 0, 1).is_err());
        assert!(TextUtils::equals_chars_range(false, Some(&chars), 0, 1, None, 0, 1).is_err());
        assert!(
            TextUtils::equals_sequence_and_chars_range(false, None, 0, 1, Some(&chars), 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::equals_sequence_and_chars_range(false, Some(&good), 0, 1, None, 0, 1)
                .is_err()
        );
        assert!(TextUtils::equals_sequences_range(false, None, 0, 1, Some(&good), 0, 1).is_err());
        assert!(TextUtils::equals_sequences_range(false, Some(&good), 0, 1, None, 0, 1).is_err());

        assert!(TextUtils::starts_with_sequences(false, None, Some(&good)).is_err());
        assert!(TextUtils::starts_with_sequences(false, Some(&good), None).is_err());
        assert!(
            TextUtils::starts_with_sequences(false, Some(&length_failure), Some(&good)).is_err()
        );
        assert!(
            TextUtils::starts_with_sequences(false, Some(&good), Some(&length_failure)).is_err()
        );
        assert!(TextUtils::starts_with_sequence_and_chars(false, None, Some(&chars)).is_err());
        assert!(TextUtils::starts_with_sequence_and_chars(false, Some(&good), None).is_err());
        assert!(TextUtils::starts_with_chars(false, None, Some(&chars)).is_err());
        assert!(TextUtils::starts_with_chars(false, Some(&chars), None).is_err());
        assert!(TextUtils::starts_with_chars_range(false, None, 0, 1, Some(&chars), 0, 1).is_err());
        assert!(TextUtils::starts_with_chars_range(false, Some(&chars), 0, 1, None, 0, 1).is_err());
        assert!(
            TextUtils::starts_with_sequence_and_chars_range(false, None, 0, 1, Some(&chars), 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::starts_with_sequence_and_chars_range(false, Some(&good), 0, 1, None, 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::starts_with_chars_and_sequence_range(false, None, 0, 1, Some(&good), 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::starts_with_chars_and_sequence_range(false, Some(&chars), 0, 1, None, 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::starts_with_sequences_range(false, None, 0, 1, Some(&good), 0, 1).is_err()
        );
        assert!(
            TextUtils::starts_with_sequences_range(false, Some(&good), 0, 1, None, 0, 1).is_err()
        );

        assert!(TextUtils::ends_with_sequences(false, None, Some(&good)).is_err());
        assert!(TextUtils::ends_with_sequences(false, Some(&good), None).is_err());
        assert!(TextUtils::ends_with_sequences(false, Some(&length_failure), Some(&good)).is_err());
        assert!(TextUtils::ends_with_sequences(false, Some(&good), Some(&length_failure)).is_err());
        assert!(TextUtils::ends_with_sequence_and_chars(false, None, Some(&chars)).is_err());
        assert!(TextUtils::ends_with_sequence_and_chars(false, Some(&good), None).is_err());
        assert!(TextUtils::ends_with_chars(false, None, Some(&chars)).is_err());
        assert!(TextUtils::ends_with_chars(false, Some(&chars), None).is_err());
        assert!(TextUtils::ends_with_chars_range(false, None, 0, 1, Some(&chars), 0, 1).is_err());
        assert!(TextUtils::ends_with_chars_range(false, Some(&chars), 0, 1, None, 0, 1).is_err());
        assert!(
            TextUtils::ends_with_sequence_and_chars_range(false, None, 0, 1, Some(&chars), 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::ends_with_sequence_and_chars_range(false, Some(&good), 0, 1, None, 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::ends_with_chars_and_sequence_range(false, None, 0, 1, Some(&good), 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::ends_with_chars_and_sequence_range(false, Some(&chars), 0, 1, None, 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::ends_with_sequences_range(false, None, 0, 1, Some(&good), 0, 1).is_err()
        );
        assert!(
            TextUtils::ends_with_sequences_range(false, Some(&good), 0, 1, None, 0, 1).is_err()
        );

        assert!(TextUtils::contains_sequences(false, None, Some(&good)).is_err());
        assert!(TextUtils::contains_sequences(false, Some(&good), None).is_err());
        assert!(TextUtils::contains_sequences(false, Some(&length_failure), Some(&good)).is_err());
        assert!(TextUtils::contains_sequences(false, Some(&good), Some(&length_failure)).is_err());
        assert!(TextUtils::contains_sequence_and_chars(false, None, Some(&chars)).is_err());
        assert!(TextUtils::contains_sequence_and_chars(false, Some(&good), None).is_err());
        assert!(TextUtils::contains_chars(false, None, Some(&chars)).is_err());
        assert!(TextUtils::contains_chars(false, Some(&chars), None).is_err());
        assert!(TextUtils::contains_chars_range(false, None, 0, 1, Some(&chars), 0, 1).is_err());
        assert!(TextUtils::contains_chars_range(false, Some(&chars), 0, 1, None, 0, 1).is_err());
        assert!(
            TextUtils::contains_sequence_and_chars_range(false, None, 0, 1, Some(&chars), 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::contains_sequence_and_chars_range(false, Some(&good), 0, 1, None, 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::contains_chars_and_sequence_range(false, None, 0, 1, Some(&good), 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::contains_chars_and_sequence_range(false, Some(&chars), 0, 1, None, 0, 1)
                .is_err()
        );
        assert!(TextUtils::contains_sequences_range(false, None, 0, 1, Some(&good), 0, 1).is_err());
        assert!(TextUtils::contains_sequences_range(false, Some(&good), 0, 1, None, 0, 1).is_err());

        assert!(TextUtils::compare_sequences(false, None, Some(&good)).is_err());
        assert!(TextUtils::compare_sequences(false, Some(&good), None).is_err());
        assert!(TextUtils::compare_sequences(false, Some(&length_failure), Some(&good)).is_err());
        assert!(TextUtils::compare_sequences(false, Some(&good), Some(&length_failure)).is_err());
        assert!(TextUtils::compare_sequence_and_chars(false, None, Some(&chars)).is_err());
        assert!(TextUtils::compare_sequence_and_chars(false, Some(&good), None).is_err());
        assert!(TextUtils::compare_chars(false, None, Some(&chars)).is_err());
        assert!(TextUtils::compare_chars(false, Some(&chars), None).is_err());
        assert!(TextUtils::compare_chars_range(false, None, 0, 1, Some(&chars), 0, 1).is_err());
        assert!(TextUtils::compare_chars_range(false, Some(&chars), 0, 1, None, 0, 1).is_err());
        assert!(
            TextUtils::compare_sequence_and_chars_range(false, None, 0, 1, Some(&chars), 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::compare_sequence_and_chars_range(false, Some(&good), 0, 1, None, 0, 1)
                .is_err()
        );
        assert!(TextUtils::compare_sequences_range(false, None, 0, 1, Some(&good), 0, 1).is_err());
        assert!(TextUtils::compare_sequences_range(false, Some(&good), 0, 1, None, 0, 1).is_err());

        let char_values = [Some(chars.as_slice())];
        let sequence_values: [Option<&dyn CharSequenceValue>; 1] = [Some(&good)];
        assert!(
            TextUtils::binary_search_chars_values_and_chars_range(
                false,
                None,
                0,
                1,
                Some(&chars),
                0,
                1
            )
            .is_err()
        );
        assert!(
            TextUtils::binary_search_chars_values_and_sequence_range(
                false,
                None,
                0,
                1,
                Some(&good),
                0,
                1
            )
            .is_err()
        );
        assert!(
            TextUtils::binary_search_sequence_values_and_chars_range(
                false,
                None,
                0,
                1,
                Some(&chars),
                0,
                1
            )
            .is_err()
        );
        assert!(
            TextUtils::binary_search_sequence_values_and_sequence_range(
                false,
                None,
                0,
                1,
                Some(&good),
                0,
                1
            )
            .is_err()
        );
        assert!(
            TextUtils::binary_search_chars_values_and_sequence(false, None, Some(&good), 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::binary_search_sequence_values_and_chars(false, None, Some(&chars), 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::binary_search_sequence_values_and_sequence(false, None, Some(&good), 0, 1)
                .is_err()
        );
        assert!(
            TextUtils::binary_search_chars_values_and_sequence_range(
                false,
                Some(&char_values),
                0,
                1,
                None,
                0,
                1
            )
            .is_err()
        );
        assert!(
            TextUtils::binary_search_sequence_values_and_chars_range(
                false,
                Some(&sequence_values),
                0,
                1,
                None,
                0,
                1
            )
            .is_err()
        );
        assert!(
            TextUtils::binary_search_sequence_values_and_sequence_range(
                false,
                Some(&sequence_values),
                0,
                1,
                None,
                0,
                1
            )
            .is_err()
        );
        let failing_values: [Option<&dyn CharSequenceValue>; 1] = [Some(&length_failure)];
        assert!(
            TextUtils::binary_search_sequence_values_and_chars(
                false,
                Some(&failing_values),
                Some(&chars),
                0,
                1
            )
            .is_err()
        );
        assert!(
            TextUtils::binary_search_sequence_values_and_sequence(
                false,
                Some(&failing_values),
                Some(&good),
                0,
                1
            )
            .is_err()
        );
        let empty_char_values: [Option<&[u16]>; 0] = [];
        assert!(
            TextUtils::binary_search_chars_values_and_sequence_range(
                false,
                Some(&empty_char_values),
                1,
                1,
                Some(&good),
                0,
                1
            )
            .is_err()
        );
        let null_char_values = [None];
        assert!(
            TextUtils::binary_search_chars_values_and_sequence_range(
                false,
                Some(&null_char_values),
                0,
                1,
                Some(&good),
                0,
                1
            )
            .is_err()
        );
        let empty_sequence_values: [Option<&dyn CharSequenceValue>; 0] = [];
        assert!(
            TextUtils::binary_search_sequence_values_and_chars_range(
                false,
                Some(&empty_sequence_values),
                1,
                1,
                Some(&chars),
                0,
                1
            )
            .is_err()
        );
        let null_sequence_values = [None];
        assert!(
            TextUtils::binary_search_sequence_values_and_sequence_range(
                false,
                Some(&null_sequence_values),
                0,
                1,
                Some(&good),
                0,
                1
            )
            .is_err()
        );

        assert!(
            equals_core(
                false,
                TextRef::Sequence(&char_failure),
                0,
                TextRef::Chars(&chars),
                0,
                1
            )
            .is_err()
        );
        assert!(
            equals_core(
                false,
                TextRef::Chars(&chars),
                0,
                TextRef::Sequence(&char_failure),
                0,
                1
            )
            .is_err()
        );
        assert!(
            ends_with_validated(
                false,
                TextRef::Sequence(&char_failure),
                0,
                1,
                TextRef::Chars(&chars),
                0,
                1
            )
            .is_err()
        );
        assert!(
            ends_with_validated(
                false,
                TextRef::Chars(&chars),
                0,
                1,
                TextRef::Sequence(&char_failure),
                0,
                1
            )
            .is_err()
        );
        assert!(
            contains_validated(
                false,
                TextRef::Chars(&chars),
                0,
                1,
                TextRef::Sequence(&char_failure),
                0,
                1
            )
            .is_err()
        );
        assert!(
            compare_core(
                false,
                TextRef::Sequence(&char_failure),
                0,
                1,
                TextRef::Chars(&chars),
                0,
                1
            )
            .is_err()
        );
        assert!(
            compare_core(
                false,
                TextRef::Chars(&chars),
                0,
                1,
                TextRef::Sequence(&char_failure),
                0,
                1
            )
            .is_err()
        );
        assert!(hash_part_whole(0, Some(&length_failure)).is_err());
        assert!(hash_part_range(0, Some(&length_failure), 0, 1).is_err());
        assert!(hash_part_range(0, None, 0, 0).is_err());
        assert_eq!(hash_part_range(0, Some(&plain), 0, 1), Ok('a' as i32));
        assert!(hash_part_range(1, Some(&char_failure), 0, 1).is_err());
        assert!(hash_part_range(1, None, 0, 1).is_err());
        assert!(TextUtils::hash_pair(None, Some(&good)).is_err());
        assert!(TextUtils::hash_triple(None, Some(&good), Some(&good)).is_err());
        assert!(TextUtils::hash_triple(Some(&good), None, Some(&good)).is_err());
        assert!(TextUtils::hash_quadruple(None, Some(&good), Some(&good), Some(&good)).is_err());
        assert!(TextUtils::hash_quadruple(Some(&good), None, Some(&good), Some(&good)).is_err());
        assert!(TextUtils::hash_quadruple(Some(&good), Some(&good), None, Some(&good)).is_err());
        assert!(
            TextUtils::hash_quintuple(None, Some(&good), Some(&good), Some(&good), Some(&good))
                .is_err()
        );
        assert!(
            TextUtils::hash_quintuple(Some(&good), None, Some(&good), Some(&good), Some(&good))
                .is_err()
        );
        assert!(
            TextUtils::hash_quintuple(Some(&good), Some(&good), None, Some(&good), Some(&good))
                .is_err()
        );
    }
}
