use std::sync::Arc;

use crate::model::IModel;
use crate::util::JavaString;

/// ProcessingInstruction Processor 的结构变更合同。
///
/// 对应 Java:
/// `org.thymeleaf.processor.processinginstruction.IProcessingInstructionStructureHandler`。
pub trait IProcessingInstructionStructureHandler {
    /// 清除已指定动作。
    fn reset(&mut self);
    /// 设置 target 和 content。
    fn set_processing_instruction(&mut self, target: JavaString, content: JavaString);
    /// 使用模型替换当前事件。
    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除当前 processing instruction。
    fn remove_processing_instruction(&mut self);
}
