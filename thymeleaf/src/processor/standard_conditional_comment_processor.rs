use std::io;
use std::sync::{Arc, Mutex};

use crate::TemplateMode;
use crate::comment::{AbstractCommentProcessor, ICommentProcessor, ICommentStructureHandler};
use crate::context::ITemplateContext;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::IComment;
use crate::processor::IProcessor;
use crate::util::{JavaString, JavaWriter, StandardConditionalCommentUtils};

type CommentCallback = Box<
    dyn Fn(
            &dyn ITemplateContext,
            &dyn IComment,
            &mut dyn ICommentStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
>;

/// 解析并处理 IE 条件注释内部模板片段的 Processor。
///
/// 条件表达式外壳保持原 UTF-16 内容，内部 markup 作为模板片段完整处理。对应 Java:
/// `org.thymeleaf.standard.processor.StandardConditionalCommentProcessor`。
pub struct StandardConditionalCommentProcessor {
    processor: AbstractCommentProcessor<CommentCallback>,
}

impl StandardConditionalCommentProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1100;

    /// 创建 HTML 条件注释 Processor。
    /// 对应 Java 语义：`StandardConditionalCommentProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new() -> Result<Self, TemplateProcessingException> {
        let callback: CommentCallback = Box::new(|context, comment, structure_handler| {
            let Some(parsing) = StandardConditionalCommentUtils::parse_conditional_comment(Some(
                comment,
            ))
            .map_err(|error| {
                Box::new(TemplateProcessingException::with_cause(
                    Some("Could not parse conditional comment".to_owned()),
                    error,
                )) as Box<dyn TemplateEngineException>
            })?
            else {
                return Ok(());
            };
            let comment_text = comment
                .get_comment()
                .map_err(|error| {
                    Box::new(TemplateProcessingException::with_cause(
                        Some("Could not read conditional comment".to_owned()),
                        error,
                    )) as Box<dyn TemplateEngineException>
                })?
                .ok_or_else(|| {
                    Box::new(TemplateProcessingException::new(Some(
                        "Conditional comment text cannot be null".to_owned(),
                    ))) as Box<dyn TemplateEngineException>
                })?;
            let content = slice(
                &comment_text,
                parsing.get_content_offset(),
                parsing.get_content_len(),
            )?;
            let owner = context.get_template_data();
            let model = context
                .get_configuration()
                .get_template_manager()
                .parse_string(
                    owner.as_ref(),
                    &content,
                    comment.get_line(),
                    comment.get_col(),
                    None,
                    true,
                )
                .map_err(|error| {
                    Box::new(TemplateProcessingException::with_cause(
                        Some("Could not parse conditional comment content".to_owned()),
                        error,
                    )) as Box<dyn TemplateEngineException>
                })?;

            let output = Arc::new(Mutex::new(Vec::new()));
            append_ascii(&output, "[");
            append_slice(
                &output,
                &comment_text,
                parsing.get_start_expression_offset(),
                parsing.get_start_expression_len(),
            )?;
            append_ascii(&output, "]>");
            context
                .get_configuration()
                .get_template_manager()
                .process(
                    model.as_ref(),
                    context,
                    Box::new(SharedWriter {
                        output: Arc::clone(&output),
                    }),
                )
                .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
            append_ascii(&output, "<![");
            append_slice(
                &output,
                &comment_text,
                parsing.get_end_expression_offset(),
                parsing.get_end_expression_len(),
            )?;
            append_ascii(&output, "]");
            let rendered = JavaString::from_utf16(
                output
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone(),
            );
            structure_handler.set_content(rendered);
            Ok(())
        });
        Ok(Self {
            processor: AbstractCommentProcessor::new(
                Some(TemplateMode::HTML),
                Self::PRECEDENCE,
                "org.thymeleaf.standard.processor.StandardConditionalCommentProcessor",
                callback,
            )
            .map_err(|error| {
                TemplateProcessingException::with_cause(
                    Some("Could not create conditional comment processor".to_owned()),
                    error,
                )
            })?,
        })
    }
}

impl IProcessor for StandardConditionalCommentProcessor {
    fn as_comment_processor(&self) -> Option<&dyn ICommentProcessor> {
        Some(self)
    }

    fn java_class_name(&self) -> &'static str {
        self.processor.java_class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.processor.get_template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.processor.get_precedence()
    }
}

impl ICommentProcessor for StandardConditionalCommentProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        comment: &dyn IComment,
        structure_handler: &mut dyn ICommentStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, comment, structure_handler)
    }
}

struct SharedWriter {
    output: Arc<Mutex<Vec<u16>>>,
}

impl JavaWriter for SharedWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        self.output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(characters);
        Ok(())
    }
}

fn slice(
    value: &JavaString,
    offset: i32,
    length: i32,
) -> Result<JavaString, Box<dyn TemplateEngineException>> {
    let start = usize::try_from(offset).map_err(|error| {
        Box::new(TemplateProcessingException::with_cause(
            Some("Conditional comment offset is negative".to_owned()),
            error,
        )) as Box<dyn TemplateEngineException>
    })?;
    let end = usize::try_from(offset.wrapping_add(length)).map_err(|error| {
        Box::new(TemplateProcessingException::with_cause(
            Some("Conditional comment end offset is negative".to_owned()),
            error,
        )) as Box<dyn TemplateEngineException>
    })?;
    let units = value.as_utf16().get(start..end).ok_or_else(|| {
        Box::new(TemplateProcessingException::new(Some(
            "Conditional comment range is out of bounds".to_owned(),
        ))) as Box<dyn TemplateEngineException>
    })?;
    Ok(JavaString::from_utf16(units.to_vec()))
}

fn append_slice(
    output: &Arc<Mutex<Vec<u16>>>,
    value: &JavaString,
    offset: i32,
    length: i32,
) -> Result<(), Box<dyn TemplateEngineException>> {
    let value = slice(value, offset, length)?;
    output
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .extend_from_slice(value.as_utf16());
    Ok(())
}

fn append_ascii(output: &Arc<Mutex<Vec<u16>>>, value: &str) {
    output
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .extend(value.encode_utf16());
}
