#![expect(
    dead_code,
    reason = "状态由后续迁移的 ProcessorTemplateHandler 统一消费"
)]

use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};

use crate::context::IEngineContext;
use crate::element::IElementModelStructureHandler;
use crate::expression::TemplateValue;
use crate::inline::IInliner;
use crate::util::JavaString;

use super::TemplateData;

/// ElementModel Processor 使用的上下文变更状态机。
///
/// 所有动作均可组合，并按变量设置、变量删除、选择目标、内联器、模板数据的固定
/// 顺序应用。对应 Java: `org.thymeleaf.engine.ElementModelStructureHandler`。
pub(crate) struct ElementModelStructureHandler {
    pub(crate) set_local_variable: bool,
    pub(crate) added_local_variables: IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>,
    pub(crate) remove_local_variable: bool,
    pub(crate) removed_local_variable_names: IndexSet<JavaString>,
    pub(crate) set_selection_target: bool,
    pub(crate) selection_target_object: Option<Arc<TemplateValue>>,
    pub(crate) set_inliner: bool,
    pub(crate) set_inliner_value: Option<Arc<dyn IInliner>>,
    pub(crate) set_template_data: bool,
    pub(crate) set_template_data_value: Option<Arc<TemplateData>>,
}

impl ElementModelStructureHandler {
    /// 创建已重置的处理器状态。
    pub(crate) fn new() -> Self {
        Self {
            set_local_variable: false,
            added_local_variables: IndexMap::new(),
            remove_local_variable: false,
            removed_local_variable_names: IndexSet::new(),
            set_selection_target: false,
            selection_target_object: None,
            set_inliner: false,
            set_inliner_value: None,
            set_template_data: false,
            set_template_data_value: None,
        }
    }

    /// 将已收集的上下文变更应用到非空引擎上下文。
    pub(crate) fn apply_context_modifications(
        &self,
        engine_context: Option<&mut dyn IEngineContext>,
    ) {
        let Some(engine_context) = engine_context else {
            return;
        };
        if self.set_local_variable {
            engine_context.set_variables(&self.added_local_variables);
        }
        if self.remove_local_variable {
            for variable_name in &self.removed_local_variable_names {
                engine_context.remove_variable(variable_name);
            }
        }
        if self.set_selection_target {
            engine_context.set_selection_target(self.selection_target_object.clone());
        }
        if self.set_inliner {
            engine_context.set_inliner(self.set_inliner_value.clone());
        }
        if self.set_template_data {
            engine_context.set_template_data(
                self.set_template_data_value
                    .clone()
                    .expect("setTemplateData action requires a value"),
            );
        }
    }
}

impl IElementModelStructureHandler for ElementModelStructureHandler {
    fn reset(&mut self) {
        self.set_local_variable = false;
        self.added_local_variables.clear();
        self.remove_local_variable = false;
        self.removed_local_variable_names.clear();
        self.set_selection_target = false;
        self.selection_target_object = None;
        self.set_inliner = false;
        self.set_inliner_value = None;
        self.set_template_data = false;
        self.set_template_data_value = None;
    }

    fn set_local_variable(&mut self, name: JavaString, value: Option<Arc<TemplateValue>>) {
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

    fn set_template_data(&mut self, template_data: Arc<TemplateData>) {
        self.set_template_data = true;
        self.set_template_data_value = Some(template_data);
    }
}
