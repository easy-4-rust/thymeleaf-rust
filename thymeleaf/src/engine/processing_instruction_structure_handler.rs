use std::sync::Arc;

use crate::model::IModel;
use crate::processinginstruction::IProcessingInstructionStructureHandler;
use crate::util::{JavaString, Validate, ValidateError};

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
        let mut handler = Self {
            set_processing_instruction: false,
            set_processing_instruction_target: None,
            set_processing_instruction_content: None,
            replace_with_model: false,
            replace_with_model_value: None,
            replace_with_model_processable: false,
            remove_processing_instruction: false,
        };
        handler.reset();
        handler
    }

    /// 设置 processing instruction 的 target 与 content。
    ///
    /// 对应 Java:
    /// `ProcessingInstructionStructureHandler#setProcessingInstruction(String, String)`。
    /// 方法先重置，再按 target、content 的顺序校验。
    pub(crate) fn set_processing_instruction_nullable(
        &mut self,
        target: Option<JavaString>,
        content: Option<JavaString>,
    ) -> Result<(), ValidateError> {
        self.reset();
        Validate::not_null(target.as_ref(), Some("Target cannot be null"))?;
        Validate::not_null(content.as_ref(), Some("Content cannot be null"))?;
        self.set_processing_instruction = true;
        self.set_processing_instruction_target = target;
        self.set_processing_instruction_content = content;
        Ok(())
    }

    /// 使用模型替换 processing instruction。对应 Java:
    /// `ProcessingInstructionStructureHandler#replaceWith(IModel, boolean)`。
    pub(crate) fn replace_with_nullable(
        &mut self,
        model: Option<Arc<dyn IModel>>,
        processable: bool,
    ) -> Result<(), ValidateError> {
        self.reset();
        Validate::not_null(model.as_deref(), Some("Model cannot be null"))?;
        self.replace_with_model = true;
        self.replace_with_model_value = model;
        self.replace_with_model_processable = processable;
        Ok(())
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
        self.set_processing_instruction_nullable(Some(target), Some(content))
            .expect("Rust non-null processing-instruction boundary must satisfy Java validation");
    }

    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.replace_with_nullable(Some(model), processable)
            .expect("Rust non-null model boundary must satisfy Java validation");
    }

    fn remove_processing_instruction(&mut self) {
        self.reset();
        self.remove_processing_instruction = true;
    }
}
