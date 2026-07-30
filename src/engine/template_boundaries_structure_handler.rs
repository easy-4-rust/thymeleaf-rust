#![expect(
    dead_code,
    reason = "状态由后续迁移的 ProcessorTemplateHandler 统一消费"
)]

use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};

use crate::context::IEngineContext;
use crate::expression::TemplateValue;
use crate::inline::IInliner;
use crate::model::IModel;
use crate::templateboundaries::ITemplateBoundariesStructureHandler;
use crate::util::JavaString;

/// 模板开始与模板结束处理器共用的结构变更状态机。
///
/// 插入文本和插入模型互斥，但局部变量、选择目标与内联器变更可以与插入动作组合。
/// 对应 Java: `org.thymeleaf.engine.TemplateBoundariesStructureHandler`。
pub(crate) struct TemplateBoundariesStructureHandler {
    pub(crate) insert_text: bool,
    pub(crate) insert_text_value: Option<JavaString>,
    pub(crate) insert_text_processable: bool,

    pub(crate) insert_model: bool,
    pub(crate) insert_model_value: Option<Arc<dyn IModel>>,
    pub(crate) insert_model_processable: bool,

    pub(crate) set_local_variable: bool,
    pub(crate) added_local_variables: IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>,

    pub(crate) remove_local_variable: bool,
    pub(crate) removed_local_variable_names: IndexSet<JavaString>,

    pub(crate) set_selection_target: bool,
    pub(crate) selection_target_object: Option<Arc<TemplateValue>>,

    pub(crate) set_inliner: bool,
    pub(crate) set_inliner_value: Option<Arc<dyn IInliner>>,
}

impl TemplateBoundariesStructureHandler {
    /// 创建已重置的结构处理器。对应 Java 构造方法。
    pub(crate) fn new() -> Self {
        let mut handler = Self {
            insert_text: false,
            insert_text_value: None,
            insert_text_processable: false,
            insert_model: false,
            insert_model_value: None,
            insert_model_processable: false,
            set_local_variable: false,
            added_local_variables: IndexMap::new(),
            remove_local_variable: false,
            removed_local_variable_names: IndexSet::new(),
            set_selection_target: false,
            selection_target_object: None,
            set_inliner: false,
            set_inliner_value: None,
        };
        handler.reset();
        handler
    }

    /// 保留可组合的上下文变更，仅清除互斥的插入动作。
    fn reset_all_but_local_variables(&mut self) {
        self.insert_text = false;
        self.insert_text_value = None;
        self.insert_text_processable = false;

        self.insert_model = false;
        self.insert_model_value = None;
        self.insert_model_processable = false;
    }

    /// 将收集到的局部上下文变更应用到引擎上下文。
    ///
    /// 对应 Java:
    /// `TemplateBoundariesStructureHandler#applyContextModifications`。
    pub(crate) fn apply_context_modifications(&self, engine_context: &dyn IEngineContext) {
        if self.set_local_variable {
            engine_context.set_variables(&self.added_local_variables);
        }

        if self.remove_local_variable {
            for variable_name in &self.removed_local_variable_names {
                engine_context.remove_variable(Some(variable_name));
            }
        }

        if self.set_selection_target {
            engine_context.set_selection_target(self.selection_target_object.clone());
        }

        if self.set_inliner {
            engine_context.set_inliner(self.set_inliner_value.clone());
        }
    }
}

impl ITemplateBoundariesStructureHandler for TemplateBoundariesStructureHandler {
    fn reset(&mut self) {
        self.reset_all_but_local_variables();

        self.set_local_variable = false;
        self.added_local_variables.clear();

        self.remove_local_variable = false;
        self.removed_local_variable_names.clear();

        self.set_selection_target = false;
        self.selection_target_object = None;

        self.set_inliner = false;
        self.set_inliner_value = None;
    }

    fn set_local_variable(&mut self, name: JavaString, value: Option<Arc<TemplateValue>>) {
        // Java Map 的 put 语义：同名变量以后一次设置为准。
        self.set_local_variable = true;
        self.added_local_variables.insert(Some(name), value);
    }

    fn remove_local_variable(&mut self, name: JavaString) {
        self.remove_local_variable = true;
        self.removed_local_variable_names.insert(name);
    }

    fn set_selection_target(&mut self, selection_target: Option<Arc<TemplateValue>>) {
        self.set_selection_target = true;
        self.selection_target_object = selection_target;
    }

    fn set_inliner(&mut self, inliner: Option<Arc<dyn IInliner>>) {
        self.set_inliner = true;
        self.set_inliner_value = inliner;
    }

    fn insert_text(&mut self, text: JavaString, processable: bool) {
        self.reset_all_but_local_variables();
        self.insert_text = true;
        self.insert_text_value = Some(text);
        self.insert_text_processable = processable;
    }

    fn insert_model(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.reset_all_but_local_variables();
        self.insert_model = true;
        self.insert_model_value = Some(model);
        self.insert_model_processable = processable;
    }
}
