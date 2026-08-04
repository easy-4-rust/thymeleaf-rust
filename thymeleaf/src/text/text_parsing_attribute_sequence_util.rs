use std::error::Error;
use std::fmt::{Display, Formatter};

use super::{ITextHandler, TextParseException, TextParsingUtil, TextParsingUtilError};
use crate::util::Utf16String;

const NULL_HANDLER_MESSAGE: &str = "Cannot invoke \"org.thymeleaf.templateparser.text.ITextHandler.handleAttribute(char[], int, int, int, int, int, int, int, int, int, int, int, int, int, int)\" because \"<parameter6>\" is null";

/// 属性序列解析中的 checked exception 与 Java 运行时异常适配。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.TextParsingAttributeSequenceUtil` 的可观察失败路径。
#[derive(Debug)]
pub(crate) enum TextParsingAttributeSequenceError {
    /// 属性处理器或属性名校验产生的 `TextParseException`。
    TextParse(Box<TextParseException>),
    /// 通用文本扫描器产生的 Java 数组运行时异常。
    Scanning(TextParsingUtilError),
    /// 实际需要派发属性事件时 handler 为 null。
    NullHandler,
    /// Java `String(char[],offset,len)` 的范围非法。
    StringRange {
        /// 起始下标。
        offset: i32,
        /// 请求长度。
        len: i32,
        /// 数组长度。
        length: usize,
    },
}

impl TextParsingAttributeSequenceError {
    /// 返回对应 Java 异常全限定名。
    pub(crate) const fn class_name(&self) -> &'static str {
        match self {
            Self::TextParse(_) => "org.thymeleaf.templateparser.text.TextParseException",
            Self::Scanning(error) => error.class_name(),
            Self::NullHandler => "java.lang.NullPointerException",
            Self::StringRange { .. } => "java.lang.StringIndexOutOfBoundsException",
        }
    }

    /// 返回 Java `String.valueOf(Throwable#getMessage())` 的 UTF-16 表示。
    /// 对应 Java 语义：`TextParsingAttributeSequenceUtil` 的 `message` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn message(&self) -> Utf16String {
        match self {
            Self::TextParse(exception) => exception
                .get_message()
                .cloned()
                .unwrap_or_else(|| Utf16String::from_rust_str("null")),
            Self::Scanning(error) => error
                .message()
                .unwrap_or_else(|| Utf16String::from_rust_str("null")),
            Self::NullHandler => Utf16String::from_rust_str(NULL_HANDLER_MESSAGE),
            Self::StringRange {
                offset,
                len,
                length,
            } => Utf16String::from_rust_str(&format!(
                "Range [{offset}, {offset} + {len}) out of bounds for length {length}"
            )),
        }
    }
}

impl Display for TextParsingAttributeSequenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message().to_string_lossy())
    }
}

impl Error for TextParsingAttributeSequenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TextParse(exception) => Some(exception),
            Self::Scanning(error) => Some(error),
            Self::NullHandler | Self::StringRange { .. } => None,
        }
    }
}

impl From<Box<TextParseException>> for TextParsingAttributeSequenceError {
    fn from(value: Box<TextParseException>) -> Self {
        Self::TextParse(value)
    }
}

impl From<TextParsingUtilError> for TextParsingAttributeSequenceError {
    fn from(value: TextParsingUtilError) -> Self {
        Self::Scanning(value)
    }
}

/// 文本模式元素中的属性序列解析工具。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.TextParsingAttributeSequenceUtil`。
///
/// 任意输入都按名称、运算符和值的顺序扫描。内部空白不产生事件；无等号属性、
/// 空值、带空白的运算符、单双引号 outer/content 范围、行列位置、处理器修改的
/// 后续可见性和 Java `int` 回绕均保持上游语义。
pub(crate) struct TextParsingAttributeSequenceUtil;

impl TextParsingAttributeSequenceUtil {
    /// 解析指定 UTF-16 范围并按源顺序派发属性事件。
    ///
    /// 对应 Java:
    /// `TextParsingAttributeSequenceUtil#parseAttributeSequence(char[],int,int,int,int,ITextHandler)`。
    ///
    /// # 参数
    /// - `buffer`：Java 参数 `buffer`，允许 null；空范围不会读取它。
    /// - `offset` / `len`：扫描范围，求和按 Java `int` 回绕。
    /// - `line` / `col`：范围起点的行列。
    /// - `handler`：Java 参数 `handler`；只有实际属性事件才访问它。
    ///
    /// # 错误
    /// 保留属性名非法的 checked exception、handler checked exception、null 和数组
    /// 越界的 Java 类别、消息及失败前已派发事件/缓冲区修改。
    #[allow(clippy::too_many_lines)]
    pub(crate) fn parse_attribute_sequence(
        mut buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
        mut handler: Option<&mut dyn ITextHandler>,
    ) -> Result<(), TextParsingAttributeSequenceError> {
        let maxi = offset.wrapping_add(len);
        let mut locator = [line, col];
        let mut index = offset;
        let mut current = index;

        while index < maxi {
            // STEP ONE：消费属性之间的空白；全部为空白时不产生事件。
            let whitespace_end = TextParsingUtil::find_next_non_whitespace_char_wildcard(
                buffer.as_deref(),
                index,
                maxi,
                Some(&mut locator),
            )?;
            if whitespace_end == -1 {
                index = maxi;
                continue;
            }
            if whitespace_end > current {
                index = whitespace_end;
                current = index;
            }

            // STEP TWO：扫描属性名，`=` 不能成为名称首字符。
            let mut artifact_line = locator[0];
            let mut artifact_col = locator[1];
            let attribute_name_end = TextParsingUtil::find_next_operator_char_wildcard(
                buffer.as_deref(),
                index,
                maxi,
                Some(&mut locator),
            )?;
            if attribute_name_end == -1 {
                dispatch_attribute(
                    &mut handler,
                    buffer.as_deref_mut(),
                    AttributeEvent {
                        name_offset: current,
                        name_len: maxi.wrapping_sub(current),
                        name_line: artifact_line,
                        name_col: artifact_col,
                        operator_offset: 0,
                        operator_len: 0,
                        operator_line: locator[0],
                        operator_col: locator[1],
                        value_content_offset: 0,
                        value_content_len: 0,
                        value_outer_offset: 0,
                        value_outer_len: 0,
                        value_line: locator[0],
                        value_col: locator[1],
                    },
                )?;
                index = maxi;
                continue;
            }
            if attribute_name_end <= current {
                let source = utf16_string_from_range(
                    buffer
                        .as_deref()
                        .expect("成功扫描属性名意味着 buffer 非 null"),
                    offset,
                    len,
                )?;
                let mut message = "Bad attribute name in sequence \""
                    .encode_utf16()
                    .collect::<Vec<_>>();
                message.extend_from_slice(&source);
                message
                    .extend("\": attribute names cannot start with an equals sign".encode_utf16());
                return Err(Box::new(TextParseException::with_message_at(
                    Some(&Utf16String::from_utf16(message)),
                    artifact_line,
                    artifact_col,
                ))
                .into());
            }

            let attribute_name_offset = current;
            let attribute_name_len = attribute_name_end.wrapping_sub(current);
            let attribute_name_line = artifact_line;
            let attribute_name_col = artifact_col;
            index = attribute_name_end;
            current = index;

            // STEP THREE：扫描由空白和等号构成的 operator。
            artifact_line = locator[0];
            artifact_col = locator[1];
            let operator_end = TextParsingUtil::find_next_non_operator_char_wildcard(
                buffer.as_deref(),
                index,
                maxi,
                Some(&mut locator),
            )?;
            if operator_end == -1 {
                let equals_present = range_contains_equals(
                    buffer
                        .as_deref()
                        .expect("成功扫描 operator 意味着 buffer 非 null"),
                    index,
                    maxi,
                );
                let event = if equals_present {
                    AttributeEvent {
                        name_offset: attribute_name_offset,
                        name_len: attribute_name_len,
                        name_line: attribute_name_line,
                        name_col: attribute_name_col,
                        operator_offset: current,
                        operator_len: maxi.wrapping_sub(current),
                        operator_line: artifact_line,
                        operator_col: artifact_col,
                        value_content_offset: 0,
                        value_content_len: 0,
                        value_outer_offset: 0,
                        value_outer_len: 0,
                        value_line: locator[0],
                        value_col: locator[1],
                    }
                } else {
                    AttributeEvent {
                        name_offset: attribute_name_offset,
                        name_len: attribute_name_len,
                        name_line: attribute_name_line,
                        name_col: attribute_name_col,
                        operator_offset: 0,
                        operator_len: 0,
                        operator_line: artifact_line,
                        operator_col: artifact_col,
                        value_content_offset: 0,
                        value_content_len: 0,
                        value_outer_offset: 0,
                        value_outer_len: 0,
                        value_line: artifact_line,
                        value_col: artifact_col,
                    }
                };
                dispatch_attribute(&mut handler, buffer.as_deref_mut(), event)?;
                index = maxi;
                continue;
            }

            if !range_contains_equals(
                buffer
                    .as_deref()
                    .expect("成功扫描 operator 意味着 buffer 非 null"),
                current,
                operator_end,
            ) {
                dispatch_attribute(
                    &mut handler,
                    buffer.as_deref_mut(),
                    AttributeEvent {
                        name_offset: attribute_name_offset,
                        name_len: attribute_name_len,
                        name_line: attribute_name_line,
                        name_col: attribute_name_col,
                        operator_offset: 0,
                        operator_len: 0,
                        operator_line: artifact_line,
                        operator_col: artifact_col,
                        value_content_offset: 0,
                        value_content_len: 0,
                        value_outer_offset: 0,
                        value_outer_len: 0,
                        value_line: artifact_line,
                        value_col: artifact_col,
                    },
                )?;
                index = operator_end;
                current = index;
                continue;
            }

            let operator_offset = current;
            let operator_len = operator_end.wrapping_sub(current);
            let operator_line = artifact_line;
            let operator_col = artifact_col;
            index = operator_end;
            current = index;

            // STEP FOUR：扫描值；只有两端同类引号都存在时才去掉 outer 引号。
            artifact_line = locator[0];
            artifact_col = locator[1];
            let current_unit = buffer
                .as_deref()
                .expect("成功扫描 value 起点意味着 buffer 非 null")[current as usize];
            let attribute_ends_with_quotes =
                index < maxi && matches!(current_unit, 0x0022 | 0x0027);
            let value_end = if attribute_ends_with_quotes {
                TextParsingUtil::find_next_any_char_avoid_quotes_wildcard(
                    buffer.as_deref(),
                    index,
                    maxi,
                    Some(&mut locator),
                )?
            } else {
                TextParsingUtil::find_next_whitespace_char_wildcard(
                    buffer.as_deref(),
                    index,
                    maxi,
                    false,
                    Some(&mut locator),
                )?
            };

            let value_outer_offset = current;
            let value_outer_len = if value_end == -1 {
                maxi.wrapping_sub(current)
            } else {
                value_end.wrapping_sub(current)
            };
            let (value_content_offset, value_content_len) = value_content_range(
                buffer
                    .as_deref()
                    .expect("成功扫描 value 意味着 buffer 非 null"),
                value_outer_offset,
                value_outer_len,
            );

            dispatch_attribute(
                &mut handler,
                buffer.as_deref_mut(),
                AttributeEvent {
                    name_offset: attribute_name_offset,
                    name_len: attribute_name_len,
                    name_line: attribute_name_line,
                    name_col: attribute_name_col,
                    operator_offset,
                    operator_len,
                    operator_line,
                    operator_col,
                    value_content_offset,
                    value_content_len,
                    value_outer_offset,
                    value_outer_len,
                    value_line: artifact_line,
                    value_col: artifact_col,
                },
            )?;

            if value_end == -1 {
                index = maxi;
            } else {
                index = value_end;
                current = index;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct AttributeEvent {
    name_offset: i32,
    name_len: i32,
    name_line: i32,
    name_col: i32,
    operator_offset: i32,
    operator_len: i32,
    operator_line: i32,
    operator_col: i32,
    value_content_offset: i32,
    value_content_len: i32,
    value_outer_offset: i32,
    value_outer_len: i32,
    value_line: i32,
    value_col: i32,
}

fn dispatch_attribute(
    handler: &mut Option<&mut dyn ITextHandler>,
    buffer: Option<&mut [u16]>,
    event: AttributeEvent,
) -> Result<(), TextParsingAttributeSequenceError> {
    let handler = handler
        .as_deref_mut()
        .ok_or(TextParsingAttributeSequenceError::NullHandler)?;
    let buffer = buffer.expect("successful attribute scans require a non-null buffer");
    handler
        .handle_attribute(
            Some(buffer),
            event.name_offset,
            event.name_len,
            event.name_line,
            event.name_col,
            event.operator_offset,
            event.operator_len,
            event.operator_line,
            event.operator_col,
            event.value_content_offset,
            event.value_content_len,
            event.value_outer_offset,
            event.value_outer_len,
            event.value_line,
            event.value_col,
        )
        .map_err(Into::into)
}

fn range_contains_equals(buffer: &[u16], start: i32, end: i32) -> bool {
    let mut index = start;
    while index < end {
        if buffer[index as usize] == u16::from(b'=') {
            return true;
        }
        index = index.wrapping_add(1);
    }
    false
}

fn value_content_range(buffer: &[u16], offset: i32, len: i32) -> (i32, i32) {
    if len < 2 {
        return (offset, len);
    }
    let first = buffer[offset as usize];
    let last = buffer[offset.wrapping_add(len).wrapping_sub(1) as usize];
    if (first == u16::from(b'"') && last == u16::from(b'"'))
        || (first == u16::from(b'\'') && last == u16::from(b'\''))
    {
        return (offset.wrapping_add(1), len.wrapping_sub(2));
    }
    (offset, len)
}

fn utf16_string_from_range(
    buffer: &[u16],
    offset: i32,
    len: i32,
) -> Result<Vec<u16>, TextParsingAttributeSequenceError> {
    let end = offset.wrapping_add(len);
    let valid = offset >= 0
        && len >= 0
        && end >= offset
        && usize::try_from(end).is_ok_and(|end| end <= buffer.len());
    if !valid {
        return Err(TextParsingAttributeSequenceError::StringRange {
            offset,
            len,
            length: buffer.len(),
        });
    }
    Ok(buffer[offset as usize..end as usize].to_vec())
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::{TextParsingAttributeSequenceError, TextParsingAttributeSequenceUtil};
    use crate::text::{ITextHandler, TextParseException};
    use crate::util::Utf16String;

    const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    const JAVA_GOLDEN: &str =
        include_str!("../../tests/fixtures/text_parsing_attribute_sequence_golden.txt");
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x100_0000_01b3;

    #[derive(Clone, Copy, Eq, PartialEq)]
    enum Mode {
        Normal,
        MutateFuture,
        CheckedFirst,
        CheckedSecond,
        RuntimeFirst,
    }

    struct RecordingHandler {
        calls: String,
        mode: Mode,
        call_count: usize,
    }

    impl RecordingHandler {
        fn new(mode: Mode) -> Self {
            Self {
                calls: String::new(),
                mode,
                call_count: 0,
            }
        }
    }

    impl ITextHandler for RecordingHandler {
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
            _: Option<&mut [u16]>,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
            _: i32,
        ) -> Result<(), Box<TextParseException>> {
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
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            name_line: i32,
            name_col: i32,
            operator_offset: i32,
            operator_len: i32,
            operator_line: i32,
            operator_col: i32,
            value_content_offset: i32,
            value_content_len: i32,
            value_outer_offset: i32,
            value_outer_len: i32,
            value_line: i32,
            value_col: i32,
        ) -> Result<(), Box<TextParseException>> {
            let buffer = buffer.expect("attribute callback receives the parser buffer");
            self.call_count += 1;
            if !self.calls.is_empty() {
                self.calls.push('|');
            }
            write!(
                self.calls,
                "A:{name_offset}:{name_len}:{name_line}:{name_col}:\
                 {operator_offset}:{operator_len}:{operator_line}:{operator_col}:\
                 {value_content_offset}:{value_content_len}:{value_outer_offset}:\
                 {value_outer_len}:{value_line}:{value_col}"
            )
            .expect("write to String");

            if self.mode == Mode::MutateFuture && self.call_count == 1 && buffer.len() > 4 {
                buffer[4] = u16::from(b'=');
            }
            if (self.mode == Mode::CheckedFirst && self.call_count == 1)
                || (self.mode == Mode::CheckedSecond && self.call_count == 2)
            {
                return Err(Box::new(TextParseException::with_message_at(
                    Some(&Utf16String::from_rust_str("handler")),
                    41,
                    43,
                )));
            }
            assert!(
                self.mode != Mode::RuntimeFirst || self.call_count != 1,
                "runtime"
            );
            Ok(())
        }
    }

    #[test]
    fn attribute_sequence_matches_java_golden() {
        let mut output = String::new();
        emit(&mut output, "baseline", JAVA_BASELINE);
        fixed_cases(&mut output);
        handler_cases(&mut output);
        runtime_cases(&mut output);
        exhaustive_cases(&mut output);
        assert_eq!(output, JAVA_GOLDEN);
    }

    #[test]
    fn error_adapter_preserves_sources_and_all_messages() {
        let text_parse = TextParsingAttributeSequenceError::TextParse(Box::default());
        assert_eq!(
            text_parse.class_name(),
            "org.thymeleaf.templateparser.text.TextParseException"
        );
        assert_eq!(text_parse.message().to_string_lossy(), "null");
        assert!(std::error::Error::source(&text_parse).is_some());

        let scan = TextParsingAttributeSequenceError::Scanning(
            super::TextParsingUtilError::NullDirectLocator,
        );
        assert_eq!(scan.class_name(), "java.lang.NullPointerException");
        assert_eq!(
            scan.to_string(),
            "Cannot load from int array because \"<parameter4>\" is null"
        );
        assert!(std::error::Error::source(&scan).is_some());

        let range = TextParsingAttributeSequenceError::StringRange {
            offset: 0,
            len: 4,
            length: 3,
        };
        assert_eq!(
            range.class_name(),
            "java.lang.StringIndexOutOfBoundsException"
        );
        assert_eq!(
            range.to_string(),
            "Range [0, 0 + 4) out of bounds for length 3"
        );
        assert!(std::error::Error::source(&range).is_none());

        let null_handler = TextParsingAttributeSequenceError::NullHandler;
        assert_eq!(null_handler.class_name(), "java.lang.NullPointerException");
        assert_eq!(null_handler.to_string(), super::NULL_HANDLER_MESSAGE);

        let mut handler = RecordingHandler::new(Mode::Normal);
        let mut buffer = [u16::from(b'x')];
        handler.handle_document_start(0, 0, 0).unwrap();
        handler.handle_document_end(0, 0, 0, 0).unwrap();
        handler.handle_text(Some(&mut buffer), 0, 0, 0, 0).unwrap();
        handler
            .handle_comment(Some(&mut buffer), 0, 0, 0, 0, 0, 0)
            .unwrap();
        handler
            .handle_standalone_element_start(Some(&mut buffer), 0, 0, false, 0, 0)
            .unwrap();
        handler
            .handle_standalone_element_end(Some(&mut buffer), 0, 0, false, 0, 0)
            .unwrap();
        handler
            .handle_open_element_start(Some(&mut buffer), 0, 0, 0, 0)
            .unwrap();
        handler
            .handle_open_element_end(Some(&mut buffer), 0, 0, 0, 0)
            .unwrap();
        handler
            .handle_close_element_start(Some(&mut buffer), 0, 0, 0, 0)
            .unwrap();
        handler
            .handle_close_element_end(Some(&mut buffer), 0, 0, 0, 0)
            .unwrap();
    }

    fn fixed_cases(output: &mut String) {
        for (key, text, offset, len, line, col) in [
            ("fixed.empty", "", 0, 0, 1, 1),
            ("fixed.whitespace", " \t\n", 0, 3, 2, 3),
            ("fixed.nameOnly", "disabled", 0, 8, 1, 1),
            ("fixed.twoNames", "a b", 0, 3, 4, 5),
            ("fixed.equalsOnly", "a=", 0, 2, 1, 1),
            ("fixed.equalsTrailingSpace", "a = \t", 0, 5, 1, 1),
            ("fixed.unquoted", "a=b", 0, 3, 1, 1),
            ("fixed.operatorSpaces", "a \t= \tb", 0, 7, 3, 7),
            ("fixed.doubleQuoted", "a=\"x y\"", 0, 7, 1, 1),
            ("fixed.singleQuoted", "a='x y'", 0, 7, 1, 1),
            ("fixed.emptyDoubleQuoted", "a=\"\"", 0, 4, 1, 1),
            ("fixed.emptySingleQuoted", "a=''", 0, 4, 1, 1),
            ("fixed.unclosedDouble", "a=\"x y", 0, 6, 1, 1),
            ("fixed.unclosedSingle", "a='x y", 0, 6, 1, 1),
            ("fixed.adjacentAfterQuote", "a=\"x\"b=y", 0, 8, 1, 1),
            ("fixed.multiple", "a=b c='d e' f = \"\" g", 0, 20, 5, 9),
            ("fixed.multipleEquals", "a==b", 0, 4, 1, 1),
            ("fixed.noEqualsThenValue", "a b=c", 0, 5, 1, 1),
            ("fixed.leadingEquals", "=a", 0, 2, 7, 11),
            ("fixed.onlyEquals", "=", 0, 1, -3, -5),
            ("fixed.embeddedRange", "xxa=\"v w\"yy", 2, 7, 8, 13),
            ("fixed.newlineBeforeName", "\n a=b", 0, 5, 10, 20),
            ("fixed.newlineOperator", "a\n=\nb", 0, 5, 10, 20),
            ("fixed.newlineQuotedValue", "a=\"x\ny\" z=q", 0, 11, 10, 20),
        ] {
            emit_case(
                output,
                key,
                Some(text),
                offset,
                len,
                line,
                col,
                Mode::Normal,
            );
        }

        let mut surrogate_buffer = vec![
            u16::from(b'a'),
            u16::from(b'='),
            0,
            u16::from(b' '),
            u16::from(b'b'),
            u16::from(b'='),
            0xD800,
        ];
        let surrogate_result = outcome(Some(&mut surrogate_buffer), 0, 7, 1, 1, Mode::Normal);
        emit(
            output,
            "fixed.nulAndSurrogate",
            surrogate_result.to_string_lossy(),
        );
        emit_case(
            output,
            "fixed.lineOverflow",
            Some("\na=b"),
            0,
            4,
            i32::MAX,
            i32::MAX,
            Mode::Normal,
        );
        emit_case(
            output,
            "fixed.columnOverflow",
            Some("ab=c"),
            0,
            4,
            1,
            i32::MAX,
            Mode::Normal,
        );
    }

    fn handler_cases(output: &mut String) {
        for (key, mode) in [
            ("handler.mutateFuture", Mode::MutateFuture),
            ("handler.checkedFirst", Mode::CheckedFirst),
            ("handler.checkedSecond", Mode::CheckedSecond),
        ] {
            emit_case(output, key, Some("a=x b=y"), 0, 7, 1, 1, mode);
        }
        for (key, text) in [
            ("handler.checkedNameOnly", "a"),
            ("handler.checkedNoOperator", "a b"),
            ("handler.checkedNoValue", "a="),
        ] {
            emit_case(
                output,
                key,
                Some(text),
                0,
                text.encode_utf16().count() as i32,
                1,
                1,
                Mode::CheckedFirst,
            );
        }
        emit_case(
            output,
            "handler.runtimeFirst",
            Some("a=x b=y"),
            0,
            7,
            1,
            1,
            Mode::RuntimeFirst,
        );

        emit_handler_null(output, "handler.nullEmpty", None, 0, 0);
        emit_handler_null(output, "handler.nullWhitespace", Some(" "), 0, 1);
        emit_handler_null(output, "handler.nullAttribute", Some("a"), 0, 1);
    }

    fn runtime_cases(output: &mut String) {
        for (key, text, offset, len) in [
            ("runtime.nullEmpty", None, 0, 0),
            ("runtime.nullNegativeLen", None, 0, -1),
            ("runtime.nullOne", None, 0, 1),
            ("runtime.negativeOffset", Some("a"), -1, 1),
            ("runtime.offsetAtEndEmpty", Some("a"), 1, 0),
            ("runtime.offsetPastEndEmpty", Some("a"), 2, 0),
            ("runtime.offsetPastEnd", Some("a"), 1, 1),
            ("runtime.negativeLen", Some("a"), 0, -1),
            ("runtime.overflowRange", Some("a"), 1, i32::MAX),
            ("runtime.badNameStringRange", Some("="), 0, 2),
            ("runtime.badNameNegativeStringRange", Some("="), 0, i32::MAX),
            ("runtime.operatorPastEnd", Some("a="), 0, 3),
            ("runtime.quotedValuePastEnd", Some("a=\""), 0, 4),
            ("runtime.unquotedValuePastEnd", Some("a=b"), 0, 4),
        ] {
            emit_case(output, key, text, offset, len, 1, 1, Mode::Normal);
        }
    }

    fn exhaustive_cases(output: &mut String) {
        let mut whitespace_hash = FNV_OFFSET;
        for unit in u16::MIN..=u16::MAX {
            let mut buffer = vec![
                u16::from(b'a'),
                unit,
                u16::from(b'b'),
                u16::from(b'='),
                u16::from(b'c'),
            ];
            whitespace_hash = mix_utf16_string(
                whitespace_hash,
                &outcome(Some(&mut buffer), 0, 5, 3, 5, Mode::Normal),
            );
        }
        emit(
            output,
            "exhaustive.whitespaceHash",
            format!("{whitespace_hash:016x}"),
        );

        let mut quoted_hash = FNV_OFFSET;
        for unit in u16::MIN..=u16::MAX {
            let mut buffer = vec![
                u16::from(b'a'),
                u16::from(b'='),
                u16::from(b'"'),
                unit,
                u16::from(b'"'),
                u16::from(b' '),
                u16::from(b'b'),
                u16::from(b'='),
                u16::from(b'z'),
            ];
            quoted_hash = mix_utf16_string(
                quoted_hash,
                &outcome(Some(&mut buffer), 0, 9, 7, 11, Mode::Normal),
            );
        }
        emit(
            output,
            "exhaustive.quotedHash",
            format!("{quoted_hash:016x}"),
        );

        let mut grammar_hash = FNV_OFFSET;
        let names = ["a", "x:y", "data-x", "", "="];
        let operators = ["", "=", " = ", "==", " \t"];
        let values = ["", "v", "\"x y\"", "'x y'", "\"\"", "\"x", "/"];
        let separators = ["", " ", "\n"];
        for first_name in names {
            for first_operator in operators {
                for first_value in values {
                    for separator in separators {
                        for second_name in names {
                            let text = format!(
                                "{first_name}{first_operator}{first_value}{separator}{second_name}=z"
                            );
                            let mut buffer = text.encode_utf16().collect::<Vec<_>>();
                            let len = buffer.len() as i32;
                            grammar_hash = mix_utf16_string(
                                grammar_hash,
                                &outcome(Some(&mut buffer), 0, len, -7, i32::MAX, Mode::Normal),
                            );
                        }
                    }
                }
            }
        }
        emit(
            output,
            "exhaustive.grammarHash",
            format!("{grammar_hash:016x}"),
        );

        let range_source = "xxa = \"v w\" yy".encode_utf16().collect::<Vec<_>>();
        let mut range_hash = FNV_OFFSET;
        for offset in -2..=range_source.len() as i32 + 2 {
            for len in -2..=range_source.len() as i32 + 4 {
                let mut buffer = range_source.clone();
                range_hash = mix_utf16_string(
                    range_hash,
                    &outcome(Some(&mut buffer), offset, len, 13, 17, Mode::Normal),
                );
            }
        }
        emit(output, "exhaustive.rangeHash", format!("{range_hash:016x}"));
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_case(
        output: &mut String,
        key: &str,
        text: Option<&str>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
        mode: Mode,
    ) {
        let mut buffer = text.map(|text| text.encode_utf16().collect::<Vec<_>>());
        let result = outcome(buffer.as_deref_mut(), offset, len, line, col, mode);
        emit(output, key, result.to_string_lossy());
    }

    fn emit_handler_null(
        output: &mut String,
        key: &str,
        text: Option<&str>,
        offset: i32,
        len: i32,
    ) {
        let mut buffer = text.map(|text| text.encode_utf16().collect::<Vec<_>>());
        let result = TextParsingAttributeSequenceUtil::parse_attribute_sequence(
            buffer.as_deref_mut(),
            offset,
            len,
            1,
            1,
            None,
        );
        let value = match result {
            Ok(()) => "OK".to_owned(),
            Err(error) => describe_error(&error),
        };
        emit(output, key, value);
    }

    #[allow(clippy::too_many_arguments)]
    fn outcome(
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
        mode: Mode,
    ) -> Utf16String {
        let mut buffer = buffer;
        let mut handler = RecordingHandler::new(mode);
        let result = catch_unwind(AssertUnwindSafe(|| {
            TextParsingAttributeSequenceUtil::parse_attribute_sequence(
                buffer.as_deref_mut(),
                offset,
                len,
                line,
                col,
                Some(&mut handler),
            )
        }));
        let prefix = match result {
            Ok(Ok(())) => format!("OK:{}", handler.calls),
            Ok(Err(error)) => format!("{}:{}", describe_error(&error), handler.calls),
            Err(_) => format!(
                "ERR:java.lang.IllegalStateException:{}:{}",
                to_utf16_hex(&Utf16String::from_rust_str("runtime")),
                handler.calls
            ),
        };
        Utf16String::from_rust_str(&format!("{prefix}:{}", describe_buffer(buffer.as_deref())))
    }

    fn describe_error(error: &TextParsingAttributeSequenceError) -> String {
        format!(
            "ERR:{}:{}",
            error.class_name(),
            to_utf16_hex(&error.message())
        )
    }

    fn describe_buffer(buffer: Option<&[u16]>) -> String {
        buffer.map_or_else(
            || "null".to_owned(),
            |buffer| to_utf16_hex(&Utf16String::from_utf16(buffer.to_vec())),
        )
    }

    fn mix_utf16_string(mut hash: u64, value: &Utf16String) -> u64 {
        for unit in value.as_utf16() {
            hash = mix(hash, i32::from((*unit & 0x00ff) as u8));
            hash = mix(hash, i32::from((*unit >> 8) as u8));
        }
        hash
    }

    fn mix(hash: u64, value: i32) -> u64 {
        (hash ^ value as i64 as u64).wrapping_mul(FNV_PRIME)
    }

    fn to_utf16_hex(value: &Utf16String) -> String {
        value
            .as_utf16()
            .iter()
            .map(|unit| format!("{unit:04x}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
        writeln!(output, "{key}={value}").expect("write to String");
    }
}
