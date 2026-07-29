use crate::util::JavaString;

use super::ITemplateEvent;

/// 不可变 XML processing instruction 事件。
///
/// 对应 Java: `org.thymeleaf.model.IProcessingInstruction`。
pub trait IProcessingInstruction: ITemplateEvent {
    /// 返回 processing instruction target。
    fn get_target(&self) -> Option<&JavaString>;
    /// 返回 processing instruction 内容。
    fn get_content(&self) -> Option<&JavaString>;
    /// 返回包含边界的完整 processing instruction。
    fn get_processing_instruction(&self) -> Option<&JavaString>;
}
