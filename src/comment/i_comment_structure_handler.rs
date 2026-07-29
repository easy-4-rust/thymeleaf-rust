use std::sync::Arc;

use crate::model::IModel;
use crate::util::JavaString;

/// Comment Processor 的结构变更合同。
///
/// 对应 Java: `org.thymeleaf.processor.comment.ICommentStructureHandler`。
pub trait ICommentStructureHandler {
    /// 清除已指定动作。
    fn reset(&mut self);
    /// 设置不含注释边界的新内容。
    fn set_content(&mut self, content: JavaString);
    /// 使用模型替换注释。
    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除注释。
    fn remove_comment(&mut self);
}
