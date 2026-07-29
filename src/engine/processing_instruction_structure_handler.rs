#![expect(
    dead_code,
    reason = "由同批后续 ProcessorTemplateHandler 复用并读取动作状态"
)]

use std::sync::Arc;

use crate::model::IModel;
use crate::processinginstruction::IProcessingInstructionStructureHandler;
use crate::util::JavaString;

/// 引擎内部 ProcessingInstruction 结构动作状态机。
///
/// 对应 Java:
/// `org.thymeleaf.engine.ProcessingInstructionStructureHandler`。
pub(crate) struct ProcessingInstructionStructureHandler {
    pub(crate) set_processing_instruction: bool,
    pub(crate) set_processing_instruction_target: Option<JavaString>,
    pub(crate) set_processing_instruction_content: Option<JavaString>,
    pub(crate) replace_with_model: bool,
    pub(crate) replace_with_model_value: Option<Arc<dyn IModel>>,
    pub(crate) replace_with_model_processable: bool,
    pub(crate) remove_processing_instruction: bool,
}

impl ProcessingInstructionStructureHandler {
    /// 创建无待执行动作的处理器。
    pub(crate) fn new() -> Self {
        Self {
            set_processing_instruction: false,
            set_processing_instruction_target: None,
            set_processing_instruction_content: None,
            replace_with_model: false,
            replace_with_model_value: None,
            replace_with_model_processable: false,
            remove_processing_instruction: false,
        }
    }
}

impl IProcessingInstructionStructureHandler for ProcessingInstructionStructureHandler {
    fn reset(&mut self) {
        self.set_processing_instruction = false;
        self.set_processing_instruction_target = None;
        self.set_processing_instruction_content = None;
        self.replace_with_model = false;
        self.replace_with_model_value = None;
        self.replace_with_model_processable = false;
        self.remove_processing_instruction = false;
    }

    fn set_processing_instruction(&mut self, target: JavaString, content: JavaString) {
        self.reset();
        self.set_processing_instruction = true;
        self.set_processing_instruction_target = Some(target);
        self.set_processing_instruction_content = Some(content);
    }

    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.reset();
        self.replace_with_model = true;
        self.replace_with_model_value = Some(model);
        self.replace_with_model_processable = processable;
    }

    fn remove_processing_instruction(&mut self) {
        self.reset();
        self.remove_processing_instruction = true;
    }
}
