use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::IComment;
use crate::processor::{AbstractProcessorAdapter, IProcessor};
use crate::util::ValidateError;

use super::{ICommentProcessor, ICommentStructureHandler};

/// 捕获 `doProcess` 异常并补充注释事件位置的抽象 Comment Processor。
///
/// 对应 Java: `org.thymeleaf.processor.comment.AbstractCommentProcessor`。
pub struct AbstractCommentProcessor<F> {
    adapter: AbstractProcessorAdapter<F>,
}

impl<F> AbstractCommentProcessor<F> {
    /// 创建以闭包表达 Java 抽象 `doProcess` 方法的 Processor。
    pub fn new(
        template_mode: Option<TemplateMode>,
        precedence: i32,
        processor_class_name: &'static str,
        do_process: F,
    ) -> Result<Self, ValidateError> {
        Ok(Self {
            adapter: AbstractProcessorAdapter::new(
                template_mode,
                precedence,
                processor_class_name,
                do_process,
            )?,
        })
    }
}

impl<F> IProcessor for AbstractCommentProcessor<F> {
    fn java_class_name(&self) -> &'static str {
        self.adapter.processor_class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.adapter.template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.adapter.precedence()
    }
}

impl<F> ICommentProcessor for AbstractCommentProcessor<F>
where
    F: Fn(
        &dyn ITemplateContext,
        &dyn IComment,
        &mut dyn ICommentStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>,
{
    fn process(
        &self,
        context: &dyn ITemplateContext,
        comment: &dyn IComment,
        structure_handler: &mut dyn ICommentStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.adapter.execute(comment, |callback| {
            callback(context, comment, structure_handler)
        })
    }
}
