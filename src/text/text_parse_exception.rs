use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use crate::util::JavaString;

const TEXT_PARSE_EXCEPTION_CLASS: &str = "org.thymeleaf.templateparser.text.TextParseException";

/// `TextParseException` 的 Java Throwable 原因适配。
///
/// 对应 Java: `java.lang.Throwable`，由
/// `org.thymeleaf.templateparser.text.TextParseException` 构造器接收。
///
/// Rust `Error` 不暴露可空 UTF-16 消息或 Java 类名，本对象显式保存这些可观察
/// 元数据，并保留底层错误对象供 [`Error::source`] 返回。由同类型解析异常创建时，
/// 还保存其可空行列，以复现上游 `instanceof TextParseException` 继承逻辑。
pub struct TextParseCause {
    error: Box<dyn Error + Send + Sync>,
    java_class_name: String,
    java_message: Option<JavaString>,
    text_parse_location: Option<(i32, i32, JavaString)>,
}

impl TextParseCause {
    /// 使用显式 Java 元数据包装普通 Rust 错误。
    ///
    /// # 参数
    /// - `error`：原因对象，所有权转移但分配身份保持不变。
    /// - `java_class_name`：Java `getClass().getName()`。
    /// - `java_message`：Java `Throwable#getMessage()`，允许 null 和孤立代理项。
    ///
    /// # 返回
    /// 不携带 TextParseException 行列继承标记的原因。
    #[must_use]
    pub fn with_java_metadata(
        error: Box<dyn Error + Send + Sync>,
        java_class_name: impl Into<String>,
        java_message: Option<JavaString>,
    ) -> Self {
        Self {
            error,
            java_class_name: java_class_name.into(),
            java_message,
            text_parse_location: None,
        }
    }

    /// 将另一个 `TextParseException` 作为原因并启用行列继承。
    ///
    /// # 参数
    /// - `exception`：被包装的同类型异常。
    ///
    /// # 返回
    /// 保存原消息、行列和 Java 类名的原因；底层 Box 供 source 链使用。
    #[must_use]
    pub fn from_text_parse(exception: TextParseException) -> Self {
        let java_message = exception.message.clone();
        // 上游所有设置 line/col 的构造器也必定设置位置前缀消息，把这一不变量编码
        // 到适配状态中，避免产生 Java 不可构造的“有位置但消息为 null”组合。
        let text_parse_location = exception.line.zip(exception.col).map(|(line, col)| {
            (
                line,
                col,
                java_message
                    .clone()
                    .expect("located TextParseException always has a message"),
            )
        });
        Self {
            error: Box::new(exception),
            java_class_name: TEXT_PARSE_EXCEPTION_CLASS.to_owned(),
            java_message,
            text_parse_location,
        }
    }

    /// 返回原因的 Java 类全限定名。
    ///
    /// # 返回
    /// 构造适配器时保存的 `Throwable#getClass().getName()`。
    #[must_use]
    pub fn java_class_name(&self) -> &str {
        &self.java_class_name
    }

    fn source_error(&self) -> &(dyn Error + 'static) {
        self.error.as_ref()
    }
}

impl Debug for TextParseCause {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TextParseCause")
            .field("java_class_name", &self.java_class_name)
            .field("java_message", &self.java_message)
            .field("text_parse_location", &self.text_parse_location)
            .finish_non_exhaustive()
    }
}

/// 文本模板解析期间产生的 checked exception。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.TextParseException`。
///
/// 本对象完整映射八个公开构造器、可空 UTF-16 消息、原因链和可空行列。用另一个
/// 带行列的 `TextParseException` 作为原因时，外层异常继承行列并按上游规则重新
/// 拼接消息；显式 location 构造器则始终使用调用方行列，不读取原因消息。
#[derive(Debug)]
pub struct TextParseException {
    message: Option<JavaString>,
    line: Option<i32>,
    col: Option<i32>,
    cause: Option<TextParseCause>,
}

impl TextParseException {
    /// 创建消息、原因和行列均为 null 的异常。
    ///
    /// 对应 Java: `TextParseException#TextParseException()`。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            message: None,
            line: None,
            col: None,
            cause: None,
        }
    }

    /// 仅使用可空消息创建异常。
    ///
    /// 对应 Java: `TextParseException#TextParseException(String)`。
    ///
    /// # 参数
    /// - `message`：原始 Java UTF-16 消息；`None` 对应 null。
    #[must_use]
    pub fn with_message(message: Option<JavaString>) -> Self {
        Self {
            message,
            line: None,
            col: None,
            cause: None,
        }
    }

    /// 使用可空消息和可空原因创建异常。
    ///
    /// 对应 Java: `TextParseException#TextParseException(String,Throwable)`。
    ///
    /// # 参数
    /// - `message`：调用方消息。
    /// - `cause`：原因；`None` 对应 Java null。
    ///
    /// # 返回
    /// 同类型且带行列的原因会把行列传播到新异常。
    #[must_use]
    pub fn with_message_and_cause(
        message: Option<JavaString>,
        cause: Option<TextParseCause>,
    ) -> Self {
        let (line, col) = inherited_location(cause.as_ref());
        let message = compose_inherited_message(message.as_ref(), cause.as_ref());
        Self {
            message,
            line,
            col,
            cause,
        }
    }

    /// 仅使用可空原因创建异常。
    ///
    /// 对应 Java: `TextParseException#TextParseException(Throwable)`。
    ///
    /// # 参数
    /// - `cause`：原因；`None` 对应 Java null。
    #[must_use]
    pub fn with_cause(cause: Option<TextParseCause>) -> Self {
        Self::with_message_and_cause(None, cause)
    }

    /// 使用显式行列创建异常。
    ///
    /// 对应 Java: `TextParseException#TextParseException(int,int)`。
    ///
    /// # 参数
    /// - `line`：原样保存的行号，包括负数。
    /// - `col`：原样保存的列号，包括负数。
    #[must_use]
    pub fn with_location(line: i32, col: i32) -> Self {
        Self {
            message: Some(message_prefix(line, col)),
            line: Some(line),
            col: Some(col),
            cause: None,
        }
    }

    /// 使用可空消息、可空原因和显式行列创建异常。
    ///
    /// 对应 Java:
    /// `TextParseException#TextParseException(String,Throwable,int,int)`。
    ///
    /// # 参数
    /// - `message`：追加到位置前缀后的消息；null 按 Java 拼接为 `"null"`。
    /// - `cause`：原因；不参与消息或位置推导。
    /// - `line`：显式行号。
    /// - `col`：显式列号。
    #[must_use]
    pub fn with_message_and_cause_at(
        message: Option<&JavaString>,
        cause: Option<TextParseCause>,
        line: i32,
        col: i32,
    ) -> Self {
        Self {
            message: Some(prefix_with_message(line, col, message)),
            line: Some(line),
            col: Some(col),
            cause,
        }
    }

    /// 使用可空消息和显式行列创建异常。
    ///
    /// 对应 Java: `TextParseException#TextParseException(String,int,int)`。
    ///
    /// # 参数
    /// - `message`：追加消息；`None` 拼接为 `"null"`。
    /// - `line`：显式行号。
    /// - `col`：显式列号。
    #[must_use]
    pub fn with_message_at(message: Option<&JavaString>, line: i32, col: i32) -> Self {
        Self::with_message_and_cause_at(message, None, line, col)
    }

    /// 使用可空原因和显式行列创建异常。
    ///
    /// 对应 Java: `TextParseException#TextParseException(Throwable,int,int)`。
    ///
    /// # 参数
    /// - `cause`：原因；不参与前缀消息计算。
    /// - `line`：显式行号。
    /// - `col`：显式列号。
    #[must_use]
    pub fn with_cause_at(cause: Option<TextParseCause>, line: i32, col: i32) -> Self {
        Self {
            message: Some(message_prefix(line, col)),
            line: Some(line),
            col: Some(col),
            cause,
        }
    }

    /// 返回可空 Java UTF-16 消息。
    ///
    /// 对应 Java: 继承的 `Throwable#getMessage()`。
    ///
    /// # 返回
    /// 构造器最终保存的消息；`None` 对应 Java null。
    #[must_use]
    pub fn get_message(&self) -> Option<&JavaString> {
        self.message.as_ref()
    }

    /// 返回可空行号。
    ///
    /// 对应 Java: `TextParseException#getLine()`。
    ///
    /// # 返回
    /// 显式位置或从同类型原因继承的位置；缺失为 `None`。
    #[must_use]
    pub const fn get_line(&self) -> Option<i32> {
        self.line
    }

    /// 返回可空列号。
    ///
    /// 对应 Java: `TextParseException#getCol()`。
    ///
    /// # 返回
    /// 显式位置或从同类型原因继承的位置；缺失为 `None`。
    #[must_use]
    pub const fn get_col(&self) -> Option<i32> {
        self.col
    }

    /// 返回原因适配对象。
    ///
    /// # 返回
    /// 原因存在时返回共享借用，用于检查 Java 类名和原因身份。
    #[must_use]
    pub const fn get_cause(&self) -> Option<&TextParseCause> {
        self.cause.as_ref()
    }
}

impl Default for TextParseException {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for TextParseException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            &self
                .message
                .as_ref()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
        )
    }
}

impl Error for TextParseException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_ref().map(TextParseCause::source_error)
    }
}

fn inherited_location(cause: Option<&TextParseCause>) -> (Option<i32>, Option<i32>) {
    cause
        .and_then(|cause| cause.text_parse_location.as_ref())
        .map_or((None, None), |(line, col, _)| (Some(*line), Some(*col)))
}

fn compose_inherited_message(
    message: Option<&JavaString>,
    cause: Option<&TextParseCause>,
) -> Option<JavaString> {
    if let Some(cause) = cause {
        if let Some((line, col, cause_message)) = cause.text_parse_location.as_ref() {
            let mut result = message_prefix(*line, *col).as_utf16().to_vec();
            match message {
                Some(message) => {
                    result.push(u16::from(b' '));
                    result.extend_from_slice(message.as_utf16());
                }
                None => result.extend_from_slice(cause_message.as_utf16()),
            }
            return Some(JavaString::from_utf16(result));
        }
    }
    if let Some(message) = message {
        return Some(message.clone());
    }
    cause.and_then(|cause| cause.java_message.clone())
}

fn message_prefix(line: i32, col: i32) -> JavaString {
    JavaString::from_rust_str(&format!("(Line = {line}, Column = {col})"))
}

fn prefix_with_message(line: i32, col: i32, message: Option<&JavaString>) -> JavaString {
    let mut result = message_prefix(line, col).as_utf16().to_vec();
    result.push(u16::from(b' '));
    match message {
        Some(message) => result.extend_from_slice(message.as_utf16()),
        None => result.extend("null".encode_utf16()),
    }
    JavaString::from_utf16(result)
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fmt::{Display, Formatter};

    use super::{TextParseCause, TextParseException};
    use crate::util::JavaString;

    #[derive(Debug)]
    struct PlainError;

    impl Display for PlainError {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("cause")
        }
    }

    impl Error for PlainError {}

    #[test]
    fn default_display_and_source_identity_are_preserved() {
        assert!(TextParseException::default().get_message().is_none());
        assert_eq!(TextParseException::new().to_string(), "null");

        let error: Box<dyn Error + Send + Sync> = Box::new(PlainError);
        let identity = error.as_ref() as *const dyn Error as *const ();
        let cause = TextParseCause::with_java_metadata(
            error,
            "example.PlainError",
            Some(JavaString::from_rust_str("cause")),
        );
        assert_eq!(PlainError.to_string(), "cause");
        assert!(format!("{cause:?}").contains("example.PlainError"));
        let exception = TextParseException::with_cause(Some(cause));
        let source_identity = exception.source().expect("source") as *const dyn Error as *const ();
        assert_eq!(source_identity, identity);
        assert_eq!(
            exception.get_cause().expect("cause").java_class_name(),
            "example.PlainError"
        );
    }
}
