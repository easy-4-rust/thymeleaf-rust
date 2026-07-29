#![expect(
    dead_code,
    reason = "状态由后续迁移的 ProcessorTemplateHandler 统一消费"
)]

use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};

use crate::context::IEngineContext;
use crate::element::IElementTagStructureHandler;
use crate::expression::TemplateValue;
use crate::inline::IInliner;
use crate::model::{AttributeValueQuotes, IModel, IProcessableElementTag};
use crate::util::{JavaString, Validate, ValidateError};

use super::{
    AttributeDefinitionValue, AttributeDefinitions, AttributeNameValue, AttributesError,
    TemplateData,
};

type SetAttributeValue = (
    Option<AttributeDefinitionValue>,
    JavaString,
    Option<JavaString>,
    Option<AttributeValueQuotes>,
);
type ReplaceAttributeValue = (
    AttributeNameValue,
    Option<AttributeDefinitionValue>,
    JavaString,
    Option<JavaString>,
    Option<AttributeValueQuotes>,
);
type RemoveAttributeValue = (
    u8,
    Option<JavaString>,
    Option<JavaString>,
    Option<AttributeNameValue>,
);

const REMOVE_COMPLETE: u8 = 0;
const REMOVE_PREFIXED: u8 = 1;
const REMOVE_NORMALIZED: u8 = 2;

/// ElementTag Processor 使用的完整结构变更状态机。
///
/// 正文、插入、替换、删除及迭代动作互斥；变量和属性动作会跨互斥动作保留并组合。
/// 属性应用严格遵循 Java 的删除、替换、设置三阶段顺序。
/// 对应 Java: `org.thymeleaf.engine.ElementTagStructureHandler`。
pub(crate) struct ElementTagStructureHandler {
    pub(crate) set_body_text: bool,
    pub(crate) set_body_text_value: Option<JavaString>,
    pub(crate) set_body_text_processable: bool,
    pub(crate) set_body_model: bool,
    pub(crate) set_body_model_value: Option<Arc<dyn IModel>>,
    pub(crate) set_body_model_processable: bool,
    pub(crate) insert_before_model: bool,
    pub(crate) insert_before_model_value: Option<Arc<dyn IModel>>,
    pub(crate) insert_immediately_after_model: bool,
    pub(crate) insert_immediately_after_model_value: Option<Arc<dyn IModel>>,
    pub(crate) insert_immediately_after_model_processable: bool,
    pub(crate) replace_with_text: bool,
    pub(crate) replace_with_text_value: Option<JavaString>,
    pub(crate) replace_with_text_processable: bool,
    pub(crate) replace_with_model: bool,
    pub(crate) replace_with_model_value: Option<Arc<dyn IModel>>,
    pub(crate) replace_with_model_processable: bool,
    pub(crate) remove_element: bool,
    pub(crate) remove_tags: bool,
    pub(crate) remove_body: bool,
    pub(crate) remove_all_but_first_child: bool,

    pub(crate) set_local_variable: bool,
    pub(crate) added_local_variables: IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>,
    pub(crate) remove_local_variable: bool,
    pub(crate) removed_local_variable_names: IndexSet<JavaString>,

    pub(crate) set_attribute: bool,
    set_attribute_values: Vec<SetAttributeValue>,
    pub(crate) replace_attribute: bool,
    replace_attribute_values: Vec<ReplaceAttributeValue>,
    pub(crate) remove_attribute: bool,
    remove_attribute_values: Vec<RemoveAttributeValue>,

    pub(crate) set_selection_target: bool,
    pub(crate) selection_target_object: Option<Arc<TemplateValue>>,
    pub(crate) set_inliner: bool,
    pub(crate) set_inliner_value: Option<Arc<dyn IInliner>>,
    pub(crate) set_template_data: bool,
    pub(crate) set_template_data_value: Option<Arc<TemplateData>>,

    pub(crate) iterate_element: bool,
    pub(crate) iter_variable_name: Option<JavaString>,
    pub(crate) iter_status_variable_name: Option<JavaString>,
    pub(crate) iterated_object: Option<Arc<TemplateValue>>,
}

impl ElementTagStructureHandler {
    /// 创建已重置的结构处理器。
    pub(crate) fn new() -> Self {
        Self {
            set_body_text: false,
            set_body_text_value: None,
            set_body_text_processable: false,
            set_body_model: false,
            set_body_model_value: None,
            set_body_model_processable: false,
            insert_before_model: false,
            insert_before_model_value: None,
            insert_immediately_after_model: false,
            insert_immediately_after_model_value: None,
            insert_immediately_after_model_processable: false,
            replace_with_text: false,
            replace_with_text_value: None,
            replace_with_text_processable: false,
            replace_with_model: false,
            replace_with_model_value: None,
            replace_with_model_processable: false,
            remove_element: false,
            remove_tags: false,
            remove_body: false,
            remove_all_but_first_child: false,
            set_local_variable: false,
            added_local_variables: IndexMap::new(),
            remove_local_variable: false,
            removed_local_variable_names: IndexSet::new(),
            set_attribute: false,
            set_attribute_values: Vec::with_capacity(3),
            replace_attribute: false,
            replace_attribute_values: Vec::with_capacity(3),
            remove_attribute: false,
            remove_attribute_values: Vec::with_capacity(3),
            set_selection_target: false,
            selection_target_object: None,
            set_inliner: false,
            set_inliner_value: None,
            set_template_data: false,
            set_template_data_value: None,
            iterate_element: false,
            iter_variable_name: None,
            iter_status_variable_name: None,
            iterated_object: None,
        }
    }

    fn reset_all_but_variables_or_attributes(&mut self) {
        self.set_body_text = false;
        self.set_body_text_value = None;
        self.set_body_text_processable = false;
        self.set_body_model = false;
        self.set_body_model_value = None;
        self.set_body_model_processable = false;
        self.insert_before_model = false;
        self.insert_before_model_value = None;
        self.insert_immediately_after_model = false;
        self.insert_immediately_after_model_value = None;
        self.insert_immediately_after_model_processable = false;
        self.replace_with_text = false;
        self.replace_with_text_value = None;
        self.replace_with_text_processable = false;
        self.replace_with_model = false;
        self.replace_with_model_value = None;
        self.replace_with_model_processable = false;
        self.remove_element = false;
        self.remove_tags = false;
        self.remove_body = false;
        self.remove_all_but_first_child = false;
        self.iterate_element = false;
        self.iter_variable_name = None;
        self.iter_status_variable_name = None;
        self.iterated_object = None;
    }

    /// 使用已解析的 AttributeDefinition 添加属性动作，供 Standard Dialect 优化。
    pub(crate) fn set_attribute_with_definition(
        &mut self,
        attribute_definition: AttributeDefinitionValue,
        attribute_name: JavaString,
        attribute_value: Option<JavaString>,
        quotes: Option<AttributeValueQuotes>,
    ) {
        self.set_attribute = true;
        self.set_attribute_values.push((
            Some(attribute_definition),
            attribute_name,
            attribute_value,
            quotes,
        ));
    }

    /// 使用已解析的新 AttributeDefinition 添加替换动作。
    pub(crate) fn replace_attribute_with_definition(
        &mut self,
        old_attribute_name: AttributeNameValue,
        attribute_definition: AttributeDefinitionValue,
        attribute_name: JavaString,
        attribute_value: Option<JavaString>,
        quotes: Option<AttributeValueQuotes>,
    ) {
        self.replace_attribute = true;
        self.replace_attribute_values.push((
            old_attribute_name,
            Some(attribute_definition),
            attribute_name,
            attribute_value,
            quotes,
        ));
    }

    /// 将上下文变更按 Java 固定顺序应用。
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

    /// 将收集的属性动作应用到不可变标签。
    pub(crate) fn apply_attributes(
        &self,
        attribute_definitions: &AttributeDefinitions,
        mut tag: Arc<dyn IProcessableElementTag>,
    ) -> Result<Arc<dyn IProcessableElementTag>, AttributesError> {
        if self.remove_attribute {
            for (kind, first, second, normalized) in &self.remove_attribute_values {
                tag = match *kind {
                    REMOVE_COMPLETE => tag.without_attribute_complete(
                        first
                            .as_ref()
                            .expect("complete-name removal requires a name"),
                    )?,
                    REMOVE_PREFIXED => tag.without_attribute_with_prefix(
                        first.as_ref(),
                        second
                            .as_ref()
                            .expect("prefixed removal requires a local name"),
                    )?,
                    REMOVE_NORMALIZED => tag.without_attribute(
                        normalized
                            .as_ref()
                            .expect("normalized removal requires AttributeName")
                            .as_attribute_name(),
                    ),
                    _ => unreachable!("unknown removal representation"),
                };
            }
        }
        if self.replace_attribute {
            for (old_name, definition, name, value, quotes) in &self.replace_attribute_values {
                tag = tag.with_replaced_attribute(
                    attribute_definitions,
                    old_name.as_attribute_name(),
                    definition.as_ref(),
                    name.clone(),
                    value.clone(),
                    *quotes,
                )?;
            }
        }
        if self.set_attribute {
            for (definition, name, value, quotes) in &self.set_attribute_values {
                tag = tag.with_attribute(
                    attribute_definitions,
                    definition.as_ref(),
                    name.clone(),
                    value.clone(),
                    *quotes,
                )?;
            }
        }
        Ok(tag)
    }
}

impl IElementTagStructureHandler for ElementTagStructureHandler {
    fn reset(&mut self) {
        self.reset_all_but_variables_or_attributes();
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
        self.set_attribute = false;
        self.set_attribute_values.clear();
        self.replace_attribute = false;
        self.replace_attribute_values.clear();
        self.remove_attribute = false;
        self.remove_attribute_values.clear();
    }

    fn set_local_variable(&mut self, name: JavaString, value: Option<Arc<TemplateValue>>) {
        self.set_local_variable = true;
        self.added_local_variables.insert(Some(name), value);
    }

    fn remove_local_variable(&mut self, name: JavaString) {
        self.remove_local_variable = true;
        self.removed_local_variable_names.insert(name);
    }

    fn set_attribute(
        &mut self,
        attribute_name: JavaString,
        attribute_value: Option<JavaString>,
        quotes: Option<AttributeValueQuotes>,
    ) {
        self.set_attribute = true;
        self.set_attribute_values
            .push((None, attribute_name, attribute_value, quotes));
    }

    fn replace_attribute(
        &mut self,
        old_attribute_name: AttributeNameValue,
        attribute_name: JavaString,
        attribute_value: Option<JavaString>,
        quotes: Option<AttributeValueQuotes>,
    ) {
        self.replace_attribute = true;
        self.replace_attribute_values.push((
            old_attribute_name,
            None,
            attribute_name,
            attribute_value,
            quotes,
        ));
    }

    fn remove_attribute(&mut self, attribute_name: JavaString) {
        self.remove_attribute = true;
        self.remove_attribute_values
            .push((REMOVE_COMPLETE, Some(attribute_name), None, None));
    }

    fn remove_attribute_with_prefix(&mut self, prefix: Option<JavaString>, name: JavaString) {
        self.remove_attribute = true;
        self.remove_attribute_values
            .push((REMOVE_PREFIXED, prefix, Some(name), None));
    }

    fn remove_attribute_name(&mut self, attribute_name: AttributeNameValue) {
        self.remove_attribute = true;
        self.remove_attribute_values
            .push((REMOVE_NORMALIZED, None, None, Some(attribute_name)));
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

    fn set_body_text(&mut self, text: JavaString, processable: bool) {
        self.reset_all_but_variables_or_attributes();
        self.set_body_text = true;
        self.set_body_text_value = Some(text);
        self.set_body_text_processable = processable;
    }

    fn set_body_model(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.reset_all_but_variables_or_attributes();
        self.set_body_model = true;
        self.set_body_model_value = Some(model);
        self.set_body_model_processable = processable;
    }

    fn insert_before(&mut self, model: Arc<dyn IModel>) {
        self.reset_all_but_variables_or_attributes();
        self.insert_before_model = true;
        self.insert_before_model_value = Some(model);
    }

    fn insert_immediately_after(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.reset_all_but_variables_or_attributes();
        self.insert_immediately_after_model = true;
        self.insert_immediately_after_model_value = Some(model);
        self.insert_immediately_after_model_processable = processable;
    }

    fn replace_with_text(&mut self, text: JavaString, processable: bool) {
        self.reset_all_but_variables_or_attributes();
        self.replace_with_text = true;
        self.replace_with_text_value = Some(text);
        self.replace_with_text_processable = processable;
    }

    fn replace_with_model(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.reset_all_but_variables_or_attributes();
        self.replace_with_model = true;
        self.replace_with_model_value = Some(model);
        self.replace_with_model_processable = processable;
    }

    fn remove_element(&mut self) {
        self.reset_all_but_variables_or_attributes();
        self.remove_element = true;
    }

    fn remove_tags(&mut self) {
        self.reset_all_but_variables_or_attributes();
        self.remove_tags = true;
    }

    fn remove_body(&mut self) {
        self.reset_all_but_variables_or_attributes();
        self.remove_body = true;
    }

    fn remove_all_but_first_child(&mut self) {
        self.reset_all_but_variables_or_attributes();
        self.remove_all_but_first_child = true;
    }

    fn iterate_element(
        &mut self,
        iter_variable_name: JavaString,
        iter_status_variable_name: Option<JavaString>,
        iterated_object: Option<Arc<TemplateValue>>,
    ) -> Result<(), ValidateError> {
        let iter_variable_name_text = iter_variable_name.to_string_lossy();
        Validate::not_empty_str(
            Some(&iter_variable_name_text),
            Some("Iteration variable name cannot be null"),
        )?;
        self.reset_all_but_variables_or_attributes();
        self.iterate_element = true;
        self.iter_variable_name = Some(iter_variable_name);
        self.iter_status_variable_name = iter_status_variable_name;
        self.iterated_object = iterated_object;
        Ok(())
    }
}
