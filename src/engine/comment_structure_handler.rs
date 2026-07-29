#![expect(
    dead_code,
    reason = "由同批后续 ProcessorTemplateHandler 复用并读取动作状态"
)]

use std::sync::Arc;

use crate::comment::ICommentStructureHandler;
use crate::model::IModel;
use crate::util::JavaString;

/// 引擎内部 Comment 结构动作状态机。
///
/// 对应 Java: `org.thymeleaf.engine.CommentStructureHandler`。
pub(crate) struct CommentStructureHandler {
    pub(crate) set_content: bool,
    pub(crate) set_content_value: Option<JavaString>,
    pub(crate) replace_with_model: bool,
    pub(crate) replace_with_model_value: Option<Arc<dyn IModel>>,
    pub(crate) replace_with_model_processable: bool,
    pub(crate) remove_comment: bool,
}

impl CommentStructureHandler {
    /// 创建无待执行动作的处理器。
    pub(crate) fn new() -> Self {
        Self {
            set_content: false,
            set_content_value: None,
            replace_with_model: false,
            replace_with_model_value: None,
            replace_with_model_processable: false,
            remove_comment: false,
        }
    }
}

impl ICommentStructureHandler for CommentStructureHandler {
    fn reset(&mut self) {
        self.set_content = false;
        self.set_content_value = None;
        self.replace_with_model = false;
        self.replace_with_model_value = None;
        self.replace_with_model_processable = false;
        self.remove_comment = false;
    }

    fn set_content(&mut self, content: JavaString) {
        self.reset();
        self.set_content = true;
        self.set_content_value = Some(content);
    }

    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.reset();
        self.replace_with_model = true;
        self.replace_with_model_value = Some(model);
        self.replace_with_model_processable = processable;
    }

    fn remove_comment(&mut self) {
        self.reset();
        self.remove_comment = true;
    }
}
