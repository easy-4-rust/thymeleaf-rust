use std::sync::Arc;

use crate::exceptions::TemplateEngineException;
use crate::raw::{IRawHandler, RawParseCause, RawParseException};
use crate::util::Utf16String;

use super::{ITemplateHandler, TemplateEnd, TemplateStart, Text};

/// 将 RAW parser 回调转换为标准模板事件并发送到 Handler 链。
///
/// parser 的解析耗时被忽略；正数行列 offset 先减一，使嵌入模板的解析行 1 对齐调用
/// 方指定位置。首行同时应用列 offset，后续行列号保持 parser 原值。
///
/// 对应 Java: `org.thymeleaf.engine.TemplateHandlerAdapterRawHandler`。
pub struct TemplateHandlerAdapterRawHandler {
    template_name: Option<Utf16String>,
    template_handler: Box<dyn ITemplateHandler>,
    line_offset: i32,
    col_offset: i32,
}

impl TemplateHandlerAdapterRawHandler {
    /// 创建 RAW parser 到模板 Handler 的适配器。
    ///
    /// # 参数
    /// - `template_name`：事件位置使用的可空模板名；
    /// - `template_handler`：接收转换后事件的非空 Handler；
    /// - `line_offset`、`col_offset`：嵌入模板起始偏移。
    #[must_use]
    ///
    /// 对应 Java 语义：`TemplateHandlerAdapterRawHandler` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_name: Option<Utf16String>,
        template_handler: Box<dyn ITemplateHandler>,
        line_offset: i32,
        col_offset: i32,
    ) -> Self {
        Self {
            template_name,
            template_handler,
            line_offset: if line_offset > 0 {
                line_offset - 1
            } else {
                line_offset
            },
            col_offset: if col_offset > 0 {
                col_offset - 1
            } else {
                col_offset
            },
        }
    }
}

impl IRawHandler for TemplateHandlerAdapterRawHandler {
    fn handle_document_start(
        &mut self,
        _start_time_nanos: i64,
        _line: i32,
        _col: i32,
    ) -> Result<(), RawParseException> {
        self.template_handler
            .handle_template_start(TemplateStart::instance())
            .map_err(handler_error)
    }

    fn handle_document_end(
        &mut self,
        _end_time_nanos: i64,
        _total_time_nanos: i64,
        _line: i32,
        _col: i32,
    ) -> Result<(), RawParseException> {
        self.template_handler
            .handle_template_end(TemplateEnd::instance())
            .map_err(handler_error)
    }

    fn handle_text(
        &mut self,
        buffer: Option<&[u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), RawParseException> {
        let buffer = buffer.ok_or_else(|| {
            RawParseException::with_message(Some(Utf16String::from_rust_str(
                "Text buffer cannot be null",
            )))
        })?;
        let start = usize::try_from(offset).map_err(|_| invalid_text_range(line, col))?;
        let len = usize::try_from(len).map_err(|_| invalid_text_range(line, col))?;
        let end = start
            .checked_add(len)
            .filter(|end| *end <= buffer.len())
            .ok_or_else(|| invalid_text_range(line, col))?;
        let text = Utf16String::from_utf16(buffer[start..end].to_vec());
        self.template_handler
            .handle_text(Arc::new(Text::with_location(
                Some(Arc::new(text)),
                self.template_name.clone(),
                self.line_offset.wrapping_add(line),
                (if line == 1 { self.col_offset } else { 0 }).wrapping_add(col),
            )))
            .map_err(handler_error)
    }
}

fn invalid_text_range(line: i32, col: i32) -> RawParseException {
    RawParseException::with_message_at(
        Some(&Utf16String::from_rust_str("Invalid text buffer range")),
        line,
        col,
    )
}

fn handler_error(error: Box<dyn TemplateEngineException>) -> RawParseException {
    let message = Some(Utf16String::from_rust_str(&error.to_string()));
    let cause = RawParseCause::with_java_metadata(
        error,
        "org.thymeleaf.exceptions.TemplateEngineException",
        message,
    );
    RawParseException::with_cause(Some(cause))
}
