use std::panic::panic_any;

use crate::engine::{AttributeNames, ElementNames};
use crate::exceptions::TemplateProcessingException;
use crate::util::{EscapedAttributeUtils, Utf16String};
use crate::{IEngineConfiguration, TemplateMode};

use super::{IInlinePreProcessorHandler, StandardInlineMode};

const DEFAULT_LEVELS_SIZE: usize = 2;

/// 把 `[[...]]` 与 `[(...)]` 输出表达式转换为标准方言块元素事件。
///
/// 本处理器同时跟踪元素执行层级和 `th:inline` 模式栈；跨模板模式的内容保留到后续
/// 重解析阶段处理。表达式结束符仅在单双引号之外识别。
///
/// 对应 Java:
/// `org.thymeleaf.standard.inline.OutputExpressionInlinePreProcessorHandler`。
pub struct OutputExpressionInlinePreProcessorHandler {
    next: Box<dyn IInlinePreProcessorHandler>,
    inline_attribute_names: Vec<Utf16String>,
    block_element_name: Vec<u16>,
    escaped_text_attribute_name: Utf16String,
    unescaped_text_attribute_name: Utf16String,
    exec_level: i32,
    inline_template_modes: Vec<Option<TemplateMode>>,
    inline_exec_levels: Vec<i32>,
    inline_index: usize,
    attribute_buffer: Vec<u16>,
}

impl OutputExpressionInlinePreProcessorHandler {
    /// 创建输出表达式预处理器并计算当前 Standard Dialect 前缀的完整名称集合。
    ///
    /// 对应 Java: `OutputExpressionInlinePreProcessorHandler#OutputExpressionInlinePreProcessorHandler`。
    pub fn new(
        _configuration: &dyn IEngineConfiguration,
        template_mode: TemplateMode,
        standard_dialect_prefix: Option<&Utf16String>,
        next: Box<dyn IInlinePreProcessorHandler>,
    ) -> Result<Self, TemplateProcessingException> {
        let inline_name = Utf16String::from_rust_str("inline");
        let block_name = Utf16String::from_rust_str("block");
        let text_name = Utf16String::from_rust_str("text");
        let utext_name = Utf16String::from_rust_str("utext");

        let inline = AttributeNames::for_name_with_prefix(
            Some(template_mode),
            standard_dialect_prefix,
            Some(&inline_name),
        )
        .map_err(processing_error)?;
        let block = ElementNames::for_name_with_prefix(
            Some(template_mode),
            standard_dialect_prefix,
            Some(&block_name),
        )
        .map_err(processing_error)?;
        let text = AttributeNames::for_name_with_prefix(
            Some(template_mode),
            standard_dialect_prefix,
            Some(&text_name),
        )
        .map_err(processing_error)?;
        let utext = AttributeNames::for_name_with_prefix(
            Some(template_mode),
            standard_dialect_prefix,
            Some(&utext_name),
        )
        .map_err(processing_error)?;

        let inline_attribute_names = inline
            .as_attribute_name()
            .get_complete_attribute_names()
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .flatten()
            .cloned()
            .collect();
        let block_element_name = first_complete_element_name(block.as_element_name())?
            .as_utf16()
            .to_vec();
        let escaped_text_attribute_name = first_complete_attribute_name(text.as_attribute_name())?;
        let unescaped_text_attribute_name =
            first_complete_attribute_name(utext.as_attribute_name())?;

        let mut inline_template_modes = vec![None; DEFAULT_LEVELS_SIZE];
        let mut inline_exec_levels = vec![-1; DEFAULT_LEVELS_SIZE];
        inline_template_modes[0] = Some(template_mode);
        inline_exec_levels[0] = 0;
        Ok(Self {
            next,
            inline_attribute_names,
            block_element_name,
            escaped_text_attribute_name,
            unescaped_text_attribute_name,
            exec_level: 0,
            inline_template_modes,
            inline_exec_levels,
            inline_index: 0,
            attribute_buffer: Vec::new(),
        })
    }

    fn increase_exec_level(&mut self) {
        self.exec_level += 1;
    }

    fn decrease_exec_level(&mut self) {
        if self.inline_exec_levels[self.inline_index] == self.exec_level {
            self.inline_template_modes[self.inline_index] = None;
            self.inline_exec_levels[self.inline_index] = -1;
            self.inline_index -= 1;
        }
        self.exec_level -= 1;
    }

    fn set_inline_template_mode(&mut self, template_mode: Option<TemplateMode>) {
        if self.inline_exec_levels[self.inline_index] != self.exec_level {
            self.inline_index += 1;
        }
        if self.inline_index >= self.inline_template_modes.len() {
            self.inline_template_modes.extend([None, None]);
            self.inline_exec_levels.extend([-1, -1]);
        }
        self.inline_template_modes[self.inline_index] = template_mode;
        self.inline_exec_levels[self.inline_index] = self.exec_level;
    }

    fn perform_inlining(
        &mut self,
        text: &mut [u16],
        offset: usize,
        len: usize,
        line: i32,
        col: i32,
    ) {
        let maxi = offset + len;
        let mut locator = [line, col];
        let mut current = offset;
        let mut index = offset;
        let mut expression = None;

        while index < maxi {
            let current_line = locator[0];
            let current_col = locator[1];
            if let Some((expression_start, closing)) = expression {
                let Some(expression_end) =
                    find_next_structure_end_avoid_quotes(text, index, maxi, closing, &mut locator)
                else {
                    self.next.handle_text(
                        Some(text),
                        as_i32(current),
                        as_i32(maxi - current),
                        current_line,
                        current_col,
                    );
                    return;
                };
                let attribute_name = if text[expression_start + 1] == u16::from(b'[') {
                    self.escaped_text_attribute_name.clone()
                } else {
                    self.unescaped_text_attribute_name.clone()
                };
                let value_offset = expression_start + 2;
                let value_len = expression_end - value_offset;
                self.prepare_attribute_buffer(&attribute_name, text, value_offset, value_len);
                let name_len = as_i32(attribute_name.len());
                let block_name_len = as_i32(self.block_element_name.len());
                let event_col = current_col + 2;

                self.next.handle_open_element_start(
                    Some(&mut self.block_element_name),
                    0,
                    block_name_len,
                    current_line,
                    event_col,
                );
                self.next.handle_attribute(
                    Some(&mut self.attribute_buffer),
                    0,
                    name_len,
                    current_line,
                    event_col,
                    name_len,
                    1,
                    current_line,
                    event_col,
                    name_len + 2,
                    as_i32(value_len),
                    name_len + 1,
                    as_i32(value_len) + 2,
                    current_line,
                    event_col,
                );
                self.next.handle_open_element_end(
                    Some(&mut self.block_element_name),
                    0,
                    block_name_len,
                    current_line,
                    event_col,
                );
                self.next.handle_close_element_start(
                    Some(&mut self.block_element_name),
                    0,
                    block_name_len,
                    current_line,
                    event_col,
                );
                self.next.handle_close_element_end(
                    Some(&mut self.block_element_name),
                    0,
                    block_name_len,
                    current_line,
                    event_col,
                );
                count_char(&mut locator, text[expression_end]);
                count_char(&mut locator, text[expression_end + 1]);
                current = expression_end + 2;
                index = current;
                expression = None;
            } else {
                let Some(expression_start) =
                    find_next_structure_start(text, index, maxi, &mut locator)
                else {
                    self.next.handle_text(
                        Some(text),
                        as_i32(current),
                        as_i32(maxi - current),
                        current_line,
                        current_col,
                    );
                    return;
                };
                if expression_start > current {
                    self.next.handle_text(
                        Some(text),
                        as_i32(current),
                        as_i32(expression_start - current),
                        current_line,
                        current_col,
                    );
                }
                let closing = if text[expression_start + 1] == u16::from(b'[') {
                    u16::from(b']')
                } else {
                    u16::from(b')')
                };
                current = expression_start;
                index = current + 2;
                expression = Some((expression_start, closing));
            }
        }

        if expression.is_some() {
            self.next.handle_text(
                Some(text),
                as_i32(current),
                as_i32(maxi - current),
                locator[0],
                locator[1],
            );
        }
    }

    fn prepare_attribute_buffer(
        &mut self,
        attribute_name: &Utf16String,
        value_text: &[u16],
        value_offset: usize,
        value_len: usize,
    ) {
        let required_len = attribute_name.len() + value_len + 3;
        if self.attribute_buffer.len() < required_len {
            self.attribute_buffer.resize(required_len.max(30), 0);
        }
        let name_len = attribute_name.len();
        self.attribute_buffer[..name_len].copy_from_slice(attribute_name.as_utf16());
        self.attribute_buffer[name_len] = u16::from(b'=');
        self.attribute_buffer[name_len + 1] = u16::from(b'"');
        self.attribute_buffer[name_len + 2..name_len + 2 + value_len]
            .copy_from_slice(&value_text[value_offset..value_offset + value_len]);
        self.attribute_buffer[required_len - 1] = u16::from(b'"');
    }
}

impl IInlinePreProcessorHandler for OutputExpressionInlinePreProcessorHandler {
    fn handle_text(
        &mut self,
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) {
        let active_mode = self.inline_template_modes[self.inline_index];
        if active_mode != self.inline_template_modes[0] {
            self.next.handle_text(buffer, offset, len, line, col);
            return;
        }
        let Some(text) = buffer else {
            self.next.handle_text(None, offset, len, line, col);
            return;
        };
        let (offset, len) = checked_range(text, offset, len);
        if !might_need_inlining(text, offset, len) {
            self.next
                .handle_text(Some(text), as_i32(offset), as_i32(len), line, col);
            return;
        }
        self.perform_inlining(text, offset, len, line, col);
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
        self.increase_exec_level();
        self.next.handle_standalone_element_start(
            buffer,
            name_offset,
            name_len,
            minimized,
            line,
            col,
        );
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
        self.decrease_exec_level();
        self.next.handle_standalone_element_end(
            buffer,
            name_offset,
            name_len,
            minimized,
            line,
            col,
        );
    }
    fn handle_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.increase_exec_level();
        self.next
            .handle_open_element_start(buffer, name_offset, name_len, line, col);
    }
    fn handle_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.next
            .handle_open_element_end(buffer, name_offset, name_len, line, col);
    }
    fn handle_auto_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.increase_exec_level();
        self.next
            .handle_auto_open_element_start(buffer, name_offset, name_len, line, col);
    }
    fn handle_auto_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.next
            .handle_auto_open_element_end(buffer, name_offset, name_len, line, col);
    }
    fn handle_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.next
            .handle_close_element_start(buffer, name_offset, name_len, line, col);
    }
    fn handle_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.decrease_exec_level();
        self.next
            .handle_close_element_end(buffer, name_offset, name_len, line, col);
    }
    fn handle_auto_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.next
            .handle_auto_close_element_start(buffer, name_offset, name_len, line, col);
    }
    fn handle_auto_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) {
        self.decrease_exec_level();
        self.next
            .handle_auto_close_element_end(buffer, name_offset, name_len, line, col);
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_attribute(
        &mut self,
        mut buffer: Option<&mut [u16]>,
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
        if let Some(values) = buffer.as_deref_mut() {
            let (name_start, name_length) = checked_range(values, name_offset, name_len);
            let case_sensitive = self.inline_template_modes[0]
                .expect("root inline mode exists")
                .is_case_sensitive();
            if self.inline_attribute_names.iter().any(|candidate| {
                text_equals(
                    case_sensitive,
                    candidate.as_utf16(),
                    &values[name_start..name_start + name_length],
                )
            }) {
                let (value_start, value_length) =
                    checked_range(values, value_content_offset, value_content_len);
                let raw = Utf16String::from_utf16(
                    values[value_start..value_start + value_length].to_vec(),
                );
                let inline_mode_value = EscapedAttributeUtils::unescape_attribute(
                    self.inline_template_modes[0],
                    Some(&raw),
                )
                .unwrap_or_else(|error| panic_any(error));
                let mode = inline_mode_value.and_then(|value| {
                    match StandardInlineMode::parse(Some(&value))
                        .unwrap_or_else(|error| panic_any(error))
                    {
                        StandardInlineMode::NONE => None,
                        StandardInlineMode::HTML => Some(TemplateMode::HTML),
                        StandardInlineMode::XML => Some(TemplateMode::XML),
                        StandardInlineMode::TEXT => Some(TemplateMode::TEXT),
                        StandardInlineMode::JAVASCRIPT => Some(TemplateMode::JAVASCRIPT),
                        StandardInlineMode::CSS => Some(TemplateMode::CSS),
                    }
                });
                self.set_inline_template_mode(mode);
            }
        }
        self.next.handle_attribute(
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
    }
}

fn first_complete_attribute_name(
    name: &crate::engine::AttributeName,
) -> Result<Utf16String, TemplateProcessingException> {
    name.get_complete_attribute_names()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .first()
        .and_then(Clone::clone)
        .ok_or_else(|| TemplateProcessingException::new(Some("Attribute name is empty".to_owned())))
}

fn first_complete_element_name(
    name: &crate::engine::ElementName,
) -> Result<Utf16String, TemplateProcessingException> {
    name.get_complete_element_names()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .first()
        .and_then(Clone::clone)
        .ok_or_else(|| TemplateProcessingException::new(Some("Element name is empty".to_owned())))
}

fn processing_error(error: impl std::fmt::Display) -> TemplateProcessingException {
    TemplateProcessingException::new(Some(error.to_string()))
}

fn checked_range(buffer: &[u16], offset: i32, len: i32) -> (usize, usize) {
    let start = usize::try_from(offset).expect("Java char[] offset must be non-negative");
    let length = usize::try_from(len).expect("Java char[] length must be non-negative");
    assert!(
        start + length <= buffer.len(),
        "Java char[] range out of bounds"
    );
    (start, length)
}

fn as_i32(value: usize) -> i32 {
    i32::try_from(value).expect("Java char[] index exceeds Integer.MAX_VALUE")
}

fn might_need_inlining(buffer: &[u16], offset: usize, len: usize) -> bool {
    buffer[offset..offset + len]
        .windows(2)
        .any(|value| value[0] == u16::from(b'[') && matches!(value[1], 0x005B | 0x0028))
}

fn count_char(locator: &mut [i32; 2], value: u16) {
    if value == u16::from(b'\n') {
        locator[0] += 1;
        locator[1] = 1;
    } else {
        locator[1] += 1;
    }
}

fn find_next_structure_start(
    text: &[u16],
    offset: usize,
    maxi: usize,
    locator: &mut [i32; 2],
) -> Option<usize> {
    let mut column_index = offset;
    let mut index = offset;
    while index < maxi {
        let value = text[index];
        if value == u16::from(b'\n') {
            column_index = index;
            locator[1] = 0;
            locator[0] += 1;
        } else if value == u16::from(b'[')
            && index + 1 < maxi
            && matches!(text[index + 1], 0x005B | 0x0028)
        {
            locator[1] += as_i32(index - column_index);
            return Some(index);
        }
        index += 1;
    }
    locator[1] += as_i32(maxi - column_index);
    None
}

fn find_next_structure_end_avoid_quotes(
    text: &[u16],
    offset: usize,
    maxi: usize,
    inner_closing: u16,
    locator: &mut [i32; 2],
) -> Option<usize> {
    let mut in_quotes = false;
    let mut in_apostrophes = false;
    let mut column_index = offset;
    let mut index = offset;
    while index < maxi {
        let value = text[index];
        if value == u16::from(b'\n') {
            column_index = index;
            locator[1] = 0;
            locator[0] += 1;
        } else if value == u16::from(b'"') && !in_apostrophes {
            in_quotes = !in_quotes;
        } else if value == u16::from(b'\'') && !in_quotes {
            in_apostrophes = !in_apostrophes;
        } else if value == inner_closing
            && !in_quotes
            && !in_apostrophes
            && index + 1 < maxi
            && text[index + 1] == u16::from(b']')
        {
            locator[1] += as_i32(index - column_index);
            return Some(index);
        }
        index += 1;
    }
    locator[1] += as_i32(maxi - column_index);
    None
}

fn text_equals(case_sensitive: bool, left: &[u16], right: &[u16]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left == right
                || (!case_sensitive
                    && char::from_u32(u32::from(*left))
                        .zip(char::from_u32(u32::from(*right)))
                        .is_some_and(|(left, right)| {
                            left.to_lowercase().to_string() == right.to_lowercase().to_string()
                        }))
        })
}
