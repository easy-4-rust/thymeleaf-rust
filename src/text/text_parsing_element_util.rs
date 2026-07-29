use std::error::Error;
use std::fmt::{Display, Formatter};

use super::{
    ITextHandler, TextParseException, TextParsingAttributeSequenceError,
    TextParsingAttributeSequenceUtil, TextParsingUtilError,
};
use crate::util::JavaString;

const NULL_ARRAY_LOAD_MESSAGE: &str =
    "Cannot load from char array because \"<parameter1>\" is null";
const NULL_STRING_VALUE_MESSAGE: &str = "Cannot read the array length because \"value\" is null";
const NULL_STANDALONE_START_MESSAGE: &str = "Cannot invoke \"org.thymeleaf.templateparser.text.ITextHandler.handleStandaloneElementStart(char[], int, int, boolean, int, int)\" because \"<parameter6>\" is null";
const NULL_STANDALONE_END_MESSAGE: &str = "Cannot invoke \"org.thymeleaf.templateparser.text.ITextHandler.handleStandaloneElementEnd(char[], int, int, boolean, int, int)\" because \"<parameter6>\" is null";
const NULL_OPEN_START_MESSAGE: &str = "Cannot invoke \"org.thymeleaf.templateparser.text.ITextHandler.handleOpenElementStart(char[], int, int, int, int)\" because \"<parameter6>\" is null";
const NULL_OPEN_END_MESSAGE: &str = "Cannot invoke \"org.thymeleaf.templateparser.text.ITextHandler.handleOpenElementEnd(char[], int, int, int, int)\" because \"<parameter6>\" is null";
const NULL_CLOSE_START_MESSAGE: &str = "Cannot invoke \"org.thymeleaf.templateparser.text.ITextHandler.handleCloseElementStart(char[], int, int, int, int)\" because \"<parameter6>\" is null";
const NULL_CLOSE_END_MESSAGE: &str = "Cannot invoke \"org.thymeleaf.templateparser.text.ITextHandler.handleCloseElementEnd(char[], int, int, int, int)\" because \"<parameter6>\" is null";

/// 文本元素解析中的 checked exception 与 Java 运行时异常适配。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.TextParsingElementUtil` 的全部可观察失败路径。
///
/// 上游在格式检查、错误消息构造、通用扫描、属性扫描和 handler 回调之间具有严格
/// 的短路顺序。本类型保留异常类别、UTF-16 消息、原因对象和失败前事件副作用。
#[derive(Debug)]
pub(crate) enum TextParsingElementError {
    /// 元素格式校验或 handler 回调产生的 `TextParseException`。
    TextParse(Box<TextParseException>),
    /// 通用文本扫描器产生的 Java 运行时异常。
    Scanning(TextParsingUtilError),
    /// 属性扫描器产生且不能折叠为前两类的 Java 运行时异常。
    Attribute(TextParsingAttributeSequenceError),
    /// 元素谓词直接读取 null `char[]`。
    NullArrayLoad,
    /// Java `String(char[],offset,len)` 收到 null 数组。
    NullStringValue,
    /// Java `char[]` 下标越界。
    ArrayIndex {
        /// 实际访问下标。
        index: i32,
        /// 数组长度。
        length: usize,
    },
    /// Java `String(char[],offset,len)` 范围非法。
    StringRange {
        /// 起始下标。
        offset: i32,
        /// 请求长度。
        len: i32,
        /// 数组长度。
        length: usize,
    },
    /// standalone start 回调的 handler 为 null。
    NullStandaloneStartHandler,
    /// standalone end 回调的 handler 为 null。
    NullStandaloneEndHandler,
    /// open start 回调的 handler 为 null。
    NullOpenStartHandler,
    /// open end 回调的 handler 为 null。
    NullOpenEndHandler,
    /// close start 回调的 handler 为 null。
    NullCloseStartHandler,
    /// close end 回调的 handler 为 null。
    NullCloseEndHandler,
}

impl TextParsingElementError {
    /// 返回对应 Java 异常全限定名。
    pub(crate) const fn java_class_name(&self) -> &'static str {
        match self {
            Self::TextParse(_) => "org.thymeleaf.templateparser.text.TextParseException",
            Self::Scanning(error) => error.java_class_name(),
            Self::Attribute(error) => error.java_class_name(),
            Self::NullArrayLoad
            | Self::NullStringValue
            | Self::NullStandaloneStartHandler
            | Self::NullStandaloneEndHandler
            | Self::NullOpenStartHandler
            | Self::NullOpenEndHandler
            | Self::NullCloseStartHandler
            | Self::NullCloseEndHandler => "java.lang.NullPointerException",
            Self::ArrayIndex { .. } => "java.lang.ArrayIndexOutOfBoundsException",
            Self::StringRange { .. } => "java.lang.StringIndexOutOfBoundsException",
        }
    }

    /// 返回 Java `String.valueOf(Throwable#getMessage())` 的 UTF-16 表示。
    pub(crate) fn java_message(&self) -> JavaString {
        match self {
            Self::TextParse(exception) => exception
                .get_message()
                .cloned()
                .unwrap_or_else(|| JavaString::from_rust_str("null")),
            Self::Scanning(error) => error
                .java_message()
                .unwrap_or_else(|| JavaString::from_rust_str("null")),
            Self::Attribute(error) => error.java_message(),
            Self::NullArrayLoad => JavaString::from_rust_str(NULL_ARRAY_LOAD_MESSAGE),
            Self::NullStringValue => JavaString::from_rust_str(NULL_STRING_VALUE_MESSAGE),
            Self::ArrayIndex { index, length } => JavaString::from_rust_str(&format!(
                "Index {index} out of bounds for length {length}"
            )),
            Self::StringRange {
                offset,
                len,
                length,
            } => JavaString::from_rust_str(&format!(
                "Range [{offset}, {offset} + {len}) out of bounds for length {length}"
            )),
            Self::NullStandaloneStartHandler => {
                JavaString::from_rust_str(NULL_STANDALONE_START_MESSAGE)
            }
            Self::NullStandaloneEndHandler => {
                JavaString::from_rust_str(NULL_STANDALONE_END_MESSAGE)
            }
            Self::NullOpenStartHandler => JavaString::from_rust_str(NULL_OPEN_START_MESSAGE),
            Self::NullOpenEndHandler => JavaString::from_rust_str(NULL_OPEN_END_MESSAGE),
            Self::NullCloseStartHandler => JavaString::from_rust_str(NULL_CLOSE_START_MESSAGE),
            Self::NullCloseEndHandler => JavaString::from_rust_str(NULL_CLOSE_END_MESSAGE),
        }
    }

    /// 返回 Java `TextParseException` 的行列；其他异常返回空。
    pub(crate) const fn text_parse_location(&self) -> Option<(i32, i32)> {
        match self {
            Self::TextParse(exception) => match (exception.get_line(), exception.get_col()) {
                (Some(line), Some(col)) => Some((line, col)),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Display for TextParsingElementError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.java_message().to_string_lossy())
    }
}

impl Error for TextParsingElementError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TextParse(exception) => Some(exception),
            Self::Scanning(error) => Some(error),
            Self::Attribute(error) => Some(error),
            _ => None,
        }
    }
}

impl From<Box<TextParseException>> for TextParsingElementError {
    fn from(value: Box<TextParseException>) -> Self {
        Self::TextParse(value)
    }
}

impl From<TextParsingUtilError> for TextParsingElementError {
    fn from(value: TextParsingUtilError) -> Self {
        Self::Scanning(value)
    }
}

impl From<TextParsingAttributeSequenceError> for TextParsingElementError {
    fn from(value: TextParsingAttributeSequenceError) -> Self {
        match value {
            TextParsingAttributeSequenceError::TextParse(exception) => Self::TextParse(exception),
            TextParsingAttributeSequenceError::Scanning(error) => Self::Scanning(error),
            other => Self::Attribute(other),
        }
    }
}

/// 文本模式元素（标签）解析工具。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.TextParsingElementUtil`。
///
/// 本无状态对象识别并解析 `[#name]`、`[#name/]` 和 `[/name]`，同步派发元素与
/// 属性事件。它保留无名元素、Java BMP 空白、引号内 `]`、关闭标签禁止属性、
/// UTF-16 代码单元、handler 修改可见性、行列回绕以及异常短路顺序。
pub(crate) struct TextParsingElementUtil;

impl TextParsingElementUtil {
    /// 解析一个 `[#name .../]` standalone 元素。
    ///
    /// 对应 Java:
    /// `TextParsingElementUtil#parseStandaloneElement(char[],int,int,int,int,ITextHandler)`。
    ///
    /// # 参数
    /// - `buffer`：Java 参数 `buffer`，允许 null 以复现运行时失败。
    /// - `offset` / `len`：完整元素范围，计算按 Java `int` 回绕。
    /// - `line` / `col`：元素起始位置。
    /// - `handler`：同步事件处理器；Java null 只在实际回调时失败。
    ///
    /// # 错误
    /// 返回格式、扫描、属性、handler checked exception 或对应 Java 运行时异常。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn parse_standalone_element(
        mut buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
        mut handler: Option<&mut dyn ITextHandler>,
    ) -> Result<(), TextParsingElementError> {
        let maxi = offset.wrapping_add(len);
        let valid = len >= 4
            && Self::is_open_element_start(buffer.as_deref(), offset, maxi)?
            && Self::is_element_end(buffer.as_deref(), maxi.wrapping_sub(2), maxi, true)?;
        if !valid {
            return Err(malformed_element(
                "Could not parse as a well-formed standalone element: \"",
                buffer.as_deref(),
                offset,
                len,
                line,
                col,
                "\"",
            )?);
        }

        let content_offset = offset.wrapping_add(2);
        let content_len = len.wrapping_sub(4);
        let content_maxi = content_offset.wrapping_add(content_len);
        let mut locator = [line, col.wrapping_add(2)];
        let element_name_end = find_next_whitespace_char_validated(
            buffer
                .as_deref()
                .expect("validated standalone element has non-null buffer"),
            content_offset,
            content_maxi,
            true,
            &mut locator,
        );

        if element_name_end == -1 {
            dispatch_standalone_start(
                &mut handler,
                buffer.as_deref_mut(),
                content_offset,
                content_len,
                line,
                col,
            )?;
            dispatch_standalone_end(
                &mut handler,
                buffer.as_deref_mut(),
                content_offset,
                content_len,
                locator[0],
                locator[1],
            )?;
            return Ok(());
        }

        let name_len = element_name_end.wrapping_sub(content_offset);
        dispatch_standalone_start(
            &mut handler,
            buffer.as_deref_mut(),
            content_offset,
            name_len,
            line,
            col,
        )?;
        let attribute_handler = handler
            .as_mut()
            .expect("successful standalone start requires a non-null handler");
        TextParsingAttributeSequenceUtil::parse_attribute_sequence(
            buffer.as_deref_mut(),
            element_name_end,
            content_maxi.wrapping_sub(element_name_end),
            locator[0],
            locator[1],
            Some(&mut **attribute_handler),
        )?;
        let _ = find_next_structure_end_avoid_quotes_validated(
            buffer
                .as_deref()
                .expect("validated standalone element has non-null buffer"),
            element_name_end,
            content_maxi,
            &mut locator,
        );
        dispatch_standalone_end(
            &mut handler,
            buffer,
            content_offset,
            name_len,
            locator[0],
            locator[1],
        )
    }

    /// 解析一个 `[#name ...]` open 元素。
    ///
    /// 对应 Java:
    /// `TextParsingElementUtil#parseOpenElement(char[],int,int,int,int,ITextHandler)`。
    ///
    /// 参数、事件时序、异常和副作用与 [`Self::parse_standalone_element`] 相同；
    /// 元素结束符仅为 `]`，回调的 minimized 参数不存在。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn parse_open_element(
        mut buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
        mut handler: Option<&mut dyn ITextHandler>,
    ) -> Result<(), TextParsingElementError> {
        let maxi = offset.wrapping_add(len);
        let valid = len >= 3
            && Self::is_open_element_start(buffer.as_deref(), offset, maxi)?
            && Self::is_element_end(buffer.as_deref(), maxi.wrapping_sub(1), maxi, false)?;
        if !valid {
            return Err(malformed_element(
                "Could not parse as a well-formed open element: \"",
                buffer.as_deref(),
                offset,
                len,
                line,
                col,
                "\"",
            )?);
        }

        let content_offset = offset.wrapping_add(2);
        let content_len = len.wrapping_sub(3);
        let content_maxi = content_offset.wrapping_add(content_len);
        let mut locator = [line, col.wrapping_add(2)];
        let element_name_end = find_next_whitespace_char_validated(
            buffer
                .as_deref()
                .expect("validated open element has non-null buffer"),
            content_offset,
            content_maxi,
            true,
            &mut locator,
        );

        if element_name_end == -1 {
            dispatch_open_start(
                &mut handler,
                buffer.as_deref_mut(),
                content_offset,
                content_len,
                line,
                col,
            )?;
            dispatch_open_end(
                &mut handler,
                buffer.as_deref_mut(),
                content_offset,
                content_len,
                locator[0],
                locator[1],
            )?;
            return Ok(());
        }

        let name_len = element_name_end.wrapping_sub(content_offset);
        dispatch_open_start(
            &mut handler,
            buffer.as_deref_mut(),
            content_offset,
            name_len,
            line,
            col,
        )?;
        let attribute_handler = handler
            .as_mut()
            .expect("successful open start requires a non-null handler");
        TextParsingAttributeSequenceUtil::parse_attribute_sequence(
            buffer.as_deref_mut(),
            element_name_end,
            content_maxi.wrapping_sub(element_name_end),
            locator[0],
            locator[1],
            Some(&mut **attribute_handler),
        )?;
        let _ = find_next_structure_end_avoid_quotes_validated(
            buffer
                .as_deref()
                .expect("validated open element has non-null buffer"),
            element_name_end,
            content_maxi,
            &mut locator,
        );
        dispatch_open_end(
            &mut handler,
            buffer,
            content_offset,
            name_len,
            locator[0],
            locator[1],
        )
    }

    /// 解析一个 `[/name]` close 元素。
    ///
    /// 对应 Java:
    /// `TextParsingElementUtil#parseCloseElement(char[],int,int,int,int,ITextHandler)`。
    ///
    /// close 元素名称之后只允许 Java 空白。若出现其他内容，start 事件已经派发，
    /// 随后才产生带原始（可能已被 handler 修改）UTF-16 元素文本的异常。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn parse_close_element(
        mut buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
        mut handler: Option<&mut dyn ITextHandler>,
    ) -> Result<(), TextParsingElementError> {
        let maxi = offset.wrapping_add(len);
        let valid = len >= 3
            && Self::is_close_element_start(buffer.as_deref(), offset, maxi)?
            && Self::is_element_end(buffer.as_deref(), maxi.wrapping_sub(1), maxi, false)?;
        if !valid {
            return Err(malformed_element(
                "Could not parse as a well-formed close element: \"",
                buffer.as_deref(),
                offset,
                len,
                line,
                col,
                "\"",
            )?);
        }

        let content_offset = offset.wrapping_add(2);
        let content_len = len.wrapping_sub(3);
        let content_maxi = content_offset.wrapping_add(content_len);
        let mut locator = [line, col.wrapping_add(2)];
        let element_name_end = find_next_whitespace_char_validated(
            buffer
                .as_deref()
                .expect("validated close element has non-null buffer"),
            content_offset,
            content_maxi,
            true,
            &mut locator,
        );

        if element_name_end == -1 {
            dispatch_close_start(
                &mut handler,
                buffer.as_deref_mut(),
                content_offset,
                content_len,
                line,
                col,
            )?;
            dispatch_close_end(
                &mut handler,
                buffer.as_deref_mut(),
                content_offset,
                content_len,
                locator[0],
                locator[1],
            )?;
            return Ok(());
        }

        let name_len = element_name_end.wrapping_sub(content_offset);
        dispatch_close_start(
            &mut handler,
            buffer.as_deref_mut(),
            content_offset,
            name_len,
            line,
            col,
        )?;
        let whitespace_end = find_next_non_whitespace_char_validated(
            buffer
                .as_deref()
                .expect("validated close element has non-null buffer"),
            element_name_end,
            content_maxi,
            &mut locator,
        );
        if whitespace_end != -1 {
            return Err(malformed_element_validated(
                "Could not parse as a well-formed closing element \"",
                buffer
                    .as_deref()
                    .expect("validated close element has non-null buffer"),
                offset,
                len,
                line,
                col,
                "\": No attributes are allowed here",
            ));
        }

        dispatch_close_end(
            &mut handler,
            buffer,
            content_offset,
            name_len,
            locator[0],
            locator[1],
        )
    }

    /// 判断当前位置是否为合法 open/standalone 元素起点。
    ///
    /// 对应 Java: `TextParsingElementUtil#isOpenElementStart(char[],int,int)`。
    ///
    /// 只有 `[#` 后接允许的名称首代码单元、空白或合法无名结束符时返回 true。
    pub(crate) fn is_open_element_start(
        buffer: Option<&[u16]>,
        offset: i32,
        maxi: i32,
    ) -> Result<bool, TextParsingElementError> {
        let len = maxi.wrapping_sub(offset);
        if len <= 2 {
            return Ok(false);
        }
        if array_unit(buffer, offset)? != u16::from(b'[')
            || array_unit(buffer, offset.wrapping_add(1))? != u16::from(b'#')
        {
            return Ok(false);
        }
        Self::is_element_name_or_end(buffer, offset.wrapping_add(2), maxi)
    }

    /// 判断当前位置是否为合法 close 元素起点。
    ///
    /// 对应 Java: `TextParsingElementUtil#isCloseElementStart(char[],int,int)`。
    pub(crate) fn is_close_element_start(
        buffer: Option<&[u16]>,
        offset: i32,
        maxi: i32,
    ) -> Result<bool, TextParsingElementError> {
        let len = maxi.wrapping_sub(offset);
        if len <= 2 {
            return Ok(false);
        }
        if array_unit(buffer, offset)? != u16::from(b'[')
            || array_unit(buffer, offset.wrapping_add(1))? != u16::from(b'/')
        {
            return Ok(false);
        }
        Self::is_element_name_or_end(buffer, offset.wrapping_add(2), maxi)
    }

    /// 判断当前位置是否为普通 `]` 或 minimized `/]` 结束符。
    ///
    /// 对应 Java:
    /// `TextParsingElementUtil#isElementEnd(char[],int,int,boolean)`。
    ///
    /// `minimized` 为 true 时要求范围至少两个代码单元且精确匹配 `/]`。
    pub(crate) fn is_element_end(
        buffer: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        minimized: bool,
    ) -> Result<bool, TextParsingElementError> {
        let len = maxi.wrapping_sub(offset);
        if len < 1 {
            return Ok(false);
        }
        if minimized {
            if len < 2 || array_unit(buffer, offset)? != u16::from(b'/') {
                return Ok(false);
            }
            return Ok(array_unit(buffer, offset.wrapping_add(1))? == u16::from(b']'));
        }
        Ok(array_unit(buffer, offset)? == u16::from(b']'))
    }

    fn is_element_name_or_end(
        buffer: Option<&[u16]>,
        offset: i32,
        maxi: i32,
    ) -> Result<bool, TextParsingElementError> {
        let unit = array_unit(buffer, offset)?;
        if is_java_whitespace(unit) {
            return Ok(true);
        }

        let len = maxi.wrapping_sub(offset);
        if len > 1 && unit == u16::from(b'/') {
            return Self::is_element_end(buffer, offset, maxi, true);
        }
        if len > 0 && unit == u16::from(b']') {
            return Self::is_element_end(buffer, offset, maxi, false);
        }
        Ok(len > 0
            && !matches!(unit, 0x002D | 0x0021 | 0x002F | 0x003F | 0x005B | 0x007B)
            && !is_java_whitespace(unit))
    }
}

fn malformed_element(
    prefix: &str,
    buffer: Option<&[u16]>,
    offset: i32,
    len: i32,
    line: i32,
    col: i32,
    suffix: &str,
) -> Result<TextParsingElementError, TextParsingElementError> {
    let source = java_string_from_range(buffer, offset, len)?;
    let mut message = prefix.encode_utf16().collect::<Vec<_>>();
    message.extend_from_slice(&source);
    message.extend(suffix.encode_utf16());
    Ok(TextParsingElementError::TextParse(Box::new(
        TextParseException::with_message_at(Some(&JavaString::from_utf16(message)), line, col),
    )))
}

fn malformed_element_validated(
    prefix: &str,
    buffer: &[u16],
    offset: i32,
    len: i32,
    line: i32,
    col: i32,
    suffix: &str,
) -> TextParsingElementError {
    let end = offset.wrapping_add(len);
    let source = &buffer[offset as usize..end as usize];
    let mut message = prefix.encode_utf16().collect::<Vec<_>>();
    message.extend_from_slice(source);
    message.extend(suffix.encode_utf16());
    TextParsingElementError::TextParse(Box::new(TextParseException::with_message_at(
        Some(&JavaString::from_utf16(message)),
        line,
        col,
    )))
}

fn find_next_whitespace_char_validated(
    buffer: &[u16],
    offset: i32,
    maxi: i32,
    avoid_quotes: bool,
    locator: &mut [i32; 2],
) -> i32 {
    let mut in_quotes = false;
    let mut in_apos = false;
    let mut index = offset;
    let mut remaining = maxi.wrapping_sub(offset);
    while remaining != 0 {
        remaining = remaining.wrapping_sub(1);
        let character = buffer[index as usize];
        if avoid_quotes && !in_apos && character == u16::from(b'"') {
            in_quotes = !in_quotes;
        } else if avoid_quotes && !in_quotes && character == u16::from(b'\'') {
            in_apos = !in_apos;
        } else if !in_quotes && !in_apos && is_java_whitespace(character) {
            return index;
        }
        count_char_validated(locator, character);
        index = index.wrapping_add(1);
    }
    -1
}

fn find_next_non_whitespace_char_validated(
    buffer: &[u16],
    offset: i32,
    maxi: i32,
    locator: &mut [i32; 2],
) -> i32 {
    let mut index = offset;
    let mut remaining = maxi.wrapping_sub(offset);
    while remaining != 0 {
        remaining = remaining.wrapping_sub(1);
        let character = buffer[index as usize];
        if !is_java_whitespace(character) {
            return index;
        }
        count_char_validated(locator, character);
        index = index.wrapping_add(1);
    }
    -1
}

fn find_next_structure_end_avoid_quotes_validated(
    buffer: &[u16],
    offset: i32,
    maxi: i32,
    locator: &mut [i32; 2],
) -> i32 {
    let mut in_quotes = false;
    let mut in_apos = false;
    let mut col_index = offset;
    let mut index = offset;
    let mut remaining = maxi.wrapping_sub(offset);
    while remaining != 0 {
        remaining = remaining.wrapping_sub(1);
        let character = buffer[index as usize];
        if character == u16::from(b'\n') {
            col_index = index;
            locator[1] = 0;
            locator[0] = locator[0].wrapping_add(1);
        } else if character == u16::from(b'"') && !in_apos {
            in_quotes = !in_quotes;
        } else if character == u16::from(b'\'') && !in_quotes {
            in_apos = !in_apos;
        } else if character == u16::from(b']') && !in_quotes && !in_apos {
            locator[1] = locator[1].wrapping_add(index.wrapping_sub(col_index));
            return index;
        }
        index = index.wrapping_add(1);
    }
    locator[1] = locator[1].wrapping_add(maxi.wrapping_sub(col_index));
    -1
}

fn count_char_validated(locator: &mut [i32; 2], character: u16) {
    if character == u16::from(b'\n') {
        locator[0] = locator[0].wrapping_add(1);
        locator[1] = 1;
    } else {
        locator[1] = locator[1].wrapping_add(1);
    }
}

fn dispatch_standalone_start(
    handler: &mut Option<&mut dyn ITextHandler>,
    buffer: Option<&mut [u16]>,
    name_offset: i32,
    name_len: i32,
    line: i32,
    col: i32,
) -> Result<(), TextParsingElementError> {
    let handler = handler
        .as_deref_mut()
        .ok_or(TextParsingElementError::NullStandaloneStartHandler)?;
    handler
        .handle_standalone_element_start(
            Some(buffer.expect("validated element has non-null buffer")),
            name_offset,
            name_len,
            true,
            line,
            col,
        )
        .map_err(Into::into)
}

fn dispatch_standalone_end(
    handler: &mut Option<&mut dyn ITextHandler>,
    buffer: Option<&mut [u16]>,
    name_offset: i32,
    name_len: i32,
    line: i32,
    col: i32,
) -> Result<(), TextParsingElementError> {
    let handler = handler
        .as_deref_mut()
        .ok_or(TextParsingElementError::NullStandaloneEndHandler)?;
    handler
        .handle_standalone_element_end(
            Some(buffer.expect("validated element has non-null buffer")),
            name_offset,
            name_len,
            true,
            line,
            col,
        )
        .map_err(Into::into)
}

fn dispatch_open_start(
    handler: &mut Option<&mut dyn ITextHandler>,
    buffer: Option<&mut [u16]>,
    name_offset: i32,
    name_len: i32,
    line: i32,
    col: i32,
) -> Result<(), TextParsingElementError> {
    let handler = handler
        .as_deref_mut()
        .ok_or(TextParsingElementError::NullOpenStartHandler)?;
    handler
        .handle_open_element_start(
            Some(buffer.expect("validated element has non-null buffer")),
            name_offset,
            name_len,
            line,
            col,
        )
        .map_err(Into::into)
}

fn dispatch_open_end(
    handler: &mut Option<&mut dyn ITextHandler>,
    buffer: Option<&mut [u16]>,
    name_offset: i32,
    name_len: i32,
    line: i32,
    col: i32,
) -> Result<(), TextParsingElementError> {
    let handler = handler
        .as_deref_mut()
        .ok_or(TextParsingElementError::NullOpenEndHandler)?;
    handler
        .handle_open_element_end(
            Some(buffer.expect("validated element has non-null buffer")),
            name_offset,
            name_len,
            line,
            col,
        )
        .map_err(Into::into)
}

fn dispatch_close_start(
    handler: &mut Option<&mut dyn ITextHandler>,
    buffer: Option<&mut [u16]>,
    name_offset: i32,
    name_len: i32,
    line: i32,
    col: i32,
) -> Result<(), TextParsingElementError> {
    let handler = handler
        .as_deref_mut()
        .ok_or(TextParsingElementError::NullCloseStartHandler)?;
    handler
        .handle_close_element_start(
            Some(buffer.expect("validated element has non-null buffer")),
            name_offset,
            name_len,
            line,
            col,
        )
        .map_err(Into::into)
}

fn dispatch_close_end(
    handler: &mut Option<&mut dyn ITextHandler>,
    buffer: Option<&mut [u16]>,
    name_offset: i32,
    name_len: i32,
    line: i32,
    col: i32,
) -> Result<(), TextParsingElementError> {
    let handler = handler
        .as_deref_mut()
        .ok_or(TextParsingElementError::NullCloseEndHandler)?;
    handler
        .handle_close_element_end(
            Some(buffer.expect("validated element has non-null buffer")),
            name_offset,
            name_len,
            line,
            col,
        )
        .map_err(Into::into)
}

fn array_unit(buffer: Option<&[u16]>, index: i32) -> Result<u16, TextParsingElementError> {
    let buffer = buffer.ok_or(TextParsingElementError::NullArrayLoad)?;
    usize::try_from(index)
        .ok()
        .and_then(|index| buffer.get(index).copied())
        .ok_or(TextParsingElementError::ArrayIndex {
            index,
            length: buffer.len(),
        })
}

fn java_string_from_range(
    buffer: Option<&[u16]>,
    offset: i32,
    len: i32,
) -> Result<Vec<u16>, TextParsingElementError> {
    let buffer = buffer.ok_or(TextParsingElementError::NullStringValue)?;
    let end = offset.wrapping_add(len);
    let valid = offset >= 0
        && len >= 0
        && end >= offset
        && usize::try_from(end).is_ok_and(|end| end <= buffer.len());
    if !valid {
        return Err(TextParsingElementError::StringRange {
            offset,
            len,
            length: buffer.len(),
        });
    }
    Ok(buffer[offset as usize..end as usize].to_vec())
}

fn is_java_whitespace(character: u16) -> bool {
    matches!(
        character,
        0x0009..=0x000D
            | 0x001C..=0x0020
            | 0x1680
            | 0x2000..=0x2006
            | 0x2008..=0x200A
            | 0x2028
            | 0x2029
            | 0x205F
            | 0x3000
    )
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::{
        TextParsingElementError, TextParsingElementUtil, dispatch_close_end, dispatch_open_end,
        dispatch_standalone_end,
    };
    use crate::text::{
        ITextHandler, TextParseException, TextParsingAttributeSequenceError, TextParsingUtilError,
    };
    use crate::util::JavaString;

    const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    const JAVA_GOLDEN: &str = include_str!("../../tests/fixtures/text_parsing_element_golden.txt");
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x100_0000_01b3;

    #[derive(Clone, Copy)]
    enum ElementKind {
        Standalone,
        Open,
        Close,
    }

    #[derive(Clone, Copy, Eq, PartialEq)]
    enum Mode {
        Normal,
        MutateStart,
        MutateAttribute,
        CheckedStart,
        CheckedAttribute,
        CheckedEnd,
        RuntimeStart,
        RuntimeAttribute,
        RuntimeEnd,
    }

    struct RecordingHandler {
        calls: String,
        mode: Mode,
        attribute_count: usize,
    }

    impl RecordingHandler {
        fn new(mode: Mode) -> Self {
            Self {
                calls: String::new(),
                mode,
                attribute_count: 0,
            }
        }

        fn record(&mut self, value: &str) {
            if !self.calls.is_empty() {
                self.calls.push('|');
            }
            self.calls.push_str(value);
        }

        fn after_start(&mut self, buffer: &mut [u16]) -> Result<(), Box<TextParseException>> {
            self.fail(Mode::CheckedStart, Mode::RuntimeStart)?;
            if self.mode == Mode::MutateStart {
                mutate_next(buffer, u16::from(b'a'));
            }
            Ok(())
        }

        fn after_end(&self) -> Result<(), Box<TextParseException>> {
            self.fail(Mode::CheckedEnd, Mode::RuntimeEnd)
        }

        fn fail(&self, checked: Mode, runtime: Mode) -> Result<(), Box<TextParseException>> {
            if self.mode == checked {
                return Err(Box::new(TextParseException::with_message_at(
                    Some(&JavaString::from_rust_str("handler")),
                    41,
                    43,
                )));
            }
            assert!(self.mode != runtime, "runtime");
            Ok(())
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
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            minimized: bool,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(&format!(
                "SS:{name_offset}:{name_len}:{minimized}:{line}:{col}"
            ));
            self.after_start(buffer.expect("element callback receives the parser buffer"))
        }

        fn handle_standalone_element_end(
            &mut self,
            _: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            minimized: bool,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(&format!(
                "SE:{name_offset}:{name_len}:{minimized}:{line}:{col}"
            ));
            self.after_end()
        }

        fn handle_open_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(&format!("OS:{name_offset}:{name_len}:{line}:{col}"));
            self.after_start(buffer.expect("element callback receives the parser buffer"))
        }

        fn handle_open_element_end(
            &mut self,
            _: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(&format!("OE:{name_offset}:{name_len}:{line}:{col}"));
            self.after_end()
        }

        fn handle_close_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(&format!("CS:{name_offset}:{name_len}:{line}:{col}"));
            self.after_start(buffer.expect("element callback receives the parser buffer"))
        }

        fn handle_close_element_end(
            &mut self,
            _: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(&format!("CE:{name_offset}:{name_len}:{line}:{col}"));
            self.after_end()
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
            self.attribute_count += 1;
            self.record(&format!(
                "A:{name_offset}:{name_len}:{name_line}:{name_col}:\
                 {operator_offset}:{operator_len}:{operator_line}:{operator_col}:\
                 {value_content_offset}:{value_content_len}:{value_outer_offset}:\
                 {value_outer_len}:{value_line}:{value_col}"
            ));
            self.fail(Mode::CheckedAttribute, Mode::RuntimeAttribute)?;
            if self.mode == Mode::MutateAttribute && self.attribute_count == 1 {
                mutate_next(
                    buffer.expect("attribute callback receives the parser buffer"),
                    u16::from(b'c'),
                );
            }
            Ok(())
        }
    }

    #[test]
    fn element_parsing_matches_java_golden() {
        let mut output = String::new();
        emit(&mut output, "baseline", JAVA_BASELINE);
        predicate_cases(&mut output);
        standalone_cases(&mut output);
        open_cases(&mut output);
        close_cases(&mut output);
        handler_cases(&mut output);
        runtime_cases(&mut output);
        exhaustive_cases(&mut output);
        let actual_lines = output.lines().collect::<Vec<_>>();
        let expected_lines = JAVA_GOLDEN.lines().collect::<Vec<_>>();
        assert_eq!(actual_lines.len(), expected_lines.len());
        for (index, (actual, expected)) in actual_lines.iter().zip(&expected_lines).enumerate() {
            require(
                actual == expected,
                &format!("Java Golden record {}", index + 1),
            );
        }
    }

    #[test]
    fn error_adapter_and_unreachable_null_end_callbacks_are_covered() {
        let text_parse = TextParsingElementError::TextParse(Box::default());
        assert_eq!(
            text_parse.java_class_name(),
            "org.thymeleaf.templateparser.text.TextParseException"
        );
        assert_eq!(text_parse.java_message().to_string_lossy(), "null");
        assert_eq!(text_parse.text_parse_location(), None);
        assert!(std::error::Error::source(&text_parse).is_some());

        let scanning = TextParsingElementError::Scanning(TextParsingUtilError::NullDirectLocator);
        assert_eq!(scanning.java_class_name(), "java.lang.NullPointerException");
        assert_eq!(
            scanning.to_string(),
            "Cannot load from int array because \"<parameter4>\" is null"
        );
        assert!(std::error::Error::source(&scanning).is_some());
        let null_scanning = TextParsingElementError::from(TextParsingUtilError::NullText);
        assert_eq!(null_scanning.java_message().to_string_lossy(), "null");

        let attribute =
            TextParsingElementError::from(TextParsingAttributeSequenceError::NullHandler);
        assert_eq!(
            attribute.java_class_name(),
            "java.lang.NullPointerException"
        );
        assert!(!attribute.java_message().as_utf16().is_empty());
        assert!(std::error::Error::source(&attribute).is_some());
        let attribute_text = TextParsingElementError::from(
            TextParsingAttributeSequenceError::TextParse(Box::default()),
        );
        require(
            attribute_text.java_class_name()
                == "org.thymeleaf.templateparser.text.TextParseException",
            "attribute TextParse conversion",
        );
        let attribute_scanning = TextParsingElementError::from(
            TextParsingAttributeSequenceError::Scanning(TextParsingUtilError::NullText),
        );
        require(
            attribute_scanning.java_message().to_string_lossy() == "null",
            "attribute scanning conversion",
        );

        let mut handler = RecordingHandler::new(Mode::Normal);
        let mut buffer = [u16::from(b'x')];
        handler.handle_document_start(0, 0, 0).unwrap();
        handler.handle_document_end(0, 0, 0, 0).unwrap();
        handler.handle_text(Some(&mut buffer), 0, 0, 0, 0).unwrap();
        handler
            .handle_comment(Some(&mut buffer), 0, 0, 0, 0, 0, 0)
            .unwrap();

        let standalone_end_error =
            dispatch_standalone_end(&mut None, Some(&mut buffer), 0, 0, 0, 0)
                .expect_err("null standalone end handler");
        require(
            standalone_end_error.to_string() == super::NULL_STANDALONE_END_MESSAGE,
            "null standalone end handler",
        );
        let open_end_error = dispatch_open_end(&mut None, Some(&mut buffer), 0, 0, 0, 0)
            .expect_err("null open end handler");
        require(
            open_end_error.to_string() == super::NULL_OPEN_END_MESSAGE,
            "null open end handler",
        );
        let close_end_error = dispatch_close_end(&mut None, Some(&mut buffer), 0, 0, 0, 0)
            .expect_err("null close end handler");
        require(
            close_end_error.to_string() == super::NULL_CLOSE_END_MESSAGE,
            "null close end handler",
        );
        for error in [
            TextParsingElementError::NullStandaloneEndHandler,
            TextParsingElementError::NullOpenEndHandler,
            TextParsingElementError::NullCloseEndHandler,
        ] {
            assert!(!error.java_message().as_utf16().is_empty());
            assert!(std::error::Error::source(&error).is_none());
        }

        let standalone_load_error =
            TextParsingElementUtil::is_element_end(Some(&[u16::from(b'/')]), 0, 2, true)
                .expect_err("standalone end second load");
        require(
            standalone_load_error.to_string() == "Index 1 out of bounds for length 1",
            "standalone end second load",
        );
        let open_prefix_error =
            TextParsingElementUtil::is_open_element_start(Some(&[u16::from(b'[')]), 0, 3)
                .expect_err("open prefix second load");
        require(
            open_prefix_error.to_string() == "Index 1 out of bounds for length 1",
            "open prefix second load",
        );
        let open_name_error = TextParsingElementUtil::is_open_element_start(
            Some(&[u16::from(b'['), u16::from(b'#')]),
            0,
            3,
        )
        .expect_err("open name first load");
        require(
            open_name_error.to_string() == "Index 2 out of bounds for length 2",
            "open name first load",
        );
        let close_prefix_error =
            TextParsingElementUtil::is_close_element_start(Some(&[u16::from(b'[')]), 0, 3)
                .expect_err("close prefix second load");
        require(
            close_prefix_error.to_string() == "Index 1 out of bounds for length 1",
            "close prefix second load",
        );
        let mut untouched = [u16::from(b'x')];
        mutate_next(&mut untouched, u16::from(b'z'));
        assert_eq!(untouched, [u16::from(b'x')]);
        let failure = catch_unwind(|| require(false, "expected coverage panic"));
        require(failure.is_err(), "require false branch must panic");
    }

    fn predicate_cases(output: &mut String) {
        for (key, text, offset, maxi) in [
            ("predicate.open", "[#name]", 0, 7),
            ("predicate.close", "[/name]", 0, 7),
            ("predicate.noNameOpen", "[#]", 0, 3),
            ("predicate.noNameStandalone", "[#/]", 0, 4),
            ("predicate.whitespaceName", "[# a=b]", 0, 7),
            ("predicate.forbiddenDash", "[#-x]", 0, 5),
            ("predicate.forbiddenBang", "[#!x]", 0, 5),
            ("predicate.forbiddenSlash", "[#/x]", 0, 5),
            ("predicate.forbiddenQuestion", "[#?x]", 0, 5),
            ("predicate.forbiddenBracket", "[#[x]", 0, 5),
            ("predicate.forbiddenBrace", "[#{x]", 0, 5),
            ("predicate.nulName", "[#\0]", 0, 4),
        ] {
            let buffer = text.encode_utf16().collect::<Vec<_>>();
            emit(
                output,
                key,
                predicates(Some(&buffer), offset, maxi).to_string_lossy(),
            );
        }
        let surrogate = [u16::from(b'['), u16::from(b'#'), 0xd800, u16::from(b']')];
        emit(
            output,
            "predicate.surrogateName",
            predicates(Some(&surrogate), 0, 4).to_string_lossy(),
        );
        for (key, text, offset, maxi) in [
            ("predicate.openEnd", "]", 0, 1),
            ("predicate.standaloneEnd", "/]", 0, 2),
            ("predicate.badStandaloneEnd", "/x", 0, 2),
            ("predicate.emptyEnd", "", 0, 0),
        ] {
            let buffer = text.encode_utf16().collect::<Vec<_>>();
            emit(
                output,
                key,
                predicates(Some(&buffer), offset, maxi).to_string_lossy(),
            );
        }
        emit(
            output,
            "predicate.nullShort",
            predicates(None, 0, 2).to_string_lossy(),
        );
        emit(
            output,
            "predicate.nullLong",
            predicates(None, 0, 3).to_string_lossy(),
        );
        let normal = "[#x]".encode_utf16().collect::<Vec<_>>();
        emit(
            output,
            "predicate.negativeOffset",
            predicates(Some(&normal), -1, 4).to_string_lossy(),
        );
        emit(
            output,
            "predicate.pastEnd",
            predicates(Some(&normal), 4, 8).to_string_lossy(),
        );
        emit(
            output,
            "predicate.wrappedMax",
            predicates(Some(&normal), i32::MAX, i32::MIN).to_string_lossy(),
        );
    }

    fn standalone_cases(output: &mut String) {
        for (key, text, offset, len, line, col) in [
            ("standalone.name", "[#x/]", 0, 5, 1, 1),
            ("standalone.noName", "[#/]", 0, 4, 2, 3),
            ("standalone.attributes", "[#x a=b c='d e'/]", 0, 17, 4, 5),
            ("standalone.noNameAttributes", "[# a=b/]", 0, 8, 7, 11),
            ("standalone.multiline", "[#x\n a=\"b\nc\"/]", 0, 14, 10, 20),
            ("standalone.quotedEnd", "[#x a=\"/\"/]", 0, 11, 1, 1),
            ("standalone.embedded", "zz[#x a=b/]yy", 2, 9, 3, 9),
            ("standalone.invalidShort", "[#]", 0, 3, 1, 1),
            ("standalone.invalidPrefix", "[/x/]", 0, 5, 1, 1),
            ("standalone.invalidEnd", "[#x]", 0, 4, 1, 1),
            ("standalone.invalidName", "[#-x/]", 0, 6, 1, 1),
        ] {
            emit_parse(
                output,
                key,
                ElementKind::Standalone,
                Some(text),
                offset,
                len,
                line,
                col,
                Mode::Normal,
            );
        }
    }

    fn open_cases(output: &mut String) {
        for (key, text, offset, len, line, col) in [
            ("open.name", "[#x]", 0, 4, 1, 1),
            ("open.noName", "[#]", 0, 3, 2, 3),
            ("open.attributes", "[#x a=b c=\"d e\"]", 0, 16, 4, 5),
            ("open.noNameAttributes", "[# a=b]", 0, 7, 7, 11),
            ("open.trailingWhitespace", "[#x \t]", 0, 6, 1, 1),
            ("open.multiline", "[#x\n a=\"b\nc\"]", 0, 13, 10, 20),
            ("open.quotedEnd", "[#x a=\"]\"]", 0, 10, 1, 1),
            ("open.doubleQuoteInName", "[#x\" y\" a=b]", 0, 12, 1, 1),
            ("open.singleQuoteInName", "[#x' y' a=b]", 0, 12, 1, 1),
            ("open.innerStructureEnd", "[#x a=b]c]", 0, 10, 1, 1),
        ] {
            emit_parse(
                output,
                key,
                ElementKind::Open,
                Some(text),
                offset,
                len,
                line,
                col,
                Mode::Normal,
            );
        }
        let mut buffer = vec![
            u16::from(b'['),
            u16::from(b'#'),
            0xd800,
            u16::from(b' '),
            u16::from(b'a'),
            u16::from(b'='),
            0,
            u16::from(b']'),
        ];
        let value = outcome(
            Some(&mut buffer),
            ElementKind::Open,
            0,
            8,
            1,
            1,
            Mode::Normal,
        );
        emit(output, "open.nulSurrogate", value.to_string_lossy());
        for (key, text, offset, len, line, col) in [
            ("open.lineOverflow", "[#x\n a=b]", 0, 9, i32::MAX, i32::MAX),
            ("open.invalidShort", "[]", 0, 2, 1, 1),
            ("open.invalidPrefix", "[/x]", 0, 4, 1, 1),
            ("open.invalidEnd", "[#x/", 0, 4, 1, 1),
            ("open.invalidName", "[#{x]", 0, 5, 1, 1),
        ] {
            emit_parse(
                output,
                key,
                ElementKind::Open,
                Some(text),
                offset,
                len,
                line,
                col,
                Mode::Normal,
            );
        }
    }

    fn close_cases(output: &mut String) {
        for (key, text, offset, len, line, col) in [
            ("close.name", "[/x]", 0, 4, 1, 1),
            ("close.noName", "[/]", 0, 3, 2, 3),
            ("close.trailingWhitespace", "[/x \t]", 0, 6, 4, 5),
            ("close.noNameWhitespace", "[/ ]", 0, 4, 7, 11),
            ("close.multiline", "[/x\n \t]", 0, 7, 10, 20),
            ("close.attributesRejected", "[/x a=b]", 0, 8, 3, 9),
            ("close.noNameAttributeRejected", "[/ a=b]", 0, 7, 3, 9),
            ("close.invalidShort", "[]", 0, 2, 1, 1),
            ("close.invalidPrefix", "[#x]", 0, 4, 1, 1),
            ("close.invalidEnd", "[/x/", 0, 4, 1, 1),
            ("close.invalidName", "[/?x]", 0, 5, 1, 1),
        ] {
            emit_parse(
                output,
                key,
                ElementKind::Close,
                Some(text),
                offset,
                len,
                line,
                col,
                Mode::Normal,
            );
        }
    }

    fn handler_cases(output: &mut String) {
        for kind in [
            ElementKind::Standalone,
            ElementKind::Open,
            ElementKind::Close,
        ] {
            let (name, text) = match kind {
                ElementKind::Standalone => ("standalone", "[#x a=b c=d/]"),
                ElementKind::Open => ("open", "[#x a=b c=d]"),
                ElementKind::Close => ("close", "[/x ]"),
            };
            for (suffix, mode) in [
                ("checkedStart", Mode::CheckedStart),
                ("checkedEnd", Mode::CheckedEnd),
                ("runtimeStart", Mode::RuntimeStart),
                ("runtimeEnd", Mode::RuntimeEnd),
            ] {
                emit_parse(
                    output,
                    &format!("handler.{name}.{suffix}"),
                    kind,
                    Some(text),
                    0,
                    text.encode_utf16().count() as i32,
                    1,
                    1,
                    mode,
                );
            }
            emit_null_handler(output, &format!("handler.{name}.null"), kind, text);

            let name_only = match kind {
                ElementKind::Standalone => "[#x/]",
                ElementKind::Open => "[#x]",
                ElementKind::Close => "[/x]",
            };
            for (suffix, mode) in [
                ("nameOnlyCheckedStart", Mode::CheckedStart),
                ("nameOnlyCheckedEnd", Mode::CheckedEnd),
            ] {
                emit_unregistered_parse(kind, name_only, mode, &format!("handler.{name}.{suffix}"));
            }
        }

        for (key, kind, text, mode) in [
            (
                "handler.standalone.checkedAttribute",
                ElementKind::Standalone,
                "[#x a=b c=d/]",
                Mode::CheckedAttribute,
            ),
            (
                "handler.open.checkedAttribute",
                ElementKind::Open,
                "[#x a=b c=d]",
                Mode::CheckedAttribute,
            ),
            (
                "handler.open.runtimeAttribute",
                ElementKind::Open,
                "[#x a=b c=d]",
                Mode::RuntimeAttribute,
            ),
            (
                "handler.open.mutateStart",
                ElementKind::Open,
                "[#x a=b c=d]",
                Mode::MutateStart,
            ),
            (
                "handler.open.mutateAttribute",
                ElementKind::Open,
                "[#x a=b c=d]",
                Mode::MutateAttribute,
            ),
        ] {
            emit_parse(
                output,
                key,
                kind,
                Some(text),
                0,
                text.encode_utf16().count() as i32,
                1,
                1,
                mode,
            );
        }
    }

    fn runtime_cases(output: &mut String) {
        for (key, kind, text, offset, len) in [
            (
                "runtime.standaloneNullShort",
                ElementKind::Standalone,
                None,
                0,
                0,
            ),
            (
                "runtime.standaloneNullLong",
                ElementKind::Standalone,
                None,
                0,
                4,
            ),
            ("runtime.openNullShort", ElementKind::Open, None, 0, 0),
            ("runtime.openNullLong", ElementKind::Open, None, 0, 3),
            ("runtime.closeNullShort", ElementKind::Close, None, 0, 0),
            ("runtime.closeNullLong", ElementKind::Close, None, 0, 3),
            (
                "runtime.negativeOffset",
                ElementKind::Open,
                Some("[#x]"),
                -1,
                4,
            ),
            (
                "runtime.offsetPastEnd",
                ElementKind::Open,
                Some("[#x]"),
                4,
                3,
            ),
            (
                "runtime.invalidStringNegativeOffset",
                ElementKind::Open,
                Some("x"),
                -1,
                0,
            ),
            (
                "runtime.invalidStringPastEnd",
                ElementKind::Open,
                Some("x"),
                2,
                0,
            ),
            (
                "runtime.wrappedRange",
                ElementKind::Open,
                Some("[#x]"),
                1,
                i32::MAX,
            ),
            ("runtime.scanPastEnd", ElementKind::Open, Some("[#x]"), 0, 5),
            ("runtime.endPastEnd", ElementKind::Open, Some("[#x"), 0, 4),
            (
                "runtime.closeEndPastEnd",
                ElementKind::Close,
                Some("[/x"),
                0,
                4,
            ),
        ] {
            emit_parse(output, key, kind, text, offset, len, 1, 1, Mode::Normal);
        }
    }

    fn exhaustive_cases(output: &mut String) {
        let mut name_hash = FNV_OFFSET;
        for unit in u16::MIN..=u16::MAX {
            let open = [u16::from(b'['), u16::from(b'#'), unit, u16::from(b']')];
            let close = [u16::from(b'['), u16::from(b'/'), unit, u16::from(b']')];
            name_hash = mix_java_string(name_hash, &predicates(Some(&open), 0, 4));
            name_hash = mix_java_string(name_hash, &predicates(Some(&close), 0, 4));
        }
        emit(
            output,
            "exhaustive.nameUnitHash",
            format!("{name_hash:016x}"),
        );

        let predicate_buffer = "xx[#a/]yy[/b]zz".encode_utf16().collect::<Vec<_>>();
        let mut predicate_range_hash = FNV_OFFSET;
        for offset in -2..=predicate_buffer.len() as i32 + 2 {
            for maxi in -2..=predicate_buffer.len() as i32 + 4 {
                predicate_range_hash = mix_java_string(
                    predicate_range_hash,
                    &predicates(Some(&predicate_buffer), offset, maxi),
                );
            }
        }
        emit(
            output,
            "exhaustive.predicateRangeHash",
            format!("{predicate_range_hash:016x}"),
        );

        let names = [
            Vec::new(),
            "x".encode_utf16().collect(),
            "x:y".encode_utf16().collect(),
            "-".encode_utf16().collect(),
            "{".encode_utf16().collect(),
            vec![0xd800],
        ];
        let attributes = ["", " ", " a=b", "\n a=\"c d\"", " a=", " a b=c"];
        let mut grammar_hash = FNV_OFFSET;
        for kind in [
            ElementKind::Standalone,
            ElementKind::Open,
            ElementKind::Close,
        ] {
            for name in &names {
                for attributes_value in attributes {
                    let mut text = match kind {
                        ElementKind::Close => "[/".encode_utf16().collect::<Vec<_>>(),
                        ElementKind::Standalone | ElementKind::Open => {
                            "[#".encode_utf16().collect::<Vec<_>>()
                        }
                    };
                    text.extend_from_slice(name);
                    text.extend(attributes_value.encode_utf16());
                    text.extend(match kind {
                        ElementKind::Standalone => "/]".encode_utf16().collect::<Vec<_>>(),
                        ElementKind::Open | ElementKind::Close => {
                            "]".encode_utf16().collect::<Vec<_>>()
                        }
                    });
                    let len = text.len() as i32;
                    grammar_hash = mix_java_string(
                        grammar_hash,
                        &outcome(Some(&mut text), kind, 0, len, -7, i32::MAX, Mode::Normal),
                    );
                }
            }
        }
        emit(
            output,
            "exhaustive.grammarHash",
            format!("{grammar_hash:016x}"),
        );

        let range_source = "xx[#a b=\"c d\"/]yy".encode_utf16().collect::<Vec<_>>();
        let mut parse_range_hash = FNV_OFFSET;
        for offset in -2..=range_source.len() as i32 + 2 {
            for len in -2..=range_source.len() as i32 + 4 {
                for kind in [
                    ElementKind::Standalone,
                    ElementKind::Open,
                    ElementKind::Close,
                ] {
                    let mut buffer = range_source.clone();
                    parse_range_hash = mix_java_string(
                        parse_range_hash,
                        &outcome(Some(&mut buffer), kind, offset, len, 13, 17, Mode::Normal),
                    );
                }
            }
        }
        emit(
            output,
            "exhaustive.parseRangeHash",
            format!("{parse_range_hash:016x}"),
        );
    }

    fn predicates(buffer: Option<&[u16]>, offset: i32, maxi: i32) -> JavaString {
        let open = TextParsingElementUtil::is_open_element_start(buffer, offset, maxi);
        let close = TextParsingElementUtil::is_close_element_start(buffer, offset, maxi);
        let regular_end = TextParsingElementUtil::is_element_end(buffer, offset, maxi, false);
        let standalone_end = TextParsingElementUtil::is_element_end(buffer, offset, maxi, true);
        JavaString::from_rust_str(&format!(
            "O={},C={},E0={},E1={}",
            describe_predicate(open),
            describe_predicate(close),
            describe_predicate(regular_end),
            describe_predicate(standalone_end)
        ))
    }

    fn describe_predicate(result: Result<bool, TextParsingElementError>) -> String {
        match result {
            Ok(value) => value.to_string(),
            Err(error) => describe_error(&error),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_parse(
        output: &mut String,
        key: &str,
        kind: ElementKind,
        text: Option<&str>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
        mode: Mode,
    ) {
        let mut buffer = text.map(|text| text.encode_utf16().collect::<Vec<_>>());
        let value = outcome(buffer.as_deref_mut(), kind, offset, len, line, col, mode);
        emit(output, key, value.to_string_lossy());
    }

    fn emit_null_handler(output: &mut String, key: &str, kind: ElementKind, text: &str) {
        let mut buffer = text.encode_utf16().collect::<Vec<_>>();
        let len = buffer.len() as i32;
        let result = invoke(kind, Some(&mut buffer), 0, len, 1, 1, None);
        let error =
            result.expect_err("valid element with null handler must fail at start callback");
        emit(output, key, describe_error(&error));
    }

    fn emit_unregistered_parse(kind: ElementKind, text: &str, mode: Mode, context: &str) {
        let mut buffer = text.encode_utf16().collect::<Vec<_>>();
        let len = buffer.len() as i32;
        let mut handler = RecordingHandler::new(mode);
        let result = invoke(kind, Some(&mut buffer), 0, len, 1, 1, Some(&mut handler));
        assert!(result.is_err(), "{context}");
    }

    #[allow(clippy::too_many_arguments)]
    fn outcome(
        buffer: Option<&mut [u16]>,
        kind: ElementKind,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
        mode: Mode,
    ) -> JavaString {
        let mut buffer = buffer;
        let mut handler = RecordingHandler::new(mode);
        let result = catch_unwind(AssertUnwindSafe(|| {
            invoke(
                kind,
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
                to_utf16_hex(&JavaString::from_rust_str("runtime")),
                handler.calls
            ),
        };
        JavaString::from_rust_str(&format!("{prefix}:{}", describe_buffer(buffer.as_deref())))
    }

    #[allow(clippy::too_many_arguments)]
    fn invoke(
        kind: ElementKind,
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
        handler: Option<&mut dyn ITextHandler>,
    ) -> Result<(), TextParsingElementError> {
        match kind {
            ElementKind::Standalone => TextParsingElementUtil::parse_standalone_element(
                buffer, offset, len, line, col, handler,
            ),
            ElementKind::Open => {
                TextParsingElementUtil::parse_open_element(buffer, offset, len, line, col, handler)
            }
            ElementKind::Close => {
                TextParsingElementUtil::parse_close_element(buffer, offset, len, line, col, handler)
            }
        }
    }

    fn describe_error(error: &TextParsingElementError) -> String {
        let base = format!(
            "ERR:{}:{}",
            error.java_class_name(),
            to_utf16_hex(&error.java_message())
        );
        error
            .text_parse_location()
            .map_or(base.clone(), |(line, col)| format!("{base}:{line}:{col}"))
    }

    fn describe_buffer(buffer: Option<&[u16]>) -> String {
        buffer.map_or_else(
            || "null".to_owned(),
            |buffer| to_utf16_hex(&JavaString::from_utf16(buffer.to_vec())),
        )
    }

    fn mutate_next(buffer: &mut [u16], expected: u16) {
        if let Some(unit) = buffer.iter_mut().find(|unit| **unit == expected) {
            *unit = u16::from(b'=');
        }
    }

    fn mix_java_string(mut hash: u64, value: &JavaString) -> u64 {
        for unit in value.as_utf16() {
            hash = mix(hash, i32::from((*unit & 0x00ff) as u8));
            hash = mix(hash, i32::from((*unit >> 8) as u8));
        }
        hash
    }

    fn mix(hash: u64, value: i32) -> u64 {
        (hash ^ value as i64 as u64).wrapping_mul(FNV_PRIME)
    }

    fn to_utf16_hex(value: &JavaString) -> String {
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

    fn require(condition: bool, context: &str) {
        if !condition {
            panic!("{context}");
        }
    }
}
