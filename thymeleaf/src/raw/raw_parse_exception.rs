use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use crate::util::Utf16String;

const RAW_PARSE_EXCEPTION_CLASS: &str = "org.thymeleaf.templateparser.raw.RawParseException";

/// `RawParseException` 接收的 Java Throwable 原因适配。
///
/// 对应 Java: `java.lang.Throwable`，由
/// `org.thymeleaf.templateparser.raw.RawParseException` 构造器接收。
///
/// Rust 的 `Error` 无法表达 Java 可空 UTF-16 消息与运行时类名，因此此对象同时
/// 保存这些可观察元数据；同类型原因还携带可继承的行列信息。
pub struct RawParseCause {
    error: Box<dyn Error + Send + Sync>,
    class_name: String,
    java_message: Option<Utf16String>,
    raw_parse_location: Option<(i32, i32, Utf16String)>,
}

impl RawParseCause {
    /// 使用 Java 异常元数据包装普通 Rust 原因。
    ///
    /// # 参数
    ///
    /// - `error`：底层 Rust 错误。
    /// - `class_name`：Java 运行时类全限定名。
    /// - `java_message`：可空的 `Throwable#getMessage()`。
    ///
    /// # 返回
    ///
    /// 不参与 RawParseException 行列继承的普通原因。
    #[must_use]
    /// 对应 Java 语义：`RawParseException` 的 `with_java_metadata` 行为（Rust 侧辅助/私有路径）。
    pub fn with_java_metadata(
        error: Box<dyn Error + Send + Sync>,
        class_name: impl Into<String>,
        java_message: Option<Utf16String>,
    ) -> Self {
        Self {
            error,
            class_name: class_name.into(),
            java_message,
            raw_parse_location: None,
        }
    }

    /// 将另一个 `RawParseException` 包装为可继承行列的原因。
    ///
    /// # 参数
    ///
    /// - `exception`：作为 cause 的同类型异常。
    ///
    /// # 返回
    ///
    /// 保留异常身份、消息以及可空行列的原因适配。
    #[must_use]
    /// 对应 Java 语义：`RawParseException` 的 `from_raw_parse` 行为（Rust 侧辅助/私有路径）。
    pub fn from_raw_parse(exception: RawParseException) -> Self {
        let java_message = exception.message.clone();
        let raw_parse_location = exception.line.zip(exception.col).map(|(line, col)| {
            (
                line,
                col,
                java_message
                    .clone()
                    .expect("located RawParseException always has a message"),
            )
        });
        Self {
            error: Box::new(exception),
            class_name: RAW_PARSE_EXCEPTION_CLASS.to_owned(),
            java_message,
            raw_parse_location,
        }
    }

    /// 返回原因的 Java 运行时类全限定名。
    #[must_use]
    /// 对应 Java 语义：`RawParseException` 的 `class_name` 行为（Rust 侧辅助/私有路径）。
    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    fn source_error(&self) -> &(dyn Error + 'static) {
        self.error.as_ref()
    }
}

impl Debug for RawParseCause {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RawParseCause")
            .field("class_name", &self.class_name)
            .field("java_message", &self.java_message)
            .field("raw_parse_location", &self.raw_parse_location)
            .finish_non_exhaustive()
    }
}

/// RAW 模板解析期间产生的 checked exception。
///
/// 对应 Java: `org.thymeleaf.templateparser.raw.RawParseException`。
///
/// 完整映射八个公开构造器、可空 UTF-16 消息、原因链及可空行列；当原因是带
/// 位置的同类型异常时，按 Java 私有 `message` 方法继承位置并重组消息。
#[derive(Debug)]
pub struct RawParseException {
    message: Option<Utf16String>,
    line: Option<i32>,
    col: Option<i32>,
    cause: Option<RawParseCause>,
}

impl RawParseException {
    /// 创建消息、原因、行列均为 null 的异常。
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
    /// # 参数
    ///
    /// - `message`：原始 Java UTF-16 消息，`None` 对应 null。
    #[must_use]
    ///
    /// 对应 Java 语义：`RawParseException` 的 `with_message` 行为（Rust 侧辅助/私有路径）。
    pub fn with_message(message: Option<Utf16String>) -> Self {
        Self {
            message,
            line: None,
            col: None,
            cause: None,
        }
    }

    /// 使用可空消息与原因创建异常，并继承同类型原因的位置。
    ///
    /// # 参数
    ///
    /// - `message`：调用方消息。
    /// - `cause`：可空原因。
    #[must_use]
    ///
    /// 对应 Java 语义：`RawParseException` 的 `with_message_and_cause` 行为（Rust 侧辅助/私有路径）。
    pub fn with_message_and_cause(
        message: Option<Utf16String>,
        cause: Option<RawParseCause>,
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
    /// # 参数
    ///
    /// - `cause`：可空原因；同类型原因的行列会被继承。
    #[must_use]
    ///
    /// 对应 Java 语义：`RawParseException` 的 `with_cause` 行为（Rust 侧辅助/私有路径）。
    pub fn with_cause(cause: Option<RawParseCause>) -> Self {
        Self::with_message_and_cause(None, cause)
    }

    /// 使用显式行列创建异常。
    ///
    /// # 参数
    ///
    /// - `line`：原样保存的行号。
    /// - `col`：原样保存的列号。
    #[must_use]
    ///
    /// 对应 Java 语义：`RawParseException` 的 `with_location` 行为（Rust 侧辅助/私有路径）。
    pub fn with_location(line: i32, col: i32) -> Self {
        Self {
            message: Some(message_prefix(line, col)),
            line: Some(line),
            col: Some(col),
            cause: None,
        }
    }

    /// 使用消息、原因与显式行列创建异常。
    ///
    /// # 参数
    ///
    /// - `message`：追加在位置前缀后的消息；null 拼接为 `"null"`。
    /// - `cause`：只作为原因链保存。
    /// - `line`：显式行号。
    /// - `col`：显式列号。
    #[must_use]
    ///
    /// 对应 Java 语义：`RawParseException` 的 `with_message_and_cause_at` 行为（Rust 侧辅助/私有路径）。
    pub fn with_message_and_cause_at(
        message: Option<&Utf16String>,
        cause: Option<RawParseCause>,
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

    /// 使用消息与显式行列创建异常。
    ///
    /// # 参数
    ///
    /// - `message`：追加消息；`None` 按 Java 拼接为 `"null"`。
    /// - `line`：显式行号。
    /// - `col`：显式列号。
    #[must_use]
    ///
    /// 对应 Java 语义：`RawParseException` 的 `with_message_at` 行为（Rust 侧辅助/私有路径）。
    pub fn with_message_at(message: Option<&Utf16String>, line: i32, col: i32) -> Self {
        Self::with_message_and_cause_at(message, None, line, col)
    }

    /// 使用原因与显式行列创建异常。
    ///
    /// # 参数
    ///
    /// - `cause`：可空原因。
    /// - `line`：显式行号。
    /// - `col`：显式列号。
    #[must_use]
    ///
    /// 对应 Java 语义：`RawParseException` 的 `with_cause_at` 行为（Rust 侧辅助/私有路径）。
    pub fn with_cause_at(cause: Option<RawParseCause>, line: i32, col: i32) -> Self {
        Self {
            message: Some(message_prefix(line, col)),
            line: Some(line),
            col: Some(col),
            cause,
        }
    }

    /// 返回构造器最终保存的可空 Java UTF-16 消息。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `getMessage()` 的 Rust 移植（`RawParseException` 继承路径）。
    pub fn get_message(&self) -> Option<&Utf16String> {
        self.message.as_ref()
    }

    /// 返回显式位置或从同类型原因继承的可空行号。
    #[must_use]
    pub const fn get_line(&self) -> Option<i32> {
        self.line
    }

    /// 返回显式位置或从同类型原因继承的可空列号。
    #[must_use]
    pub const fn get_col(&self) -> Option<i32> {
        self.col
    }

    /// 返回可空原因适配对象。
    #[must_use]
    pub const fn get_cause(&self) -> Option<&RawParseCause> {
        self.cause.as_ref()
    }
}

impl Default for RawParseException {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for RawParseException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            &self
                .message
                .as_ref()
                .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy),
        )
    }
}

impl Error for RawParseException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_ref().map(RawParseCause::source_error)
    }
}

fn inherited_location(cause: Option<&RawParseCause>) -> (Option<i32>, Option<i32>) {
    cause
        .and_then(|cause| cause.raw_parse_location.as_ref())
        .map_or((None, None), |(line, col, _)| (Some(*line), Some(*col)))
}

fn compose_inherited_message(
    message: Option<&Utf16String>,
    cause: Option<&RawParseCause>,
) -> Option<Utf16String> {
    if let Some(cause) = cause
        && let Some((line, col, cause_message)) = cause.raw_parse_location.as_ref()
    {
        let mut result = message_prefix(*line, *col).as_utf16().to_vec();
        match message {
            Some(message) => {
                result.push(u16::from(b' '));
                result.extend_from_slice(message.as_utf16());
            }
            None => result.extend_from_slice(cause_message.as_utf16()),
        }
        return Some(Utf16String::from_utf16(result));
    }
    if let Some(message) = message {
        return Some(message.clone());
    }
    cause.and_then(|cause| cause.java_message.clone())
}

fn message_prefix(line: i32, col: i32) -> Utf16String {
    Utf16String::from_rust_str(&format!("(Line = {line}, Column = {col})"))
}

fn prefix_with_message(line: i32, col: i32, message: Option<&Utf16String>) -> Utf16String {
    let mut result = message_prefix(line, col).as_utf16().to_vec();
    result.push(u16::from(b' '));
    match message {
        Some(message) => result.extend_from_slice(message.as_utf16()),
        None => result.extend("null".encode_utf16()),
    }
    Utf16String::from_utf16(result)
}
