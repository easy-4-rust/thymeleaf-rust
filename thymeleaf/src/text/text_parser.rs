use std::any::Any;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::panic::{AssertUnwindSafe, catch_unwind, panic_any, resume_unwind};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use super::parsing_locator_util::ParsingLocatorError;
use super::{
    ChainedTextHandlerRuntimeError, CommentProcessorTextHandler,
    CommentProcessorTextHandlerRuntimeError, EventProcessorTextHandler,
    EventProcessorTextHandlerRuntimeError, ITextHandler, ParsingLocatorUtil, TextParseCause,
    TextParseException, TextParseStatus, TextParsingCommentError, TextParsingCommentUtil,
    TextParsingElementError, TextParsingElementUtil, TextParsingLiteralUtil, TextParsingUtil,
    TextParsingUtilError,
};
use crate::util::JavaString;

const DOCUMENT_NULL_MESSAGE: &str = "Document cannot be null";
const READER_NULL_MESSAGE: &str = "Reader cannot be null";
const HANDLER_NULL_MESSAGE: &str = "Handler cannot be null";
const NULL_PARSE_DOCUMENT_READER_MESSAGE: &str =
    "Cannot invoke \"java.io.Reader.read(char[])\" because \"reader\" is null";
const NULL_PARSE_DOCUMENT_HANDLER_MESSAGE: &str = "Cannot invoke \"org.thymeleaf.templateparser.text.ITextHandler.handleDocumentStart(long, int, int)\" because \"handler\" is null";

/// `TextParser` 直接产生的 Java 未检查异常适配。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.TextParser` 与其内部 `BufferPool`。
///
/// 参数校验在 `parse` 入口外抛；解析体内的运行时异常由 Java 的
/// `catch (Exception)` 包装为 [`TextParseException`]。本类型同时保存数组创建、
/// 数组访问和非法状态的 JVM 类别与 UTF-16 消息，以便两个调用层级采用不同传播
/// 策略而不丢失原因对象。
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TextParserRuntimeError {
    java_class_name: &'static str,
    java_message: Option<JavaString>,
}

impl TextParserRuntimeError {
    fn illegal_argument(message: &'static str) -> Self {
        Self {
            java_class_name: "java.lang.IllegalArgumentException",
            java_message: Some(JavaString::from_rust_str(message)),
        }
    }

    fn negative_array_size(size: i32) -> Self {
        Self {
            java_class_name: "java.lang.NegativeArraySizeException",
            java_message: Some(JavaString::from_rust_str(&size.to_string())),
        }
    }

    fn array_index(index: i32, length: usize) -> Self {
        Self {
            java_class_name: "java.lang.ArrayIndexOutOfBoundsException",
            java_message: Some(JavaString::from_rust_str(&format!(
                "Index {index} out of bounds for length {length}"
            ))),
        }
    }

    fn string_range(offset: i32, len: i32, length: usize) -> Self {
        Self {
            java_class_name: "java.lang.StringIndexOutOfBoundsException",
            java_message: Some(JavaString::from_rust_str(&format!(
                "Range [{offset}, {offset} + {len}) out of bounds for length {length}"
            ))),
        }
    }

    fn arraycopy_negative_length(length: i32) -> Self {
        Self::with_java_metadata(
            "java.lang.ArrayIndexOutOfBoundsException",
            Some(JavaString::from_rust_str(&format!(
                "arraycopy: length {length} is negative"
            ))),
        )
    }

    fn arraycopy_source_index(index: i32, length: usize) -> Self {
        Self::with_java_metadata(
            "java.lang.ArrayIndexOutOfBoundsException",
            Some(JavaString::from_rust_str(&format!(
                "arraycopy: source index {index} out of bounds for char[{length}]"
            ))),
        )
    }

    fn arraycopy_destination_index(index: i32, length: usize) -> Self {
        Self::with_java_metadata(
            "java.lang.ArrayIndexOutOfBoundsException",
            Some(JavaString::from_rust_str(&format!(
                "arraycopy: destination index {index} out of bounds for char[{length}]"
            ))),
        )
    }

    fn arraycopy_last_source(index: i64, length: usize) -> Self {
        Self::with_java_metadata(
            "java.lang.ArrayIndexOutOfBoundsException",
            Some(JavaString::from_rust_str(&format!(
                "arraycopy: last source index {index} out of bounds for char[{length}]"
            ))),
        )
    }

    fn arraycopy_last_destination(index: i64, length: usize) -> Self {
        Self::with_java_metadata(
            "java.lang.ArrayIndexOutOfBoundsException",
            Some(JavaString::from_rust_str(&format!(
                "arraycopy: last destination index {index} out of bounds for char[{length}]"
            ))),
        )
    }

    fn null_reader() -> Self {
        Self {
            java_class_name: "java.lang.NullPointerException",
            java_message: Some(JavaString::from_rust_str(
                NULL_PARSE_DOCUMENT_READER_MESSAGE,
            )),
        }
    }

    fn null_handler() -> Self {
        Self {
            java_class_name: "java.lang.NullPointerException",
            java_message: Some(JavaString::from_rust_str(
                NULL_PARSE_DOCUMENT_HANDLER_MESSAGE,
            )),
        }
    }

    /// 创建供 Rust handler/Reader 适配层表达 Java RuntimeException 的错误。
    ///
    /// # 参数
    /// - `java_class_name`：Java 异常全限定名；
    /// - `java_message`：可空 Java UTF-16 消息。
    #[must_use]
    pub(crate) fn with_java_metadata(
        java_class_name: &'static str,
        java_message: Option<JavaString>,
    ) -> Self {
        Self {
            java_class_name,
            java_message,
        }
    }

    /// 返回 Java 异常全限定名。
    #[must_use]
    pub(crate) const fn java_class_name(&self) -> &'static str {
        self.java_class_name
    }

    /// 返回 Java `Throwable#getMessage()`。
    #[must_use]
    pub(crate) fn java_message(&self) -> Option<JavaString> {
        self.java_message.clone()
    }
}

impl Display for TextParserRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            &self
                .java_message
                .as_ref()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
        )
    }
}

impl Error for TextParserRuntimeError {}

/// Java `Reader` 读取失败的元数据适配。
///
/// 对应 Java: `java.io.Reader` 的实现可抛出的任意 `Exception`。解析器会把该异常
/// 包装为 `TextParseException`，同时保留类名、可空消息与 Rust source 身份。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextParserReaderError {
    java_class_name: String,
    java_message: Option<JavaString>,
}

impl TextParserReaderError {
    /// 创建带 Java 元数据的 Reader 失败。
    #[must_use]
    pub fn new(java_class_name: &str, java_message: Option<JavaString>) -> Self {
        Self {
            java_class_name: java_class_name.to_owned(),
            java_message,
        }
    }

    /// 创建 `java.io.IOException`。
    #[must_use]
    pub fn io(message: &str) -> Self {
        Self::new(
            "java.io.IOException",
            Some(JavaString::from_rust_str(message)),
        )
    }

    /// 返回 Java 异常全限定名。
    #[must_use]
    pub fn java_class_name(&self) -> &str {
        &self.java_class_name
    }

    /// 返回可空 Java 消息。
    #[must_use]
    pub fn java_message(&self) -> Option<JavaString> {
        self.java_message.clone()
    }
}

impl Display for TextParserReaderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            &self
                .java_message
                .as_ref()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
        )
    }
}

impl Error for TextParserReaderError {}

/// `java.io.Reader` 的 UTF-16 读取合同。
///
/// 对应 Java: `java.io.Reader`。`TextParser` 按 Java `char[]` 而非 UTF-8 字节
/// 消费输入，因此该 Reader 适配合同直接读取 UTF-16 code unit，并分别保留
/// `read(char[])`、`read(char[],off,len)` 与 `close()` 三个动态调用点。
pub trait TextParserReader {
    /// 对应 `Reader#read(char[])`。
    fn read_buffer(&mut self, buffer: &mut [u16]) -> Result<i32, TextParserReaderError> {
        self.read_range(buffer, 0, buffer.len() as i32)
    }

    /// 对应 `Reader#read(char[],int,int)`。
    fn read_range(
        &mut self,
        buffer: &mut [u16],
        offset: i32,
        len: i32,
    ) -> Result<i32, TextParserReaderError>;

    /// 对应 `Reader#close()`；解析器忽略这里的任何 Throwable。
    fn close(&mut self) -> Result<(), TextParserReaderError> {
        Ok(())
    }
}

/// Java `StringReader` 的 UTF-16 顺序读取实现。
#[derive(Debug)]
struct StringTextParserReader {
    input: Vec<u16>,
    position: usize,
}

impl StringTextParserReader {
    fn new(document: &JavaString) -> Self {
        Self {
            input: document.as_utf16().to_vec(),
            position: 0,
        }
    }
}

impl TextParserReader for StringTextParserReader {
    fn read_range(
        &mut self,
        buffer: &mut [u16],
        offset: i32,
        len: i32,
    ) -> Result<i32, TextParserReaderError> {
        if len == 0 {
            return Ok(0);
        }
        if self.position >= self.input.len() {
            return Ok(-1);
        }
        let copied = (len as usize).min(self.input.len() - self.position);
        let destination = offset as usize;
        buffer[destination..destination + copied]
            .copy_from_slice(&self.input[self.position..self.position + copied]);
        self.position += copied;
        Ok(copied as i32)
    }
}

/// 非阻塞 Java `char[]` 缓冲池。
///
/// 对应 Java: `TextParser.BufferPool`。默认大小的空闲槽按数组顺序复用；池满时
/// 创建不归池的新数组；不同大小永不入池。Rust 在分配时暂时取出槽内 `Vec`，
/// 释放时按原槽放回，等价保留 Java 数组身份和并发占用语义。
#[derive(Debug)]
struct BufferPool {
    state: Mutex<BufferPoolState>,
    pool_buffer_size: i32,
}

#[derive(Debug)]
struct BufferPoolState {
    buffers: Vec<Option<Vec<u16>>>,
}

#[derive(Debug)]
struct AllocatedBuffer {
    buffer: Vec<u16>,
    pool_index: Option<usize>,
}

impl BufferPool {
    fn new(pool_size: i32, pool_buffer_size: i32) -> Self {
        if pool_size < 0 {
            panic_runtime(TextParserRuntimeError::negative_array_size(pool_size));
        }
        let mut buffers = Vec::with_capacity(pool_size as usize);
        for _ in 0..pool_size {
            buffers.push(Some(java_char_array(pool_buffer_size)));
        }
        Self {
            state: Mutex::new(BufferPoolState { buffers }),
            pool_buffer_size,
        }
    }

    #[inline(always)]
    fn allocate_buffer(&self, buffer_size: i32) -> AllocatedBuffer {
        if buffer_size != self.pool_buffer_size {
            return AllocatedBuffer {
                buffer: java_char_array(buffer_size),
                pool_index: None,
            };
        }
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        for (pool_index, slot) in state.buffers.iter_mut().enumerate() {
            if let Some(buffer) = slot.take() {
                return AllocatedBuffer {
                    buffer,
                    pool_index: Some(pool_index),
                };
            }
        }
        AllocatedBuffer {
            buffer: java_char_array(buffer_size),
            pool_index: None,
        }
    }

    #[inline(always)]
    fn release_buffer(&self, allocated: Option<AllocatedBuffer>) {
        let Some(allocated) = allocated else {
            return;
        };
        if allocated.buffer.len() as i32 != self.pool_buffer_size {
            return;
        }
        let Some(pool_index) = allocated.pool_index else {
            return;
        };
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.buffers[pool_index] = Some(allocated.buffer);
    }
}

/// Thymeleaf TEXT/JAVASCRIPT/CSS 模式的流式 UTF-16 解析器。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.TextParser`。
///
/// 本对象按输入缓冲区扫描文本元素、注释、行注释与字符串/正则字面量；未完成结构
/// 会移动到缓冲区开头并在需要时倍增容量。公开解析入口按配置装配
/// `EventProcessorTextHandler` 与 `CommentProcessorTextHandler`，包级
/// `parse_document` 则使用调用方已经装配的 handler。所有路径最终释放缓冲区并
/// 尝试关闭 Reader，关闭失败包括 Java Error 在内均被忽略。
pub struct TextParser {
    pool: BufferPool,
    process_comments_and_literals: bool,
    standard_dialect_present: bool,
}

impl TextParser {
    /// 创建文本解析器。
    ///
    /// 对应 Java:
    /// `TextParser#TextParser(int,int,boolean,boolean)`。
    ///
    /// # 参数
    /// - `pool_size`：默认大小缓冲区槽位数；
    /// - `buffer_size`：默认 UTF-16 缓冲区长度；
    /// - `process_comments_and_literals`：是否识别 JS/CSS 注释和字面量；
    /// - `standard_dialect_present`：注释处理器是否启用标准内联表达式过滤。
    #[must_use]
    pub fn new(
        pool_size: i32,
        buffer_size: i32,
        process_comments_and_literals: bool,
        standard_dialect_present: bool,
    ) -> Self {
        Self {
            pool: BufferPool::new(pool_size, buffer_size),
            process_comments_and_literals,
            standard_dialect_present,
        }
    }

    /// 解析 Java UTF-16 字符串。
    ///
    /// 对应 Java: `TextParser#parse(String,ITextHandler)`。
    ///
    /// document null 在 handler 检查前抛出 `IllegalArgumentException`；非 null
    /// 文档通过独立 `StringReader` 进入 Reader 重载。
    pub fn parse(
        &self,
        document: Option<&JavaString>,
        handler: Option<Box<dyn ITextHandler>>,
    ) -> Result<(), Box<TextParseException>> {
        let Some(document) = document else {
            panic_runtime(TextParserRuntimeError::illegal_argument(
                DOCUMENT_NULL_MESSAGE,
            ));
        };
        self.parse_reader(
            Some(Box::new(StringTextParserReader::new(document))),
            handler,
        )
    }

    /// 解析 UTF-16 Reader。
    ///
    /// 对应 Java: `TextParser#parse(Reader,ITextHandler)`。
    ///
    /// # 参数
    /// reader 与 handler 按 Java 顺序校验；随后始终安装事件结构处理器，并按配置
    /// 选择是否在其外层安装注释处理器。
    pub fn parse_reader(
        &self,
        reader: Option<Box<dyn TextParserReader>>,
        handler: Option<Box<dyn ITextHandler>>,
    ) -> Result<(), Box<TextParseException>> {
        let Some(reader) = reader else {
            panic_runtime(TextParserRuntimeError::illegal_argument(
                READER_NULL_MESSAGE,
            ));
        };
        if handler.is_none() {
            panic_runtime(TextParserRuntimeError::illegal_argument(
                HANDLER_NULL_MESSAGE,
            ));
        }

        let mut handler_chain: Box<dyn ITextHandler> =
            Box::new(EventProcessorTextHandler::new(handler));
        if self.process_comments_and_literals {
            handler_chain = Box::new(CommentProcessorTextHandler::new(
                self.standard_dialect_present,
                Some(handler_chain),
            ));
        }
        self.parse_document(
            Some(reader),
            self.pool.pool_buffer_size,
            Some(handler_chain),
        )
    }

    /// 使用调用方指定缓冲区大小和已装配 handler 解析 Reader。
    ///
    /// 对应 Java: `TextParser#parseDocument(Reader,int,ITextHandler)`。
    ///
    /// 本入口不做 null 参数预校验。解析体的 checked exception 保持原 Box；
    /// Java `Exception` 类 panic 被包装为带 cause 的 `TextParseException`；未知
    /// Rust panic 等价于 Java Error，在 finally 清理后继续传播。
    #[inline(always)]
    pub(crate) fn parse_document(
        &self,
        mut reader: Option<Box<dyn TextParserReader>>,
        suggested_buffer_size: i32,
        mut handler: Option<Box<dyn ITextHandler>>,
    ) -> Result<(), Box<TextParseException>> {
        let mut allocated_buffer = None;
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            self.parse_document_body(
                reader.as_deref_mut(),
                suggested_buffer_size,
                handler.as_deref_mut(),
                &mut allocated_buffer,
            )
        }));

        self.pool.release_buffer(allocated_buffer.take());
        if let Some(reader) = reader.as_deref_mut() {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let _ = reader.close();
            }));
        }

        match outcome {
            Ok(result) => result,
            Err(payload) => match panic_payload_to_cause(payload) {
                Ok(cause) => Err(Box::new(TextParseException::with_cause(Some(cause)))),
                Err(payload) => resume_unwind(payload),
            },
        }
    }

    fn parse_document_body(
        &self,
        reader: Option<&mut (dyn TextParserReader + 'static)>,
        suggested_buffer_size: i32,
        handler: Option<&mut (dyn ITextHandler + 'static)>,
        allocated_buffer: &mut Option<AllocatedBuffer>,
    ) -> Result<(), Box<TextParseException>> {
        let parsing_start_time_nanos = nano_time();
        let Some(handler) = handler else {
            panic_runtime(TextParserRuntimeError::null_handler());
        };

        let mut status = TextParseStatus::new();
        handler.handle_document_start(parsing_start_time_nanos, 1, 1)?;

        let mut buffer_size = suggested_buffer_size;
        *allocated_buffer = Some(self.pool.allocate_buffer(buffer_size));
        let Some(reader) = reader else {
            panic_runtime(TextParserRuntimeError::null_reader());
        };
        let mut buffer_content_size = reader
            .read_buffer(
                &mut allocated_buffer
                    .as_mut()
                    .expect("buffer allocated before first read")
                    .buffer,
            )
            .map_err(reader_error_as_text_parse)?;

        let mut cont = buffer_content_size != -1;
        status.offset = -1;
        status.line = 1;
        status.col = 1;
        status.in_structure = false;
        status.in_comment_line = false;
        status.literal_marker = 0;

        while cont {
            self.parse_buffer(
                &mut allocated_buffer.as_mut().expect("active buffer").buffer,
                0,
                buffer_content_size,
                handler,
                &mut status,
            )?;

            let mut read_offset = 0;
            let mut read_len = buffer_size;

            if status.offset == 0 {
                if buffer_content_size == buffer_size {
                    self.grow_buffer(&mut buffer_size, buffer_content_size, allocated_buffer);
                }
                read_offset = buffer_content_size;
                read_len = buffer_size.wrapping_sub(read_offset);
            } else if status.offset < buffer_content_size {
                let content_to_move = buffer_content_size.wrapping_sub(status.offset);
                let active = &mut allocated_buffer.as_mut().expect("active buffer").buffer;
                java_arraycopy_within(active, status.offset, 0, content_to_move);
                read_offset = content_to_move;
                read_len = buffer_size.wrapping_sub(read_offset);
                status.offset = 0;
                buffer_content_size = read_offset;
            }

            let read = reader
                .read_range(
                    &mut allocated_buffer.as_mut().expect("active buffer").buffer,
                    read_offset,
                    read_len,
                )
                .map_err(reader_error_as_text_parse)?;
            if read != -1 {
                buffer_content_size = read_offset.wrapping_add(read);
            } else {
                cont = false;
            }
        }

        let mut last_line = status.line;
        let mut last_col = status.col;
        let last_start = status.offset;
        let last_len = buffer_content_size.wrapping_sub(last_start);

        if last_len > 0 {
            let active = &mut allocated_buffer.as_mut().expect("active buffer").buffer;
            if status.in_structure && !status.in_comment_line {
                let source = java_string_from_range(active, last_start, last_len);
                let mut message = "Incomplete structure: \""
                    .encode_utf16()
                    .collect::<Vec<_>>();
                message.extend_from_slice(source.as_utf16());
                message.push(u16::from(b'"'));
                return Err(Box::new(TextParseException::with_message_at(
                    Some(&JavaString::from_utf16(message)),
                    status.line,
                    status.col,
                )));
            }

            handler.handle_text(Some(active), last_start, last_len, status.line, status.col)?;
            let maxi = last_start.wrapping_add(last_len);
            let mut index = last_start;
            while index < maxi {
                let character = array_unit(active, index);
                if character == u16::from(b'\n') {
                    last_line = last_line.wrapping_add(1);
                    last_col = 1;
                } else {
                    last_col = last_col.wrapping_add(1);
                }
                index = index.wrapping_add(1);
            }
        }

        let parsing_end_time_nanos = nano_time();
        handler.handle_document_end(
            parsing_end_time_nanos,
            parsing_end_time_nanos.wrapping_sub(parsing_start_time_nanos),
            last_line,
            last_col,
        )
    }

    #[inline(always)]
    fn grow_buffer(
        &self,
        buffer_size: &mut i32,
        buffer_content_size: i32,
        allocated_buffer: &mut Option<AllocatedBuffer>,
    ) {
        // Java 在扩容局部块中捕获 `Exception`：逻辑大小已经按 `int` 回绕翻倍，
        // 但新缓冲分配或复制失败不会立刻结束解析。只有 Rust 未知 panic（等价于
        // Java `Error`）才在释放候选缓冲后继续传播。
        *buffer_size = buffer_size.wrapping_mul(2);
        let mut new_buffer = None;
        let growth = catch_unwind(AssertUnwindSafe(|| {
            new_buffer = Some(self.pool.allocate_buffer(*buffer_size));
            java_arraycopy(
                &allocated_buffer.as_ref().expect("old buffer").buffer,
                0,
                &mut new_buffer.as_mut().expect("new buffer").buffer,
                0,
                buffer_content_size,
            );
        }));
        match growth {
            Ok(()) => {
                let old_buffer =
                    allocated_buffer.replace(new_buffer.take().expect("new buffer allocated"));
                self.pool.release_buffer(old_buffer);
            }
            Err(payload) if payload.is::<TextParserRuntimeError>() => {
                self.pool.release_buffer(new_buffer.take());
            }
            Err(payload) => {
                self.pool.release_buffer(new_buffer.take());
                resume_unwind(payload);
            }
        }
    }

    fn parse_buffer(
        &self,
        buffer: &mut [u16],
        offset: i32,
        len: i32,
        handler: &mut dyn ITextHandler,
        status: &mut TextParseStatus,
    ) -> Result<(), Box<TextParseException>> {
        let mut locator = [status.line, status.col];
        let mut current_line = locator[0];
        let mut current_col = locator[1];
        let maxi = offset.wrapping_add(len);
        let mut index = offset;
        let mut current = index;

        let mut in_open_element = false;
        let mut in_close_element = false;
        let mut in_comment_block = false;
        let mut in_comment_line = false;
        let mut in_literal = false;

        let mut position;
        let mut tag_start = index;
        let mut tag_end = index;

        while index < maxi {
            let mut in_structure = in_open_element
                || in_close_element
                || in_comment_block
                || in_comment_line
                || in_literal;

            if !in_structure {
                position = util_i32_value(
                    TextParsingUtil::find_next_structure_start_or_literal_marker(
                        Some(buffer),
                        index,
                        maxi,
                        Some(&mut locator),
                        self.process_comments_and_literals,
                    ),
                );
                if position == -1 {
                    set_terminal_status(status, current, current_line, current_col);
                    return Ok(());
                }

                let (
                    mut character,
                    classified_open,
                    classified_close,
                    classified_comment_block,
                    classified_comment_line,
                    classified_literal,
                    literal_marker,
                ) = classify_structure_start(
                    buffer,
                    position,
                    maxi,
                    self.process_comments_and_literals,
                );
                in_open_element = classified_open;
                in_close_element = classified_close;
                in_comment_block = classified_comment_block;
                in_comment_line = classified_comment_line;
                in_literal = classified_literal;
                if let Some(literal_marker) = literal_marker {
                    status.literal_marker = literal_marker;
                }

                in_structure = in_open_element
                    || in_close_element
                    || in_comment_block
                    || in_comment_line
                    || in_literal;
                if in_structure && !in_literal {
                    tag_start = position;
                }

                while !in_structure {
                    count_locator(&mut locator, character);
                    position = util_i32_value(
                        TextParsingUtil::find_next_structure_start_or_literal_marker(
                            Some(buffer),
                            position.wrapping_add(1),
                            maxi,
                            Some(&mut locator),
                            self.process_comments_and_literals,
                        ),
                    );
                    if position == -1 {
                        set_terminal_status(status, current, current_line, current_col);
                        return Ok(());
                    }

                    let classification = classify_structure_start(
                        buffer,
                        position,
                        maxi,
                        self.process_comments_and_literals,
                    );
                    character = classification.0;
                    in_open_element = classification.1;
                    in_close_element = classification.2;
                    in_comment_block = classification.3;
                    in_comment_line = classification.4;
                    in_literal = classification.5;
                    if let Some(literal_marker) = classification.6 {
                        status.literal_marker = literal_marker;
                    }
                    in_structure = in_open_element
                        || in_close_element
                        || in_comment_block
                        || in_comment_line
                        || in_literal;
                    if in_structure && !in_literal {
                        tag_start = position;
                    }
                }

                if tag_start > current {
                    handler.handle_text(
                        Some(buffer),
                        current,
                        tag_start.wrapping_sub(current),
                        current_line,
                        current_col,
                    )?;
                }
                if tag_start == position {
                    current = tag_start;
                    current_line = locator[0];
                    current_col = locator[1];
                }
                index = position;
            } else {
                position = if in_literal {
                    util_i32_value(TextParsingUtil::find_next_literal_end(
                        Some(buffer),
                        index,
                        maxi,
                        Some(&mut locator),
                        status.literal_marker,
                    ))
                } else if in_comment_block {
                    util_i32_value(TextParsingUtil::find_next_comment_block_end(
                        Some(buffer),
                        index,
                        maxi,
                        Some(&mut locator),
                    ))
                } else if in_comment_line {
                    util_i32_value(TextParsingUtil::find_next_comment_line_end(
                        Some(buffer),
                        index,
                        maxi,
                        Some(&mut locator),
                    ))
                } else {
                    util_i32_value(TextParsingUtil::find_next_structure_end_avoid_quotes(
                        Some(buffer),
                        index,
                        maxi,
                        Some(&mut locator),
                    ))
                };

                if position < 0 {
                    status.offset = current;
                    status.line = current_line;
                    status.col = current_col;
                    status.in_structure = true;
                    status.in_comment_line = in_comment_line;
                    status.literal_marker = 0;
                    return Ok(());
                }

                if in_open_element {
                    tag_end = position;
                    if array_unit(buffer, tag_end.wrapping_sub(1)) == u16::from(b'/') {
                        element_parse(TextParsingElementUtil::parse_standalone_element(
                            Some(buffer),
                            current,
                            tag_end.wrapping_sub(current).wrapping_add(1),
                            current_line,
                            current_col,
                            Some(handler),
                        ))?;
                    } else {
                        element_parse(TextParsingElementUtil::parse_open_element(
                            Some(buffer),
                            current,
                            tag_end.wrapping_sub(current).wrapping_add(1),
                            current_line,
                            current_col,
                            Some(handler),
                        ))?;
                    }
                    in_open_element = false;
                } else if in_close_element {
                    tag_end = position;
                    element_parse(TextParsingElementUtil::parse_close_element(
                        Some(buffer),
                        current,
                        tag_end.wrapping_sub(current).wrapping_add(1),
                        current_line,
                        current_col,
                        Some(handler),
                    ))?;
                    in_close_element = false;
                } else if in_comment_block {
                    tag_end = position;
                    comment_parse(TextParsingCommentUtil::parse_comment(
                        Some(buffer),
                        current,
                        tag_end.wrapping_sub(current).wrapping_add(1),
                        current_line,
                        current_col,
                        handler,
                    ))?;
                    in_comment_block = false;
                } else if in_comment_line {
                    tag_end = position;
                    handler.handle_text(
                        Some(buffer),
                        current,
                        tag_end.wrapping_sub(current).wrapping_add(1),
                        current_line,
                        current_col,
                    )?;
                    in_comment_line = false;
                } else {
                    // 到达结构结束分支时，五个局部状态至少一个为 true；前四类已
                    // 依次排除，因此这里必为 literal。该不变量由同一循环内的
                    // `in_structure` 计算建立，Java 最后的 IllegalStateException
                    // 防御分支在任何参数和 status 输入下都不可达。
                    in_literal = false;
                    status.literal_marker = 0;
                }

                count_locator(&mut locator, array_unit(buffer, position));
                if tag_end == position {
                    current = tag_end.wrapping_add(1);
                    current_line = locator[0];
                    current_col = locator[1];
                }
                index = position.wrapping_add(1);
            }
        }

        set_terminal_status(status, current, current_line, current_col);
        Ok(())
    }
}

type StructureStartClassification = (u16, bool, bool, bool, bool, bool, Option<u16>);

fn classify_structure_start(
    buffer: &[u16],
    position: i32,
    maxi: i32,
    process_comments_and_literals: bool,
) -> StructureStartClassification {
    let character = array_unit(buffer, position);
    let in_open_element = element_bool_value(TextParsingElementUtil::is_open_element_start(
        Some(buffer),
        position,
        maxi,
    ));
    if in_open_element {
        return (character, true, false, false, false, false, None);
    }

    let in_close_element = element_bool_value(TextParsingElementUtil::is_close_element_start(
        Some(buffer),
        position,
        maxi,
    ));
    if in_close_element {
        return (character, false, true, false, false, false, None);
    }
    if !process_comments_and_literals {
        return (character, false, false, false, false, false, None);
    }

    let in_comment_block = comment_bool_value(TextParsingCommentUtil::is_comment_block_start(
        Some(buffer),
        position,
        maxi,
    ));
    if in_comment_block {
        return (character, false, false, true, false, false, None);
    }

    let in_comment_line = comment_bool_value(TextParsingCommentUtil::is_comment_line_start(
        Some(buffer),
        position,
        maxi,
    ));
    if in_comment_line {
        return (character, false, false, false, true, false, None);
    }

    let in_literal = matches!(character, 0x0027 | 0x0022 | 0x0060)
        || comment_bool_value(TextParsingLiteralUtil::is_regex_literal_start(
            Some(buffer),
            position,
            maxi,
        ));
    (
        character,
        false,
        false,
        false,
        false,
        in_literal,
        Some(if in_literal { character } else { 0 }),
    )
}

fn set_terminal_status(
    status: &mut TextParseStatus,
    current: i32,
    current_line: i32,
    current_col: i32,
) {
    status.offset = current;
    status.line = current_line;
    status.col = current_col;
    status.in_structure = false;
    status.in_comment_line = false;
    status.literal_marker = 0;
}

fn nano_time() -> i64 {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    let elapsed = ORIGIN.get_or_init(Instant::now).elapsed().as_nanos();
    elapsed as i64
}

fn java_char_array(size: i32) -> Vec<u16> {
    if size < 0 {
        panic_runtime(TextParserRuntimeError::negative_array_size(size));
    }
    vec![0; size as usize]
}

fn java_string_from_range(buffer: &[u16], offset: i32, len: i32) -> JavaString {
    if offset < 0 || len < 0 {
        panic_runtime(TextParserRuntimeError::string_range(
            offset,
            len,
            buffer.len(),
        ));
    }
    let end = i64::from(offset) + i64::from(len);
    if end > buffer.len() as i64 {
        panic_runtime(TextParserRuntimeError::string_range(
            offset,
            len,
            buffer.len(),
        ));
    }
    JavaString::from_utf16(buffer[offset as usize..end as usize].to_vec())
}

fn array_unit(buffer: &[u16], index: i32) -> u16 {
    if index < 0 || index as usize >= buffer.len() {
        panic_runtime(TextParserRuntimeError::array_index(index, buffer.len()));
    }
    buffer[index as usize]
}

fn java_arraycopy(
    source: &[u16],
    source_offset: i32,
    destination: &mut [u16],
    destination_offset: i32,
    len: i32,
) {
    if len < 0 {
        panic_runtime(TextParserRuntimeError::arraycopy_negative_length(len));
    }
    if source_offset < 0 {
        panic_runtime(TextParserRuntimeError::arraycopy_source_index(
            source_offset,
            source.len(),
        ));
    }
    if destination_offset < 0 {
        panic_runtime(TextParserRuntimeError::arraycopy_destination_index(
            destination_offset,
            destination.len(),
        ));
    }
    let source_end = i64::from(source_offset) + i64::from(len);
    if source_end > source.len() as i64 {
        panic_runtime(TextParserRuntimeError::arraycopy_last_source(
            source_end,
            source.len(),
        ));
    }
    let destination_end = i64::from(destination_offset) + i64::from(len);
    if destination_end > destination.len() as i64 {
        panic_runtime(TextParserRuntimeError::arraycopy_last_destination(
            destination_end,
            destination.len(),
        ));
    }
    destination[destination_offset as usize..destination_end as usize]
        .copy_from_slice(&source[source_offset as usize..source_end as usize]);
}

fn java_arraycopy_within(
    buffer: &mut [u16],
    source_offset: i32,
    destination_offset: i32,
    len: i32,
) {
    if len < 0 {
        panic_runtime(TextParserRuntimeError::arraycopy_negative_length(len));
    }
    if source_offset < 0 {
        panic_runtime(TextParserRuntimeError::arraycopy_source_index(
            source_offset,
            buffer.len(),
        ));
    }
    if destination_offset < 0 {
        panic_runtime(TextParserRuntimeError::arraycopy_destination_index(
            destination_offset,
            buffer.len(),
        ));
    }
    let source_end = i64::from(source_offset) + i64::from(len);
    if source_end > buffer.len() as i64 {
        panic_runtime(TextParserRuntimeError::arraycopy_last_source(
            source_end,
            buffer.len(),
        ));
    }
    let destination_end = i64::from(destination_offset) + i64::from(len);
    if destination_end > buffer.len() as i64 {
        panic_runtime(TextParserRuntimeError::arraycopy_last_destination(
            destination_end,
            buffer.len(),
        ));
    }
    buffer.copy_within(
        source_offset as usize..source_end as usize,
        destination_offset as usize,
    );
}

fn util_i32_value(result: Result<i32, TextParsingUtilError>) -> i32 {
    match result {
        Ok(value) => value,
        Err(error) => panic_any(error),
    }
}

fn element_bool_value(result: Result<bool, TextParsingElementError>) -> bool {
    match result {
        Ok(value) => value,
        Err(error) => panic_any(error),
    }
}

fn comment_bool_value(result: Result<bool, TextParsingCommentError>) -> bool {
    match result {
        Ok(value) => value,
        Err(error) => panic_any(error),
    }
}

fn element_parse(
    result: Result<(), TextParsingElementError>,
) -> Result<(), Box<TextParseException>> {
    match result {
        Ok(()) => Ok(()),
        Err(TextParsingElementError::TextParse(exception)) => Err(exception),
        Err(error) => panic_any(error),
    }
}

fn comment_parse(
    result: Result<(), TextParsingCommentError>,
) -> Result<(), Box<TextParseException>> {
    match result {
        Ok(()) => Ok(()),
        Err(TextParsingCommentError::TextParse(exception)) => Err(exception),
        Err(error) => panic_any(error),
    }
}

fn count_locator(locator: &mut [i32], character: u16) {
    if let Err(error) = ParsingLocatorUtil::count_char(Some(locator), character) {
        panic_any(error);
    }
}

fn reader_error_as_text_parse(error: TextParserReaderError) -> Box<TextParseException> {
    let java_class_name = error.java_class_name.clone();
    let java_message = error.java_message.clone();
    Box::new(TextParseException::with_cause(Some(
        TextParseCause::with_java_metadata(Box::new(error), java_class_name, java_message),
    )))
}

fn panic_payload_to_cause(
    payload: Box<dyn Any + Send>,
) -> Result<TextParseCause, Box<dyn Any + Send>> {
    let payload = match payload.downcast::<TextParserRuntimeError>() {
        Ok(error) => {
            let class_name = error.java_class_name();
            let message = error.java_message();
            return Ok(TextParseCause::with_java_metadata(
                error, class_name, message,
            ));
        }
        Err(payload) => payload,
    };
    let payload = match payload.downcast::<TextParsingUtilError>() {
        Ok(error) => {
            let class_name = error.java_class_name();
            let message = error.java_message();
            return Ok(TextParseCause::with_java_metadata(
                error, class_name, message,
            ));
        }
        Err(payload) => payload,
    };
    let payload = match payload.downcast::<TextParsingElementError>() {
        Ok(error) => {
            let class_name = error.java_class_name();
            let message = Some(error.java_message());
            return Ok(TextParseCause::with_java_metadata(
                error, class_name, message,
            ));
        }
        Err(payload) => payload,
    };
    let payload = match payload.downcast::<TextParsingCommentError>() {
        Ok(error) => {
            let class_name = error.java_class_name();
            let message = Some(error.java_message());
            return Ok(TextParseCause::with_java_metadata(
                error, class_name, message,
            ));
        }
        Err(payload) => payload,
    };
    let payload = match payload.downcast::<ParsingLocatorError>() {
        Ok(error) => {
            let class_name = error.java_class_name();
            let message = Some(error.message());
            return Ok(TextParseCause::with_java_metadata(
                error, class_name, message,
            ));
        }
        Err(payload) => payload,
    };
    let payload = match payload.downcast::<EventProcessorTextHandlerRuntimeError>() {
        Ok(error) => {
            let class_name = error.java_class_name();
            let message = Some(error.java_message().clone());
            return Ok(TextParseCause::with_java_metadata(
                error, class_name, message,
            ));
        }
        Err(payload) => payload,
    };
    let payload = match payload.downcast::<CommentProcessorTextHandlerRuntimeError>() {
        Ok(error) => {
            let class_name = error.java_class_name();
            let message = error.java_message();
            return Ok(TextParseCause::with_java_metadata(
                error, class_name, message,
            ));
        }
        Err(payload) => payload,
    };
    match payload.downcast::<ChainedTextHandlerRuntimeError>() {
        Ok(error) => {
            let class_name = error.java_class_name();
            let message = Some(error.java_message());
            Ok(TextParseCause::with_java_metadata(
                error, class_name, message,
            ))
        }
        Err(payload) => Err(payload),
    }
}

fn panic_runtime(error: TextParserRuntimeError) -> ! {
    panic_any(error)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::error::Error;
    use std::fmt::{Display, Write};
    use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};
    use std::rc::Rc;

    use super::{
        AllocatedBuffer, BufferPool, ITextHandler, JavaString, ParsingLocatorError,
        StringTextParserReader, TextParseException, TextParser, TextParserReader,
        TextParserReaderError, TextParserRuntimeError, array_unit, comment_bool_value,
        comment_parse, count_locator, element_bool_value, element_parse, java_arraycopy,
        java_arraycopy_within, java_string_from_range, panic_payload_to_cause, util_i32_value,
    };
    use crate::text::{
        AbstractChainedTextHandler, AbstractTextHandler, CommentProcessorTextHandlerRuntimeError,
        EventProcessorTextHandler, TextParsingCommentError, TextParsingElementError,
        TextParsingUtilError,
    };

    const JAVA_GOLDEN: &str = include_str!("../../tests/fixtures/text_parser_golden.txt");

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum CloseMode {
        None,
        Io,
        Assertion,
    }

    #[derive(Debug)]
    struct ScriptedReaderState {
        input: Vec<u16>,
        max_chunk: usize,
        zero_reads: i32,
        fail_call: i32,
        close_mode: CloseMode,
        position: usize,
        read_calls: i32,
        close_count: i32,
        requests: Vec<String>,
    }

    struct ScriptedReader {
        state: Rc<RefCell<ScriptedReaderState>>,
    }

    impl ScriptedReader {
        fn new(
            input: JavaString,
            max_chunk: usize,
            zero_reads: i32,
            fail_call: i32,
            close_mode: CloseMode,
        ) -> (Self, Rc<RefCell<ScriptedReaderState>>) {
            let state = Rc::new(RefCell::new(ScriptedReaderState {
                input: input.as_utf16().to_vec(),
                max_chunk,
                zero_reads,
                fail_call,
                close_mode,
                position: 0,
                read_calls: 0,
                close_count: 0,
                requests: Vec::new(),
            }));
            (
                Self {
                    state: Rc::clone(&state),
                },
                state,
            )
        }
    }

    impl TextParserReader for ScriptedReader {
        fn read_range(
            &mut self,
            buffer: &mut [u16],
            offset: i32,
            len: i32,
        ) -> Result<i32, TextParserReaderError> {
            let mut state = self.state.borrow_mut();
            state.requests.push(format!("{offset}:{len}"));
            state.read_calls += 1;
            if state.fail_call == state.read_calls {
                let message = format!("reader-boom-{}", state.read_calls);
                return Err(TextParserReaderError::io(&message));
            }
            if state.zero_reads > 0 {
                state.zero_reads -= 1;
                return Ok(0);
            }
            if state.position >= state.input.len() {
                return Ok(-1);
            }
            let copied = (len as usize)
                .min(state.max_chunk)
                .min(state.input.len() - state.position);
            let input_start = state.position;
            let input_end = input_start + copied;
            buffer[offset as usize..offset as usize + copied]
                .copy_from_slice(&state.input[input_start..input_end]);
            state.position = input_end;
            Ok(copied as i32)
        }

        fn close(&mut self) -> Result<(), TextParserReaderError> {
            let mut state = self.state.borrow_mut();
            state.close_count += 1;
            match state.close_mode {
                CloseMode::None => Ok(()),
                CloseMode::Io => Err(TextParserReaderError::io("close-boom")),
                CloseMode::Assertion => panic_any("close-error"),
            }
        }
    }

    #[derive(Default)]
    struct RecordingState {
        events: String,
        semantic: bool,
        fail_event: Option<&'static str>,
        runtime_fail_event: Option<&'static str>,
        unknown_runtime_fail_event: Option<&'static str>,
    }

    struct RecordingHandler {
        state: Rc<RefCell<RecordingState>>,
    }

    impl RecordingHandler {
        fn new(semantic: bool) -> (Self, Rc<RefCell<RecordingState>>) {
            let state = Rc::new(RefCell::new(RecordingState {
                semantic,
                ..RecordingState::default()
            }));
            (
                Self {
                    state: Rc::clone(&state),
                },
                state,
            )
        }

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
                    state.events.push_str(&hex(
                        &value[offset as usize..offset.wrapping_add(len) as usize]
                    ));
                }
                Some(_) => write!(state.events, "range({offset},{len})").unwrap(),
            }
            if state.fail_event == Some(event) {
                return Err(Box::new(TextParseException::with_message_at(
                    Some(&JavaString::from_rust_str(&format!("checked-{event}"))),
                    71,
                    72,
                )));
            }
            if state.runtime_fail_event == Some(event) {
                panic_any(TextParserRuntimeError::with_java_metadata(
                    "java.lang.IllegalStateException",
                    Some(JavaString::from_rust_str(&format!("runtime-{event}"))),
                ));
            }
            if state.unknown_runtime_fail_event == Some(event) {
                panic_any("unknown-handler-panic");
            }
            Ok(())
        }

        fn arguments(&self, detailed: String, semantic: String) -> String {
            if self.state.borrow().semantic {
                semantic
            } else {
                detailed
            }
        }
    }

    impl ITextHandler for RecordingHandler {
        fn handle_document_start(
            &mut self,
            _start_time_nanos: i64,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record("documentStart", None, 0, 0, format!("{line},{col}"))
        }

        fn handle_document_end(
            &mut self,
            _end_time_nanos: i64,
            total_time_nanos: i64,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                "documentEnd",
                None,
                0,
                0,
                format!("{},{line},{col}", total_time_nanos >= 0),
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
            let arguments = self.arguments(
                format!("{offset},{len},{line},{col}"),
                format!("{line},{col}"),
            );
            self.record("text", buffer, offset, len, arguments)
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
            let arguments = self.arguments(
                format!("{content_offset},{content_len},{outer_offset},{outer_len},{line},{col}"),
                format!("{line},{col}"),
            );
            self.record("comment", buffer, outer_offset, outer_len, arguments)
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
            let arguments = self.arguments(
                format!("{name_offset},{name_len},{minimized},{line},{col}"),
                format!("{minimized},{line},{col}"),
            );
            self.record("standaloneStart", buffer, name_offset, name_len, arguments)
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
            let arguments = self.arguments(
                format!("{name_offset},{name_len},{minimized},{line},{col}"),
                format!("{minimized},{line},{col}"),
            );
            self.record("standaloneEnd", buffer, name_offset, name_len, arguments)
        }

        fn handle_open_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            let arguments = self.arguments(
                format!("{name_offset},{name_len},{line},{col}"),
                format!("{line},{col}"),
            );
            self.record("openStart", buffer, name_offset, name_len, arguments)
        }

        fn handle_open_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            let arguments = self.arguments(
                format!("{name_offset},{name_len},{line},{col}"),
                format!("{line},{col}"),
            );
            self.record("openEnd", buffer, name_offset, name_len, arguments)
        }

        fn handle_close_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            let arguments = self.arguments(
                format!("{name_offset},{name_len},{line},{col}"),
                format!("{line},{col}"),
            );
            self.record("closeStart", buffer, name_offset, name_len, arguments)
        }

        fn handle_close_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            let arguments = self.arguments(
                format!("{name_offset},{name_len},{line},{col}"),
                format!("{line},{col}"),
            );
            self.record("closeEnd", buffer, name_offset, name_len, arguments)
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
            let arguments = self.arguments(
                format!(
                    "{name_offset},{name_len},{name_line},{name_col},{operator_offset},{operator_len},{operator_line},{operator_col},{value_content_offset},{value_content_len},{value_outer_offset},{value_outer_len},{value_line},{value_col}"
                ),
                format!(
                    "{name_line},{name_col},{operator_line},{operator_col},{value_line},{value_col}"
                ),
            );
            self.record("attribute", buffer, name_offset, name_len, arguments)
        }
    }

    fn handler(semantic: bool) -> (Box<dyn ITextHandler>, Rc<RefCell<RecordingState>>) {
        let (handler, state) = RecordingHandler::new(semantic);
        (Box::new(handler), state)
    }

    fn scripted_reader(
        input: &JavaString,
        max_chunk: usize,
        zero_reads: i32,
        fail_call: i32,
        close_mode: CloseMode,
    ) -> (Box<dyn TextParserReader>, Rc<RefCell<ScriptedReaderState>>) {
        let (reader, state) =
            ScriptedReader::new(input.clone(), max_chunk, zero_reads, fail_call, close_mode);
        (Box::new(reader), state)
    }

    fn generate_golden() -> String {
        let mut output = String::new();
        emit(
            &mut output,
            "baseline",
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add",
        );
        validation_cases(&mut output);
        document_cases(&mut output);
        split_matrix_cases(&mut output);
        reader_cases(&mut output);
        handler_failure_cases(&mut output);
        incomplete_cases(&mut output);
        buffer_pool_cases(&mut output);
        constructor_cases(&mut output);
        output
    }

    fn validation_cases(output: &mut String) {
        let parser = TextParser::new(1, 4, true, true);
        emit(
            output,
            "validation.nullDocument",
            throwable(|| {
                let (handler, _) = handler(false);
                parser.parse(None, Some(handler))
            }),
        );
        emit(
            output,
            "validation.nullStringHandler",
            throwable(|| parser.parse(Some(&java("x")), None)),
        );
        emit(
            output,
            "validation.nullReader",
            throwable(|| {
                let (handler, _) = handler(false);
                parser.parse_reader(None, Some(handler))
            }),
        );
        emit(
            output,
            "validation.nullReaderHandler",
            throwable(|| {
                let (reader, _) = scripted_reader(&java("x"), 1, 0, -1, CloseMode::None);
                parser.parse_reader(Some(reader), None)
            }),
        );
    }

    fn document_cases(output: &mut String) {
        let mut documents = vec![
            java(""),
            java("plain"),
            java("a\nb\r\nc"),
            java("[#root]x[/root]"),
            java("[#img src=\"hello\" alt='x'/]"),
            java("[# one]"),
            java("[[#hello]]...[[/hello]]"),
            java("/*hello*/"),
            java("/*[#hello/]*/tail"),
            java("/*[(hello)]*/something;"),
            java("a//line\n[#x/]b"),
            java("\"[#not]\" [#yes/]"),
            java("'a\\'[#not]' [#yes/]"),
            java("`[#not]` [#yes/]"),
            java("/[#not]/ [#yes/]"),
        ];
        documents.push(JavaString::from_utf16(vec![
            0xd800,
            u16::from(b'['),
            u16::from(b'#'),
            u16::from(b'x'),
            u16::from(b'/'),
            u16::from(b']'),
            0xdc00,
        ]));

        for process_comments in [false, true] {
            for standard_dialect in [false, true] {
                for (index, document) in documents.iter().enumerate() {
                    let (handler, state) = handler(false);
                    let parser = TextParser::new(2, 3, process_comments, standard_dialect);
                    emit(
                        output,
                        &format!(
                            "document.{process_comments}.{standard_dialect}.{index}.throwable"
                        ),
                        throwable(|| parser.parse(Some(document), Some(handler))),
                    );
                    emit(
                        output,
                        &format!("document.{process_comments}.{standard_dialect}.{index}.events"),
                        state.borrow().events.clone(),
                    );
                }
            }
        }
    }

    fn split_matrix_cases(output: &mut String) {
        let documents = [
            java("before[#root a=\"x]y\"]line\n[#single/][/root]after"),
            java("/*[(hello)]*/ [1,\n 2,3] tail;"),
            java("a//comment\nb/*[#x/]*/c"),
            java("\"quoted\\\\\\\" [#no]\" [#yes/]"),
            java("[#template a='zero' b='one']\n\naaaaa\n\n[/template]"),
        ];
        for (index, document) in documents.iter().enumerate() {
            for process_comments in [false, true] {
                let expected = parse_with_buffer(document, 64, process_comments);
                let mut digest = String::new();
                for buffer_size in 1..=96 {
                    let actual = parse_with_buffer(document, buffer_size, process_comments);
                    assert_eq!(
                        actual, expected,
                        "split mismatch document={index}, process={process_comments}, buffer={buffer_size}"
                    );
                    write!(
                        digest,
                        "{}:{};",
                        java_string_hash(&actual),
                        actual.encode_utf16().count()
                    )
                    .unwrap();
                }
                emit(
                    output,
                    &format!("split.{index}.{process_comments}"),
                    format!("{expected};matrixHash={}", java_string_hash(&digest)),
                );
            }
        }
    }

    fn parse_with_buffer(
        document: &JavaString,
        buffer_size: i32,
        process_comments: bool,
    ) -> String {
        let parser = TextParser::new(2, buffer_size, process_comments, true);
        let (reader, _) = scripted_reader(document, usize::MAX, 0, -1, CloseMode::None);
        let (handler, state) = handler(true);
        let mut chain: Box<dyn ITextHandler> =
            Box::new(super::EventProcessorTextHandler::new(Some(handler)));
        if process_comments {
            chain = Box::new(super::CommentProcessorTextHandler::new(true, Some(chain)));
        }
        match parser.parse_document(Some(reader), buffer_size, Some(chain)) {
            Ok(()) => state.borrow().events.clone(),
            Err(error) => describe_text_parse(&error),
        }
    }

    fn reader_cases(output: &mut String) {
        run_reader(
            output,
            "reader.chunk1",
            &java("[#x/]tail"),
            1,
            0,
            -1,
            CloseMode::None,
            3,
        );
        run_reader(
            output,
            "reader.chunk2",
            &java("[#x/]tail"),
            2,
            0,
            -1,
            CloseMode::None,
            3,
        );
        run_reader(
            output,
            "reader.zeroThenData",
            &java("[#x/]"),
            2,
            2,
            -1,
            CloseMode::None,
            3,
        );
        run_reader(
            output,
            "reader.readFailure",
            &java("[#x/]tail"),
            2,
            0,
            3,
            CloseMode::None,
            3,
        );
        run_reader(
            output,
            "reader.closeIOException",
            &java("plain"),
            2,
            0,
            -1,
            CloseMode::Io,
            3,
        );
        run_reader(
            output,
            "reader.closeAssertion",
            &java("plain"),
            2,
            0,
            -1,
            CloseMode::Assertion,
            3,
        );
        run_reader(
            output,
            "reader.empty",
            &java(""),
            2,
            0,
            -1,
            CloseMode::None,
            3,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn run_reader(
        output: &mut String,
        key: &str,
        input: &JavaString,
        max_chunk: usize,
        zero_reads: i32,
        fail_call: i32,
        close_mode: CloseMode,
        buffer_size: i32,
    ) {
        let parser = TextParser::new(1, buffer_size, false, true);
        let (reader, reader_state) =
            scripted_reader(input, max_chunk, zero_reads, fail_call, close_mode);
        let (handler, handler_state) = handler(false);
        emit(
            output,
            &format!("{key}.throwable"),
            throwable(|| parser.parse_document(Some(reader), buffer_size, Some(handler))),
        );
        emit(
            output,
            &format!("{key}.events"),
            handler_state.borrow().events.clone(),
        );
        emit(
            output,
            &format!("{key}.requests"),
            reader_state.borrow().requests.join(","),
        );
        emit(
            output,
            &format!("{key}.closeCount"),
            reader_state.borrow().close_count,
        );
    }

    fn handler_failure_cases(output: &mut String) {
        for event in ["documentStart", "text", "openStart", "documentEnd"] {
            let parser = TextParser::new(1, 3, false, true);
            let (reader, reader_state) =
                scripted_reader(&java("[#x]text[/x]"), 2, 0, -1, CloseMode::None);
            let (handler, handler_state) = handler(false);
            handler_state.borrow_mut().fail_event = Some(event);
            emit(
                output,
                &format!("handler.checked.{event}"),
                throwable(|| parser.parse_document(Some(reader), 3, Some(handler))),
            );
            emit(
                output,
                &format!("handler.checked.{event}.events"),
                handler_state.borrow().events.clone(),
            );
            emit(
                output,
                &format!("handler.checked.{event}.closeCount"),
                reader_state.borrow().close_count,
            );
        }

        for event in ["documentStart", "text", "standaloneStart", "documentEnd"] {
            let document = if event == "standaloneStart" {
                java("[#x/]")
            } else {
                java("text")
            };
            let parser = TextParser::new(1, 3, false, true);
            let (reader, reader_state) = scripted_reader(&document, 2, 0, -1, CloseMode::None);
            let (handler, handler_state) = handler(false);
            handler_state.borrow_mut().runtime_fail_event = Some(event);
            emit(
                output,
                &format!("handler.runtime.{event}"),
                throwable(|| parser.parse_document(Some(reader), 3, Some(handler))),
            );
            emit(
                output,
                &format!("handler.runtime.{event}.events"),
                handler_state.borrow().events.clone(),
            );
            emit(
                output,
                &format!("handler.runtime.{event}.closeCount"),
                reader_state.borrow().close_count,
            );
        }
    }

    fn incomplete_cases(output: &mut String) {
        let documents = [
            java("[#open"),
            java("[/close"),
            java("/*block"),
            java("//line"),
            java("\"literal"),
            java("'literal"),
            java("`literal"),
            java("/regex"),
        ];
        for process_comments in [false, true] {
            for (index, document) in documents.iter().enumerate() {
                let parser = TextParser::new(1, 2, process_comments, true);
                let (reader, _) = scripted_reader(document, 1, 0, -1, CloseMode::None);
                let (handler, state) = handler(false);
                emit(
                    output,
                    &format!("incomplete.{process_comments}.{index}.throwable"),
                    throwable(|| parser.parse_document(Some(reader), 2, Some(handler))),
                );
                emit(
                    output,
                    &format!("incomplete.{process_comments}.{index}.events"),
                    state.borrow().events.clone(),
                );
            }
        }
    }

    fn buffer_pool_cases(output: &mut String) {
        let pool = BufferPool::new(2, 4);
        let first = pool.allocate_buffer(4);
        let first_pointer = first.buffer.as_ptr();
        let second = pool.allocate_buffer(4);
        let second_pointer = second.buffer.as_ptr();
        let overflow = pool.allocate_buffer(4);
        let overflow_pointer = overflow.buffer.as_ptr();
        emit(
            output,
            "pool.distinct",
            format!(
                "{},{},{}",
                first_pointer != second_pointer,
                first_pointer != overflow_pointer,
                second_pointer != overflow_pointer
            ),
        );
        pool.release_buffer(Some(first));
        let reused_first = pool.allocate_buffer(4);
        emit(
            output,
            "pool.reusedFirst",
            reused_first.buffer.as_ptr() == first_pointer,
        );
        pool.release_buffer(Some(AllocatedBuffer {
            buffer: vec![0; 4],
            pool_index: None,
        }));
        let still_overflow = pool.allocate_buffer(4);
        emit(
            output,
            "pool.foreignIgnored",
            still_overflow.buffer.as_ptr() != first_pointer
                && still_overflow.buffer.as_ptr() != second_pointer,
        );
        pool.release_buffer(Some(second));
        let reused_second = pool.allocate_buffer(4);
        emit(
            output,
            "pool.reusedSecond",
            reused_second.buffer.as_ptr() == second_pointer,
        );
        let different_one = pool.allocate_buffer(3);
        let different_two = pool.allocate_buffer(3);
        emit(
            output,
            "pool.differentSize",
            format!(
                "{},{},{}",
                different_one.buffer.len(),
                different_two.buffer.len(),
                different_one.buffer.as_ptr() != different_two.buffer.as_ptr()
            ),
        );
        pool.release_buffer(None);
        emit(output, "pool.releaseNull", "NO_ERROR");
        emit(
            output,
            "pool.negativeAllocate",
            panic_throwable(|| {
                let _ = pool.allocate_buffer(-1);
            }),
        );
        emit(
            output,
            "pool.negativePoolSize",
            panic_throwable(|| {
                let _ = BufferPool::new(-1, 4);
            }),
        );
        emit(
            output,
            "pool.negativeBufferSize",
            panic_throwable(|| {
                let _ = BufferPool::new(1, -1);
            }),
        );
        let zero_negative = BufferPool::new(0, -1);
        emit(output, "pool.zeroPoolNegativeBuffer", true);
        emit(
            output,
            "pool.zeroPoolNegativeAllocate",
            panic_throwable(|| {
                let _ = zero_negative.allocate_buffer(-1);
            }),
        );
    }

    fn constructor_cases(output: &mut String) {
        emit(
            output,
            "constructor.negativePool",
            panic_throwable(|| {
                let _ = TextParser::new(-1, 4, false, true);
            }),
        );
        emit(
            output,
            "constructor.negativeBuffer",
            panic_throwable(|| {
                let _ = TextParser::new(1, -1, false, true);
            }),
        );
        emit(
            output,
            "constructor.zeroPoolNegativeBuffer",
            panic_throwable(|| {
                let _ = TextParser::new(0, -1, false, true);
            }),
        );
        let parser = TextParser::new(0, 1, false, true);
        let (first_handler, first_state) = handler(false);
        let (second_handler, second_state) = handler(false);
        emit(
            output,
            "constructor.zeroPool.first",
            throwable(|| parser.parse(Some(&java("a")), Some(first_handler))),
        );
        emit(
            output,
            "constructor.zeroPool.second",
            throwable(|| parser.parse(Some(&java("b")), Some(second_handler))),
        );
        emit(
            output,
            "constructor.zeroPool.events",
            format!(
                "{}|{}",
                first_state.borrow().events,
                second_state.borrow().events
            ),
        );
    }

    fn throwable(operation: impl FnOnce() -> Result<(), Box<TextParseException>>) -> String {
        throwable_boxed(Box::new(operation))
    }

    fn throwable_boxed(
        operation: Box<dyn FnOnce() -> Result<(), Box<TextParseException>> + '_>,
    ) -> String {
        match catch_unwind(AssertUnwindSafe(operation)) {
            Ok(Ok(())) => "NO_ERROR".to_owned(),
            Ok(Err(error)) => describe_text_parse(&error),
            Err(payload) => describe_panic(payload),
        }
    }

    fn panic_throwable(operation: impl FnOnce()) -> String {
        panic_throwable_boxed(Box::new(operation))
    }

    fn panic_throwable_boxed(operation: Box<dyn FnOnce() + '_>) -> String {
        match catch_unwind(AssertUnwindSafe(operation)) {
            Ok(()) => "NO_ERROR".to_owned(),
            Err(payload) => describe_panic(payload),
        }
    }

    fn describe_text_parse(error: &TextParseException) -> String {
        let mut result = format!(
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
        );
        if let Some(cause) = error.get_cause() {
            write!(
                result,
                ";causeClass={};causeMessage={}",
                cause.java_class_name(),
                hex(&error
                    .source()
                    .expect("TextParseCause always owns its source")
                    .to_string()
                    .encode_utf16()
                    .collect::<Vec<_>>())
            )
            .unwrap();
        }
        result
    }

    fn describe_panic(payload: Box<dyn std::any::Any + Send>) -> String {
        match payload.downcast::<TextParserRuntimeError>() {
            Ok(error) => format!(
                "{};message={}",
                error.java_class_name(),
                error
                    .java_message()
                    .map_or_else(|| "null".to_owned(), |message| hex(message.as_utf16()))
            ),
            Err(_) => panic!("unknown panic payload"),
        }
    }

    fn java(value: &str) -> JavaString {
        JavaString::from_rust_str(value)
    }

    fn java_string_hash(value: &str) -> i32 {
        value.encode_utf16().fold(0_i32, |hash, unit| {
            hash.wrapping_mul(31).wrapping_add(i32::from(unit))
        })
    }

    fn hex(value: &[u16]) -> String {
        value
            .iter()
            .map(|unit| format!("{unit:04x}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn emit(output: &mut String, key: &str, value: impl Display) {
        writeln!(output, "{key}={value}").unwrap();
    }

    #[test]
    fn java_golden_matches_streaming_parser_pool_and_failure_semantics() {
        assert_eq!(generate_golden(), JAVA_GOLDEN);
    }

    /// RUST_OBLIGATION：扩容局部异常不能越过 Java `catch (Exception)`。
    ///
    /// 若实现把扩容失败交给 `parse_document` 外层包装，本测试会观察到 panic；
    /// 同时校验 Java `int` 乘法已发生且旧缓冲仍是活动缓冲。
    #[test]
    fn buffer_growth_runtime_exception_is_ignored_after_java_int_wrap() {
        let parser = TextParser::new(0, 1, false, false);
        let mut buffer_size = i32::MAX;
        let mut allocated_buffer = Some(AllocatedBuffer {
            buffer: vec![u16::from(b'x')],
            pool_index: None,
        });

        parser.grow_buffer(&mut buffer_size, 1, &mut allocated_buffer);

        assert_eq!(buffer_size, -2);
        assert_eq!(
            allocated_buffer
                .as_ref()
                .expect("old buffer remains active")
                .buffer,
            vec![u16::from(b'x')]
        );
    }

    /// RUST_OBLIGATION：Java Reader、数组和字符串边界必须归一到精确 JVM 错误合同。
    ///
    /// 本测试会杀死“统一成一个越界错误”、改变校验顺序或丢失 null 消息的实现。
    #[test]
    fn runtime_adapters_preserve_distinct_jvm_failure_contracts() {
        let error = TextParserReaderError::new("example.ReaderError", None);
        assert_eq!(error.java_class_name(), "example.ReaderError");
        assert_eq!(error.java_message(), None);
        assert_eq!(error.to_string(), "null");
        let io_error = TextParserReaderError::io("reader-message");
        assert_eq!(io_error.java_class_name(), "java.io.IOException");
        assert_eq!(
            io_error
                .java_message()
                .expect("IOException has a message")
                .to_string_lossy(),
            "reader-message"
        );

        let document = java("xy");
        let mut reader = StringTextParserReader::new(&document);
        let mut destination = [0_u16; 2];
        assert_eq!(reader.read_range(&mut destination, 0, 0), Ok(0));

        let cases = [
            (
                TextParserRuntimeError::array_index(-1, 2),
                "java.lang.ArrayIndexOutOfBoundsException",
                "Index -1 out of bounds for length 2",
            ),
            (
                TextParserRuntimeError::string_range(-1, 1, 2),
                "java.lang.StringIndexOutOfBoundsException",
                "Range [-1, -1 + 1) out of bounds for length 2",
            ),
            (
                TextParserRuntimeError::arraycopy_negative_length(-1),
                "java.lang.ArrayIndexOutOfBoundsException",
                "arraycopy: length -1 is negative",
            ),
            (
                TextParserRuntimeError::arraycopy_source_index(-1, 2),
                "java.lang.ArrayIndexOutOfBoundsException",
                "arraycopy: source index -1 out of bounds for char[2]",
            ),
            (
                TextParserRuntimeError::arraycopy_destination_index(-1, 2),
                "java.lang.ArrayIndexOutOfBoundsException",
                "arraycopy: destination index -1 out of bounds for char[2]",
            ),
            (
                TextParserRuntimeError::arraycopy_last_source(3, 2),
                "java.lang.ArrayIndexOutOfBoundsException",
                "arraycopy: last source index 3 out of bounds for char[2]",
            ),
            (
                TextParserRuntimeError::arraycopy_last_destination(3, 2),
                "java.lang.ArrayIndexOutOfBoundsException",
                "arraycopy: last destination index 3 out of bounds for char[2]",
            ),
        ];
        for (error, expected_class, expected_message) in cases {
            assert_eq!(error.java_class_name(), expected_class);
            assert_eq!(error.to_string(), expected_message);
        }

        assert_runtime_error(|| {
            let _ = array_unit(&[1], -1);
        });
        assert_runtime_error(|| {
            let _ = java_string_from_range(&[1], -1, 1);
        });
        assert_runtime_error(|| {
            let _ = java_string_from_range(&[1], 0, 2);
        });

        let mut destination = [0_u16; 2];
        assert_runtime_error(|| java_arraycopy(&[1, 2], 0, &mut destination, 0, -1));
        assert_runtime_error(|| java_arraycopy(&[1, 2], -1, &mut destination, 0, 1));
        assert_runtime_error(|| java_arraycopy(&[1, 2], 0, &mut destination, -1, 1));
        assert_runtime_error(|| java_arraycopy(&[1, 2], 1, &mut destination, 0, 2));
        assert_runtime_error(|| java_arraycopy(&[1, 2], 0, &mut destination, 1, 2));
        java_arraycopy(&[1, 2], 0, &mut destination, 0, 2);
        assert_eq!(destination, [1, 2]);

        assert_runtime_error(|| java_arraycopy_within(&mut [1, 2], 0, 0, -1));
        assert_runtime_error(|| java_arraycopy_within(&mut [1, 2], -1, 0, 1));
        assert_runtime_error(|| java_arraycopy_within(&mut [1, 2], 0, -1, 1));
        assert_runtime_error(|| java_arraycopy_within(&mut [1, 2], 1, 0, 2));
        assert_runtime_error(|| java_arraycopy_within(&mut [1, 2], 0, 1, 2));
        let mut overlapping = [1, 2, 3];
        java_arraycopy_within(&mut overlapping, 0, 1, 2);
        assert_eq!(overlapping, [1, 1, 2]);
    }

    /// RUST_OBLIGATION：Rust 内部默认方法、空消息格式化和互斥锁中毒恢复不能形成
    /// 未验证的旁路；这些适配不改变 Java 可观察合同，但必须确定性地继续工作。
    #[test]
    fn internal_defaults_null_formatting_and_poison_recovery_are_deterministic() {
        let runtime = TextParserRuntimeError::with_java_metadata("example.Runtime", None);
        assert_eq!(runtime.to_string(), "null");
        assert_eq!(
            describe_panic(Box::new(runtime)),
            "example.Runtime;message=null"
        );
        assert_eq!(
            describe_text_parse(&TextParseException::new()),
            "org.thymeleaf.templateparser.text.TextParseException;message=null;line=null;col=null"
        );

        let mut reader = StringTextParserReader::new(&java("xy"));
        let mut buffer = [0_u16; 2];
        assert_eq!(reader.read_buffer(&mut buffer), Ok(2));
        assert_eq!(buffer, [u16::from(b'x'), u16::from(b'y')]);
        assert_eq!(reader.close(), Ok(()));

        let pool = BufferPool::new(1, 2);
        let poison = catch_unwind(AssertUnwindSafe(|| {
            let _guard = pool.state.lock().expect("fresh mutex");
            panic!("poison buffer pool");
        }));
        assert!(poison.is_err());
        let allocated = pool.allocate_buffer(2);
        assert_eq!(allocated.pool_index, Some(0));
        pool.release_buffer(Some(allocated));
        assert!(
            pool.state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .buffers[0]
                .is_some()
        );
    }

    /// RUST_OBLIGATION：解析体必须在所有已知异常上保留 Java cause 分类，在未知
    /// panic 上完成清理后继续传播。
    #[test]
    fn parse_document_normalizes_known_causes_and_cleans_up_unknown_panics() {
        let parser = TextParser::new(1, 2, false, false);
        let (reader, reader_state) = ScriptedReader::new(java("x"), 1, 0, 1, CloseMode::None);
        let (handler, _) = RecordingHandler::new(false);
        let initial_read = parser
            .parse_document(Some(Box::new(reader)), 2, Some(Box::new(handler)))
            .expect_err("initial Reader failure is preserved");
        assert_eq!(
            initial_read
                .get_cause()
                .expect("reader cause")
                .java_class_name(),
            "java.io.IOException"
        );
        assert_eq!(reader_state.borrow().close_count, 1);

        let (reader, reader_state) = ScriptedReader::new(java("plain"), 5, 0, -1, CloseMode::None);
        let (handler, handler_state) = RecordingHandler::new(false);
        handler_state.borrow_mut().fail_event = Some("text");
        let final_text = parser
            .parse_document(Some(Box::new(reader)), 5, Some(Box::new(handler)))
            .expect_err("terminal text checked failure is preserved");
        assert_eq!(final_text.get_line(), Some(71));
        assert_eq!(final_text.get_col(), Some(72));
        assert_eq!(reader_state.borrow().close_count, 1);

        let (handler, _) = RecordingHandler::new(false);
        let null_reader = parser
            .parse_document(None, 2, Some(Box::new(handler)))
            .expect_err("null reader is wrapped");
        assert_eq!(
            null_reader.get_cause().expect("cause").java_class_name(),
            "java.lang.NullPointerException"
        );

        let (reader, reader_state) = ScriptedReader::new(java("x"), 1, 0, -1, CloseMode::None);
        let null_handler = parser
            .parse_document(Some(Box::new(reader)), 2, None)
            .expect_err("null handler is wrapped");
        assert_eq!(
            null_handler.get_cause().expect("cause").java_class_name(),
            "java.lang.NullPointerException"
        );
        assert_eq!(reader_state.borrow().close_count, 1);

        let (reader, reader_state) = ScriptedReader::new(java("x"), 1, 0, -1, CloseMode::None);
        let (handler, handler_state) = RecordingHandler::new(false);
        handler_state.borrow_mut().unknown_runtime_fail_event = Some("documentStart");
        let payload = catch_unwind(AssertUnwindSafe(|| {
            let _ = parser.parse_document(Some(Box::new(reader)), 2, Some(Box::new(handler)));
        }))
        .expect_err("unknown panic must resume");
        assert_eq!(
            payload
                .downcast_ref::<&'static str>()
                .copied()
                .expect("original payload"),
            "unknown-handler-panic"
        );
        assert_eq!(reader_state.borrow().close_count, 1);
    }

    /// VALUE_ADD：所有 panic taxonomy 分支必须保留 Java 类别和原始 source。
    #[test]
    fn panic_taxonomy_preserves_every_java_runtime_category() {
        let variants: Vec<Box<dyn std::any::Any + Send>> = vec![
            Box::new(TextParserRuntimeError::negative_array_size(-1)),
            Box::new(TextParsingUtilError::NullText),
            Box::new(TextParsingElementError::NullArrayLoad),
            Box::new(TextParsingCommentError::NullArrayLoad),
            Box::new(ParsingLocatorError::NullLocator),
            Box::new(CommentProcessorTextHandlerRuntimeError::NullCharArrayLoad),
        ];
        for variant in variants {
            let cause = panic_payload_to_cause(variant).expect("known Java exception");
            assert!(cause.java_class_name().starts_with("java."));
        }

        let event_payload = catch_unwind(AssertUnwindSafe(|| {
            let mut handler =
                EventProcessorTextHandler::new(Some(Box::new(AbstractTextHandler::new())));
            let _ = handler.handle_open_element_start(None, 0, 1, 1, 1);
        }))
        .expect_err("null text reaches EventProcessor runtime adapter");
        assert_eq!(
            panic_payload_to_cause(event_payload)
                .expect("event runtime adapter")
                .java_class_name(),
            "java.lang.IllegalArgumentException"
        );

        let chained_payload = catch_unwind(AssertUnwindSafe(|| {
            let mut handler = AbstractChainedTextHandler::new(None);
            let _ = handler.handle_text(None, 0, 0, 1, 1);
        }))
        .expect_err("null next reaches chained runtime adapter");
        assert_eq!(
            panic_payload_to_cause(chained_payload)
                .expect("chained runtime adapter")
                .java_class_name(),
            "java.lang.NullPointerException"
        );

        assert!(panic_payload_to_cause(Box::new("rust-error")).is_err());
        assert_runtime_payload::<&'static str>(|| {
            let _ = describe_panic(Box::new("unknown"));
        });
    }

    /// VALUE_ADD：内部扫描适配必须分别保护成功、checked 和 runtime 三条路径。
    #[test]
    fn parser_helpers_keep_checked_and_runtime_channels_separate() {
        assert_eq!(util_i32_value(Ok(7)), 7);
        assert!(element_bool_value(Ok(true)));
        assert!(comment_bool_value(Ok(true)));
        assert_runtime_payload::<TextParsingUtilError>(|| {
            let _ = util_i32_value(Err(TextParsingUtilError::NullText));
        });
        assert_runtime_payload::<TextParsingElementError>(|| {
            let _ = element_bool_value(Err(TextParsingElementError::NullArrayLoad));
        });
        assert_runtime_payload::<TextParsingCommentError>(|| {
            let _ = comment_bool_value(Err(TextParsingCommentError::NullArrayLoad));
        });

        let checked = Box::new(TextParseException::with_message(Some(
            JavaString::from_rust_str("checked"),
        )));
        assert!(element_parse(Err(TextParsingElementError::TextParse(checked))).is_err());
        assert_runtime_payload::<TextParsingElementError>(|| {
            let _ = element_parse(Err(TextParsingElementError::NullArrayLoad));
        });
        let checked = Box::new(TextParseException::with_message(Some(
            JavaString::from_rust_str("checked"),
        )));
        assert!(comment_parse(Err(TextParsingCommentError::TextParse(checked))).is_err());
        assert_runtime_payload::<TextParsingCommentError>(|| {
            let _ = comment_parse(Err(TextParsingCommentError::NullArrayLoad));
        });
        assert!(element_parse(Ok(())).is_ok());
        assert!(comment_parse(Ok(())).is_ok());
        assert_runtime_payload::<ParsingLocatorError>(|| count_locator(&mut [], u16::from(b'x')));
    }

    /// SOURCE_PARITY：直接 `parseDocument` 覆盖四类结构结束和非字面量 `/` 分支，
    /// 对应 `TextParserTest` 的跨缓冲结构事件合同。
    #[test]
    fn raw_parse_document_emits_each_terminal_structure_event() {
        for document in [
            "[#x/]", "[#x]", "[/x]", "/*x*/", "//x\n", "a/", "a/b/", "'x'", "\"x\"",
        ] {
            let parser = TextParser::new(0, 64, true, false);
            let (reader, _) = ScriptedReader::new(java(document), 64, 0, -1, CloseMode::None);
            let (handler, state) = RecordingHandler::new(false);
            parser
                .parse_document(Some(Box::new(reader)), 64, Some(Box::new(handler)))
                .expect("raw structure parses");
            assert!(
                state.borrow().events.contains("documentEnd"),
                "missing terminal event for {document:?}"
            );
        }

        let (handler, _) = RecordingHandler::new(false);
        handler
            .record("manual", Some(&mut [1]), 2, 1, "invalid".to_owned())
            .expect("recording harness accepts invalid ranges");
        let mut handler = handler;
        handler
            .handle_comment(Some(&mut [1, 2, 3, 4]), 1, 1, 0, 4, 1, 1)
            .expect("comment callback is recorded");
    }

    /// SOURCE_PARITY：四类结构结束点都必须原样传播 handler checked exception。
    ///
    /// 若某一 `?` 被误删或错误被 panic 包装，本测试会观察到成功或错误 cause 改变。
    #[test]
    fn raw_structure_endpoints_propagate_checked_handler_errors() {
        for (document, fail_event) in [
            ("[#x/]", "standaloneStart"),
            ("[/x]", "closeStart"),
            ("/*x*/", "comment"),
            ("//x\n", "text"),
        ] {
            let parser = TextParser::new(0, 8, true, false);
            let (reader, _) = ScriptedReader::new(java(document), 8, 0, -1, CloseMode::None);
            let (handler, state) = RecordingHandler::new(false);
            state.borrow_mut().fail_event = Some(fail_event);
            let error = parser
                .parse_document(Some(Box::new(reader)), 8, Some(Box::new(handler)))
                .expect_err("handler checked failure propagates");
            assert_eq!(error.get_line(), Some(71));
            assert_eq!(error.get_col(), Some(72));
        }
    }

    /// RUST_OBLIGATION：扩容捕获仅吞 Java `Exception` 适配，未知 panic 必须恢复。
    #[test]
    fn buffer_growth_resumes_unknown_panic_after_releasing_candidate() {
        let parser = TextParser::new(0, 1, false, false);
        let mut buffer_size = 1;
        let mut allocated_buffer = None;
        let payload = catch_unwind(AssertUnwindSafe(|| {
            parser.grow_buffer(&mut buffer_size, 1, &mut allocated_buffer);
        }))
        .expect_err("missing old buffer is an unknown Rust invariant panic");
        assert!(payload.downcast_ref::<TextParserRuntimeError>().is_none());
        assert_eq!(buffer_size, 2);
        assert!(allocated_buffer.is_none());
    }

    fn assert_runtime_error(operation: impl FnOnce()) {
        assert_runtime_payload::<TextParserRuntimeError>(operation);
    }

    fn assert_runtime_payload<T: std::any::Any + Send>(operation: impl FnOnce()) {
        let payload = catch_unwind(AssertUnwindSafe(operation)).expect_err("operation must panic");
        assert!(payload.downcast::<T>().is_ok());
    }
}
