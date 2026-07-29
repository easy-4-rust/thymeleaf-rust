use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::IComment;
use crate::processor::IProcessor;

use super::ICommentStructureHandler;

/// Comment 事件 Processor 合同。
///
/// 对应 Java: `org.thymeleaf.processor.comment.ICommentProcessor`。
pub trait ICommentProcessor: IProcessor {
    /// 处理注释事件。
    fn process(
        &self,
        context: &dyn ITemplateContext,
        comment: &dyn IComment,
        structure_handler: &mut dyn ICommentStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
}
