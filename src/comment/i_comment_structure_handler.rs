use std::sync::Arc;

use crate::model::IModel;
use crate::util::{JavaCharSequence, JavaString};

/// Comment Processor 的结构变更合同。
///
/// 对应 Java: `org.thymeleaf.processor.comment.ICommentStructureHandler`。
pub trait ICommentStructureHandler {
    /// 清除已指定动作。
    fn reset(&mut self);
    /// 设置不含注释边界的新内容。
    fn set_content(&mut self, content: JavaString);
    /// 使用任意 Java CharSequence 设置内容，保留延迟 Writer 输出能力。
    fn set_content_sequence(&mut self, content: Arc<dyn JavaCharSequence>);
    /// 使用模型替换注释。
    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除注释。
    fn remove_comment(&mut self);
}
