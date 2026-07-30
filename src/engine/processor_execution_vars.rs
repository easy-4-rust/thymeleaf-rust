use super::SkipBody;
use super::element_processor_iterator::ElementProcessorIterator;
use super::model::Model;

/// 单个元素事件执行 Processor 时累积的结构变更状态。
///
/// 对应 Java: `org.thymeleaf.engine.ProcessorExecutionVars`。
pub(crate) struct ProcessorExecutionVars {
    pub(crate) processor_iterator: ElementProcessorIterator,
    pub(crate) model_before: Option<Model>,
    pub(crate) model_after: Option<Model>,
    pub(crate) model_after_processable: bool,
    pub(crate) discard_event: bool,
    pub(crate) skip_body: SkipBody,
    pub(crate) skip_close_tag: bool,
}

impl ProcessorExecutionVars {
    /// 创建全部标志为默认处理状态的执行变量。
    ///
    /// 对应 Java: `ProcessorExecutionVars#ProcessorExecutionVars()`。
    pub(crate) fn new() -> Self {
        Self {
            processor_iterator: ElementProcessorIterator::new(),
            model_before: None,
            model_after: None,
            model_after_processable: false,
            discard_event: false,
            skip_body: SkipBody::Process,
            skip_close_tag: false,
        }
    }

    /// 克隆迭代器位置、模型事件身份及全部控制标志。
    ///
    /// 对应 Java: `ProcessorExecutionVars#cloneVars()`。
    pub(crate) fn clone_vars(&self) -> Self {
        let mut clone = Self::new();
        clone
            .processor_iterator
            .reset_as_clone_of(&self.processor_iterator);
        clone.model_before = self.model_before.as_ref().map(clone_model);
        clone.model_after = self.model_after.as_ref().map(clone_model);
        clone.model_after_processable = self.model_after_processable;
        clone.discard_event = self.discard_event;
        clone.skip_body = self.skip_body;
        clone.skip_close_tag = self.skip_close_tag;
        clone
    }
}

fn clone_model(model: &Model) -> Model {
    let mut clone = Model::new(
        model.get_configuration_arc(),
        model.get_template_mode_value(),
    );
    clone.reset_as_clone_of(model);
    clone
}
