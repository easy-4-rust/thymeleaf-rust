use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::IProcessingInstruction;
use crate::processor::IProcessor;

use super::IProcessingInstructionStructureHandler;

/// ProcessingInstruction 事件 Processor 合同。
///
/// 对应 Java:
/// `org.thymeleaf.processor.processinginstruction.IProcessingInstructionProcessor`。
pub trait IProcessingInstructionProcessor: IProcessor {
    /// 处理 processing instruction。
    ///
    /// 对应 Java:
    /// `IProcessingInstructionProcessor#process(ITemplateContext,
    /// IProcessingInstruction, IProcessingInstructionStructureHandler)`。
    /// 事件不可变，结构变更通过 handler 声明。
    fn process(
        &self,
        context: &dyn ITemplateContext,
        processing_instruction: &dyn IProcessingInstruction,
        structure_handler: &mut dyn IProcessingInstructionStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
}
