use std::cell::RefCell;
use std::panic::panic_any;
use std::rc::Rc;

use crate::exceptions::TemplateProcessingException;
use crate::inline::{IInlinePreProcessorHandler, OutputExpressionInlinePreProcessorHandler};
use crate::util::JavaString;
use crate::{IEngineConfiguration, TemplateMode};

use super::{ITextHandler, TextParseException};

/// 在文本 parser 事件链中执行 Standard Dialect 输出表达式预处理。
///
/// 文档与注释事件原样转发；文本、元素和属性事件交给
/// `OutputExpressionInlinePreProcessorHandler`，使 `[[...]]`/`[(...)]` 转换结果与
/// markup parser 共用同一核心算法。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.InlinedOutputExpressionTextHandler`。
pub struct InlinedOutputExpressionTextHandler {
    next: Rc<RefCell<Box<dyn ITextHandler>>>,
    inline_handler: OutputExpressionInlinePreProcessorHandler,
}

impl InlinedOutputExpressionTextHandler {
    /// 创建文本输出表达式 handler。
    ///
    /// 对应 Java: `InlinedOutputExpressionTextHandler#InlinedOutputExpressionTextHandler`。
    pub fn new(
        configuration: &dyn IEngineConfiguration,
        template_mode: TemplateMode,
        standard_dialect_prefix: Option<&JavaString>,
        next: Box<dyn ITextHandler>,
    ) -> Result<Self, TemplateProcessingException> {
        let next = Rc::new(RefCell::new(next));
        let adapter = InlineTextAdapterPreProcessorHandler { next: next.clone() };
        let inline_handler = OutputExpressionInlinePreProcessorHandler::new(
            configuration,
            template_mode,
            standard_dialect_prefix,
            Box::new(adapter),
        )?;
        Ok(Self {
            next,
            inline_handler,
        })
    }
}

impl ITextHandler for InlinedOutputExpressionTextHandler {
    fn handle_document_start(
        &mut self,
        start_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.next
            .borrow_mut()
            .handle_document_start(start_time_nanos, line, col)
    }

    fn handle_document_end(
        &mut self,
        end_time_nanos: i64,
        total_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.next
            .borrow_mut()
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
        self.inline_handler
            .handle_text(buffer, offset, len, line, col);
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
        self.next.borrow_mut().handle_comment(
            buffer,
            content_offset,
            content_len,
            outer_offset,
            outer_len,
            line,
            col,
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
        self.inline_handler.handle_standalone_element_start(
            buffer,
            name_offset,
            name_len,
            minimized,
            line,
            col,
        );
        Ok(())
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
        self.inline_handler.handle_standalone_element_end(
            buffer,
            name_offset,
            name_len,
            minimized,
            line,
            col,
        );
        Ok(())
    }
    fn handle_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.inline_handler
            .handle_open_element_start(buffer, name_offset, name_len, line, col);
        Ok(())
    }
    fn handle_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.inline_handler
            .handle_open_element_end(buffer, name_offset, name_len, line, col);
        Ok(())
    }
    fn handle_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.inline_handler
            .handle_close_element_start(buffer, name_offset, name_len, line, col);
        Ok(())
    }
    fn handle_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.inline_handler
            .handle_close_element_end(buffer, name_offset, name_len, line, col);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
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
        self.inline_handler.handle_attribute(
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
        );
        Ok(())
    }
}

struct InlineTextAdapterPreProcessorHandler {
    next: Rc<RefCell<Box<dyn ITextHandler>>>,
}

impl InlineTextAdapterPreProcessorHandler {
    fn forward(&self, result: Result<(), Box<TextParseException>>) {
        if let Err(error) = result {
            panic_any(TemplateProcessingException::new(Some(format!(
                "Parse exception during processing of inlining: {error}"
            ))));
        }
    }
}

impl IInlinePreProcessorHandler for InlineTextAdapterPreProcessorHandler {
    fn handle_text(
        &mut self,
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) {
        let result = self
            .next
            .borrow_mut()
            .handle_text(buffer, offset, len, line, col);
        self.forward(result);
    }
    fn handle_standalone_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        minimized: bool,
        line: i32,
        col: i32,
    ) {
        let result = self.next.borrow_mut().handle_standalone_element_start(
            buffer,
            name_offset,
            name_len,
            minimized,
            line,
            col,
        );
        self.forward(result);
    }
    fn handle_standalone_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        minimized: bool,
        line: i32,
        col: i32,
    ) {
        let result = self.next.borrow_mut().handle_standalone_element_end(
            buffer,
            name_offset,
            name_len,
            minimized,
            line,
            col,
        );
        self.forward(result);
    }
    fn handle_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        let result = self.next.borrow_mut().handle_open_element_start(
            buffer,
            name_offset,
            name_len,
            line,
            col,
        );
        self.forward(result);
    }
    fn handle_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        let result = self.next.borrow_mut().handle_open_element_end(
            buffer,
            name_offset,
            name_len,
            line,
            col,
        );
        self.forward(result);
    }
    fn handle_auto_open_element_start(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _line: i32,
        _col: i32,
    ) {
        panic_any(TemplateProcessingException::new(Some(
            "Parse exception during processing of inlining: auto-open not allowed in text mode"
                .to_owned(),
        )));
    }
    fn handle_auto_open_element_end(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _line: i32,
        _col: i32,
    ) {
        panic_any(TemplateProcessingException::new(Some(
            "Parse exception during processing of inlining: auto-open not allowed in text mode"
                .to_owned(),
        )));
    }
    fn handle_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        let result = self.next.borrow_mut().handle_close_element_start(
            buffer,
            name_offset,
            name_len,
            line,
            col,
        );
        self.forward(result);
    }
    fn handle_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        let result = self.next.borrow_mut().handle_close_element_end(
            buffer,
            name_offset,
            name_len,
            line,
            col,
        );
        self.forward(result);
    }
    fn handle_auto_close_element_start(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _line: i32,
        _col: i32,
    ) {
        panic_any(TemplateProcessingException::new(Some(
            "Parse exception during processing of inlining: auto-close not allowed in text mode"
                .to_owned(),
        )));
    }
    fn handle_auto_close_element_end(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _line: i32,
        _col: i32,
    ) {
        panic_any(TemplateProcessingException::new(Some(
            "Parse exception during processing of inlining: auto-close not allowed in text mode"
                .to_owned(),
        )));
    }
    #[allow(clippy::too_many_arguments)]
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
    ) {
        let result = self.next.borrow_mut().handle_attribute(
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
        );
        self.forward(result);
    }
}
