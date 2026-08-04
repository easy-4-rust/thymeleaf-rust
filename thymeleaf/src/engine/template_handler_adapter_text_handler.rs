use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::TemplateMode;
use crate::exceptions::TemplateEngineException;
use crate::model::AttributeValueQuotes;
use crate::text::{ITextHandler, TextParseCause, TextParseException};
use crate::util::Utf16String;

use super::{
    Attribute, Attributes, CloseElementTag, ITemplateHandler, OpenElementTag, StandaloneElementTag,
    TemplateEnd, TemplateStart, Text,
};

const DEFAULT_OPERATOR: &[u16] = &[0x003D];

/// 将 TEXT/JAVASCRIPT/CSS parser 回调转换为标准 Engine 模板事件。
///
/// 元素开始回调只保存源位置并清空属性；元素结束回调统一解析 ElementDefinition、
/// 组装合成属性空白并发送不可变标签。正数偏移先减一，因而嵌入模板的第一行和
/// 第一列与 Java 版完全相同。
///
/// 对应 Java: `org.thymeleaf.engine.TemplateHandlerAdapterTextHandler`。
pub struct TemplateHandlerAdapterTextHandler {
    template_name: Option<Utf16String>,
    template_handler: Box<dyn ITemplateHandler>,
    configuration: Arc<dyn IEngineConfiguration>,
    template_mode: TemplateMode,
    line_offset: i32,
    col_offset: i32,
    current_element_line: i32,
    current_element_col: i32,
    current_element_attributes: Vec<Arc<Attribute>>,
}

impl TemplateHandlerAdapterTextHandler {
    /// 创建文本 parser 到 Engine Handler 的适配器。
    ///
    /// `configuration` 持有 Java 中 ElementDefinitions 与 AttributeDefinitions 两个
    /// repository 的共同所有者，避免 Rust 借用越过 parser handler 生命周期。
    #[must_use]
    /// 对应 Java 语义：`TemplateHandlerAdapterTextHandler` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_name: Option<Utf16String>,
        template_handler: Box<dyn ITemplateHandler>,
        configuration: Arc<dyn IEngineConfiguration>,
        template_mode: TemplateMode,
        line_offset: i32,
        col_offset: i32,
    ) -> Self {
        Self {
            template_name,
            template_handler,
            configuration,
            template_mode,
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
            current_element_line: -1,
            current_element_col: -1,
            current_element_attributes: Vec::with_capacity(10),
        }
    }

    fn location(&self, line: i32, col: i32) -> (i32, i32) {
        (
            self.line_offset.wrapping_add(line),
            (if line == 1 { self.col_offset } else { 0 }).wrapping_add(col),
        )
    }

    fn start_element(&mut self, line: i32, col: i32) {
        self.current_element_line = line;
        self.current_element_col = col;
        self.current_element_attributes.clear();
    }

    fn attributes(&self) -> Option<Arc<Attributes>> {
        if self.current_element_attributes.is_empty() {
            return None;
        }
        let spaces = vec![Utf16String::from_rust_str(" "); self.current_element_attributes.len()];
        Some(Attributes::new(
            Some(self.current_element_attributes.clone()),
            Some(spaces),
        ))
    }
}

impl ITextHandler for TemplateHandlerAdapterTextHandler {
    fn handle_document_start(
        &mut self,
        _start_time_nanos: i64,
        _line: i32,
        _col: i32,
    ) -> Result<(), Box<TextParseException>> {
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
    ) -> Result<(), Box<TextParseException>> {
        self.template_handler
            .handle_template_end(TemplateEnd::instance())
            .map_err(handler_error)
    }

    fn handle_text(
        &mut self,
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        let text = slice(buffer.as_deref(), offset, len, line, col)?;
        let (line, col) = self.location(line, col);
        self.template_handler
            .handle_text(Arc::new(Text::with_location(
                Some(Arc::new(text)),
                self.template_name.clone(),
                line,
                col,
            )))
            .map_err(handler_error)
    }

    fn handle_comment(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _content_offset: i32,
        _content_len: i32,
        _outer_offset: i32,
        _outer_len: i32,
        _line: i32,
        _col: i32,
    ) -> Result<(), Box<TextParseException>> {
        Ok(())
    }

    fn handle_standalone_element_start(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _minimized: bool,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.start_element(line, col);
        Ok(())
    }

    fn handle_standalone_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        minimized: bool,
        _line: i32,
        _col: i32,
    ) -> Result<(), Box<TextParseException>> {
        let name = slice(
            buffer.as_deref(),
            name_offset,
            name_len,
            self.current_element_line,
            self.current_element_col,
        )?;
        let definition = self
            .configuration
            .get_element_definitions()
            .for_name(Some(self.template_mode), Some(&name))
            .map_err(definition_error)?;
        let (line, col) = self.location(self.current_element_line, self.current_element_col);
        let tag = StandaloneElementTag::with_location(
            self.template_mode,
            definition,
            name,
            self.attributes(),
            false,
            minimized,
            self.template_name.clone(),
            line,
            col,
        )
        .map_err(|error| parse_cause(error, line, col))?;
        self.template_handler
            .handle_standalone_element(Arc::new(tag))
            .map_err(handler_error)
    }

    fn handle_open_element_start(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.start_element(line, col);
        Ok(())
    }

    fn handle_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        _line: i32,
        _col: i32,
    ) -> Result<(), Box<TextParseException>> {
        let name = slice(
            buffer.as_deref(),
            name_offset,
            name_len,
            self.current_element_line,
            self.current_element_col,
        )?;
        let definition = self
            .configuration
            .get_element_definitions()
            .for_name(Some(self.template_mode), Some(&name))
            .map_err(definition_error)?;
        let (line, col) = self.location(self.current_element_line, self.current_element_col);
        self.template_handler
            .handle_open_element(Arc::new(OpenElementTag::with_location(
                self.template_mode,
                definition,
                name,
                self.attributes(),
                false,
                self.template_name.clone(),
                line,
                col,
            )))
            .map_err(handler_error)
    }

    fn handle_close_element_start(
        &mut self,
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.start_element(line, col);
        Ok(())
    }

    fn handle_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        _line: i32,
        _col: i32,
    ) -> Result<(), Box<TextParseException>> {
        let name = slice(
            buffer.as_deref(),
            name_offset,
            name_len,
            self.current_element_line,
            self.current_element_col,
        )?;
        let definition = self
            .configuration
            .get_element_definitions()
            .for_name(Some(self.template_mode), Some(&name))
            .map_err(definition_error)?;
        let (line, col) = self.location(self.current_element_line, self.current_element_col);
        self.template_handler
            .handle_close_element(Arc::new(CloseElementTag::with_location(
                self.template_mode,
                definition,
                name,
                None,
                false,
                false,
                self.template_name.clone(),
                line,
                col,
            )))
            .map_err(handler_error)
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
        _operator_line: i32,
        _operator_col: i32,
        value_content_offset: i32,
        value_content_len: i32,
        value_outer_offset: i32,
        _value_outer_len: i32,
        _value_line: i32,
        _value_col: i32,
    ) -> Result<(), Box<TextParseException>> {
        let buffer = buffer.as_deref();
        let name = slice(buffer, name_offset, name_len, name_line, name_col)?;
        let definition = self
            .configuration
            .get_attribute_definitions()
            .for_name(Some(self.template_mode), Some(&name))
            .map_err(definition_error)?;
        let operator = if operator_len > 0 {
            let value = slice(buffer, operator_offset, operator_len, name_line, name_col)?;
            Some(if value.as_utf16() == DEFAULT_OPERATOR {
                Utf16String::from_rust_str("=")
            } else {
                value
            })
        } else {
            None
        };
        let value = operator
            .as_ref()
            .map(|_| {
                slice(
                    buffer,
                    value_content_offset,
                    value_content_len,
                    name_line,
                    name_col,
                )
            })
            .transpose()?;
        let quotes = value.as_ref().map(|_| {
            if value_outer_offset == value_content_offset {
                AttributeValueQuotes::NONE
            } else {
                match buffer.and_then(|buffer| {
                    usize::try_from(value_outer_offset)
                        .ok()
                        .and_then(|index| buffer.get(index).copied())
                }) {
                    Some(value) if value == u16::from(b'"') => AttributeValueQuotes::DOUBLE,
                    Some(value) if value == u16::from(b'\'') => AttributeValueQuotes::SINGLE,
                    _ => AttributeValueQuotes::NONE,
                }
            }
        });
        let (line, col) = self.location(name_line, name_col);
        self.current_element_attributes
            .push(Arc::new(Attribute::new(
                definition,
                name,
                operator,
                value,
                quotes,
                self.template_name.clone(),
                line,
                col,
            )));
        Ok(())
    }
}

fn slice(
    buffer: Option<&[u16]>,
    offset: i32,
    len: i32,
    line: i32,
    col: i32,
) -> Result<Utf16String, Box<TextParseException>> {
    let buffer = buffer.ok_or_else(|| invalid_range(line, col))?;
    let start = usize::try_from(offset).map_err(|_| invalid_range(line, col))?;
    let len = usize::try_from(len).map_err(|_| invalid_range(line, col))?;
    let end = start
        .checked_add(len)
        .filter(|end| *end <= buffer.len())
        .ok_or_else(|| invalid_range(line, col))?;
    Ok(Utf16String::from_utf16(buffer[start..end].to_vec()))
}

fn invalid_range(line: i32, col: i32) -> Box<TextParseException> {
    Box::new(TextParseException::with_message_at(
        Some(&Utf16String::from_rust_str("Invalid text buffer range")),
        line,
        col,
    ))
}

fn handler_error(error: Box<dyn TemplateEngineException>) -> Box<TextParseException> {
    Box::new(TextParseException::with_cause(Some(
        TextParseCause::with_java_metadata(
            error,
            "org.thymeleaf.exceptions.TemplateEngineException",
            None,
        ),
    )))
}

fn definition_error<E>(error: E) -> Box<TextParseException>
where
    E: std::error::Error + Send + Sync + 'static,
{
    Box::new(TextParseException::with_cause(Some(
        TextParseCause::with_java_metadata(
            Box::new(error),
            "java.lang.IllegalArgumentException",
            None,
        ),
    )))
}

fn parse_cause<E>(error: E, line: i32, col: i32) -> Box<TextParseException>
where
    E: std::error::Error + Send + Sync + 'static,
{
    Box::new(TextParseException::with_cause_at(
        Some(TextParseCause::with_java_metadata(
            Box::new(error),
            "java.lang.IllegalArgumentException",
            None,
        )),
        line,
        col,
    ))
}
