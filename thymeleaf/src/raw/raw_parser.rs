use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use crate::util::Utf16String;

use super::{IRawHandler, RawParseCause, RawParseException};

/// Java `Reader` 的 UTF-16 读取与关闭合同。
/// 对应 Java 语义：`RawParser` 的 Rust 侧类型 `RawReader`。
pub trait RawReader {
    /// 向目标范围读取 Java `char`，返回读取数或 `-1`。
    fn read_utf16(&mut self, buffer: &mut [u16], offset: usize, length: usize) -> io::Result<i32>;

    /// 关闭 Reader；调用方按上游语义忽略关闭错误。
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// 内存 Java String 的 Reader 实现。
/// 对应 Java 语义：`RawParser` 的 Rust 侧类型 `RawStringReader`。
pub struct RawStringReader {
    value: Utf16String,
    position: usize,
    closed: bool,
}

impl RawStringReader {
    /// 创建从字符串起点读取的 Reader。
    #[must_use]
    pub const fn new(value: Utf16String) -> Self {
        Self {
            value,
            position: 0,
            closed: false,
        }
    }
}

impl RawReader for RawStringReader {
    fn read_utf16(&mut self, buffer: &mut [u16], offset: usize, length: usize) -> io::Result<i32> {
        if self.closed {
            return Err(io::Error::other("Stream closed"));
        }
        if offset > buffer.len() || length > buffer.len().saturating_sub(offset) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "reader target range out of bounds",
            ));
        }
        if self.position >= self.value.len() {
            return Ok(-1);
        }
        let count = length.min(self.value.len() - self.position);
        if count == 0 {
            return Ok(0);
        }
        buffer[offset..offset + count]
            .copy_from_slice(&self.value.as_utf16()[self.position..self.position + count]);
        self.position += count;
        Ok(count as i32)
    }

    fn close(&mut self) -> io::Result<()> {
        self.closed = true;
        Ok(())
    }
}

/// RAW parser 公开入口的运行时参数错误或 checked 解析异常。
#[derive(Debug)]
/// 对应 Java 语义：`RawParser` 的 Rust 侧类型 `RawParserError`。
pub enum RawParserError {
    /// Java `IllegalArgumentException`。
    IllegalArgument(&'static str),
    /// Java `RawParseException`。
    Parse(RawParseException),
}

impl RawParserError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::IllegalArgument(_) => "java.lang.IllegalArgumentException",
            Self::Parse(_) => "org.thymeleaf.templateparser.raw.RawParseException",
        }
    }
}

impl Display for RawParserError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalArgument(message) => formatter.write_str(message),
            Self::Parse(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for RawParserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IllegalArgument(_) => None,
            Self::Parse(error) => Some(error),
        }
    }
}

/// 把整个资源读入可增长 UTF-16 buffer 并产生单个 text 事件的 RAW parser。
///
/// 对应 Java: `org.thymeleaf.templateparser.raw.RawParser`。
pub struct RawParser {
    pool: BufferPool,
}

impl RawParser {
    /// 创建具有非阻塞固定大小 buffer 池的解析器。
    #[must_use]
    /// 对应 Java 语义：`RawParser` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(pool_size: usize, buffer_size: usize) -> Self {
        Self {
            pool: BufferPool::new(pool_size, buffer_size),
        }
    }

    /// 解析完整 Java String。
    #[expect(
        clippy::result_large_err,
        reason = "公开 API 保留具体 Java 对照异常，不用 Box 改变调用方合同"
    )]
    /// 对应 Java 语义：Java 接口/超类方法 `parseString()` 的 Rust 移植（`RawParser` 继承路径）。
    pub fn parse_string(
        &self,
        document: Option<Utf16String>,
        handler: Option<&mut dyn IRawHandler>,
    ) -> Result<(), RawParserError> {
        let document =
            document.ok_or(RawParserError::IllegalArgument("Document cannot be null"))?;
        let mut reader = RawStringReader::new(document);
        self.parse_reader(Some(&mut reader), handler)
    }

    /// 解析 UTF-16 Reader，并始终尝试关闭它。
    #[expect(
        clippy::result_large_err,
        reason = "公开 API 保留具体 Java 对照异常，不用 Box 改变调用方合同"
    )]
    /// 对应 Java 语义：`RawParser` 的 `parse_reader` 行为（Rust 侧辅助/私有路径）。
    pub fn parse_reader(
        &self,
        reader: Option<&mut dyn RawReader>,
        handler: Option<&mut dyn IRawHandler>,
    ) -> Result<(), RawParserError> {
        let reader = reader.ok_or(RawParserError::IllegalArgument("Reader cannot be null"))?;
        let handler = handler.ok_or(RawParserError::IllegalArgument("Handler cannot be null"))?;
        self.parse_document(reader, self.pool.pool_buffer_size, handler)
            .map_err(RawParserError::Parse)
    }

    /// 使用指定建议 buffer 大小执行解析，供同包测试覆盖增长边界。
    #[expect(
        clippy::result_large_err,
        reason = "解析热路径保留具体 RawParseException 的行列与原因字段"
    )]
    /// 对应 Java: `RawParser#parseDocument()`。
    pub fn parse_document(
        &self,
        reader: &mut dyn RawReader,
        suggested_buffer_size: usize,
        handler: &mut dyn IRawHandler,
    ) -> Result<(), RawParseException> {
        let parsing_start_time_nanos = nano_time();
        let mut buffer = Vec::new();
        let mut buffer_allocated = false;

        #[expect(
            clippy::result_large_err,
            reason = "闭包与外层解析合同共享具体 RawParseException"
        )]
        let result = (|| {
            handler.handle_document_start(parsing_start_time_nanos, 1, 1)?;

            let mut buffer_size = suggested_buffer_size;
            buffer = self.pool.allocate_buffer(buffer_size);
            buffer_allocated = true;
            let first_read = reader
                .read_utf16(&mut buffer, 0, buffer_size)
                .map_err(wrap_io)?;
            let mut buffer_content_size = first_read;
            let mut cont = buffer_content_size != -1;

            while cont {
                if buffer_content_size as usize == buffer_size {
                    buffer_size = buffer_size.wrapping_mul(2);
                    let mut new_buffer = self.pool.allocate_buffer(buffer_size);
                    new_buffer[..buffer_content_size as usize]
                        .copy_from_slice(&buffer[..buffer_content_size as usize]);
                    self.pool.release_buffer(std::mem::take(&mut buffer));
                    buffer = new_buffer;
                }

                let offset = buffer_content_size as usize;
                let read = reader
                    .read_utf16(&mut buffer, offset, buffer_size - offset)
                    .map_err(wrap_io)?;
                if read != -1 {
                    buffer_content_size += read;
                } else {
                    cont = false;
                }
            }

            handler.handle_text(Some(&buffer), 0, buffer_content_size, 1, 1)?;
            if buffer_content_size < 0 {
                return Err(wrap_java_runtime(
                    "java.lang.ArrayIndexOutOfBoundsException",
                    "Index 0 out of bounds for length 0",
                ));
            }
            let (line, col) = compute_last_line_col(&buffer, buffer_content_size as usize);
            let parsing_end_time_nanos = nano_time();
            handler.handle_document_end(
                parsing_end_time_nanos,
                parsing_end_time_nanos.wrapping_sub(parsing_start_time_nanos),
                line,
                col,
            )
        })();

        if buffer_allocated {
            self.pool.release_buffer(buffer);
        }
        let _ = reader.close();
        result
    }
}

fn compute_last_line_col(buffer: &[u16], buffer_content_size: usize) -> (i32, i32) {
    if buffer_content_size == 0 {
        return (1, 1);
    }
    let mut line = 1_i32;
    let mut last_line_feed = 0_usize;
    for (index, character) in buffer[..buffer_content_size].iter().enumerate() {
        if *character == u16::from(b'\n') {
            line = line.wrapping_add(1);
            last_line_feed = index;
        }
    }
    (line, (buffer_content_size - last_line_feed) as i32)
}

fn nano_time() -> i64 {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    ORIGIN.get_or_init(Instant::now).elapsed().as_nanos() as i64
}

fn wrap_io(error: io::Error) -> RawParseException {
    let message = Utf16String::from_rust_str(&error.to_string());
    RawParseException::with_cause(Some(RawParseCause::with_java_metadata(
        Box::new(error),
        "java.io.IOException",
        Some(message),
    )))
}

fn wrap_java_runtime(class_name: &'static str, message: &'static str) -> RawParseException {
    RawParseException::with_cause(Some(RawParseCause::with_java_metadata(
        Box::new(JavaRuntimeError(message)),
        class_name,
        Some(Utf16String::from_rust_str(message)),
    )))
}

#[derive(Debug)]
struct JavaRuntimeError(&'static str);

impl Display for JavaRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl Error for JavaRuntimeError {}

struct BufferPool {
    buffers: Mutex<Vec<Vec<u16>>>,
    pool_buffer_size: usize,
}

impl BufferPool {
    fn new(pool_size: usize, pool_buffer_size: usize) -> Self {
        Self {
            buffers: Mutex::new((0..pool_size).map(|_| vec![0; pool_buffer_size]).collect()),
            pool_buffer_size,
        }
    }

    fn allocate_buffer(&self, buffer_size: usize) -> Vec<u16> {
        if buffer_size == self.pool_buffer_size
            && let Some(buffer) = lock(&self.buffers).pop()
        {
            return buffer;
        }
        vec![0; buffer_size]
    }

    fn release_buffer(&self, buffer: Vec<u16>) {
        if buffer.len() != self.pool_buffer_size {
            return;
        }
        let mut buffers = lock(&self.buffers);
        if buffers.capacity() == 0 || buffers.len() < buffers.capacity() {
            buffers.push(buffer);
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
