use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::IProcessingInstruction;
use thymeleaf::processinginstruction::{
    AbstractProcessingInstructionProcessor, IProcessingInstructionProcessor,
    IProcessingInstructionStructureHandler,
};
use thymeleaf::processor::IProcessor;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &dyn IProcessingInstruction,
    &mut dyn IProcessingInstructionStructureHandler,
) -> ProcessResult;

/// 删除遇到的 ProcessingInstruction 模板事件。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.remove.RemoveProcessingInstructionProcessor`。
pub struct RemoveProcessingInstructionProcessor {
    processor: AbstractProcessingInstructionProcessor<ProcessCallback>,
}

impl RemoveProcessingInstructionProcessor {
    /// 创建 HTML 模式、precedence 1000 的事件处理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            processor: AbstractProcessingInstructionProcessor::new(
                Some(TemplateMode::HTML),
                1000,
                "org.thymeleaf.templateengine.processors.dialects.remove.RemoveProcessingInstructionProcessor",
                remove_event as ProcessCallback,
            )
            .expect("the fixed remove processor configuration is valid"),
        }
    }
}

impl Default for RemoveProcessingInstructionProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl IProcessor for RemoveProcessingInstructionProcessor {
    fn as_processing_instruction_processor(&self) -> Option<&dyn IProcessingInstructionProcessor> {
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

impl IProcessingInstructionProcessor for RemoveProcessingInstructionProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        event: &dyn IProcessingInstruction,
        structure_handler: &mut dyn IProcessingInstructionStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, event, structure_handler)
    }
}

fn remove_event(
    _context: &dyn ITemplateContext,
    _event: &dyn IProcessingInstruction,
    structure_handler: &mut dyn IProcessingInstructionStructureHandler,
) -> ProcessResult {
    structure_handler.remove_processing_instruction();
    Ok(())
}
