use std::sync::Arc;

use crate::model::IModel;
use crate::util::JavaString;

/// Text Processor 指示引擎修改当前事件的合同。
///
/// 对应 Java: `org.thymeleaf.processor.text.ITextStructureHandler`。
pub trait ITextStructureHandler {
    /// 清除当前 Processor 已指定的所有动作。
    fn reset(&mut self);
    /// 设置新的文本内容。
    fn set_text(&mut self, text: JavaString);
    /// 使用模型替换当前事件。
    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除当前文本事件。
    fn remove_text(&mut self);
}
