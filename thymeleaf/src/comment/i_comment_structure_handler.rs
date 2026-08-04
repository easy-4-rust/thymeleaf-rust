use std::sync::Arc;

use crate::model::IModel;
use crate::util::{JavaCharSequence, Utf16String};

/// Comment Processor 的结构变更合同。
///
/// 对应 Java: `org.thymeleaf.processor.comment.ICommentStructureHandler`。
pub trait ICommentStructureHandler {
    /// 清除已指定动作。对应 Java: `ICommentStructureHandler#reset()`。
    fn reset(&mut self);
    /// 设置不含注释边界的新内容。对应 Java:
    /// `ICommentStructureHandler#setContent(CharSequence)`。
    fn set_content(&mut self, content: Utf16String);
    /// 使用任意 Java `CharSequence` 设置内容，保留对象身份和延迟 Writer 输出能力。
    ///
    /// 对应 Java: `ICommentStructureHandler#setContent(CharSequence)`。
    fn set_content_sequence(&mut self, content: Arc<dyn JavaCharSequence>);
    /// 使用模型替换注释。对应 Java:
    /// `ICommentStructureHandler#replaceWith(IModel, boolean)`。
    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除注释。对应 Java: `ICommentStructureHandler#removeComment()`。
    fn remove_comment(&mut self);
}
