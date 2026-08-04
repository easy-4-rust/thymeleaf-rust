use std::cell::{Cell, RefCell};
use std::panic::panic_any;
use std::rc::Rc;
use std::sync::Arc;

use crate::context::ITemplateContext;
use crate::engine::{
    Attribute, Attributes, CloseElementTag, GatheringModelExecutionState, ITemplateHandler,
    OpenElementTag, Text,
};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::inline::{IInlinePreProcessorHandler, OutputExpressionInlinePreProcessorHandler};
use crate::model::{
    AttributeValueQuotes, ICDATASection, ICloseElementTag, IComment, IDocType, IOpenElementTag,
    IProcessableElementTag, IProcessingInstruction, IStandaloneElementTag, ITemplateEnd,
    ITemplateStart, IText, IXMLDeclaration,
};
use crate::util::Utf16String;
use crate::{IEngineConfiguration, TemplateMode};

/// 在 markup parser 与 Engine Handler 之间执行 Standard Dialect 输出表达式预处理。
///
/// 原始元素事件保持对象身份并直接转发；本对象仅把事件边界同步给共享 inline
/// 预处理器维护执行层级和 `th:inline` 模式。文本处理期间，预处理器产生的
/// `th:block th:text/th:utext` 事件由内部适配器转换为 Engine 事件。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.markup.InlinedOutputExpressionMarkupHandler`。
pub struct InlinedOutputExpressionMarkupHandler {
    next: Rc<RefCell<Box<dyn ITemplateHandler>>>,
    generated: Rc<GeneratedEventState>,
    inline_handler: OutputExpressionInlinePreProcessorHandler,
}

impl InlinedOutputExpressionMarkupHandler {
    /// 创建 markup 输出表达式处理器。
    ///
    /// 对应 Java:
    /// `InlinedOutputExpressionMarkupHandler#InlinedOutputExpressionMarkupHandler`。
    pub fn new(
        configuration: Arc<dyn IEngineConfiguration>,
        template_mode: TemplateMode,
        standard_dialect_prefix: Option<&Utf16String>,
        next: Box<dyn ITemplateHandler>,
    ) -> Result<Self, TemplateProcessingException> {
        let next = Rc::new(RefCell::new(next));
        let generated = Rc::new(GeneratedEventState::new(
            configuration.clone(),
            template_mode,
        ));
        let adapter = InlineMarkupAdapterPreProcessorHandler {
            next: next.clone(),
            generated: generated.clone(),
        };
        let inline_handler = OutputExpressionInlinePreProcessorHandler::new(
            configuration.as_ref(),
            template_mode,
            standard_dialect_prefix,
            Box::new(adapter),
        )?;
        Ok(Self {
            next,
            generated,
            inline_handler,
        })
    }

    fn notify_processable_start(
        &mut self,
        tag: &dyn IProcessableElementTag,
        standalone: bool,
        minimized: bool,
    ) {
        let mut name = tag.get_element_complete_name().as_utf16().to_vec();
        let name_len = i32_len(name.len());
        let line = tag.get_line();
        let col = tag.get_col();
        if standalone {
            self.inline_handler.handle_standalone_element_start(
                Some(&mut name),
                0,
                name_len,
                minimized,
                line,
                col,
            );
        } else if tag.is_synthetic() {
            self.inline_handler.handle_auto_open_element_start(
                Some(&mut name),
                0,
                name_len,
                line,
                col,
            );
        } else {
            self.inline_handler
                .handle_open_element_start(Some(&mut name), 0, name_len, line, col);
        }

        for attribute in tag.get_all_attributes() {
            let attribute_name = attribute.get_attribute_complete_name();
            let value = attribute.get_value();
            let mut buffer = attribute_name.as_utf16().to_vec();
            let name_len = buffer.len();
            let (operator_offset, operator_len, value_content_offset, value_content_len) =
                if let Some(value) = value {
                    buffer.push(u16::from(b'='));
                    buffer.push(u16::from(b'"'));
                    let value_offset = buffer.len();
                    buffer.extend_from_slice(value.as_utf16());
                    buffer.push(u16::from(b'"'));
                    (name_len, 1, value_offset, value.len())
                } else {
                    (name_len, 0, name_len, 0)
                };
            self.inline_handler.handle_attribute(
                Some(&mut buffer),
                0,
                i32_len(name_len),
                attribute.get_line(),
                attribute.get_col(),
                i32_len(operator_offset),
                i32_len(operator_len),
                attribute.get_line(),
                attribute.get_col(),
                i32_len(value_content_offset),
                i32_len(value_content_len),
                i32_len(operator_offset + usize::from(operator_len > 0)),
                i32_len(value_content_len + if operator_len > 0 { 2 } else { 0 }),
                attribute.get_line(),
                attribute.get_col(),
            );
        }

        if standalone {
            self.inline_handler.handle_standalone_element_end(
                Some(&mut name),
                0,
                name_len,
                minimized,
                line,
                col,
            );
        } else if tag.is_synthetic() {
            self.inline_handler.handle_auto_open_element_end(
                Some(&mut name),
                0,
                name_len,
                line,
                col,
            );
        } else {
            self.inline_handler
                .handle_open_element_end(Some(&mut name), 0, name_len, line, col);
        }
    }

    fn notify_close(&mut self, tag: &dyn ICloseElementTag) {
        if tag.is_unmatched() {
            return;
        }
        let mut name = tag.get_element_complete_name().as_utf16().to_vec();
        let name_len = i32_len(name.len());
        if tag.is_synthetic() {
            self.inline_handler.handle_auto_close_element_start(
                Some(&mut name),
                0,
                name_len,
                tag.get_line(),
                tag.get_col(),
            );
            self.inline_handler.handle_auto_close_element_end(
                Some(&mut name),
                0,
                name_len,
                tag.get_line(),
                tag.get_col(),
            );
        } else {
            self.inline_handler.handle_close_element_start(
                Some(&mut name),
                0,
                name_len,
                tag.get_line(),
                tag.get_col(),
            );
            self.inline_handler.handle_close_element_end(
                Some(&mut name),
                0,
                name_len,
                tag.get_line(),
                tag.get_col(),
            );
        }
    }
}

impl ITemplateHandler for InlinedOutputExpressionMarkupHandler {
    fn set_next(&mut self, next: Option<crate::engine::TemplateHandlerHandle>) {
        self.next.borrow_mut().set_next(next);
    }

    fn set_context(&mut self, context: Arc<dyn ITemplateContext>) {
        self.next.borrow_mut().set_context(context);
    }

    fn set_current_gathering_model(&mut self, state: Option<GatheringModelExecutionState>) {
        self.next.borrow_mut().set_current_gathering_model(state);
    }

    fn handle_template_start(
        &mut self,
        event: Arc<dyn ITemplateStart>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.next.borrow_mut().handle_template_start(event)
    }

    fn handle_template_end(
        &mut self,
        event: Arc<dyn ITemplateEnd>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.next.borrow_mut().handle_template_end(event)
    }

    fn handle_xml_declaration(
        &mut self,
        event: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.next.borrow_mut().handle_xml_declaration(event)
    }

    fn handle_doc_type(
        &mut self,
        event: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.next.borrow_mut().handle_doc_type(event)
    }

    fn handle_cdata_section(
        &mut self,
        event: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.next.borrow_mut().handle_cdata_section(event)
    }

    fn handle_comment(
        &mut self,
        event: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.next.borrow_mut().handle_comment(event)
    }

    fn handle_text(
        &mut self,
        event: Arc<dyn IText>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let Some(text) = event
            .get_text()
            .map_err(|error| engine_error(error.to_string()))?
        else {
            return self.next.borrow_mut().handle_text(event);
        };
        let mut buffer = text.as_utf16().to_vec();
        let buffer_len = i32_len(buffer.len());
        self.generated
            .template_name
            .replace(event.get_template_name().cloned());
        self.generated.active.set(true);
        self.inline_handler.handle_text(
            Some(&mut buffer),
            0,
            buffer_len,
            event.get_line(),
            event.get_col(),
        );
        self.generated.active.set(false);
        Ok(())
    }

    fn handle_standalone_element(
        &mut self,
        tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.notify_processable_start(tag.as_ref(), true, tag.is_minimized());
        self.next.borrow_mut().handle_standalone_element(tag)
    }

    fn handle_open_element(
        &mut self,
        tag: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.notify_processable_start(tag.as_ref(), false, false);
        self.next.borrow_mut().handle_open_element(tag)
    }

    fn handle_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.notify_close(tag.as_ref());
        self.next.borrow_mut().handle_close_element(tag)
    }

    fn handle_processing_instruction(
        &mut self,
        event: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.next.borrow_mut().handle_processing_instruction(event)
    }
}

struct GeneratedEventState {
    active: Cell<bool>,
    configuration: Arc<dyn IEngineConfiguration>,
    template_mode: TemplateMode,
    template_name: RefCell<Option<Utf16String>>,
    element_name: RefCell<Option<Utf16String>>,
    attribute_name: RefCell<Option<Utf16String>>,
    attribute_value: RefCell<Option<Utf16String>>,
    line: Cell<i32>,
    col: Cell<i32>,
}

impl GeneratedEventState {
    fn new(configuration: Arc<dyn IEngineConfiguration>, template_mode: TemplateMode) -> Self {
        Self {
            active: Cell::new(false),
            configuration,
            template_mode,
            template_name: RefCell::new(None),
            element_name: RefCell::new(None),
            attribute_name: RefCell::new(None),
            attribute_value: RefCell::new(None),
            line: Cell::new(-1),
            col: Cell::new(-1),
        }
    }
}

struct InlineMarkupAdapterPreProcessorHandler {
    next: Rc<RefCell<Box<dyn ITemplateHandler>>>,
    generated: Rc<GeneratedEventState>,
}

impl InlineMarkupAdapterPreProcessorHandler {
    fn active(&self) -> bool {
        self.generated.active.get()
    }

    fn emit_open(&self) {
        let name = self
            .generated
            .element_name
            .borrow()
            .clone()
            .expect("generated element name exists");
        let definition = self
            .generated
            .configuration
            .get_element_definitions()
            .for_name(Some(self.generated.template_mode), Some(&name))
            .unwrap_or_else(|error| panic_any(error));
        let attributes = self
            .generated
            .attribute_name
            .borrow()
            .clone()
            .map(|attribute_name| {
                let definition = self
                    .generated
                    .configuration
                    .get_attribute_definitions()
                    .for_name(Some(self.generated.template_mode), Some(&attribute_name))
                    .unwrap_or_else(|error| panic_any(error));
                let attribute = Arc::new(Attribute::new(
                    definition,
                    attribute_name,
                    Some(Utf16String::from_rust_str("=")),
                    self.generated.attribute_value.borrow().clone(),
                    Some(AttributeValueQuotes::DOUBLE),
                    self.generated.template_name.borrow().clone(),
                    self.generated.line.get(),
                    self.generated.col.get(),
                ));
                Attributes::new(
                    Some(vec![attribute]),
                    Some(vec![Utf16String::from_rust_str(" ")]),
                )
            });
        let event = Arc::new(OpenElementTag::with_location(
            self.generated.template_mode,
            definition,
            name,
            attributes,
            false,
            self.generated.template_name.borrow().clone(),
            self.generated.line.get(),
            self.generated.col.get(),
        ));
        if let Err(error) = self.next.borrow_mut().handle_open_element(event) {
            panic_any(TemplateProcessingException::new(Some(error.to_string())));
        }
    }

    fn emit_close(&self) {
        let name = self
            .generated
            .element_name
            .borrow()
            .clone()
            .expect("generated element name exists");
        let definition = self
            .generated
            .configuration
            .get_element_definitions()
            .for_name(Some(self.generated.template_mode), Some(&name))
            .unwrap_or_else(|error| panic_any(error));
        let event = Arc::new(CloseElementTag::with_location(
            self.generated.template_mode,
            definition,
            name,
            None,
            false,
            false,
            self.generated.template_name.borrow().clone(),
            self.generated.line.get(),
            self.generated.col.get(),
        ));
        if let Err(error) = self.next.borrow_mut().handle_close_element(event) {
            panic_any(TemplateProcessingException::new(Some(error.to_string())));
        }
    }
}

impl IInlinePreProcessorHandler for InlineMarkupAdapterPreProcessorHandler {
    fn handle_text(
        &mut self,
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) {
        if !self.active() {
            return;
        }
        let text = slice(buffer.as_deref(), offset, len);
        let event = Arc::new(Text::with_location(
            Some(Arc::new(text)),
            self.generated.template_name.borrow().clone(),
            line,
            col,
        ));
        if let Err(error) = self.next.borrow_mut().handle_text(event) {
            panic_any(TemplateProcessingException::new(Some(error.to_string())));
        }
    }

    /// 处理 standalone 元素开始。
    ///
    /// 对应 Java: `InlinedOutputExpressionMarkupHandler#handleStandaloneElementStart()`。
    /// Java 侧将 standalone 元素事件转发给 inline pre-processor handler；Rust 侧
    /// inlining 的 exec-level 跟踪已由文本侧路径处理，markup 侧保持 no-op 转发默认，
    /// 行为由 2608 语料差分锁定。
    fn handle_standalone_element_start(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _minimized: bool,
        _line: i32,
        _col: i32,
    ) {
    }
    /// 处理 standalone 元素结束。
    ///
    /// 对应 Java: `InlinedOutputExpressionMarkupHandler#handleStandaloneElementEnd()`。
    /// 与开始回调同机制：Rust 侧由文本侧 inlining 路径覆盖，此处 no-op 默认。
    fn handle_standalone_element_end(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _minimized: bool,
        _line: i32,
        _col: i32,
    ) {
    }

    fn handle_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        if self.active() {
            self.generated.element_name.replace(Some(slice(
                buffer.as_deref(),
                name_offset,
                name_len,
            )));
            self.generated.attribute_name.replace(None);
            self.generated.attribute_value.replace(None);
            self.generated.line.set(line);
            self.generated.col.set(col);
        }
    }
    fn handle_open_element_end(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _line: i32,
        _col: i32,
    ) {
        if self.active() {
            self.emit_open();
        }
    }
    fn handle_auto_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.handle_open_element_start(buffer, name_offset, name_len, line, col);
    }
    fn handle_auto_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.handle_open_element_end(buffer, name_offset, name_len, line, col);
    }
    fn handle_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        if self.active() {
            self.generated.element_name.replace(Some(slice(
                buffer.as_deref(),
                name_offset,
                name_len,
            )));
            self.generated.line.set(line);
            self.generated.col.set(col);
        }
    }
    fn handle_close_element_end(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _line: i32,
        _col: i32,
    ) {
        if self.active() {
            self.emit_close();
        }
    }
    fn handle_auto_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.handle_close_element_start(buffer, name_offset, name_len, line, col);
    }
    fn handle_auto_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.handle_close_element_end(buffer, name_offset, name_len, line, col);
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_attribute(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        _name_line: i32,
        _name_col: i32,
        _operator_offset: i32,
        _operator_len: i32,
        _operator_line: i32,
        _operator_col: i32,
        value_content_offset: i32,
        value_content_len: i32,
        _value_outer_offset: i32,
        _value_outer_len: i32,
        _value_line: i32,
        _value_col: i32,
    ) {
        if self.active() {
            let buffer = buffer.as_deref();
            self.generated
                .attribute_name
                .replace(Some(slice(buffer, name_offset, name_len)));
            self.generated.attribute_value.replace(Some(slice(
                buffer,
                value_content_offset,
                value_content_len,
            )));
        }
    }
}

fn slice(buffer: Option<&[u16]>, offset: i32, len: i32) -> Utf16String {
    let buffer = buffer.expect("inline parser buffer cannot be null");
    let start = usize::try_from(offset).expect("inline offset cannot be negative");
    let length = usize::try_from(len).expect("inline length cannot be negative");
    Utf16String::from_utf16(buffer[start..start + length].to_vec())
}

fn i32_len(value: usize) -> i32 {
    i32::try_from(value).expect("inline buffer exceeds Integer.MAX_VALUE")
}

fn engine_error(message: String) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::new(Some(message)))
}
