use std::sync::Arc;

use crate::comment::ICommentStructureHandler;
use crate::model::IModel;
use crate::util::{JavaCharSequence, JavaString, Validate, ValidateError};

/// 引擎内部 Comment 结构动作状态机。
///
/// 对应 Java: `org.thymeleaf.engine.CommentStructureHandler`。
pub(crate) struct CommentStructureHandler {
    pub(crate) set_content: bool,
    pub(crate) set_content_value: Option<Arc<dyn JavaCharSequence>>,
    pub(crate) replace_with_model: bool,
    pub(crate) replace_with_model_value: Option<Arc<dyn IModel>>,
    pub(crate) replace_with_model_processable: bool,
    pub(crate) remove_comment: bool,
}

impl CommentStructureHandler {
    /// 创建无待执行动作的处理器。
    /// 对应 Java 语义：`CommentStructureHandler` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new() -> Self {
        let mut handler = Self {
            set_content: false,
            set_content_value: None,
            replace_with_model: false,
            replace_with_model_value: None,
            replace_with_model_processable: false,
            remove_comment: false,
        };
        handler.reset();
        handler
    }

    /// 设置不含注释边界的内容。对应 Java:
    /// `CommentStructureHandler#setContent(CharSequence)`。
    ///
    /// 方法先重置，再校验非空；失败消息精确为 `"Content cannot be null"`。
    pub(crate) fn set_content_nullable(
        &mut self,
        content: Option<Arc<dyn JavaCharSequence>>,
    ) -> Result<(), ValidateError> {
        self.reset();
        Validate::not_null(content.as_deref(), Some("Content cannot be null"))?;
        self.set_content = true;
        self.set_content_value = content;
        Ok(())
    }

    /// 使用模型替换注释。对应 Java:
    /// `CommentStructureHandler#replaceWith(IModel, boolean)`。
    pub(crate) fn replace_with_nullable(
        &mut self,
        model: Option<Arc<dyn IModel>>,
        processable: bool,
    ) -> Result<(), ValidateError> {
        self.reset();
        Validate::not_null(model.as_deref(), Some("Model cannot be null"))?;
        self.replace_with_model = true;
        self.replace_with_model_value = model;
        self.replace_with_model_processable = processable;
        Ok(())
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
        self.set_content_nullable(Some(Arc::new(content)))
            .expect("Rust non-null content boundary must satisfy Java validation");
    }

    fn set_content_sequence(&mut self, content: Arc<dyn JavaCharSequence>) {
        self.set_content_nullable(Some(content))
            .expect("Rust non-null content boundary must satisfy Java validation");
    }

    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.replace_with_nullable(Some(model), processable)
            .expect("Rust non-null model boundary must satisfy Java validation");
    }

    fn remove_comment(&mut self) {
        self.reset();
        self.remove_comment = true;
    }
}
