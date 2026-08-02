use std::sync::Arc;

use crate::model::IModel;
use crate::util::JavaString;

/// ProcessingInstruction Processor 的结构变更合同。
///
/// 对应 Java:
/// `org.thymeleaf.processor.processinginstruction.IProcessingInstructionStructureHandler`。
pub trait IProcessingInstructionStructureHandler {
    /// 清除已指定动作。对应 Java:
    /// `IProcessingInstructionStructureHandler#reset()`。
    fn reset(&mut self);
    /// 设置非空 target 和 content。对应 Java:
    /// `IProcessingInstructionStructureHandler#setProcessingInstruction(String, String)`。
    fn set_processing_instruction(&mut self, target: JavaString, content: JavaString);
    /// 使用模型替换当前事件。对应 Java:
    /// `IProcessingInstructionStructureHandler#replaceWith(IModel, boolean)`。
    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除当前 processing instruction。对应 Java:
    /// `IProcessingInstructionStructureHandler#removeProcessingInstruction()`。
    fn remove_processing_instruction(&mut self);
}
