#![expect(
    dead_code,
    reason = "由同批后续 ProcessorTemplateHandler 复用并读取动作状态"
)]

use std::sync::Arc;

use crate::model::IModel;
use crate::text::ITextStructureHandler;
use crate::util::JavaString;

/// 引擎内部 Text 结构动作状态机。
///
/// 每个互斥动作先执行 `reset`；因此同一个 Processor 执行中最后一次
/// set/replace/remove 调用获胜。
///
/// 对应 Java: `org.thymeleaf.engine.TextStructureHandler`。
pub(crate) struct TextStructureHandler {
    pub(crate) set_text: bool,
    pub(crate) set_text_value: Option<JavaString>,
    pub(crate) replace_with_model: bool,
    pub(crate) replace_with_model_value: Option<Arc<dyn IModel>>,
    pub(crate) replace_with_model_processable: bool,
    pub(crate) remove_text: bool,
}

impl TextStructureHandler {
    /// 创建无待执行动作的处理器。
    pub(crate) fn new() -> Self {
        Self {
            set_text: false,
            set_text_value: None,
            replace_with_model: false,
            replace_with_model_value: None,
            replace_with_model_processable: false,
            remove_text: false,
        }
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

    fn set_text(&mut self, text: JavaString) {
        self.reset();
        self.set_text = true;
        self.set_text_value = Some(text);
    }

    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.reset();
        self.replace_with_model = true;
        self.replace_with_model_value = Some(model);
        self.replace_with_model_processable = processable;
    }

    fn remove_text(&mut self) {
        self.reset();
        self.remove_text = true;
    }
}
