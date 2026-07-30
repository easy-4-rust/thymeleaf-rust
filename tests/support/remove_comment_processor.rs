use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::IComment;
use thymeleaf::processor::IProcessor;
use thymeleaf::comment::{AbstractCommentProcessor, ICommentProcessor, ICommentStructureHandler};

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback =
    fn(&dyn ITemplateContext, &dyn IComment, &mut dyn ICommentStructureHandler) -> ProcessResult;

/// 删除遇到的 Comment 模板事件。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.remove.RemoveCommentProcessor`。
pub struct RemoveCommentProcessor {
    processor: AbstractCommentProcessor<ProcessCallback>,
}

impl RemoveCommentProcessor {
    /// 创建 HTML 模式、precedence 1000 的事件处理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            processor: AbstractCommentProcessor::new(
                Some(TemplateMode::HTML),
                1000,
                "org.thymeleaf.templateengine.processors.dialects.remove.RemoveCommentProcessor",
                remove_event as ProcessCallback,
            )
            .expect("the fixed remove processor configuration is valid"),
        }
    }
}

impl Default for RemoveCommentProcessor {
    fn default() -> Self { Self::new() }
}

impl IProcessor for RemoveCommentProcessor {
    fn as_comment_processor(&self) -> Option<&dyn ICommentProcessor> { Some(self) }
    fn java_class_name(&self) -> &'static str { self.processor.java_class_name() }
    fn get_template_mode(&self) -> Option<TemplateMode> { self.processor.get_template_mode() }
    fn get_precedence(&self) -> i32 { self.processor.get_precedence() }
}

impl ICommentProcessor for RemoveCommentProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        event: &dyn IComment,
        structure_handler: &mut dyn ICommentStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, event, structure_handler)
    }
}

fn remove_event(
    _context: &dyn ITemplateContext,
    _event: &dyn IComment,
    structure_handler: &mut dyn ICommentStructureHandler,
) -> ProcessResult {
    structure_handler.remove_comment();
    Ok(())
}

