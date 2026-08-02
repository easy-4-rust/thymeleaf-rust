use std::io;
use std::sync::{Arc, Mutex};

use crate::context::ITemplateContext;
use crate::engine::EngineEventUtils;
use crate::exceptions::TemplateProcessingException;
use crate::expression::{
    StandardExpressionError, StandardExpressionResult, StandardExpressions, TemplateValue,
};
use crate::model::{ICDATASection, IComment, IModel, IText};
use crate::serializer::{IStandardCSSSerializer, IStandardJavaScriptSerializer};
use crate::util::{
    EscapedAttributeUtils, JavaCharSequence, JavaString, JavaWriter, LazyProcessingCharSequence,
};
use crate::{IEngineConfiguration, TemplateMode};

/// Standard 内联器共享的扫描、表达式执行与跨模板模式处理实现。
///
/// 对应 Java: `org.thymeleaf.standard.inline.AbstractStandardInliner`。
pub struct AbstractStandardInliner {
    template_mode: TemplateMode,
    write_texts_to_output: bool,
    escaping: StandardInlinerEscaping,
}
/// 对应 Java 语义：`AbstractStandardInliner` 的 Rust 侧类型 `StandardInlinerEscaping`。

pub(crate) enum StandardInlinerEscaping {
    Html,
    Xml,
    JavaScript(Arc<dyn IStandardJavaScriptSerializer>),
    Css(Arc<dyn IStandardCSSSerializer>),
}

impl AbstractStandardInliner {
    /// 创建指定模板模式和转义策略的共享实现。
    /// 对应 Java 语义：`AbstractStandardInliner` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(
        configuration: &dyn IEngineConfiguration,
        template_mode: TemplateMode,
        escaping: StandardInlinerEscaping,
    ) -> Self {
        let write_texts_to_output = configuration.get_post_processors(template_mode).is_empty()
            && configuration.get_text_processors(template_mode).len() <= 1;
        Self {
            template_mode,
            write_texts_to_output,
            escaping,
        }
    }

    /// 对 Text 执行 Standard inline。
    /// 对应 Java 语义：`AbstractStandardInliner` 的 `inline_text` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn inline_text(
        &self,
        context: &dyn ITemplateContext,
        text: &dyn IText,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        if context.get_template_mode() != self.template_mode {
            let content = text
                .get_text()
                .map_err(box_error)?
                .ok_or_else(null_text_error)?;
            return self.inline_switch_template_mode(
                context,
                content,
                text.get_line(),
                text.get_col(),
                true,
            );
        }
        if !EngineEventUtils::is_inlineable_text(Some(text)).map_err(box_error)? {
            return Ok(None);
        }
        let content = text.java_to_string().map_err(box_error)?;
        self.perform_inlining(
            context,
            &content,
            0,
            content.len(),
            text.get_template_name(),
            text.get_line(),
            text.get_col(),
        )
        .map(|value| Some(Box::new(value) as Box<dyn JavaCharSequence>))
    }

    /// 对 CDATA 执行 Standard inline。
    /// 对应 Java 语义：`AbstractStandardInliner` 的 `inline_cdata_section` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn inline_cdata_section(
        &self,
        context: &dyn ITemplateContext,
        cdata_section: &dyn ICDATASection,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        if context.get_template_mode() != self.template_mode {
            let content = cdata_section
                .get_content()
                .map_err(box_error)?
                .ok_or_else(null_text_error)?;
            return self.inline_switch_template_mode(
                context,
                content,
                cdata_section.get_line(),
                cdata_section.get_col().wrapping_add(9),
                false,
            );
        }
        if !EngineEventUtils::is_inlineable_cdata(Some(cdata_section)).map_err(box_error)? {
            return Ok(None);
        }
        let content = cdata_section.java_to_string().map_err(box_error)?;
        self.perform_inlining(
            context,
            &content,
            9,
            content.len().saturating_sub(12),
            cdata_section.get_template_name(),
            cdata_section.get_line(),
            cdata_section.get_col(),
        )
        .map(|value| Some(Box::new(value) as Box<dyn JavaCharSequence>))
    }

    /// 对 Comment 执行 Standard inline。
    /// 对应 Java 语义：`AbstractStandardInliner` 的 `inline_comment` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn inline_comment(
        &self,
        context: &dyn ITemplateContext,
        comment: &dyn IComment,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        if context.get_template_mode() != self.template_mode {
            let content = comment
                .get_content()
                .map_err(box_error)?
                .ok_or_else(null_text_error)?;
            return self.inline_switch_template_mode(
                context,
                content,
                comment.get_line(),
                comment.get_col().wrapping_add(4),
                false,
            );
        }
        if !EngineEventUtils::is_inlineable_comment(Some(comment)).map_err(box_error)? {
            return Ok(None);
        }
        let content = comment.java_to_string().map_err(box_error)?;
        self.perform_inlining(
            context,
            &content,
            4,
            content.len().saturating_sub(7),
            comment.get_template_name(),
            comment.get_line(),
            comment.get_col(),
        )
        .map(|value| Some(Box::new(value) as Box<dyn JavaCharSequence>))
    }

    fn inline_switch_template_mode(
        &self,
        context: &dyn ITemplateContext,
        content: JavaString,
        line: i32,
        col: i32,
        allow_lazy: bool,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        let model = context
            .get_configuration()
            .get_template_manager()
            .parse_string(
                context.get_template_data().as_ref(),
                &content,
                line,
                col,
                Some(self.template_mode),
                true,
            )
            .map_err(box_error)?;
        let model: Arc<dyn IModel> = Arc::from(model);
        if allow_lazy
            && self.write_texts_to_output
            && let Some(engine_context) = context.get_engine_context_arc()
        {
            let template_context: Arc<dyn ITemplateContext> = engine_context;
            return Ok(Some(Box::new(LazyProcessingCharSequence::new(
                template_context,
                model,
            ))));
        }
        let output = Arc::new(Mutex::new(Vec::new()));
        context
            .get_configuration()
            .get_template_manager()
            .process(
                model.as_ref(),
                context,
                Box::new(SharedUtf16Writer {
                    output: Arc::clone(&output),
                }),
            )
            .map_err(box_error)?;
        Ok(Some(Box::new(JavaString::from_utf16(
            output
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        ))))
    }

    #[allow(clippy::too_many_arguments)]
    fn perform_inlining(
        &self,
        context: &dyn ITemplateContext,
        text: &JavaString,
        offset: usize,
        len: usize,
        template_name: Option<&JavaString>,
        line: i32,
        col: i32,
    ) -> StandardExpressionResult<JavaString> {
        let expression_parser =
            StandardExpressions::get_expression_parser(context.get_configuration())?;
        let units = text.as_utf16();
        let max = offset.saturating_add(len).min(units.len());
        let mut output = Vec::with_capacity(len + len / 2);
        let mut locator = [line, col];
        let mut index = offset;
        let mut current = offset;
        let mut expression: Option<(usize, u16, i32, i32)> = None;

        while index < max {
            if let Some((start, closing, current_line, current_col)) = expression {
                let Some(end) = find_structure_end(units, index, max, closing, &mut locator) else {
                    output.extend_from_slice(&units[current..max]);
                    return Ok(JavaString::from_utf16(output));
                };
                let expression_text = JavaString::from_utf16(units[start + 2..end].to_vec());
                let escaped = closing == u16::from(b']');
                let result = self.process_expression(
                    context,
                    expression_parser.as_ref(),
                    &expression_text,
                    escaped,
                    template_name,
                    current_line,
                    current_col.wrapping_add(2),
                )?;
                output.extend_from_slice(result.as_utf16());
                count_char(&mut locator, units[end]);
                count_char(&mut locator, units[end + 1]);
                current = end + 2;
                index = current;
                expression = None;
            } else {
                let Some(start) = find_structure_start(units, index, max, &mut locator) else {
                    output.extend_from_slice(&units[current..max]);
                    return Ok(JavaString::from_utf16(output));
                };
                // Java 在下一次循环（进入 inExpression 分支）才读取 locator，因此这里
                // 必须保存扫描到表达式起点后的行列，而不是当前文本事件的起始行列。
                let current_line = locator[0];
                let current_col = locator[1];
                output.extend_from_slice(&units[current..start]);
                let closing = if units[start + 1] == u16::from(b'[') {
                    u16::from(b']')
                } else {
                    u16::from(b')')
                };
                current = start;
                index = start + 2;
                expression = Some((start, closing, current_line, current_col));
            }
        }
        if expression.is_some() {
            output.extend_from_slice(&units[current..max]);
        }
        Ok(JavaString::from_utf16(output))
    }

    #[allow(clippy::too_many_arguments)]
    fn process_expression(
        &self,
        context: &dyn ITemplateContext,
        expression_parser: &dyn crate::expression::IStandardExpressionParser,
        expression: &JavaString,
        escape: bool,
        template_name: Option<&JavaString>,
        line: i32,
        col: i32,
    ) -> StandardExpressionResult<JavaString> {
        let result = (|| {
            let unescaped = EscapedAttributeUtils::unescape_attribute(
                Some(self.template_mode),
                Some(expression),
            )
            .map_err(box_error)?;
            let Some(unescaped) = unescaped else {
                return Ok(None);
            };
            expression_parser
                .parse_expression(context, Some(&unescaped))?
                .execute(context)
        })();
        match result {
            Ok(value) if escape => self.produce_escaped_output(value.as_deref()),
            Ok(value) => Ok(value
                .as_deref()
                .filter(|value| !matches!(value, TemplateValue::Null))
                .and_then(TemplateValue::to_java_string)
                .unwrap_or_else(|| JavaString::from_rust_str(""))),
            Err(mut error) => {
                if let Some(processing) = error.downcast_mut::<TemplateProcessingException>() {
                    if !processing.has_template_name() {
                        processing
                            .set_template_name(template_name.map(JavaString::to_string_lossy));
                    }
                    if !processing.has_line_and_col() {
                        processing.set_line_and_col(line, col);
                    }
                    return Err(error);
                }
                Err(Box::new(
                    TemplateProcessingException::with_location_and_cause(
                        Some(format!(
                            "Error during execution of inlined expression '{}'",
                            expression.to_string_lossy()
                        )),
                        template_name.map(JavaString::to_string_lossy),
                        line,
                        col,
                        InlinerExpressionCause(error.to_string()),
                    ),
                ))
            }
        }
    }

    fn produce_escaped_output(
        &self,
        input: Option<&TemplateValue>,
    ) -> StandardExpressionResult<JavaString> {
        match &self.escaping {
            StandardInlinerEscaping::Html => {
                let text = input
                    .filter(|value| !matches!(value, TemplateValue::Null))
                    .and_then(TemplateValue::to_java_string)
                    .unwrap_or_else(|| JavaString::from_rust_str(""));
                Ok(escape_html4_xml(&text))
            }
            StandardInlinerEscaping::Xml => {
                let text = input
                    .filter(|value| !matches!(value, TemplateValue::Null))
                    .and_then(TemplateValue::to_java_string)
                    .unwrap_or_else(|| JavaString::from_rust_str(""));
                Ok(escape_xml10(&text))
            }
            StandardInlinerEscaping::JavaScript(serializer) => {
                serialize_value(input, |writer| serializer.serialize_value(input, writer))
            }
            StandardInlinerEscaping::Css(serializer) => {
                serialize_value(input, |writer| serializer.serialize_value(input, writer))
            }
        }
    }
}

fn find_structure_start(
    text: &[u16],
    offset: usize,
    max: usize,
    locator: &mut [i32; 2],
) -> Option<usize> {
    let mut col_index = offset;
    let mut index = offset;
    while index < max {
        let unit = text[index];
        if unit == u16::from(b'\n') {
            col_index = index;
            locator[1] = 0;
            locator[0] = locator[0].wrapping_add(1);
        } else if unit == u16::from(b'[')
            && index + 1 < max
            && matches!(text[index + 1], 0x005B | 0x0028)
        {
            locator[1] = locator[1].wrapping_add((index - col_index) as i32);
            return Some(index);
        }
        index += 1;
    }
    locator[1] = locator[1].wrapping_add((max - col_index) as i32);
    None
}

fn find_structure_end(
    text: &[u16],
    offset: usize,
    max: usize,
    closing: u16,
    locator: &mut [i32; 2],
) -> Option<usize> {
    let mut in_quotes = false;
    let mut in_apostrophes = false;
    let mut col_index = offset;
    let mut index = offset;
    while index < max {
        let unit = text[index];
        if unit == u16::from(b'\n') {
            col_index = index;
            locator[1] = 0;
            locator[0] = locator[0].wrapping_add(1);
        } else if unit == u16::from(b'"') && !in_apostrophes {
            in_quotes = !in_quotes;
        } else if unit == u16::from(b'\'') && !in_quotes {
            in_apostrophes = !in_apostrophes;
        } else if unit == closing
            && !in_quotes
            && !in_apostrophes
            && index + 1 < max
            && text[index + 1] == u16::from(b']')
        {
            locator[1] = locator[1].wrapping_add((index - col_index) as i32);
            return Some(index);
        }
        index += 1;
    }
    locator[1] = locator[1].wrapping_add((max - col_index) as i32);
    None
}

fn count_char(locator: &mut [i32; 2], unit: u16) {
    if unit == u16::from(b'\n') {
        locator[0] = locator[0].wrapping_add(1);
        locator[1] = 1;
    } else {
        locator[1] = locator[1].wrapping_add(1);
    }
}

fn escape_html4_xml(input: &JavaString) -> JavaString {
    let mut output = Vec::with_capacity(input.len());
    for &unit in input.as_utf16() {
        match unit {
            0x0022 => output.extend("&quot;".encode_utf16()),
            0x0026 => output.extend("&amp;".encode_utf16()),
            0x0027 => output.extend("&#39;".encode_utf16()),
            0x003C => output.extend("&lt;".encode_utf16()),
            0x003E => output.extend("&gt;".encode_utf16()),
            _ => output.push(unit),
        }
    }
    JavaString::from_utf16(output)
}

fn escape_xml10(input: &JavaString) -> JavaString {
    let mut output = Vec::with_capacity(input.len());
    for &unit in input.as_utf16() {
        match unit {
            0x0026 => output.extend("&amp;".encode_utf16()),
            0x003C => output.extend("&lt;".encode_utf16()),
            0x003E => output.extend("&gt;".encode_utf16()),
            0x0009 | 0x000A | 0x000D | 0x0020..=0xD7FF | 0xE000..=0xFFFD => {
                output.push(unit);
            }
            _ => {}
        }
    }
    JavaString::from_utf16(output)
}

fn serialize_value(
    input: Option<&TemplateValue>,
    operation: impl FnOnce(&mut dyn JavaWriter) -> Result<(), TemplateProcessingException>,
) -> StandardExpressionResult<JavaString> {
    let output = Arc::new(Mutex::new(Vec::new()));
    let mut writer = SharedUtf16Writer {
        output: Arc::clone(&output),
    };
    let _ = input;
    operation(&mut writer).map_err(box_error)?;
    Ok(JavaString::from_utf16(
        output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone(),
    ))
}

struct SharedUtf16Writer {
    output: Arc<Mutex<Vec<u16>>>,
}

impl JavaWriter for SharedUtf16Writer {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        self.output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(characters);
        Ok(())
    }
}

#[derive(Debug)]
struct InlinerExpressionCause(String);

impl std::fmt::Display for InlinerExpressionCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for InlinerExpressionCause {}

fn box_error<E>(error: E) -> StandardExpressionError
where
    E: std::error::Error + Send + Sync + 'static,
{
    Box::new(error)
}

fn null_text_error() -> StandardExpressionError {
    Box::new(crate::util::TextUtilsError::NullPointer)
}
