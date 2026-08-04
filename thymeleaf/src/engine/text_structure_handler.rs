use std::sync::Arc;

use crate::model::IModel;
use crate::text::ITextStructureHandler;
use crate::util::{JavaCharSequence, Utf16String, Validate, ValidateError};

/// 引擎内部 Text 结构动作状态机。
///
/// 每个互斥动作先执行 `reset`；因此同一个 Processor 执行中最后一次
/// set/replace/remove 调用获胜。
///
/// 对应 Java: `org.thymeleaf.engine.TextStructureHandler`。
pub(crate) struct TextStructureHandler {
    pub(crate) set_text: bool,
    pub(crate) set_text_value: Option<Arc<dyn JavaCharSequence>>,
    pub(crate) replace_with_model: bool,
    pub(crate) replace_with_model_value: Option<Arc<dyn IModel>>,
    pub(crate) replace_with_model_processable: bool,
    pub(crate) remove_text: bool,
}

impl TextStructureHandler {
    /// 创建无待执行动作的处理器。对应 Java:
    /// `TextStructureHandler#TextStructureHandler()`。
    pub(crate) fn new() -> Self {
        let mut handler = Self {
            set_text: false,
            set_text_value: None,
            replace_with_model: false,
            replace_with_model_value: None,
            replace_with_model_processable: false,
            remove_text: false,
        };
        handler.reset();
        handler
    }

    /// 设置任意 `CharSequence` 文本，并保留 Java 的空值错误与清理顺序。
    ///
    /// 对应 Java: `TextStructureHandler#setText(CharSequence)`。方法先清除之前的
    /// 互斥动作，再校验 `text`；因此传入 `None` 时返回
    /// `"Text cannot be null"`，且处理器保持已重置状态。
    pub(crate) fn set_text_nullable(
        &mut self,
        text: Option<Arc<dyn JavaCharSequence>>,
    ) -> Result<(), ValidateError> {
        self.reset();
        Validate::not_null(text.as_deref(), Some("Text cannot be null"))?;
        self.set_text = true;
        self.set_text_value = text;
        Ok(())
    }

    /// 使用模型替换文本，并保留 Java 的空值错误与清理顺序。
    ///
    /// 对应 Java: `TextStructureHandler#replaceWith(IModel, boolean)`。
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

impl ITextStructureHandler for TextStructureHandler {
    fn reset(&mut self) {
        self.set_text = false;
        self.set_text_value = None;
        self.replace_with_model = false;
        self.replace_with_model_value = None;
        self.replace_with_model_processable = false;
        self.remove_text = false;
    }

    fn set_text(&mut self, text: Utf16String) {
        self.set_text_nullable(Some(Arc::new(text)))
            .expect("Rust non-null text boundary must satisfy Java validation");
    }

    fn set_text_sequence(&mut self, text: Arc<dyn JavaCharSequence>) {
        self.set_text_nullable(Some(text))
            .expect("Rust non-null text boundary must satisfy Java validation");
    }

    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.replace_with_nullable(Some(model), processable)
            .expect("Rust non-null model boundary must satisfy Java validation");
    }

    fn remove_text(&mut self) {
        self.reset();
        self.remove_text = true;
    }
}
