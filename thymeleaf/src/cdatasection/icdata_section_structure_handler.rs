use std::sync::Arc;

use crate::model::IModel;
use crate::util::{CharSequenceValue, Utf16String};

/// CDATA Processor 的结构变更合同。
///
/// 对应 Java: `org.thymeleaf.processor.cdatasection.ICDATASectionStructureHandler`。
pub trait ICDATASectionStructureHandler {
    /// 清除已指定动作。对应 Java: `ICDATASectionStructureHandler#reset()`。
    fn reset(&mut self);
    /// 设置不含 CDATA 边界的新内容。对应 Java:
    /// `ICDATASectionStructureHandler#setContent(CharSequence)`。
    fn set_content(&mut self, content: Utf16String);
    /// 使用任意 Java `CharSequence` 设置内容，保留对象身份和延迟 Writer 输出能力。
    ///
    /// 对应 Java: `ICDATASectionStructureHandler#setContent(CharSequence)`。
    fn set_content_sequence(&mut self, content: Arc<dyn CharSequenceValue>);
    /// 使用模型替换当前事件。对应 Java:
    /// `ICDATASectionStructureHandler#replaceWith(IModel, boolean)`。
    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除当前 CDATA。对应 Java:
    /// `ICDATASectionStructureHandler#removeCDATASection()`。
    fn remove_cdata_section(&mut self);
}
