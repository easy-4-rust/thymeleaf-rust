use std::sync::Arc;

use crate::TemplateMode;
use crate::exceptions::TemplateInputException;
use crate::util::JavaString;

use super::{DecoupledInjectedAttribute, DecoupledTemplateLogic};

const TAG_NAME_LOGIC: &str = "thlogic";
const TAG_NAME_ATTR: &str = "attr";
const ATTRIBUTE_NAME_SEL: &str = "sel";

/// 解析解耦逻辑资源并构建 `DecoupledTemplateLogic`。
///
/// 仅 `<thlogic>` 内的 `<attr>` 标签具有语义；`sel` 可位于标签属性的任意位置，
/// 其他属性按原始名称、操作符、引号和值保存为注入属性。嵌套 `<attr>` 的 selector
/// 按层级连接，复现 AttoParser handler 的同步状态机。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.markup.decoupled.DecoupledTemplateLogicBuilderMarkupHandler`。
pub struct DecoupledTemplateLogicBuilderMarkupHandler {
    template_name: JavaString,
    template_mode: TemplateMode,
    decoupled_template_logic: DecoupledTemplateLogic,
    in_logic_body: bool,
    in_attr_tag: bool,
    selector: Selector,
    current_injected_attributes: Vec<Arc<DecoupledInjectedAttribute>>,
}

impl DecoupledTemplateLogicBuilderMarkupHandler {
    /// 创建指定模板和模式的构建 handler。
    ///
    /// # 错误
    ///
    /// 空模板名返回与 Java `Validate.notEmpty` 对应的输入错误。
    ///
    /// 对应 Java:
    /// `DecoupledTemplateLogicBuilderMarkupHandler#DecoupledTemplateLogicBuilderMarkupHandler`。
    pub fn new(
        template_name: JavaString,
        template_mode: TemplateMode,
    ) -> Result<Self, TemplateInputException> {
        if template_name.is_empty() {
            return Err(TemplateInputException::new(Some(
                "Template name cannot be null or empty".to_owned(),
            )));
        }
        Ok(Self {
            template_name,
            template_mode,
            decoupled_template_logic: DecoupledTemplateLogic::new(),
            in_logic_body: false,
            in_attr_tag: false,
            selector: Selector::new(),
            current_injected_attributes: Vec::with_capacity(8),
        })
    }

    /// 返回当前已构建的解耦逻辑。
    ///
    /// 对应 Java:
    /// `DecoupledTemplateLogicBuilderMarkupHandler#getDecoupledTemplateLogic()`。
    #[must_use]
    pub const fn get_decoupled_template_logic(&self) -> &DecoupledTemplateLogic {
        &self.decoupled_template_logic
    }

    /// 消费 handler 并返回构建结果。
    ///
    /// 这是 Rust 所有权入口；可观察内容等同 Java getter 返回的同一容器。
    #[must_use]
    /// 对应 Java 语义：`DecoupledTemplateLogicBuilderMarkupHandler` 的 `into_decoupled_template_logic` 行为（Rust 侧辅助/私有路径）。
    pub fn into_decoupled_template_logic(self) -> DecoupledTemplateLogic {
        self.decoupled_template_logic
    }

    /// 解析完整解耦逻辑标记文本。
    ///
    /// Parser 只负责把原始标签边界按顺序送入与 Java callback 相同的状态转换；
    /// 文本、注释、声明及无关标签被忽略。
    ///
    /// # 错误
    ///
    /// 缺少/重复 `sel`、属性结构损坏或标签未闭合时返回带模板位置的输入错误。
    /// 对应 Java 语义：Java 接口/超类方法 `parse()` 的 Rust 移植（`DecoupledTemplateLogicBuilderMarkupHandler` 继承路径）。
    pub fn parse(&mut self, source: &str) -> Result<(), TemplateInputException> {
        let mut position = 0;
        while let Some(relative_start) = source[position..].find('<') {
            let start = position + relative_start;
            if source[start..].starts_with("<!--") {
                position = source[start + 4..]
                    .find("-->")
                    .map_or(source.len(), |end| start + 4 + end + 3);
                continue;
            }
            if source[start..].starts_with("<![CDATA[") {
                position = source[start + 9..]
                    .find("]]>")
                    .map_or(source.len(), |end| start + 9 + end + 3);
                continue;
            }
            let end = find_tag_end(source, start).ok_or_else(|| {
                self.error_at(source, start, "Unclosed tag in decoupled logic file")
            })?;
            if source[start..end].starts_with("<?") || source[start..end].starts_with("<!") {
                position = end;
                continue;
            }

            let closing = source[start + 1..end].trim_start().starts_with('/');
            let standalone = !closing && source[start + 1..end - 1].trim_end().ends_with('/');
            let Some(tag) = ParsedTag::parse(source, start, end, closing) else {
                return Err(self.error_at(source, start, "Malformed decoupled logic tag"));
            };
            if closing {
                self.handle_close(&tag.name)?;
            } else {
                self.handle_open(source, &tag, standalone)?;
                if standalone {
                    self.handle_standalone_end(&tag.name)?;
                }
            }
            position = end;
        }
        Ok(())
    }

    fn handle_open(
        &mut self,
        source: &str,
        tag: &ParsedTag,
        standalone: bool,
    ) -> Result<(), TemplateInputException> {
        if !self.in_logic_body {
            if self.name_equals(&tag.name, TAG_NAME_LOGIC) && !standalone {
                self.in_logic_body = true;
            }
            return Ok(());
        }
        if !self.name_equals(&tag.name, TAG_NAME_ATTR) {
            return Ok(());
        }

        self.selector.increase_level();
        self.in_attr_tag = true;
        self.current_injected_attributes.clear();
        for attribute in &tag.attributes {
            if self.name_equals(&source[attribute.name.clone()], ATTRIBUTE_NAME_SEL) {
                if !self.selector.is_level_empty() {
                    return Err(self.error_at(
                        source,
                        attribute.name.start,
                        "Error while processing decoupled logic file: selector (\"sel\") attribute found more than once in attr injection tag",
                    ));
                }
                let value = attribute
                    .value_content
                    .as_ref()
                    .map_or("", |range| &source[range.clone()]);
                if value.is_empty() {
                    return Err(self.error_at(
                        source,
                        attribute.name.start,
                        "String index out of range: 0",
                    ));
                }
                self.selector.set_selector(value);
            } else {
                self.current_injected_attributes.push(Arc::new(
                    attribute.to_injected_attribute(source).map_err(|error| {
                        self.error_at(source, attribute.name.start, &error.to_string())
                    })?,
                ));
            }
        }
        self.finish_attr_tag(source, tag.start)
    }

    fn finish_attr_tag(
        &mut self,
        source: &str,
        position: usize,
    ) -> Result<(), TemplateInputException> {
        if self.in_attr_tag && self.selector.is_level_empty() {
            return Err(self.error_at(
                source,
                position,
                "Error while processing decoupled logic file: <attr> injection tag does not contain any \"sel\" selector attributes.",
            ));
        }
        let current_selector = JavaString::from_rust_str(self.selector.get_current_selector());
        for attribute in self.current_injected_attributes.drain(..) {
            self.decoupled_template_logic
                .add_injected_attribute(current_selector.clone(), attribute);
        }
        self.in_attr_tag = false;
        Ok(())
    }

    fn handle_standalone_end(&mut self, name: &str) -> Result<(), TemplateInputException> {
        if self.in_logic_body && self.name_equals(name, TAG_NAME_ATTR) {
            self.selector
                .decrease_level()
                .map_err(|message| TemplateInputException::new(Some(message.to_owned())))?;
        }
        Ok(())
    }

    fn handle_close(&mut self, name: &str) -> Result<(), TemplateInputException> {
        if !self.in_logic_body {
            return Ok(());
        }
        if self.name_equals(name, TAG_NAME_LOGIC) {
            self.in_logic_body = false;
        } else if self.name_equals(name, TAG_NAME_ATTR) {
            self.selector
                .decrease_level()
                .map_err(|message| TemplateInputException::new(Some(message.to_owned())))?;
        }
        Ok(())
    }

    fn name_equals(&self, left: &str, right: &str) -> bool {
        if self.template_mode.is_case_sensitive() {
            left == right
        } else {
            left.eq_ignore_ascii_case(right)
        }
    }

    fn error_at(&self, source: &str, offset: usize, message: &str) -> TemplateInputException {
        let (line, col) = source_location(source, offset);
        TemplateInputException::with_location(
            Some(message.to_owned()),
            Some(self.template_name.to_string_lossy()),
            line,
            col,
        )
    }
}

struct Selector {
    level: i32,
    selector_levels: Vec<String>,
    current_selector: Option<String>,
}

impl Selector {
    fn new() -> Self {
        Self {
            level: -1,
            selector_levels: Vec::with_capacity(5),
            current_selector: None,
        }
    }

    fn increase_level(&mut self) {
        self.level = self.level.wrapping_add(1);
    }

    fn decrease_level(&mut self) -> Result<(), &'static str> {
        if self.level < 0 {
            return Err("Cannot decrease level when the selector is clean");
        }
        if self.selector_levels.len() > self.level as usize {
            self.selector_levels.remove(self.level as usize);
        }
        self.level -= 1;
        self.current_selector = None;
        Ok(())
    }

    fn set_selector(&mut self, selector: &str) {
        self.selector_levels.push(if selector.starts_with('/') {
            selector.to_owned()
        } else {
            format!("//{selector}")
        });
        self.current_selector = None;
    }

    fn is_level_empty(&self) -> bool {
        self.selector_levels.len() <= self.level.max(0) as usize
    }

    fn get_current_selector(&mut self) -> &str {
        if self.current_selector.is_none() {
            self.current_selector = Some(self.selector_levels.concat());
        }
        self.current_selector.as_deref().unwrap_or("")
    }
}

#[derive(Clone)]
struct ParsedAttribute {
    name: std::ops::Range<usize>,
    operator: Option<std::ops::Range<usize>>,
    value_content: Option<std::ops::Range<usize>>,
    value_outer: Option<std::ops::Range<usize>>,
}

impl ParsedAttribute {
    fn to_injected_attribute(
        &self,
        source: &str,
    ) -> Result<DecoupledInjectedAttribute, super::DecoupledInjectedAttributeError> {
        let buffer: Vec<u16> = source.encode_utf16().collect();
        let name_offset = utf16_offset(source, self.name.start);
        let name_len = utf16_offset(source, self.name.end) - name_offset;
        let (operator_offset, operator_len) = self.operator.as_ref().map_or((0, 0), |range| {
            let offset = utf16_offset(source, range.start);
            (offset, utf16_offset(source, range.end) - offset)
        });
        let (value_content_offset, value_content_len) =
            self.value_content.as_ref().map_or((0, 0), |range| {
                let offset = utf16_offset(source, range.start);
                (offset, utf16_offset(source, range.end) - offset)
            });
        let (value_outer_offset, value_outer_len) =
            self.value_outer.as_ref().map_or((0, 0), |range| {
                let offset = utf16_offset(source, range.start);
                (offset, utf16_offset(source, range.end) - offset)
            });
        DecoupledInjectedAttribute::create_attribute(
            Some(&buffer),
            name_offset,
            name_len,
            operator_offset,
            operator_len,
            value_content_offset,
            value_content_len,
            value_outer_offset,
            value_outer_len,
        )
    }
}

struct ParsedTag {
    start: usize,
    name: String,
    attributes: Vec<ParsedAttribute>,
}

impl ParsedTag {
    fn parse(source: &str, start: usize, end: usize, closing: bool) -> Option<Self> {
        let mut position = start + if closing { 2 } else { 1 };
        position = consume_space(source, position, end - 1);
        let name_start = position;
        while position < end - 1 {
            let byte = source.as_bytes()[position];
            if byte.is_ascii_whitespace() || matches!(byte, b'/' | b'>') {
                break;
            }
            position += source[position..].chars().next()?.len_utf8();
        }
        if position == name_start {
            return None;
        }
        let name = source[name_start..position].to_owned();
        let mut attributes = Vec::new();
        if !closing {
            let content_end = if source[start..end - 1].trim_end().ends_with('/') {
                source[..end - 1].trim_end().len() - 1
            } else {
                end - 1
            };
            while position < content_end {
                position = consume_space(source, position, content_end);
                if position >= content_end {
                    break;
                }
                let attr_start = position;
                while position < content_end {
                    let byte = source.as_bytes()[position];
                    if byte.is_ascii_whitespace() || matches!(byte, b'=' | b'/' | b'>') {
                        break;
                    }
                    position += source[position..].chars().next()?.len_utf8();
                }
                let name_range = attr_start..position;
                position = consume_space(source, position, content_end);
                let mut operator = None;
                let mut value_content = None;
                let mut value_outer = None;
                if position < content_end && source.as_bytes()[position] == b'=' {
                    operator = Some(position..position + 1);
                    position += 1;
                    position = consume_space(source, position, content_end);
                    if position < content_end {
                        let quote = source.as_bytes()[position];
                        if quote == b'\'' || quote == b'"' {
                            let outer_start = position;
                            position += 1;
                            let content_start = position;
                            while position < content_end && source.as_bytes()[position] != quote {
                                position += source[position..].chars().next()?.len_utf8();
                            }
                            value_content = Some(content_start..position);
                            if position < content_end {
                                position += 1;
                            }
                            value_outer = Some(outer_start..position);
                        } else {
                            let content_start = position;
                            while position < content_end
                                && !source.as_bytes()[position].is_ascii_whitespace()
                            {
                                position += source[position..].chars().next()?.len_utf8();
                            }
                            value_content = Some(content_start..position);
                            value_outer = Some(content_start..position);
                        }
                    }
                }
                attributes.push(ParsedAttribute {
                    name: name_range,
                    operator,
                    value_content,
                    value_outer,
                });
            }
        }
        Some(Self {
            start,
            name,
            attributes,
        })
    }
}

fn find_tag_end(source: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (relative, character) in source[start + 1..].char_indices() {
        let position = start + 1 + relative;
        if let Some(expected) = quote {
            if character == expected {
                quote = None;
            }
        } else if character == '\'' || character == '"' {
            quote = Some(character);
        } else if character == '>' {
            return Some(position + 1);
        }
    }
    None
}

fn consume_space(source: &str, mut position: usize, end: usize) -> usize {
    while position < end && source.as_bytes()[position].is_ascii_whitespace() {
        position += 1;
    }
    position
}

fn utf16_offset(source: &str, byte_offset: usize) -> i32 {
    source[..byte_offset].encode_utf16().count() as i32
}

fn source_location(source: &str, offset: usize) -> (i32, i32) {
    let mut line = 1_i32;
    let mut col = 1_i32;
    let mut characters = source[..offset].chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\r' => {
                if characters.peek() == Some(&'\n') {
                    characters.next();
                }
                line += 1;
                col = 1;
            }
            '\n' => {
                line += 1;
                col = 1;
            }
            value => col += value.len_utf16() as i32,
        }
    }
    (line, col)
}
