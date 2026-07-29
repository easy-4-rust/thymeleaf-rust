use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::IProcessingInstruction;
use crate::processor::{AbstractProcessorAdapter, IProcessor};
use crate::util::ValidateError;

use super::{IProcessingInstructionProcessor, IProcessingInstructionStructureHandler};

/// 捕获 `doProcess` 异常并补充 processing instruction 位置的抽象 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.processor.processinginstruction.AbstractProcessingInstructionProcessor`。
pub struct AbstractProcessingInstructionProcessor<F> {
    adapter: AbstractProcessorAdapter<F>,
}

impl<F> AbstractProcessingInstructionProcessor<F> {
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

impl<F> IProcessor for AbstractProcessingInstructionProcessor<F> {
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

impl<F> IProcessingInstructionProcessor for AbstractProcessingInstructionProcessor<F>
where
    F: Fn(
        &dyn ITemplateContext,
        &dyn IProcessingInstruction,
        &mut dyn IProcessingInstructionStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>,
{
    fn process(
        &self,
        context: &dyn ITemplateContext,
        processing_instruction: &dyn IProcessingInstruction,
        structure_handler: &mut dyn IProcessingInstructionStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.adapter.execute(processing_instruction, |callback| {
            callback(context, processing_instruction, structure_handler)
        })
    }
}
