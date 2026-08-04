use std::sync::Arc;

use crate::TemplateMode;
use crate::comment::{AbstractCommentProcessor, ICommentProcessor, ICommentStructureHandler};
use crate::context::ITemplateContext;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::IComment;
use crate::processor::IProcessor;

use super::expression_processing_error;

type CommentCallback = Box<
    dyn Fn(
            &dyn ITemplateContext,
            &dyn IComment,
            &mut dyn ICommentStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
>;

/// 对 Comment 内容应用当前上下文 Inliner 的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardInliningCommentProcessor`。
pub struct StandardInliningCommentProcessor {
    processor: AbstractCommentProcessor<CommentCallback>,
}

impl StandardInliningCommentProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;

    /// 创建指定模板模式的内联 Processor。
    /// 对应 Java 语义：`StandardInliningCommentProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(template_mode: TemplateMode) -> Result<Self, TemplateProcessingException> {
        let callback: CommentCallback = Box::new(|context, comment, structure_handler| {
            let Some(inliner) = context.get_inliner() else {
                return Ok(());
            };
            if let Some(inlined) = inliner
                .inline_comment(context, comment)
                .map_err(|error| expression_processing_error("Could not inline comment", error))?
            {
                structure_handler.set_content_sequence(Arc::from(inlined));
            }
            Ok(())
        });
        Ok(Self {
            processor: AbstractCommentProcessor::new(
                Some(template_mode),
                Self::PRECEDENCE,
                "org.thymeleaf.standard.processor.StandardInliningCommentProcessor",
                callback,
            )
            .map_err(|error| {
                TemplateProcessingException::with_cause(
                    Some("Could not create inlining comment processor".to_owned()),
                    error,
                )
            })?,
        })
    }
}

impl IProcessor for StandardInliningCommentProcessor {
    fn as_comment_processor(&self) -> Option<&dyn ICommentProcessor> {
        Some(self)
    }

    fn class_name(&self) -> &'static str {
        self.processor.class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.processor.get_template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.processor.get_precedence()
    }
}

impl ICommentProcessor for StandardInliningCommentProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        comment: &dyn IComment,
        structure_handler: &mut dyn ICommentStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, comment, structure_handler)
    }
}
