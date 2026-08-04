use std::sync::Arc;

use crate::model::IModel;
use crate::util::{JavaCharSequence, Utf16String};

/// Text Processor 指示引擎修改当前事件的合同。
///
/// 对应 Java: `org.thymeleaf.processor.text.ITextStructureHandler`。
pub trait ITextStructureHandler {
    /// 清除当前 Processor 已指定的所有动作。对应 Java:
    /// `ITextStructureHandler#reset()`。
    fn reset(&mut self);
    /// 设置新的文本内容。对应 Java:
    /// `ITextStructureHandler#setText(CharSequence)`。
    ///
    /// Rust 的非空参数在调用边界排除 Java null；引擎实现仍以 nullable 入口验证
    /// Java 精确错误。调用后将取消此前的替换或删除动作。
    fn set_text(&mut self, text: Utf16String);
    /// 使用任意 Java `CharSequence` 设置文本并保留对象身份及延迟 Writer 输出能力。
    ///
    /// 对应 Java: `ITextStructureHandler#setText(CharSequence)`。
    fn set_text_sequence(&mut self, text: Arc<dyn JavaCharSequence>);
    /// 使用模型替换当前事件。对应 Java:
    /// `ITextStructureHandler#replaceWith(IModel, boolean)`。
    ///
    /// `processable` 决定替换模型是否再次经过 Processor 链。
    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除当前文本事件。对应 Java: `ITextStructureHandler#removeText()`。
    fn remove_text(&mut self);
}
