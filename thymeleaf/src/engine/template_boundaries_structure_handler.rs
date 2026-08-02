use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};

use crate::context::IEngineContext;
use crate::expression::TemplateValue;
use crate::inline::IInliner;
use crate::model::IModel;
use crate::templateboundaries::ITemplateBoundariesStructureHandler;
use crate::util::{JavaString, Validate, ValidateError};

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
    pub(crate) removed_local_variable_names: IndexSet<Option<JavaString>>,

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
        self.apply_context_modifications_with(
            |variables| engine_context.set_variables(variables),
            |variable_name| engine_context.remove_variable(variable_name),
            |selection_target| engine_context.set_selection_target(selection_target),
            |inliner| engine_context.set_inliner(inliner),
        );
    }

    /// 以可观察回调执行 Java 固定的上下文提交顺序。
    /// 对应 Java 语义：`TemplateBoundariesStructureHandler` 的 `apply_context_modifications_with` 行为（Rust 侧辅助/私有路径）。
    pub(super) fn apply_context_modifications_with(
        &self,
        mut set_variables: impl FnMut(&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>),
        mut remove_variable: impl FnMut(Option<&JavaString>),
        mut set_selection_target: impl FnMut(Option<Arc<TemplateValue>>),
        mut set_inliner: impl FnMut(Option<Arc<dyn IInliner>>),
    ) {
        // 对应 Java 的固定提交顺序：批量设置、逐项删除、selection target、inliner。
        if self.set_local_variable {
            set_variables(&self.added_local_variables);
        }

        if self.remove_local_variable {
            for variable_name in &self.removed_local_variable_names {
                remove_variable(variable_name.as_ref());
            }
        }

        if self.set_selection_target {
            set_selection_target(self.selection_target_object.clone());
        }

        if self.set_inliner {
            set_inliner(self.set_inliner_value.clone());
        }
    }

    /// 插入文本，并保留 Java 的空值错误与清理顺序。
    ///
    /// 对应 Java: `TemplateBoundariesStructureHandler#insert(String, boolean)`。
    /// 仅清除互斥插入动作，保留已收集的上下文变更；随后校验文本非空。
    pub(crate) fn insert_text_nullable(
        &mut self,
        text: Option<JavaString>,
        processable: bool,
    ) -> Result<(), ValidateError> {
        self.reset_all_but_local_variables();
        Validate::not_null(text.as_ref(), Some("Text cannot be null"))?;
        self.insert_text = true;
        self.insert_text_value = text;
        self.insert_text_processable = processable;
        Ok(())
    }

    /// 插入模型，并保留 Java 的空值错误与清理顺序。
    ///
    /// 对应 Java: `TemplateBoundariesStructureHandler#insert(IModel, boolean)`。
    pub(crate) fn insert_model_nullable(
        &mut self,
        model: Option<Arc<dyn IModel>>,
        processable: bool,
    ) -> Result<(), ValidateError> {
        self.reset_all_but_local_variables();
        Validate::not_null(model.as_deref(), Some("Model cannot be null"))?;
        self.insert_model = true;
        self.insert_model_value = model;
        self.insert_model_processable = processable;
        Ok(())
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

    fn set_local_variable(&mut self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        // 可与其他动作组合，无需清除已收集状态。
        // Java Map 的 put 语义：同名变量以后一次设置为准。
        self.set_local_variable = true;
        self.added_local_variables.insert(name, value);
    }

    fn remove_local_variable(&mut self, name: Option<JavaString>) {
        // 可与其他动作组合，无需清除已收集状态。
        self.remove_local_variable = true;
        self.removed_local_variable_names.insert(name);
    }

    fn set_selection_target(&mut self, selection_target: Option<Arc<TemplateValue>>) {
        // 可与其他动作组合，无需清除已收集状态。
        self.set_selection_target = true;
        self.selection_target_object = selection_target;
    }

    fn set_inliner(&mut self, inliner: Option<Arc<dyn IInliner>>) {
        // 可与其他动作组合，无需清除已收集状态。
        self.set_inliner = true;
        self.set_inliner_value = inliner;
    }

    fn insert_text(&mut self, text: JavaString, processable: bool) {
        self.insert_text_nullable(Some(text), processable)
            .expect("Rust non-null text boundary must satisfy Java validation");
    }

    fn insert_model(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.insert_model_nullable(Some(model), processable)
            .expect("Rust non-null model boundary must satisfy Java validation");
    }
}
