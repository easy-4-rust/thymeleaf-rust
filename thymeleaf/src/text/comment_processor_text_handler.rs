use std::error::Error;
use std::fmt::{Display, Formatter};
use std::panic::panic_any;

use super::parsing_locator_util::ParsingLocatorError;
use super::{
    AbstractChainedTextHandler, ITextHandler, ParsingLocatorUtil, TextParseException,
    TextParsingElementError, TextParsingElementUtil,
};
use crate::util::Utf16String;

const FILTER_BUFFER_INCREMENT: i32 = 256;
const NULL_CHAR_ARRAY_MESSAGE: &str =
    "Cannot load from char array because \"<parameter1>\" is null";
const NULL_NEXT_TEXT_MESSAGE: &str = "Cannot invoke \"org.thymeleaf.templateparser.text.ITextHandler.handleText(char[], int, int, int, int)\" because the return value of \"org.thymeleaf.templateparser.text.CommentProcessorTextHandler.getNext()\" is null";

/// 注释预处理器产生的 Java 未检查异常适配。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.CommentProcessorTextHandler`。
///
/// 本类型保存本对象直接数组访问、`System.arraycopy`、locator 更新、元素解析和
/// `getNext()` 空返回值产生的 Java 异常类别与可空 UTF-16 消息。下游
/// [`TextParseException`] 仍经 `Result` 原对象传播。
#[derive(Debug)]
pub(crate) enum CommentProcessorTextHandlerRuntimeError {
    /// 直接读取的 Java `char[]` 为 null。
    NullCharArrayLoad,
    /// 直接数组下标访问失败。
    ArrayIndex { index: i32, length: usize },
    /// `System.arraycopy` 参数校验失败。
    ArrayCopy {
        java_class_name: &'static str,
        java_message: Option<Utf16String>,
    },
    /// `getNext()` 返回 null 后直接调用 `handleText`。
    NullNextText,
    /// locator 更新失败。
    Locator(ParsingLocatorError),
    /// 元素谓词或元素事件解包失败。
    Element(TextParsingElementError),
}

impl CommentProcessorTextHandlerRuntimeError {
    fn arraycopy_null_source() -> Self {
        Self::ArrayCopy {
            java_class_name: "java.lang.NullPointerException",
            java_message: None,
        }
    }

    fn arraycopy_negative_length(length: i32) -> Self {
        Self::ArrayCopy {
            java_class_name: "java.lang.ArrayIndexOutOfBoundsException",
            java_message: Some(Utf16String::from_rust_str(&format!(
                "arraycopy: length {length} is negative"
            ))),
        }
    }

    fn arraycopy_source_index(index: i32, length: usize) -> Self {
        Self::ArrayCopy {
            java_class_name: "java.lang.ArrayIndexOutOfBoundsException",
            java_message: Some(Utf16String::from_rust_str(&format!(
                "arraycopy: source index {index} out of bounds for char[{length}]"
            ))),
        }
    }

    fn arraycopy_destination_index(index: i32, length: usize) -> Self {
        Self::ArrayCopy {
            java_class_name: "java.lang.ArrayIndexOutOfBoundsException",
            java_message: Some(Utf16String::from_rust_str(&format!(
                "arraycopy: destination index {index} out of bounds for char[{length}]"
            ))),
        }
    }

    fn arraycopy_last_source(index: i64, length: usize) -> Self {
        Self::ArrayCopy {
            java_class_name: "java.lang.ArrayIndexOutOfBoundsException",
            java_message: Some(Utf16String::from_rust_str(&format!(
                "arraycopy: last source index {index} out of bounds for char[{length}]"
            ))),
        }
    }

    fn arraycopy_last_destination(index: i64, length: usize) -> Self {
        Self::ArrayCopy {
            java_class_name: "java.lang.ArrayIndexOutOfBoundsException",
            java_message: Some(Utf16String::from_rust_str(&format!(
                "arraycopy: last destination index {index} out of bounds for char[{length}]"
            ))),
        }
    }

    /// 返回对应 Java 异常全限定名。
    ///
    /// # 返回
    /// 与固定 JVM Oracle 中 `Throwable#getClass().getName()` 一致的名称。
    #[must_use]
    pub(crate) const fn java_class_name(&self) -> &'static str {
        match self {
            Self::NullCharArrayLoad | Self::NullNextText => "java.lang.NullPointerException",
            Self::ArrayIndex { .. } => "java.lang.ArrayIndexOutOfBoundsException",
            Self::ArrayCopy {
                java_class_name, ..
            } => java_class_name,
            Self::Locator(error) => error.java_class_name(),
            Self::Element(error) => error.java_class_name(),
        }
    }

    /// 返回对应 Java 异常的可空 UTF-16 消息。
    ///
    /// # 返回
    /// `None` 仅表示 HotSpot `System.arraycopy` 的 null-source NPE 消息为 null。
    #[must_use]
    pub(crate) fn java_message(&self) -> Option<Utf16String> {
        match self {
            Self::NullCharArrayLoad => Some(Utf16String::from_rust_str(NULL_CHAR_ARRAY_MESSAGE)),
            Self::ArrayIndex { index, length } => Some(Utf16String::from_rust_str(&format!(
                "Index {index} out of bounds for length {length}"
            ))),
            Self::ArrayCopy { java_message, .. } => java_message.clone(),
            Self::NullNextText => Some(Utf16String::from_rust_str(NULL_NEXT_TEXT_MESSAGE)),
            Self::Locator(error) => Some(error.message()),
            Self::Element(error) => Some(error.java_message()),
        }
    }
}

impl Display for CommentProcessorTextHandlerRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self.java_message() {
            Some(message) => formatter.write_str(&message.to_string_lossy()),
            None => formatter.write_str("null"),
        }
    }
}

impl Error for CommentProcessorTextHandlerRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Locator(error) => Some(error),
            Self::Element(error) => Some(error),
            _ => None,
        }
    }
}

/// 将文本注释转换为注释元素事件或内联表达式文本，并过滤其自然模板内容。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.CommentProcessorTextHandler`。
///
/// 普通注释始终作为 text 事件转发，绝不产生 comment 事件。`[#...]`、
/// `[/...]` 会被解包为元素事件；标准方言存在时，`[(...)]` 与 `[[...]]`
/// 先输出表达式本身，再累计并过滤后续自然文本，直到遇到规定的刷新事件。
pub(crate) struct CommentProcessorTextHandler {
    chained: AbstractChainedTextHandler,
    standard_dialect_present: bool,
    filter_texts: bool,
    filtered_text_buffer: Option<Vec<u16>>,
    filtered_text_size: i32,
    filtered_text_locator: Option<[i32; 2]>,
}

impl CommentProcessorTextHandler {
    /// 创建注释预处理器。
    ///
    /// 对应 Java:
    /// `CommentProcessorTextHandler#CommentProcessorTextHandler(boolean,ITextHandler)`。
    ///
    /// # 参数
    /// - `standard_dialect_present`：是否启用标准方言内联表达式转换。
    /// - `handler`：下游 handler；`None` 保留 Java null 的延迟失败。
    #[must_use]
    pub(crate) fn new(
        standard_dialect_present: bool,
        handler: Option<Box<dyn ITextHandler>>,
    ) -> Self {
        Self {
            chained: AbstractChainedTextHandler::new(handler),
            standard_dialect_present,
            filter_texts: false,
            filtered_text_buffer: None,
            filtered_text_size: 0,
            filtered_text_locator: None,
        }
    }

    fn is_comment_processable(
        &self,
        buffer: Option<&[u16]>,
        content_offset: i32,
        content_len: i32,
    ) -> bool {
        let maxi = content_offset.wrapping_add(content_len);
        if content_len < 3
            || array_unit(buffer, content_offset) != u16::from(b'[')
            || array_unit(buffer, maxi.wrapping_sub(1)) != u16::from(b']')
        {
            return false;
        }
        if content_len >= 4
            && array_unit(buffer, content_offset.wrapping_add(1)) == u16::from(b'(')
            && array_unit(buffer, maxi.wrapping_sub(2)) == u16::from(b')')
        {
            return true;
        }
        if content_len >= 4
            && array_unit(buffer, content_offset.wrapping_add(1)) == u16::from(b'[')
            && array_unit(buffer, maxi.wrapping_sub(2)) == u16::from(b']')
        {
            return true;
        }

        if element_bool(TextParsingElementUtil::is_open_element_start(
            buffer,
            content_offset,
            maxi,
        )) {
            return element_bool(TextParsingElementUtil::is_element_end(
                buffer,
                maxi.wrapping_sub(1),
                maxi,
                false,
            ));
        }
        if element_bool(TextParsingElementUtil::is_close_element_start(
            buffer,
            content_offset,
            maxi,
        )) {
            return element_bool(TextParsingElementUtil::is_element_end(
                buffer,
                maxi.wrapping_sub(1),
                maxi,
                false,
            ));
        }
        false
    }

    fn filter_text(&mut self, buffer: Option<&[u16]>, offset: i32, len: i32, line: i32, col: i32) {
        if self.filtered_text_buffer.is_none() {
            let allocation_len = FILTER_BUFFER_INCREMENT.max(len);
            self.filtered_text_buffer = Some(vec![0; allocation_len as usize]);
            self.filtered_text_size = 0;
            self.filtered_text_locator = Some([0, 0]);
        } else {
            let buffer_len = self
                .filtered_text_buffer
                .as_ref()
                .expect("filtered buffer exists")
                .len() as i32;
            let required_len = self.filtered_text_size.wrapping_add(len);
            if required_len > buffer_len {
                let new_len = buffer_len
                    .wrapping_add(FILTER_BUFFER_INCREMENT)
                    .max(required_len);
                let mut grown = vec![0; new_len as usize];
                let old = self
                    .filtered_text_buffer
                    .as_ref()
                    .expect("filtered buffer exists");
                grown[..self.filtered_text_size as usize]
                    .copy_from_slice(&old[..self.filtered_text_size as usize]);
                self.filtered_text_buffer = Some(grown);
            }
        }

        java_arraycopy(
            buffer,
            offset,
            self.filtered_text_buffer
                .as_mut()
                .expect("filter initialization creates buffer"),
            self.filtered_text_size,
            len,
        );
        self.filtered_text_size = self.filtered_text_size.wrapping_add(len);
        let locator = self
            .filtered_text_locator
            .as_mut()
            .expect("filter initialization creates locator");
        locator[0] = line;
        locator[1] = col;
    }

    fn process_filtered_texts(&mut self) -> Result<(), Box<TextParseException>> {
        if !self.filter_texts {
            return Ok(());
        }

        let filter_offset = compute_filter_offset(
            self.filtered_text_buffer.as_deref(),
            0,
            self.filtered_text_size,
            self.filtered_text_locator
                .as_mut()
                .map(|locator| locator.as_mut_slice()),
        );

        if filter_offset < self.filtered_text_size {
            let locator = self
                .filtered_text_locator
                .expect("non-empty filtered buffer has locator");
            self.chained.handle_text(
                self.filtered_text_buffer.as_deref_mut(),
                filter_offset,
                self.filtered_text_size.wrapping_sub(filter_offset),
                locator[0],
                locator[1],
            )?;
        }

        self.filtered_text_size = 0;
        self.filter_texts = false;
        Ok(())
    }

    fn direct_next_text(
        &mut self,
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        match self.chained.get_next() {
            Some(next) => next.handle_text(buffer, offset, len, line, col),
            None => panic_runtime(CommentProcessorTextHandlerRuntimeError::NullNextText),
        }
    }
}

impl ITextHandler for CommentProcessorTextHandler {
    fn handle_document_start(
        &mut self,
        start_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.chained
            .handle_document_start(start_time_nanos, line, col)
    }

    fn handle_document_end(
        &mut self,
        end_time_nanos: i64,
        total_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.process_filtered_texts()?;
        self.chained
            .handle_document_end(end_time_nanos, total_time_nanos, line, col)
    }

    fn handle_text(
        &mut self,
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        if self.filter_texts {
            self.filter_text(buffer.as_deref(), offset, len, line, col);
            return Ok(());
        }
        self.chained.handle_text(buffer, offset, len, line, col)
    }

    fn handle_comment(
        &mut self,
        mut buffer: Option<&mut [u16]>,
        content_offset: i32,
        content_len: i32,
        outer_offset: i32,
        outer_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.process_filtered_texts()?;

        if !self.is_comment_processable(buffer.as_deref(), content_offset, content_len) {
            return self
                .chained
                .handle_text(buffer, outer_offset, outer_len, line, col);
        }

        let maxi = content_offset.wrapping_add(content_len);
        if element_bool(TextParsingElementUtil::is_open_element_start(
            buffer.as_deref(),
            content_offset,
            maxi,
        )) {
            if element_bool(TextParsingElementUtil::is_element_end(
                buffer.as_deref(),
                maxi.wrapping_sub(2),
                maxi,
                true,
            )) {
                return element_parse(TextParsingElementUtil::parse_standalone_element(
                    buffer,
                    content_offset,
                    content_len,
                    line,
                    col.wrapping_add(2),
                    self.chained.get_next(),
                ));
            }

            // `is_comment_processable` 已用同一组纯谓词确认：非 minimized 的开放
            // 元素必定以 `]` 结束。仍执行 Java 的第二次谓词调用以保留异常顺序，
            // 随后直接进入开放元素解析，避免为逻辑不可达的 false 分支制造状态。
            let _ = element_bool(TextParsingElementUtil::is_element_end(
                buffer.as_deref(),
                maxi.wrapping_sub(1),
                maxi,
                false,
            ));
            return element_parse(TextParsingElementUtil::parse_open_element(
                buffer,
                content_offset,
                content_len,
                line,
                col.wrapping_add(2),
                self.chained.get_next(),
            ));
        } else if element_bool(TextParsingElementUtil::is_close_element_start(
            buffer.as_deref(),
            content_offset,
            maxi,
        )) && element_bool(TextParsingElementUtil::is_element_end(
            buffer.as_deref(),
            maxi.wrapping_sub(1),
            maxi,
            false,
        )) {
            return element_parse(TextParsingElementUtil::parse_close_element(
                buffer,
                content_offset,
                content_len,
                line,
                col.wrapping_add(2),
                self.chained.get_next(),
            ));
        }

        if self.standard_dialect_present {
            self.direct_next_text(
                buffer.as_deref_mut(),
                content_offset,
                content_len,
                line,
                col.wrapping_add(2),
            )?;
            self.filter_texts = true;
        } else {
            self.direct_next_text(buffer, outer_offset, outer_len, line, col)?;
        }
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
        self.process_filtered_texts()?;
        self.chained.handle_standalone_element_start(
            buffer,
            name_offset,
            name_len,
            minimized,
            line,
            col,
        )
    }

    fn handle_standalone_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        minimized: bool,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.chained.handle_standalone_element_end(
            buffer,
            name_offset,
            name_len,
            minimized,
            line,
            col,
        )
    }

    fn handle_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.process_filtered_texts()?;
        self.chained
            .handle_open_element_start(buffer, name_offset, name_len, line, col)
    }

    fn handle_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.chained
            .handle_open_element_end(buffer, name_offset, name_len, line, col)
    }

    fn handle_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.process_filtered_texts()?;
        self.chained
            .handle_close_element_start(buffer, name_offset, name_len, line, col)
    }

    fn handle_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.chained
            .handle_close_element_end(buffer, name_offset, name_len, line, col)
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
        self.chained.handle_attribute(
            buffer,
            name_offset,
            name_len,
            name_line,
            name_col,
            operator_offset,
            operator_len,
            operator_line,
            operator_col,
            value_content_offset,
            value_content_len,
            value_outer_offset,
            value_outer_len,
            value_line,
            value_col,
        )
    }
}

fn compute_filter_offset(
    buffer: Option<&[u16]>,
    offset: i32,
    maxi: i32,
    mut locator: Option<&mut [i32]>,
) -> i32 {
    if offset == maxi {
        return 0;
    }

    let mut literal_delimiter = 0_u16;
    let mut array_level = 0_i32;
    let mut object_level = 0_i32;
    let mut index = offset;

    while index < maxi {
        let character = array_unit(buffer, index);
        index = index.wrapping_add(1);

        if literal_delimiter != 0 {
            if character == literal_delimiter
                && array_unit(buffer, index.wrapping_sub(2)) != u16::from(b'\\')
            {
                literal_delimiter = 0;
            }
            count_locator(locator.as_deref_mut(), character);
            continue;
        }

        if character == u16::from(b'\'') || character == u16::from(b'"') {
            literal_delimiter = character;
            count_locator(locator.as_deref_mut(), character);
            continue;
        }

        if character == u16::from(b'{') {
            object_level = object_level.wrapping_add(1);
            count_locator(locator.as_deref_mut(), character);
            continue;
        } else if object_level > 0 && character == u16::from(b'}') {
            object_level = object_level.wrapping_sub(1);
            count_locator(locator.as_deref_mut(), character);
            continue;
        } else if character == u16::from(b'[') {
            array_level = array_level.wrapping_add(1);
            count_locator(locator.as_deref_mut(), character);
            continue;
        } else if array_level > 0 && character == u16::from(b']') {
            array_level = array_level.wrapping_sub(1);
            count_locator(locator.as_deref_mut(), character);
            continue;
        }

        if array_level == 0 && object_level == 0 {
            if character == u16::from(b'\n') {
                return index.wrapping_sub(1);
            }
            if matches!(character, 0x003B | 0x002C | 0x0029 | 0x007D | 0x005D) {
                return index.wrapping_sub(1);
            }
            if character == u16::from(b'/')
                && index < maxi
                && array_unit(buffer, index) == u16::from(b'/')
            {
                return index.wrapping_sub(1);
            }
        }

        count_locator(locator.as_deref_mut(), character);
    }

    maxi
}

fn element_bool(result: Result<bool, TextParsingElementError>) -> bool {
    match result {
        Ok(value) => value,
        Err(error) => panic_runtime(CommentProcessorTextHandlerRuntimeError::Element(error)),
    }
}

fn element_parse(
    result: Result<(), TextParsingElementError>,
) -> Result<(), Box<TextParseException>> {
    match result {
        Ok(()) => Ok(()),
        Err(TextParsingElementError::TextParse(exception)) => Err(exception),
        Err(error) => panic_runtime(CommentProcessorTextHandlerRuntimeError::Element(error)),
    }
}

fn count_locator(locator: Option<&mut [i32]>, character: u16) {
    if let Err(error) = ParsingLocatorUtil::count_char(locator, character) {
        panic_runtime(CommentProcessorTextHandlerRuntimeError::Locator(error));
    }
}

fn array_unit(buffer: Option<&[u16]>, index: i32) -> u16 {
    let Some(buffer) = buffer else {
        panic_runtime(CommentProcessorTextHandlerRuntimeError::NullCharArrayLoad);
    };
    if index < 0 || index as usize >= buffer.len() {
        panic_runtime(CommentProcessorTextHandlerRuntimeError::ArrayIndex {
            index,
            length: buffer.len(),
        });
    }
    buffer[index as usize]
}

fn java_arraycopy(
    source: Option<&[u16]>,
    source_offset: i32,
    destination: &mut [u16],
    destination_offset: i32,
    len: i32,
) {
    let Some(source) = source else {
        panic_runtime(CommentProcessorTextHandlerRuntimeError::arraycopy_null_source());
    };
    if len < 0 {
        panic_runtime(CommentProcessorTextHandlerRuntimeError::arraycopy_negative_length(len));
    }
    if source_offset < 0 {
        panic_runtime(
            CommentProcessorTextHandlerRuntimeError::arraycopy_source_index(
                source_offset,
                source.len(),
            ),
        );
    }
    if destination_offset < 0 {
        panic_runtime(
            CommentProcessorTextHandlerRuntimeError::arraycopy_destination_index(
                destination_offset,
                destination.len(),
            ),
        );
    }
    let source_end = i64::from(source_offset) + i64::from(len);
    if source_end > source.len() as i64 {
        panic_runtime(
            CommentProcessorTextHandlerRuntimeError::arraycopy_last_source(
                source_end,
                source.len(),
            ),
        );
    }
    let destination_end = i64::from(destination_offset) + i64::from(len);
    if destination_end > destination.len() as i64 {
        panic_runtime(
            CommentProcessorTextHandlerRuntimeError::arraycopy_last_destination(
                destination_end,
                destination.len(),
            ),
        );
    }
    destination[destination_offset as usize..destination_end as usize]
        .copy_from_slice(&source[source_offset as usize..source_end as usize]);
}

fn panic_runtime(error: CommentProcessorTextHandlerRuntimeError) -> ! {
    panic_any(error)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fmt::{Display, Write};
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::rc::Rc;

    use super::{
        CommentProcessorTextHandler, CommentProcessorTextHandlerRuntimeError, ITextHandler,
        TextParseException, compute_filter_offset,
    };
    use crate::text::ChainedTextHandlerRuntimeError;
    use crate::util::Utf16String;

    const JAVA_GOLDEN: &str =
        include_str!("../../tests/fixtures/comment_processor_text_handler_golden.txt");

    #[derive(Default)]
    struct RecordingState {
        events: String,
        fail_event: Option<&'static str>,
        mutate_first_unit: bool,
    }

    struct RecordingHandler {
        state: Rc<RefCell<RecordingState>>,
    }

    impl RecordingHandler {
        fn record(
            &self,
            event: &'static str,
            buffer: Option<&mut [u16]>,
            offset: i32,
            len: i32,
            arguments: String,
        ) -> Result<(), Box<TextParseException>> {
            let mut state = self.state.borrow_mut();
            if !state.events.is_empty() {
                state.events.push('|');
            }
            write!(state.events, "{event}({arguments})@").unwrap();
            match buffer.as_deref() {
                None => state.events.push_str("null"),
                Some(value)
                    if offset >= 0
                        && len >= 0
                        && i64::from(offset) + i64::from(len) <= value.len() as i64 =>
                {
                    state
                        .events
                        .push_str(&hex(&value[offset as usize..(offset + len) as usize]));
                }
                Some(_) => write!(state.events, "range({offset},{len})").unwrap(),
            }
            if state.mutate_first_unit && len > 0 {
                buffer.expect("mutable event buffer")[offset as usize] = u16::from(b'!');
            }
            if state.fail_event == Some(event) {
                return Err(Box::new(TextParseException::with_message_at(
                    Some(&Utf16String::from_rust_str(&format!("downstream-{event}"))),
                    71,
                    72,
                )));
            }
            Ok(())
        }
    }

    impl ITextHandler for RecordingHandler {
        fn handle_document_start(
            &mut self,
            start_time_nanos: i64,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                "documentStart",
                None,
                0,
                0,
                format!("{start_time_nanos},{line},{col}"),
            )
        }

        fn handle_document_end(
            &mut self,
            end_time_nanos: i64,
            total_time_nanos: i64,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                "documentEnd",
                None,
                0,
                0,
                format!("{end_time_nanos},{total_time_nanos},{line},{col}"),
            )
        }

        fn handle_text(
            &mut self,
            buffer: Option<&mut [u16]>,
            offset: i32,
            len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                "text",
                buffer,
                offset,
                len,
                format!("{offset},{len},{line},{col}"),
            )
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
            self.record(
                "comment",
                buffer,
                outer_offset,
                outer_len,
                format!("{content_offset},{content_len},{outer_offset},{outer_len},{line},{col}"),
            )
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
            self.record(
                "standaloneStart",
                buffer,
                name_offset,
                name_len,
                format!("{name_offset},{name_len},{minimized},{line},{col}"),
            )
        }

        fn handle_standalone_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            minimized: bool,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                "standaloneEnd",
                buffer,
                name_offset,
                name_len,
                format!("{name_offset},{name_len},{minimized},{line},{col}"),
            )
        }

        fn handle_open_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                "openStart",
                buffer,
                name_offset,
                name_len,
                format!("{name_offset},{name_len},{line},{col}"),
            )
        }

        fn handle_open_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                "openEnd",
                buffer,
                name_offset,
                name_len,
                format!("{name_offset},{name_len},{line},{col}"),
            )
        }

        fn handle_close_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                "closeStart",
                buffer,
                name_offset,
                name_len,
                format!("{name_offset},{name_len},{line},{col}"),
            )
        }

        fn handle_close_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                "closeEnd",
                buffer,
                name_offset,
                name_len,
                format!("{name_offset},{name_len},{line},{col}"),
            )
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
            self.record(
                "attribute",
                buffer,
                name_offset,
                name_len,
                format!(
                    "{name_offset},{name_len},{name_line},{name_col},{operator_offset},{operator_len},{operator_line},{operator_col},{value_content_offset},{value_content_len},{value_outer_offset},{value_outer_len},{value_line},{value_col}"
                ),
            )
        }
    }

    fn handler(
        standard_dialect_present: bool,
    ) -> (CommentProcessorTextHandler, Rc<RefCell<RecordingState>>) {
        let state = Rc::new(RefCell::new(RecordingState::default()));
        let next = RecordingHandler {
            state: Rc::clone(&state),
        };
        (
            CommentProcessorTextHandler::new(standard_dialect_present, Some(Box::new(next))),
            state,
        )
    }

    fn java_chars(value: &str) -> Vec<u16> {
        value.encode_utf16().collect()
    }

    fn comment(
        handler: &mut CommentProcessorTextHandler,
        content: &str,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        let mut buffer = java_chars(&format!("/*{content}*/"));
        let outer_len = buffer.len() as i32;
        let content_len = java_chars(content).len() as i32;
        handler.handle_comment(Some(&mut buffer), 2, content_len, 0, outer_len, line, col)
    }

    fn state(handler: &CommentProcessorTextHandler) -> String {
        let buffer = handler.filtered_text_buffer.as_ref().map_or_else(
            || "null".to_owned(),
            |buffer| {
                format!(
                    "{}:{}",
                    buffer.len(),
                    hex(&buffer[..(handler.filtered_text_size.max(0) as usize).min(12)])
                )
            },
        );
        let locator = handler.filtered_text_locator.map_or_else(
            || "null".to_owned(),
            |locator| format!("[{}, {}]", locator[0], locator[1]),
        );
        format!(
            "standard={};filter={};size={};buffer={buffer};locator={locator}",
            handler.standard_dialect_present, handler.filter_texts, handler.filtered_text_size
        )
    }

    fn hex(value: &[u16]) -> String {
        value
            .iter()
            .map(|unit| format!("{unit:04x}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn throwable(operation: &mut dyn FnMut() -> Result<(), Box<TextParseException>>) -> String {
        match catch_unwind(AssertUnwindSafe(operation)) {
            Ok(Ok(())) => "NO_ERROR".to_owned(),
            Ok(Err(error)) => format!(
                "org.thymeleaf.templateparser.text.TextParseException;message={};line={};col={}",
                error
                    .get_message()
                    .map_or_else(|| "null".to_owned(), |message| hex(message.as_utf16())),
                error
                    .get_line()
                    .map_or_else(|| "null".to_owned(), |line| line.to_string()),
                error
                    .get_col()
                    .map_or_else(|| "null".to_owned(), |col| col.to_string())
            ),
            Err(payload) => describe_panic(payload),
        }
    }

    fn describe_panic(payload: Box<dyn std::any::Any + Send>) -> String {
        let payload = match payload.downcast::<CommentProcessorTextHandlerRuntimeError>() {
            Ok(error) => {
                return format!(
                    "{};message={}",
                    error.java_class_name(),
                    error
                        .java_message()
                        .map_or_else(|| "null".to_owned(), |message| hex(message.as_utf16()))
                );
            }
            Err(payload) => payload,
        };
        match payload.downcast::<ChainedTextHandlerRuntimeError>() {
            Ok(error) => format!(
                "{};message={}",
                error.java_class_name(),
                hex(error.java_message().as_utf16())
            ),
            Err(_) => panic!("unknown panic payload"),
        }
    }

    fn emit(output: &mut String, key: &str, value: impl Display) {
        writeln!(output, "{key}={value}").unwrap();
    }

    fn fire_trigger(
        handler: &mut CommentProcessorTextHandler,
        trigger: &str,
    ) -> Result<(), Box<TextParseException>> {
        let mut name = java_chars("x");
        match trigger {
            "documentEnd" => handler.handle_document_end(1, 2, 3, 4),
            "comment" => comment(handler, "normal", 30, 31),
            "standaloneStart" => {
                handler.handle_standalone_element_start(Some(&mut name), 0, 1, true, 30, 31)
            }
            "openStart" => handler.handle_open_element_start(Some(&mut name), 0, 1, 30, 31),
            "closeStart" => handler.handle_close_element_start(Some(&mut name), 0, 1, 30, 31),
            _ => panic!("unknown trigger"),
        }
    }

    fn emit_compute(
        output: &mut String,
        key: &str,
        buffer: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        mut locator: Option<&mut [i32]>,
    ) {
        let result = catch_unwind(AssertUnwindSafe(|| {
            compute_filter_offset(buffer, offset, maxi, locator.as_deref_mut())
        }));
        match result {
            Ok(filter_offset) => emit(
                output,
                key,
                format!(
                    "offset={filter_offset};locator={}",
                    describe_locator(locator.as_deref())
                ),
            ),
            Err(payload) => emit(
                output,
                key,
                format!(
                    "{};locator={}",
                    describe_panic(payload),
                    describe_locator(locator.as_deref())
                ),
            ),
        }
    }

    fn describe_locator(locator: Option<&[i32]>) -> String {
        locator.map_or_else(
            || "null".to_owned(),
            |locator| {
                format!(
                    "[{}]",
                    locator
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
        )
    }

    fn generate_golden() -> String {
        let mut output = String::new();
        emit(
            &mut output,
            "baseline",
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add",
        );

        let (mut inherited, inherited_state) = handler(true);
        emit(&mut output, "initial.state", state(&inherited));
        inherited.handle_document_start(1, 2, 3).unwrap();
        let mut plain = java_chars("plain");
        inherited.handle_text(Some(&mut plain), 0, 5, 4, 5).unwrap();
        for (kind, value, line, col) in [
            ("standaloneEnd", "s", 6, 7),
            ("openEnd", "o", 8, 9),
            ("closeEnd", "c", 10, 11),
        ] {
            let mut chars = java_chars(value);
            match kind {
                "standaloneEnd" => inherited
                    .handle_standalone_element_end(Some(&mut chars), 0, 1, true, line, col)
                    .unwrap(),
                "openEnd" => inherited
                    .handle_open_element_end(Some(&mut chars), 0, 1, line, col)
                    .unwrap(),
                _ => inherited
                    .handle_close_element_end(Some(&mut chars), 0, 1, line, col)
                    .unwrap(),
            }
        }
        let mut attribute = java_chars("a");
        inherited
            .handle_attribute(
                Some(&mut attribute),
                0,
                1,
                12,
                13,
                -1,
                0,
                12,
                14,
                -1,
                0,
                -1,
                0,
                12,
                15,
            )
            .unwrap();
        emit(
            &mut output,
            "inherited.events",
            inherited_state.borrow().events.clone(),
        );
        emit(&mut output, "inherited.state", state(&inherited));

        let comments = ["x", "[x]", "[#x", "[?x]", "[#x]tail", ""];
        for standard in [false, true] {
            let (mut ordinary, ordinary_state) = handler(standard);
            for content in comments {
                comment(&mut ordinary, content, 20, 30).unwrap();
            }
            emit(
                &mut output,
                &format!("ordinary.{standard}.events"),
                ordinary_state.borrow().events.clone(),
            );
            emit(
                &mut output,
                &format!("ordinary.{standard}.state"),
                state(&ordinary),
            );
        }
        let (mut short, short_state) = handler(true);
        short
            .handle_comment(None, i32::MIN, 2, -7, 9, 1, 2)
            .unwrap();
        emit(
            &mut output,
            "ordinary.shortNull.events",
            short_state.borrow().events.clone(),
        );

        let (mut elements, element_state) = handler(true);
        for (content, line, col) in [
            ("[#root]", 7, 9),
            ("[#single/]", 11, 13),
            ("[/root]", 17, 19),
            ("[#item a='b']", 23, 29),
        ] {
            comment(&mut elements, content, line, col).unwrap();
        }
        emit(
            &mut output,
            "element.events",
            element_state.borrow().events.clone(),
        );
        emit(&mut output, "element.state", state(&elements));

        let (mut mutation, mutation_state) = handler(true);
        mutation_state.borrow_mut().mutate_first_unit = true;
        comment(&mut mutation, "[#x]", 1, 1).unwrap();
        emit(
            &mut output,
            "element.mutation.events",
            mutation_state.borrow().events.clone(),
        );

        let (mut failed_element, failed_element_state) = handler(true);
        failed_element_state.borrow_mut().fail_event = Some("openStart");
        emit(
            &mut output,
            "element.checked",
            throwable(&mut || comment(&mut failed_element, "[#x]", 31, 32)),
        );
        emit(&mut output, "element.checked.state", state(&failed_element));

        for standard in [false, true] {
            let (mut expressions, expression_state) = handler(standard);
            comment(&mut expressions, "[(${x})]", 41, i32::MAX).unwrap();
            comment(&mut expressions, "[[${y}]]", 43, 44).unwrap();
            emit(
                &mut output,
                &format!("expression.{standard}.events"),
                expression_state.borrow().events.clone(),
            );
            emit(
                &mut output,
                &format!("expression.{standard}.state"),
                state(&expressions),
            );
        }
        let (mut failed_expression, failed_expression_state) = handler(true);
        failed_expression_state.borrow_mut().fail_event = Some("text");
        emit(
            &mut output,
            "expression.checked",
            throwable(&mut || comment(&mut failed_expression, "[(${x})]", 45, 46)),
        );
        emit(
            &mut output,
            "expression.checked.state",
            state(&failed_expression),
        );

        let texts = [
            "abc;rest",
            "abc,rest",
            "abc)rest",
            "abc}rest",
            "abc]rest",
            "abc\nrest",
            "abc//rest",
            "{a;b};rest",
            "[a,b],rest",
            "'a;b';rest",
            "\"a,b\",rest",
            "'a\\';b';rest",
            "all-filtered",
        ];
        for (index, text) in texts.iter().enumerate() {
            let (mut delimiter, delimiter_state) = handler(true);
            comment(&mut delimiter, "[(${x})]", 1, 2).unwrap();
            let mut chars = java_chars(text);
            let len = chars.len() as i32;
            delimiter
                .handle_text(
                    Some(&mut chars),
                    0,
                    len,
                    50 + index as i32,
                    60 + index as i32,
                )
                .unwrap();
            delimiter.handle_document_end(7, 8, 70, 80).unwrap();
            emit(
                &mut output,
                &format!("delimiter.{index}.events"),
                delimiter_state.borrow().events.clone(),
            );
            emit(
                &mut output,
                &format!("delimiter.{index}.state"),
                state(&delimiter),
            );
        }

        for trigger in [
            "documentEnd",
            "comment",
            "standaloneStart",
            "openStart",
            "closeStart",
        ] {
            let (mut flush, flush_state) = handler(true);
            comment(&mut flush, "[(${x})]", 1, 2).unwrap();
            let mut text = java_chars("abc;rest");
            flush.handle_text(Some(&mut text), 0, 8, 10, 20).unwrap();
            fire_trigger(&mut flush, trigger).unwrap();
            emit(
                &mut output,
                &format!("flush.{trigger}.events"),
                flush_state.borrow().events.clone(),
            );
            emit(
                &mut output,
                &format!("flush.{trigger}.state"),
                state(&flush),
            );
        }

        let (mut delayed, delayed_state) = handler(true);
        comment(&mut delayed, "[(${x})]", 1, 2).unwrap();
        let mut text = java_chars("abc;rest");
        delayed.handle_text(Some(&mut text), 0, 8, 10, 20).unwrap();
        let mut x = java_chars("x");
        delayed
            .handle_open_element_end(Some(&mut x), 0, 1, 30, 31)
            .unwrap();
        let mut a = java_chars("a");
        delayed
            .handle_attribute(
                Some(&mut a),
                0,
                1,
                32,
                33,
                -1,
                0,
                32,
                34,
                -1,
                0,
                -1,
                0,
                32,
                35,
            )
            .unwrap();
        emit(
            &mut output,
            "flush.nonTriggers.before",
            delayed_state.borrow().events.clone(),
        );
        emit(&mut output, "flush.nonTriggers.state", state(&delayed));
        delayed.handle_document_end(1, 2, 3, 4).unwrap();
        emit(
            &mut output,
            "flush.nonTriggers.after",
            delayed_state.borrow().events.clone(),
        );

        let (mut empty, empty_state) = handler(true);
        comment(&mut empty, "[(${x})]", 1, 2).unwrap();
        empty.handle_text(Some(&mut []), 0, 0, 90, 91).unwrap();
        empty.handle_document_end(1, 2, 3, 4).unwrap();
        emit(
            &mut output,
            "flush.empty.events",
            empty_state.borrow().events.clone(),
        );
        emit(&mut output, "flush.empty.state", state(&empty));

        let (mut chunks, chunks_state) = handler(true);
        comment(&mut chunks, "[(${x})]", 1, 2).unwrap();
        let mut first = java_chars("ab");
        chunks
            .handle_text(Some(&mut first), 0, 2, 100, 101)
            .unwrap();
        let mut second = java_chars("c;rest");
        chunks
            .handle_text(Some(&mut second), 0, 6, 200, 201)
            .unwrap();
        chunks.handle_document_end(1, 2, 3, 4).unwrap();
        emit(
            &mut output,
            "flush.chunks.events",
            chunks_state.borrow().events.clone(),
        );
        emit(&mut output, "flush.chunks.state", state(&chunks));

        let (mut failure, failure_state) = handler(true);
        comment(&mut failure, "[(${x})]", 1, 2).unwrap();
        let mut failure_text = java_chars("abc;rest");
        failure
            .handle_text(Some(&mut failure_text), 0, 8, 10, 20)
            .unwrap();
        failure_state.borrow_mut().fail_event = Some("text");
        emit(
            &mut output,
            "failure.flush1",
            throwable(&mut || failure.handle_document_end(1, 2, 3, 4)),
        );
        emit(&mut output, "failure.flush1.state", state(&failure));
        emit(
            &mut output,
            "failure.flush2",
            throwable(&mut || failure.handle_document_end(1, 2, 3, 4)),
        );
        emit(&mut output, "failure.flush2.state", state(&failure));
        failure_state.borrow_mut().fail_event = None;
        failure.handle_document_end(1, 2, 3, 4).unwrap();
        emit(
            &mut output,
            "failure.recovered.events",
            failure_state.borrow().events.clone(),
        );
        emit(&mut output, "failure.recovered.state", state(&failure));

        let (mut after_flush, after_flush_state) = handler(true);
        comment(&mut after_flush, "[(${x})]", 1, 2).unwrap();
        let mut after_text = java_chars("abc;rest");
        after_flush
            .handle_text(Some(&mut after_text), 0, 8, 10, 20)
            .unwrap();
        after_flush_state.borrow_mut().fail_event = Some("openStart");
        let mut x = java_chars("x");
        emit(
            &mut output,
            "failure.afterFlush",
            throwable(&mut || after_flush.handle_open_element_start(Some(&mut x), 0, 1, 5, 6)),
        );
        emit(&mut output, "failure.afterFlush.state", state(&after_flush));

        let (mut growth, _) = handler(true);
        comment(&mut growth, "[(${x})]", 1, 2).unwrap();
        let mut first = vec![u16::from(b'a'); 200];
        growth.handle_text(Some(&mut first), 0, 200, 1, 2).unwrap();
        first[0] = u16::from(b'z');
        emit(&mut output, "growth.200", state(&growth));
        let mut second = vec![u16::from(b'b'); 100];
        growth.handle_text(Some(&mut second), 0, 100, 3, 4).unwrap();
        emit(&mut output, "growth.300", state(&growth));
        let mut third = vec![u16::from(b'c'); 500];
        growth.handle_text(Some(&mut third), 0, 500, 5, 6).unwrap();
        emit(&mut output, "growth.800", state(&growth));
        emit(
            &mut output,
            "growth.copyHead",
            hex(&growth.filtered_text_buffer.as_ref().unwrap()[..3]),
        );

        let compute_values = [
            "",
            "abc",
            ";x",
            "\nx",
            "//x",
            "{a;b};x",
            "[a,b],x",
            "'a;b';x",
            "\"a,b\",x",
            "'a\\';b';x",
            "{{a}b}c]x",
            "[[a]b]c)x",
        ];
        for (index, value) in compute_values.iter().enumerate() {
            let chars = java_chars(value);
            let mut locator = [10, 20];
            emit_compute(
                &mut output,
                &format!("compute.{index}"),
                Some(&chars),
                0,
                chars.len() as i32,
                Some(&mut locator),
            );
        }
        emit_compute(&mut output, "compute.emptyNull", None, 7, 7, None);
        emit_compute(&mut output, "compute.reverseNull", None, 8, 7, None);

        let mut destination = [0_u16];
        emit(
            &mut output,
            "arraycopy.nullSource",
            throwable(&mut || {
                super::java_arraycopy(None, 0, &mut destination, 0, 1);
                Ok(())
            }),
        );
        let source = [0_u16];
        emit(
            &mut output,
            "arraycopy.negativeLen",
            throwable(&mut || {
                super::java_arraycopy(Some(&source), 0, &mut destination, 0, -1);
                Ok(())
            }),
        );
        emit(
            &mut output,
            "arraycopy.sourceIndex",
            throwable(&mut || {
                super::java_arraycopy(Some(&source), -1, &mut destination, 0, 1);
                Ok(())
            }),
        );
        emit(
            &mut output,
            "arraycopy.destinationIndex",
            throwable(&mut || {
                super::java_arraycopy(Some(&source), 0, &mut destination, -1, 1);
                Ok(())
            }),
        );
        let mut wide_destination = [0_u16; 2];
        emit(
            &mut output,
            "arraycopy.lastSource",
            throwable(&mut || {
                super::java_arraycopy(Some(&source), 0, &mut wide_destination, 0, 2);
                Ok(())
            }),
        );
        let wide_source = [0_u16; 2];
        emit(
            &mut output,
            "arraycopy.lastDestination",
            throwable(&mut || {
                super::java_arraycopy(Some(&wide_source), 0, &mut destination, 0, 2);
                Ok(())
            }),
        );

        let (mut predicate, _) = handler(true);
        emit(
            &mut output,
            "invalid.comment.null",
            throwable(&mut || predicate.handle_comment(None, 0, 3, 0, 0, 1, 1)),
        );
        let mut abc = java_chars("abc");
        emit(
            &mut output,
            "invalid.comment.negativeOffset",
            throwable(&mut || predicate.handle_comment(Some(&mut abc), -1, 3, 0, 3, 1, 1)),
        );
        emit(
            &mut output,
            "invalid.comment.overflow",
            throwable(&mut || predicate.handle_comment(Some(&mut abc), i32::MAX, 3, 0, 3, 1, 1)),
        );

        let (mut null_filter, _) = handler(true);
        comment(&mut null_filter, "[(${x})]", 1, 2).unwrap();
        emit(
            &mut output,
            "invalid.filter.null",
            throwable(&mut || null_filter.handle_text(None, 0, 1, 1, 1)),
        );
        emit(
            &mut output,
            "invalid.filter.null.state",
            state(&null_filter),
        );

        let (mut negative_filter, _) = handler(true);
        comment(&mut negative_filter, "[(${x})]", 1, 2).unwrap();
        let mut a = java_chars("a");
        emit(
            &mut output,
            "invalid.filter.negativeLen",
            throwable(&mut || negative_filter.handle_text(Some(&mut a), 0, -1, 1, 1)),
        );
        emit(
            &mut output,
            "invalid.filter.negativeLen.state",
            state(&negative_filter),
        );

        let mut locator = [1, 2];
        emit_compute(
            &mut output,
            "invalid.compute.nullBuffer",
            None,
            0,
            1,
            Some(&mut locator),
        );
        let chars = java_chars("a");
        let mut locator = [1, 2];
        emit_compute(
            &mut output,
            "invalid.compute.negativeOffset",
            Some(&chars),
            -1,
            1,
            Some(&mut locator),
        );
        let terminator = java_chars(";");
        emit_compute(
            &mut output,
            "invalid.compute.nullLocatorTerminator",
            Some(&terminator),
            0,
            1,
            None,
        );
        emit_compute(
            &mut output,
            "invalid.compute.nullLocatorText",
            Some(&chars),
            0,
            1,
            None,
        );
        let newline = java_chars("\n");
        let mut short_locator = [i32::MAX];
        emit_compute(
            &mut output,
            "invalid.compute.shortLocatorLf",
            Some(&newline),
            0,
            1,
            Some(&mut short_locator),
        );

        let mut null_expression = CommentProcessorTextHandler::new(true, None);
        emit(
            &mut output,
            "invalid.next.expression",
            throwable(&mut || comment(&mut null_expression, "[(${x})]", 1, 2)),
        );
        let mut null_normal = CommentProcessorTextHandler::new(true, None);
        emit(
            &mut output,
            "invalid.next.normal",
            throwable(&mut || comment(&mut null_normal, "normal", 1, 2)),
        );
        let mut null_element = CommentProcessorTextHandler::new(true, None);
        emit(
            &mut output,
            "invalid.next.element",
            throwable(&mut || comment(&mut null_element, "[#x]", 1, 2)),
        );

        output
    }

    #[test]
    fn java_golden_matches_comment_unwrapping_filtering_and_failure_state() {
        assert_eq!(generate_golden(), JAVA_GOLDEN);
    }

    #[test]
    fn internal_error_and_harness_defensive_branches_are_observable() {
        use std::error::Error;

        use super::{ParsingLocatorError, TextParsingElementError};

        let with_message = CommentProcessorTextHandlerRuntimeError::arraycopy_negative_length(-1);
        assert_eq!(with_message.to_string(), "arraycopy: length -1 is negative");
        assert!(with_message.source().is_none());

        let without_message = CommentProcessorTextHandlerRuntimeError::arraycopy_null_source();
        assert_eq!(without_message.to_string(), "null");

        let locator =
            CommentProcessorTextHandlerRuntimeError::Locator(ParsingLocatorError::NullLocator);
        assert!(locator.source().is_some());
        let element = CommentProcessorTextHandlerRuntimeError::Element(
            TextParsingElementError::NullArrayLoad,
        );
        assert!(element.source().is_some());

        let element_panic = catch_unwind(AssertUnwindSafe(|| {
            super::element_bool(Err(TextParsingElementError::NullArrayLoad));
        }))
        .expect_err("元素谓词错误应按 Java 运行时异常传播");
        let element_error = element_panic
            .downcast::<CommentProcessorTextHandlerRuntimeError>()
            .expect("panic payload 应为注释处理器运行时异常");
        assert_eq!(
            element_error.java_class_name(),
            "java.lang.NullPointerException"
        );

        let state = Rc::new(RefCell::new(RecordingState::default()));
        let mut recording = RecordingHandler {
            state: Rc::clone(&state),
        };
        let mut comment_buffer = java_chars("/*x*/");
        recording
            .handle_comment(Some(&mut comment_buffer), 2, 1, 0, 5, 1, 2)
            .unwrap();
        let mut invalid_range = java_chars("x");
        recording
            .record("range", Some(&mut invalid_range), -1, 2, String::new())
            .unwrap();
        assert!(state.borrow().events.contains("comment("));
        assert!(state.borrow().events.contains("range(-1,2)"));

        assert_eq!(throwable(&mut || Ok(())), "NO_ERROR");
        assert_eq!(
            throwable(&mut || Err(Box::new(TextParseException::new()))),
            "org.thymeleaf.templateparser.text.TextParseException;message=null;line=null;col=null"
        );
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                describe_panic(Box::new("unknown"));
            }))
            .is_err()
        );

        let (mut unknown_handler, _) = handler(true);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let _ = fire_trigger(&mut unknown_handler, "unknown");
            }))
            .is_err()
        );

        let (mut filtered, filtered_state) = handler(true);
        comment(&mut filtered, "[(${x})]", 1, 2).unwrap();
        let mut remainder = java_chars(";tail");
        filtered
            .handle_text(Some(&mut remainder), 0, 5, 3, 4)
            .unwrap();
        filtered_state.borrow_mut().fail_event = Some("text");
        assert!(comment(&mut filtered, "normal", 5, 6).is_err());
        let mut name = java_chars("x");
        assert!(
            filtered
                .handle_standalone_element_start(Some(&mut name), 0, 1, true, 5, 6)
                .is_err()
        );
        assert!(
            filtered
                .handle_open_element_start(Some(&mut name), 0, 1, 5, 6)
                .is_err()
        );
        assert!(
            filtered
                .handle_close_element_start(Some(&mut name), 0, 1, 5, 6)
                .is_err()
        );

        let (mut nonstandard, nonstandard_state) = handler(false);
        nonstandard_state.borrow_mut().fail_event = Some("text");
        assert!(comment(&mut nonstandard, "[(${x})]", 7, 8).is_err());
    }
}
