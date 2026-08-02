use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::TemplateMode;
use crate::decoupled::DecoupledInjectedAttribute;
use crate::exceptions::TemplateInputException;
use crate::model::AttributeValueQuotes;
use crate::templateparser::TemplateParserError;
use crate::util::{JavaCharSequence, JavaString};

use super::{
    Attribute, Attributes, CDATASection, CloseElementTag, Comment, DocType, ITemplateHandler,
    OpenElementTag, ProcessingInstruction, StandaloneElementTag, TemplateEnd, TemplateStart, Text,
    XMLDeclaration,
};

/// 将 HTML/XML 标记解析事件转换为 Engine 模板事件。
///
/// 本对象保留标签名大小写、属性操作符、属性值引号、属性间空白、合成/未匹配标志与
/// 源码位置。正数行列偏移先减一，使嵌入模板第一行、第一列的定位与 Java 完全一致。
///
/// 对应 Java: `org.thymeleaf.engine.TemplateHandlerAdapterMarkupHandler`。
pub struct TemplateHandlerAdapterMarkupHandler {
    template_name: Option<JavaString>,
    template_handler: Box<dyn ITemplateHandler>,
    configuration: Arc<dyn IEngineConfiguration>,
    template_mode: TemplateMode,
    line_offset: i32,
    col_offset: i32,
}

impl TemplateHandlerAdapterMarkupHandler {
    /// 创建标记 parser 到 Engine Handler 的适配器。
    ///
    /// 对应 Java: `TemplateHandlerAdapterMarkupHandler#TemplateHandlerAdapterMarkupHandler`。
    #[must_use]
    pub fn new(
        template_name: Option<JavaString>,
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
        }
    }

    /// 发送文档开始事件；parser 计时信息与 Java 版一样被忽略。
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `document_start` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn document_start(&mut self) -> Result<(), TemplateParserError> {
        self.template_handler
            .handle_template_start(TemplateStart::instance())
            .map_err(handler_error)
    }

    /// 发送文档结束事件；parser 计时信息与 Java 版一样被忽略。
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `document_end` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn document_end(&mut self) -> Result<(), TemplateParserError> {
        self.template_handler
            .handle_template_end(TemplateEnd::instance())
            .map_err(handler_error)
    }

    /// 发送保留完整源码形态的文本事件。
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `text` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn text(
        &mut self,
        source: &str,
        start: usize,
        end: usize,
    ) -> Result<(), TemplateParserError> {
        if start == end {
            return Ok(());
        }
        let (line, col) = self.location(source, start);
        self.template_handler
            .handle_text(Arc::new(Text::with_location(
                Some(Arc::new(JavaString::from_rust_str(&source[start..end]))),
                self.template_name.clone(),
                line,
                col,
            )))
            .map_err(handler_error)
    }

    /// 发送保留 parser 实际边界的注释事件。
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `comment` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn comment(
        &mut self,
        source: &str,
        start: usize,
        content_start: usize,
        content_end: usize,
        end: usize,
    ) -> Result<(), TemplateParserError> {
        let (line, col) = self.location(source, start);
        let content: Arc<dyn JavaCharSequence> = Arc::new(JavaString::from_rust_str(
            &source[content_start..content_end],
        ));
        self.template_handler
            .handle_comment(Arc::new(Comment::with_boundaries_and_location(
                JavaString::from_rust_str(&source[start..content_start]),
                Some(content),
                JavaString::from_rust_str(&source[content_end..end]),
                self.template_name.clone(),
                line,
                col,
            )))
            .map_err(handler_error)
    }

    /// 发送保留 parser 实际边界的 CDATA 事件。
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `cdata` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn cdata(
        &mut self,
        source: &str,
        start: usize,
        content_start: usize,
        content_end: usize,
        end: usize,
    ) -> Result<(), TemplateParserError> {
        let (line, col) = self.location(source, start);
        let content: Arc<dyn JavaCharSequence> = Arc::new(JavaString::from_rust_str(
            &source[content_start..content_end],
        ));
        self.template_handler
            .handle_cdata_section(Arc::new(CDATASection::with_boundaries_and_location(
                JavaString::from_rust_str(&source[start..content_start]),
                Some(content),
                JavaString::from_rust_str(&source[content_end..end]),
                self.template_name.clone(),
                line,
                col,
            )))
            .map_err(handler_error)
    }

    /// 发送 XML declaration，并保留完整声明文本及分解字段。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `xml_declaration` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn xml_declaration(
        &mut self,
        source: &str,
        start: usize,
        end: usize,
        keyword: &str,
        version: Option<&str>,
        encoding: Option<&str>,
        standalone: Option<&str>,
    ) -> Result<(), TemplateParserError> {
        let (line, col) = self.location(source, start);
        self.template_handler
            .handle_xml_declaration(Arc::new(XMLDeclaration::with_location(
                Some(JavaString::from_rust_str(&source[start..end])),
                Some(JavaString::from_rust_str(keyword)),
                version.map(JavaString::from_rust_str),
                encoding.map(JavaString::from_rust_str),
                standalone.map(JavaString::from_rust_str),
                self.template_name.clone(),
                line,
                col,
            )))
            .map_err(handler_error)
    }

    /// 发送 DOCTYPE，并保留完整声明文本及分解字段。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `doc_type` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn doc_type(
        &mut self,
        source: &str,
        start: usize,
        end: usize,
        keyword: &str,
        root_element_name: &str,
        public_id: Option<&str>,
        system_id: Option<&str>,
        internal_subset: Option<&str>,
    ) -> Result<(), TemplateParserError> {
        let (line, col) = self.location(source, start);
        let event = DocType::with_location(
            Some(JavaString::from_rust_str(&source[start..end])),
            Some(JavaString::from_rust_str(keyword)),
            Some(JavaString::from_rust_str(root_element_name)),
            public_id.map(JavaString::from_rust_str),
            system_id.map(JavaString::from_rust_str),
            internal_subset.map(JavaString::from_rust_str),
            self.template_name.clone(),
            line,
            col,
        )
        .map_err(|error| input_error(error.to_string(), Some(line), Some(col)))?;
        self.template_handler
            .handle_doc_type(Arc::new(event))
            .map_err(handler_error)
    }

    /// 发送 processing instruction，并保留完整文本。
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `processing_instruction` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn processing_instruction(
        &mut self,
        source: &str,
        start: usize,
        end: usize,
        target: &str,
        content: Option<&str>,
    ) -> Result<(), TemplateParserError> {
        let (line, col) = self.location(source, start);
        self.template_handler
            .handle_processing_instruction(Arc::new(ProcessingInstruction::with_location(
                Some(JavaString::from_rust_str(&source[start..end])),
                Some(JavaString::from_rust_str(target)),
                content.map(JavaString::from_rust_str),
                self.template_name.clone(),
                line,
                col,
            )))
            .map_err(handler_error)
    }

    /// 解析原始开始标签并发送 standalone/open Engine 事件。
    ///
    /// `synthetic` 对应 AttoParser 的 auto-open；`standalone` 表示词法上或 HTML
    /// 元素定义上不具有独立 body。
    #[expect(dead_code, reason = "保留 AttoParser 原始 elementStart 回调合同")]
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `element_start` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn element_start(
        &mut self,
        source: &str,
        start: usize,
        end: usize,
        name_start: usize,
        name_end: usize,
        standalone: bool,
        minimized: bool,
        synthetic: bool,
    ) -> Result<(), TemplateParserError> {
        self.element_start_with_injected(
            source,
            start,
            end,
            name_start,
            name_end,
            standalone,
            minimized,
            synthetic,
            &[],
        )
    }

    /// 解析开始标签并在 parser 原有属性之后注入 decoupled logic 属性。
    ///
    /// 注入顺序、合成空白和属性位置对应 Java
    /// `DecoupledTemplateLogicMarkupHandler#processInjectedAttributes`。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn element_start_with_injected(
        &mut self,
        source: &str,
        start: usize,
        end: usize,
        name_start: usize,
        name_end: usize,
        standalone: bool,
        minimized: bool,
        synthetic: bool,
        injected_attributes: &[Arc<DecoupledInjectedAttribute>],
    ) -> Result<(), TemplateParserError> {
        let complete_name = JavaString::from_rust_str(&source[name_start..name_end]);
        let definition = self
            .configuration
            .get_element_definitions()
            .for_name(Some(self.template_mode), Some(&complete_name))
            .map_err(|error| input_error(error.to_string(), None, None))?;
        let attributes = self.append_injected_attributes(
            self.parse_attributes(source, name_end, tag_content_end(source, start, end))?,
            injected_attributes,
            source,
            start,
        )?;
        let (line, col) = self.location(source, start);
        if standalone {
            let event = StandaloneElementTag::with_location(
                self.template_mode,
                definition,
                complete_name,
                attributes,
                synthetic,
                minimized,
                self.template_name.clone(),
                line,
                col,
            )
            .map_err(|error| input_error(error.to_string(), Some(line), Some(col)))?;
            self.template_handler
                .handle_standalone_element(Arc::new(event))
                .map_err(handler_error)
        } else {
            self.template_handler
                .handle_open_element(Arc::new(OpenElementTag::with_location(
                    self.template_mode,
                    definition,
                    complete_name,
                    attributes,
                    synthetic,
                    self.template_name.clone(),
                    line,
                    col,
                )))
                .map_err(handler_error)
        }
    }

    fn append_injected_attributes(
        &self,
        attributes: Option<Arc<Attributes>>,
        injected_attributes: &[Arc<DecoupledInjectedAttribute>],
        source: &str,
        position: usize,
    ) -> Result<Option<Arc<Attributes>>, TemplateParserError> {
        if injected_attributes.is_empty() {
            return Ok(attributes);
        }
        let mut values = attributes
            .as_ref()
            .and_then(|value| value.as_attribute_slice())
            .map_or_else(Vec::new, <[Arc<Attribute>]>::to_vec);
        let mut spaces = attributes
            .as_ref()
            .and_then(|value| value.inner_white_spaces())
            .map_or_else(Vec::new, <[JavaString]>::to_vec);
        let (line, col) = self.location(source, position);

        for injected in injected_attributes {
            if spaces.len() <= values.len() {
                spaces.push(JavaString::from_rust_str(" "));
            }
            let (_, _, _, _, operator_len, _, _, _, value_outer_len) = injected.parser_parts();
            let complete_name = injected
                .get_name()
                .map_err(|error| input_error(error.to_string(), Some(line), Some(col)))?;
            let definition = self
                .configuration
                .get_attribute_definitions()
                .for_name(Some(self.template_mode), Some(&complete_name))
                .map_err(|error| input_error(error.to_string(), Some(line), Some(col)))?;
            let operator = (operator_len > 0)
                .then(|| injected.get_operator())
                .transpose()
                .map_err(|error| input_error(error.to_string(), Some(line), Some(col)))?;
            let value = (operator_len > 0)
                .then(|| injected.get_value_content())
                .transpose()
                .map_err(|error| input_error(error.to_string(), Some(line), Some(col)))?;
            let quotes = if value_outer_len <= 0 {
                None
            } else {
                let outer = injected
                    .get_value_outer()
                    .map_err(|error| input_error(error.to_string(), Some(line), Some(col)))?;
                match outer.as_utf16().first().copied() {
                    Some(value) if value == u16::from(b'"') => Some(AttributeValueQuotes::DOUBLE),
                    Some(value) if value == u16::from(b'\'') => Some(AttributeValueQuotes::SINGLE),
                    _ => Some(AttributeValueQuotes::NONE),
                }
            };
            values.push(Arc::new(Attribute::new(
                definition,
                complete_name,
                operator,
                value,
                quotes,
                self.template_name.clone(),
                line,
                col,
            )));
        }
        Ok(Some(Attributes::new(Some(values), Some(spaces))))
    }

    /// 发送 close/auto-close/unmatched-close Engine 事件。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `element_end` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn element_end(
        &mut self,
        source: &str,
        start: usize,
        end: usize,
        name_start: usize,
        name_end: usize,
        synthetic: bool,
        unmatched: bool,
    ) -> Result<(), TemplateParserError> {
        let complete_name = JavaString::from_rust_str(&source[name_start..name_end]);
        let definition = self
            .configuration
            .get_element_definitions()
            .for_name(Some(self.template_mode), Some(&complete_name))
            .map_err(|error| input_error(error.to_string(), None, None))?;
        let trailing = source[name_end..tag_content_end(source, start, end)]
            .trim_end_matches('/')
            .to_owned();
        let trailing_white_space = if trailing.is_empty() {
            None
        } else {
            Some(JavaString::from_rust_str(&trailing))
        };
        let (line, col) = self.location(source, start);
        self.template_handler
            .handle_close_element(Arc::new(CloseElementTag::with_location(
                self.template_mode,
                definition,
                complete_name,
                trailing_white_space,
                synthetic,
                unmatched,
                self.template_name.clone(),
                line,
                col,
            )))
            .map_err(handler_error)
    }

    /// 发送 HTML 平衡过程自动补出的关闭标签。
    ///
    /// 合成标签使用触发自动闭合的当前位置，而不是原始开放标签的位置；名称来自
    /// 元素栈，因此不要求 `source` 中当前位置实际包含该名称。
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `synthetic_element_end` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn synthetic_element_end(
        &mut self,
        source: &str,
        position: usize,
        complete_name: &str,
    ) -> Result<(), TemplateParserError> {
        let complete_name = JavaString::from_rust_str(complete_name);
        let definition = self
            .configuration
            .get_element_definitions()
            .for_name(Some(self.template_mode), Some(&complete_name))
            .map_err(|error| input_error(error.to_string(), None, None))?;
        let (line, col) = self.location(source, position);
        self.template_handler
            .handle_close_element(Arc::new(CloseElementTag::with_location(
                self.template_mode,
                definition,
                complete_name,
                None,
                true,
                false,
                self.template_name.clone(),
                line,
                col,
            )))
            .map_err(handler_error)
    }

    /// 发送 HTML 平衡过程自动补出的开放标签。
    #[expect(
        dead_code,
        reason = "保留 Java handleAutoOpenElementStart 合同；当前 HTMLTemplateParser 使用 AUTO_CLOSE"
    )]
    /// 对应 Java 语义：`TemplateHandlerAdapterMarkupHandler` 的 `synthetic_element_start` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn synthetic_element_start(
        &mut self,
        source: &str,
        position: usize,
        complete_name: &str,
        injected_attributes: &[Arc<DecoupledInjectedAttribute>],
    ) -> Result<(), TemplateParserError> {
        let complete_name = JavaString::from_rust_str(complete_name);
        let definition = self
            .configuration
            .get_element_definitions()
            .for_name(Some(self.template_mode), Some(&complete_name))
            .map_err(|error| input_error(error.to_string(), None, None))?;
        let attributes =
            self.append_injected_attributes(None, injected_attributes, source, position)?;
        let (line, col) = self.location(source, position);
        self.template_handler
            .handle_open_element(Arc::new(OpenElementTag::with_location(
                self.template_mode,
                definition,
                complete_name,
                attributes,
                true,
                self.template_name.clone(),
                line,
                col,
            )))
            .map_err(handler_error)
    }

    fn parse_attributes(
        &self,
        source: &str,
        mut position: usize,
        content_end: usize,
    ) -> Result<Option<Arc<Attributes>>, TemplateParserError> {
        let mut attributes = Vec::with_capacity(10);
        let mut spaces = Vec::with_capacity(10);

        while position < content_end {
            let white_start = position;
            position = consume_whitespace(source, position, content_end);
            if position == content_end {
                if position > white_start {
                    spaces.push(JavaString::from_rust_str(&source[white_start..position]));
                }
                break;
            }
            spaces.push(JavaString::from_rust_str(&source[white_start..position]));

            let name_start = position;
            position = consume_attribute_name(source, position, content_end);
            if position == name_start {
                return Err(input_error(
                    "Malformed attribute name".to_owned(),
                    None,
                    None,
                ));
            }
            let name_end = position;
            let after_name = position;
            let after_name_whitespace = consume_whitespace(source, position, content_end);

            let mut operator = None;
            let mut value = None;
            let mut quotes = None;
            if after_name_whitespace < content_end
                && source.as_bytes()[after_name_whitespace] == b'='
            {
                // 对应 attoparser/Java `handleAttribute` 的 operatorOffset/Len：
                // 操作符文本从名称结束处（含名称与 `=` 之间的空白）一直延伸到
                // 值前空白之后 —— 如 `href = "x"` 的 operator 为 ` = `，在
                // toString/重写时原样保留（Java Attribute#write 逐字写出 operator）。
                let operator_start = after_name;
                position = after_name_whitespace;
                position += 1;
                position = consume_whitespace(source, position, content_end);
                operator = Some(JavaString::from_rust_str(&source[operator_start..position]));
                if position < content_end {
                    let quote = source.as_bytes()[position];
                    if quote == b'\'' || quote == b'"' {
                        quotes = Some(if quote == b'"' {
                            AttributeValueQuotes::DOUBLE
                        } else {
                            AttributeValueQuotes::SINGLE
                        });
                        position += 1;
                        let value_start = position;
                        while position < content_end && source.as_bytes()[position] != quote {
                            position = next_char_boundary(source, position);
                        }
                        value = Some(JavaString::from_rust_str(&source[value_start..position]));
                        if position < content_end {
                            position += 1;
                        } else if self.template_mode == TemplateMode::XML {
                            return Err(input_error(
                                "Unclosed XML attribute value".to_owned(),
                                None,
                                None,
                            ));
                        }
                    } else {
                        if self.template_mode == TemplateMode::XML {
                            return Err(input_error(
                                "XML attribute values must be quoted".to_owned(),
                                None,
                                None,
                            ));
                        }
                        quotes = Some(AttributeValueQuotes::NONE);
                        let value_start = position;
                        while position < content_end
                            && !is_markup_whitespace(source.as_bytes()[position])
                        {
                            position = next_char_boundary(source, position);
                        }
                        value = Some(JavaString::from_rust_str(&source[value_start..position]));
                    }
                } else {
                    value = Some(JavaString::from_rust_str(""));
                    quotes = Some(AttributeValueQuotes::DOUBLE);
                }
            } else {
                // 无值 HTML 属性之后的空白属于下一属性的前导空白，不能作为
                // “属性名与等号之间的空白”吞掉。对应 attoparser 的属性序列事件。
                position = after_name;
            }

            let complete_name = JavaString::from_rust_str(&source[name_start..name_end]);
            if attributes.iter().any(|attribute: &Arc<Attribute>| {
                use crate::model::IAttribute;
                if self.template_mode == TemplateMode::HTML {
                    attribute
                        .get_attribute_complete_name()
                        .to_string_lossy()
                        .eq_ignore_ascii_case(&complete_name.to_string_lossy())
                } else {
                    attribute.get_attribute_complete_name() == &complete_name
                }
            }) {
                return Err(input_error(
                    format!(
                        "Attribute \"{}\" appears more than once in element",
                        complete_name.to_string_lossy()
                    ),
                    None,
                    None,
                ));
            }
            let definition = self
                .configuration
                .get_attribute_definitions()
                .for_name(Some(self.template_mode), Some(&complete_name))
                .map_err(|error| input_error(error.to_string(), None, None))?;
            let (line, col) = self.location(source, name_start);
            attributes.push(Arc::new(Attribute::new(
                definition,
                complete_name,
                operator,
                value,
                quotes,
                self.template_name.clone(),
                line,
                col,
            )));
        }

        if attributes.is_empty() && spaces.is_empty() {
            Ok(None)
        } else {
            while spaces.len() < attributes.len() {
                spaces.push(JavaString::from_rust_str(""));
            }
            Ok(Some(Attributes::new(Some(attributes), Some(spaces))))
        }
    }

    fn location(&self, source: &str, byte_offset: usize) -> (i32, i32) {
        let (line, col) = source_location(source, byte_offset);
        (
            self.line_offset.wrapping_add(line),
            (if line == 1 { self.col_offset } else { 0 }).wrapping_add(col),
        )
    }
}

fn tag_content_end(source: &str, start: usize, end: usize) -> usize {
    let mut position = end.saturating_sub(1);
    if position > start && source.as_bytes().get(position) == Some(&b'>') {
        position -= 1;
    }
    if position > start && source.as_bytes().get(position) == Some(&b'/') {
        position -= 1;
    }
    position + 1
}

fn consume_whitespace(source: &str, mut position: usize, end: usize) -> usize {
    while position < end && is_markup_whitespace(source.as_bytes()[position]) {
        position += 1;
    }
    position
}

fn consume_attribute_name(source: &str, mut position: usize, end: usize) -> usize {
    while position < end {
        let byte = source.as_bytes()[position];
        if is_markup_whitespace(byte) || matches!(byte, b'=' | b'/' | b'>') {
            break;
        }
        position = next_char_boundary(source, position);
    }
    position
}

const fn is_markup_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0C)
}

fn next_char_boundary(source: &str, position: usize) -> usize {
    position + source[position..].chars().next().map_or(1, char::len_utf8)
}

fn source_location(source: &str, byte_offset: usize) -> (i32, i32) {
    let mut line = 1_i32;
    let mut col = 1_i32;
    let mut chars = source[..byte_offset].chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                line = line.wrapping_add(1);
                col = 1;
            }
            '\n' => {
                line = line.wrapping_add(1);
                col = 1;
            }
            other => {
                col = col.wrapping_add(other.len_utf16() as i32);
            }
        }
    }
    (line, col)
}

fn handler_error(
    error: Box<dyn crate::exceptions::TemplateEngineException>,
) -> TemplateParserError {
    input_error(error.to_string(), None, None)
}

fn input_error(message: String, line: Option<i32>, col: Option<i32>) -> TemplateParserError {
    TemplateParserError::Input(match (line, col) {
        (Some(line), Some(col)) => {
            TemplateInputException::with_location(Some(message), None, line, col)
        }
        _ => TemplateInputException::new(Some(message)),
    })
}
