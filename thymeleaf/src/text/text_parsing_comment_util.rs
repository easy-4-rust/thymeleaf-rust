use std::error::Error;
use std::fmt::{Display, Formatter};

use super::{ITextHandler, TextParseException};
use crate::util::JavaString;

/// 文本注释解析中的 checked exception 与 Java 运行时异常适配。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.TextParsingCommentUtil` 的可观察失败路径。
#[derive(Debug)]
pub(crate) enum TextParsingCommentError {
    /// 处理器或格式校验产生的 `TextParseException`。
    TextParse(Box<TextParseException>),
    /// Java `char[]` 取值指令收到 null。
    NullArrayLoad,
    /// Java `String(char[],int,int)` 收到 null。
    NullStringValue,
    /// Java `char[]` 下标越界。
    ArrayIndex { index: i32, size: usize },
    /// Java 字符串构造范围越界。
    StringRange { offset: i32, len: i32, size: usize },
}

impl TextParsingCommentError {
    /// 返回 Java 异常全限定名。
    pub(crate) const fn java_class_name(&self) -> &'static str {
        match self {
            Self::TextParse(_) => "org.thymeleaf.templateparser.text.TextParseException",
            Self::NullArrayLoad | Self::NullStringValue => "java.lang.NullPointerException",
            Self::ArrayIndex { .. } => "java.lang.ArrayIndexOutOfBoundsException",
            Self::StringRange { .. } => "java.lang.StringIndexOutOfBoundsException",
        }
    }

    /// 返回 Java `Throwable#getMessage()` 的 UTF-16 表示。
    /// 对应 Java 语义：`TextParsingCommentUtil` 的 `java_message` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn java_message(&self) -> JavaString {
        match self {
            Self::TextParse(exception) => exception
                .get_message()
                .cloned()
                .unwrap_or_else(|| JavaString::from_rust_str("null")),
            Self::NullArrayLoad => JavaString::from_rust_str(
                "Cannot load from char array because \"<parameter1>\" is null",
            ),
            Self::NullStringValue => {
                JavaString::from_rust_str("Cannot read the array length because \"value\" is null")
            }
            Self::ArrayIndex { index, size } => {
                JavaString::from_rust_str(&format!("Index {index} out of bounds for length {size}"))
            }
            Self::StringRange { offset, len, size } => JavaString::from_rust_str(&format!(
                "Range [{offset}, {offset} + {len}) out of bounds for length {size}"
            )),
        }
    }
}

impl Display for TextParsingCommentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.java_message().to_string_lossy())
    }
}

impl Error for TextParsingCommentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TextParse(exception) => Some(exception),
            _ => None,
        }
    }
}

impl From<Box<TextParseException>> for TextParsingCommentError {
    fn from(value: Box<TextParseException>) -> Self {
        Self::TextParse(value)
    }
}

/// 文本模式的 JavaScript/CSS 注释解析工具。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.TextParsingCommentUtil`。
///
/// 本无状态对象保留 `/*...*/` 的边界校验、UTF-16 原文错误消息、Java `int`
/// 回绕、数组运行时异常以及向 [`ITextHandler`] 的同步回调。
pub(crate) struct TextParsingCommentUtil;

impl TextParsingCommentUtil {
    /// 校验并分派一个完整块注释。
    ///
    /// 对应 Java: `TextParsingCommentUtil#parseComment`。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn parse_comment(
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
        handler: &mut dyn ITextHandler,
    ) -> Result<(), TextParsingCommentError> {
        let maxi = offset.wrapping_add(len);
        let valid = len >= 4
            && Self::is_comment_block_start(buffer.as_deref(), offset, maxi)?
            && Self::is_comment_block_end(buffer.as_deref(), maxi.wrapping_sub(2), maxi)?;
        if !valid {
            let source = java_string_from_range(buffer.as_deref(), offset, len)?;
            let mut message = "Could not parse as a well-formed Comment: \""
                .encode_utf16()
                .collect::<Vec<_>>();
            message.extend_from_slice(&source);
            message.push(u16::from(b'"'));
            return Err(Box::new(TextParseException::with_message_at(
                Some(&JavaString::from_utf16(message)),
                line,
                col,
            ))
            .into());
        }

        let buffer = buffer.expect("validated comment dereferenced the buffer");
        handler
            .handle_comment(
                Some(buffer),
                offset.wrapping_add(2),
                len.wrapping_sub(4),
                offset,
                len,
                line,
                col,
            )
            .map_err(Into::into)
    }

    /// 判断当前位置是否为 `/*`。对应 Java: `isCommentBlockStart`。
    pub(crate) fn is_comment_block_start(
        buffer: Option<&[u16]>,
        offset: i32,
        maxi: i32,
    ) -> Result<bool, TextParsingCommentError> {
        two_units_equal(buffer, offset, maxi, u16::from(b'/'), u16::from(b'*'))
    }

    /// 判断当前位置是否为 `*/`。对应 Java: `isCommentBlockEnd`。
    pub(crate) fn is_comment_block_end(
        buffer: Option<&[u16]>,
        offset: i32,
        maxi: i32,
    ) -> Result<bool, TextParsingCommentError> {
        two_units_equal(buffer, offset, maxi, u16::from(b'*'), u16::from(b'/'))
    }

    /// 判断当前位置是否为 `//`。对应 Java: `isCommentLineStart`。
    pub(crate) fn is_comment_line_start(
        buffer: Option<&[u16]>,
        offset: i32,
        maxi: i32,
    ) -> Result<bool, TextParsingCommentError> {
        two_units_equal(buffer, offset, maxi, u16::from(b'/'), u16::from(b'/'))
    }
}

fn two_units_equal(
    buffer: Option<&[u16]>,
    offset: i32,
    maxi: i32,
    first: u16,
    second: u16,
) -> Result<bool, TextParsingCommentError> {
    if maxi.wrapping_sub(offset) <= 1 {
        return Ok(false);
    }
    let buffer = buffer.ok_or(TextParsingCommentError::NullArrayLoad)?;
    if array_unit(buffer, offset)? != first {
        return Ok(false);
    }
    Ok(array_unit(buffer, offset.wrapping_add(1))? == second)
}

fn array_unit(buffer: &[u16], index: i32) -> Result<u16, TextParsingCommentError> {
    usize::try_from(index)
        .ok()
        .and_then(|index| buffer.get(index).copied())
        .ok_or(TextParsingCommentError::ArrayIndex {
            index,
            size: buffer.len(),
        })
}

fn java_string_from_range(
    buffer: Option<&[u16]>,
    offset: i32,
    len: i32,
) -> Result<Vec<u16>, TextParsingCommentError> {
    let buffer = buffer.ok_or(TextParsingCommentError::NullStringValue)?;
    let end = offset.wrapping_add(len);
    let valid = offset >= 0
        && len >= 0
        && end >= offset
        && usize::try_from(end).is_ok_and(|end| end <= buffer.len());
    if !valid {
        return Err(TextParsingCommentError::StringRange {
            offset,
            len,
            size: buffer.len(),
        });
    }
    Ok(buffer[offset as usize..end as usize].to_vec())
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::{TextParsingCommentError, TextParsingCommentUtil};
    use crate::text::{ITextHandler, TextParseException};

    struct CommentHandler {
        args: Option<(i32, i32, i32, i32, i32, i32)>,
        mutate: bool,
        fail: bool,
    }

    impl ITextHandler for CommentHandler {
        fn handle_document_start(
            &mut self,
            _: i64,
            _: i32,
            _: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }
        fn handle_document_end(
            &mut self,
            _: i64,
            _: i64,
            _: i32,
            _: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }
        fn handle_text(
            &mut self,
            _: Option<&mut [u16]>,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }
        fn handle_comment(
            &mut self,
            buffer: Option<&mut [u16]>,
            content_offset: i32,
            content_len: i32,
            outer_offset: i32,
            outer_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.args = Some((
                content_offset,
                content_len,
                outer_offset,
                outer_len,
                line,
                col,
            ));
            if self.mutate {
                let buffer = buffer.expect("comment callback receives the parser buffer");
                buffer[content_offset as usize] = u16::from(b'Z');
            }
            if self.fail {
                return Err(Box::new(TextParseException::with_message_at(
                    Some(&crate::util::JavaString::from_rust_str("handler")),
                    41,
                    43,
                )));
            }
            Ok(())
        }
        fn handle_standalone_element_start(
            &mut self,
            _: Option<&mut [u16]>,
            _: i32,
            _: i32,
            _: bool,
            _: i32,
            _: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }
        fn handle_standalone_element_end(
            &mut self,
            _: Option<&mut [u16]>,
            _: i32,
            _: i32,
            _: bool,
            _: i32,
            _: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }
        fn handle_open_element_start(
            &mut self,
            _: Option<&mut [u16]>,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }
        fn handle_open_element_end(
            &mut self,
            _: Option<&mut [u16]>,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }
        fn handle_close_element_start(
            &mut self,
            _: Option<&mut [u16]>,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }
        fn handle_close_element_end(
            &mut self,
            _: Option<&mut [u16]>,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }
        fn handle_attribute(
            &mut self,
            _: Option<&mut [u16]>,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }
    }

    fn handler(mutate: bool, fail: bool) -> CommentHandler {
        CommentHandler {
            args: None,
            mutate,
            fail,
        }
    }

    #[test]
    fn parses_empty_embedded_mutable_and_handler_failure_cases() {
        let mut buffer: Vec<u16> = "x/*a*/y".encode_utf16().collect();
        let mut receiver = handler(true, false);
        TextParsingCommentUtil::parse_comment(Some(&mut buffer), 1, 5, 7, 9, &mut receiver)
            .expect("valid comment");
        assert_eq!(receiver.args, Some((3, 1, 1, 5, 7, 9)));
        assert_eq!(buffer[3], u16::from(b'Z'));

        let mut empty: Vec<u16> = "/**/".encode_utf16().collect();
        let mut receiver = handler(false, false);
        TextParsingCommentUtil::parse_comment(Some(&mut empty), 0, 4, -1, -2, &mut receiver)
            .expect("empty comment");
        assert_eq!(receiver.args, Some((2, 0, 0, 4, -1, -2)));

        let mut failed: Vec<u16> = "/*a*/".encode_utf16().collect();
        let error = TextParsingCommentUtil::parse_comment(
            Some(&mut failed),
            0,
            5,
            2,
            4,
            &mut handler(false, true),
        )
        .expect_err("handler failure");
        assert_eq!(
            error.java_class_name(),
            "org.thymeleaf.templateparser.text.TextParseException"
        );
        assert_eq!(
            error.java_message().to_string_lossy(),
            "(Line = 41, Column = 43) handler"
        );
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn preserves_predicate_short_circuit_and_runtime_failures() {
        assert!(!TextParsingCommentUtil::is_comment_block_start(None, 0, 1).expect("short"));
        let error =
            TextParsingCommentUtil::is_comment_block_start(None, 0, 2).expect_err("null load");
        assert_eq!(error.java_class_name(), "java.lang.NullPointerException");
        assert_eq!(
            error.java_message().to_string_lossy(),
            "Cannot load from char array because \"<parameter1>\" is null"
        );
        let buffer: Vec<u16> = "/*//*/".encode_utf16().collect();
        assert!(TextParsingCommentUtil::is_comment_block_start(Some(&buffer), 0, 2).unwrap());
        assert!(TextParsingCommentUtil::is_comment_line_start(Some(&buffer), 2, 4).unwrap());
        assert!(TextParsingCommentUtil::is_comment_block_end(Some(&buffer), 4, 6).unwrap());
        assert!(
            !TextParsingCommentUtil::is_comment_block_start(Some(&buffer), i32::MIN, i32::MAX)
                .unwrap()
        );
        let error = TextParsingCommentUtil::is_comment_block_start(Some(&buffer), -1, 2)
            .expect_err("negative index");
        assert_eq!(
            error.java_message().to_string_lossy(),
            "Index -1 out of bounds for length 6"
        );
    }

    #[test]
    fn reproduces_malformed_string_construction_order_and_messages() {
        let mut handler = handler(false, false);
        let short = TextParsingCommentUtil::parse_comment(None, 0, 3, 1, 2, &mut handler)
            .expect_err("null string");
        assert_eq!(
            short.java_message().to_string_lossy(),
            "Cannot read the array length because \"value\" is null"
        );
        let long = TextParsingCommentUtil::parse_comment(None, 0, 4, 1, 2, &mut handler)
            .expect_err("null load");
        assert_eq!(
            long.java_message().to_string_lossy(),
            "Cannot load from char array because \"<parameter1>\" is null"
        );

        let mut truncated: Vec<u16> = "/*".encode_utf16().collect();
        let end_error =
            TextParsingCommentUtil::parse_comment(Some(&mut truncated), 0, 4, 1, 2, &mut handler)
                .expect_err("block end array access");
        assert_eq!(
            end_error.java_message().to_string_lossy(),
            "Index 2 out of bounds for length 2"
        );

        let mut buffer: Vec<u16> = "/*a*/".encode_utf16().collect();
        let range =
            TextParsingCommentUtil::parse_comment(Some(&mut buffer), 0, 6, 1, 2, &mut handler)
                .expect_err("string range");
        assert_eq!(
            range.java_message().to_string_lossy(),
            "Range [0, 0 + 6) out of bounds for length 5"
        );
        let malformed =
            TextParsingCommentUtil::parse_comment(Some(&mut buffer), 0, 3, 1, 2, &mut handler)
                .expect_err("malformed");
        assert_eq!(
            malformed.java_message().to_string_lossy(),
            "(Line = 1, Column = 2) Could not parse as a well-formed Comment: \"/*a\""
        );
    }

    #[test]
    fn covers_interface_callbacks_and_every_runtime_error_adapter() {
        let mut receiver = handler(false, false);
        let mut buffer: Vec<u16> = "n=o".encode_utf16().collect();
        receiver.handle_document_start(11, 1, 2).unwrap();
        receiver.handle_document_end(13, 2, 3, 4).unwrap();
        receiver.handle_text(Some(&mut buffer), 0, 3, 5, 6).unwrap();
        receiver
            .handle_standalone_element_start(Some(&mut buffer), 0, 1, true, 9, 10)
            .unwrap();
        receiver
            .handle_standalone_element_end(Some(&mut buffer), 0, 1, false, 11, 12)
            .unwrap();
        receiver
            .handle_open_element_start(Some(&mut buffer), 0, 1, 13, 14)
            .unwrap();
        receiver
            .handle_open_element_end(Some(&mut buffer), 0, 1, 15, 16)
            .unwrap();
        receiver
            .handle_close_element_start(Some(&mut buffer), 0, 1, 17, 18)
            .unwrap();
        receiver
            .handle_close_element_end(Some(&mut buffer), 0, 1, 19, 20)
            .unwrap();
        receiver
            .handle_attribute(
                Some(&mut buffer),
                0,
                1,
                21,
                22,
                1,
                1,
                23,
                24,
                2,
                1,
                1,
                2,
                25,
                26,
            )
            .unwrap();

        let runtime_errors = [
            TextParsingCommentError::NullStringValue,
            TextParsingCommentError::ArrayIndex { index: -1, size: 3 },
            TextParsingCommentError::StringRange {
                offset: 0,
                len: 4,
                size: 3,
            },
        ];
        assert_eq!(
            runtime_errors[0].java_message().to_string_lossy(),
            "Cannot read the array length because \"value\" is null"
        );
        assert_eq!(
            runtime_errors[1].java_message().to_string_lossy(),
            "Index -1 out of bounds for length 3"
        );
        assert_eq!(
            runtime_errors[2].java_class_name(),
            "java.lang.StringIndexOutOfBoundsException"
        );
        assert_eq!(
            runtime_errors[2].to_string(),
            "Range [0, 0 + 4) out of bounds for length 3"
        );
        assert!(runtime_errors.iter().all(|error| error.source().is_none()));

        let null_message = TextParsingCommentError::TextParse(Box::default());
        assert_eq!(null_message.java_message().to_string_lossy(), "null");
    }

    #[test]
    fn exhausts_comment_predicate_offset_and_maximum_matrix() {
        let buffer: Vec<u16> = "/*//*/".encode_utf16().collect();
        let mut outcomes = (0_u32, 0_u32);
        for offset in -2..=8 {
            for maxi in -2..=8 {
                for result in [
                    TextParsingCommentUtil::is_comment_block_start(Some(&buffer), offset, maxi),
                    TextParsingCommentUtil::is_comment_block_end(Some(&buffer), offset, maxi),
                    TextParsingCommentUtil::is_comment_line_start(Some(&buffer), offset, maxi),
                ] {
                    if result.is_ok() {
                        outcomes.0 += 1;
                    } else {
                        outcomes.1 += 1;
                    }
                }
            }
        }
        assert_eq!(outcomes, (305, 58));
    }
}
