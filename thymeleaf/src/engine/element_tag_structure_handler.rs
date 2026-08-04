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
use crate::util::{JavaCharSequence, Utf16String, Validate, ValidateError};

use super::{
    AttributeDefinitionValue, AttributeDefinitions, AttributeNameValue, AttributesError,
    TemplateData,
};

type SetAttributeValue = (
    Option<AttributeDefinitionValue>,
    Utf16String,
    Option<Utf16String>,
    Option<AttributeValueQuotes>,
);
type ReplaceAttributeValue = (
    AttributeNameValue,
    Option<AttributeDefinitionValue>,
    Utf16String,
    Option<Utf16String>,
    Option<AttributeValueQuotes>,
);
type RemoveAttributeValue = (
    u8,
    Option<Utf16String>,
    Option<Utf16String>,
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
    pub(crate) set_body_text_value: Option<Arc<dyn JavaCharSequence>>,
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
    pub(crate) replace_with_text_value: Option<Utf16String>,
    pub(crate) replace_with_text_processable: bool,
    pub(crate) replace_with_model: bool,
    pub(crate) replace_with_model_value: Option<Arc<dyn IModel>>,
    pub(crate) replace_with_model_processable: bool,
    pub(crate) remove_element: bool,
    pub(crate) remove_tags: bool,
    pub(crate) remove_body: bool,
    pub(crate) remove_all_but_first_child: bool,

    pub(crate) set_local_variable: bool,
    pub(crate) added_local_variables: IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>,
    pub(crate) remove_local_variable: bool,
    pub(crate) removed_local_variable_names: IndexSet<Utf16String>,

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
    pub(crate) iter_variable_name: Option<Utf16String>,
    pub(crate) iter_status_variable_name: Option<Utf16String>,
    pub(crate) iterated_object: Option<Arc<TemplateValue>>,
}

impl ElementTagStructureHandler {
    /// 创建已重置的结构处理器。
    /// 对应 Java 语义：`ElementTagStructureHandler` 的 `new` 行为（Rust 侧辅助/私有路径）。
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
    /// 对应 Java 语义：`ElementTagStructureHandler` 的 `set_attribute_with_definition` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn set_attribute_with_definition(
        &mut self,
        attribute_definition: AttributeDefinitionValue,
        attribute_name: Utf16String,
        attribute_value: Option<Utf16String>,
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
    /// 对应 Java 语义：`ElementTagStructureHandler` 的 `replace_attribute_with_definition` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn replace_attribute_with_definition(
        &mut self,
        old_attribute_name: AttributeNameValue,
        attribute_definition: AttributeDefinitionValue,
        attribute_name: Utf16String,
        attribute_value: Option<Utf16String>,
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
    /// 对应 Java: `ElementTagStructureHandler#applyContextModifications()`。
    pub(crate) fn apply_context_modifications(&self, engine_context: Option<&dyn IEngineContext>) {
        let Some(engine_context) = engine_context else {
            return;
        };
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
        if self.set_template_data {
            engine_context.set_template_data(
                self.set_template_data_value
                    .clone()
                    .expect("setTemplateData action requires a value"),
            );
        }
    }

    /// 将收集的属性动作应用到不可变标签。
    /// 对应 Java: `ElementTagStructureHandler#applyAttributes()`。
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

    fn set_local_variable(&mut self, name: Utf16String, value: Option<Arc<TemplateValue>>) {
        self.set_local_variable = true;
        self.added_local_variables.insert(Some(name), value);
    }

    fn remove_local_variable(&mut self, name: Utf16String) {
        self.remove_local_variable = true;
        self.removed_local_variable_names.insert(name);
    }

    fn set_attribute(
        &mut self,
        attribute_name: Utf16String,
        attribute_value: Option<Utf16String>,
        quotes: Option<AttributeValueQuotes>,
    ) {
        self.set_attribute = true;
        self.set_attribute_values
            .push((None, attribute_name, attribute_value, quotes));
    }

    fn replace_attribute(
        &mut self,
        old_attribute_name: AttributeNameValue,
        attribute_name: Utf16String,
        attribute_value: Option<Utf16String>,
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

    fn remove_attribute(&mut self, attribute_name: Utf16String) {
        self.remove_attribute = true;
        self.remove_attribute_values
            .push((REMOVE_COMPLETE, Some(attribute_name), None, None));
    }

    fn remove_attribute_with_prefix(&mut self, prefix: Option<Utf16String>, name: Utf16String) {
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

    fn set_body_text(&mut self, text: Utf16String, processable: bool) {
        self.reset_all_but_variables_or_attributes();
        self.set_body_text = true;
        self.set_body_text_value = Some(Arc::new(text));
        self.set_body_text_processable = processable;
    }

    fn set_body_sequence(&mut self, text: Arc<dyn JavaCharSequence>, processable: bool) {
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

    fn replace_with_text(&mut self, text: Utf16String, processable: bool) {
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
        iter_variable_name: Utf16String,
        iter_status_variable_name: Option<Utf16String>,
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use indexmap::IndexMap;

    use super::ElementTagStructureHandler;
    use crate::cache::AlwaysValidCacheEntryValidity;
    use crate::context::{EngineContext, IContext, ITemplateContext};
    use crate::element::IElementTagStructureHandler;
    use crate::engine::{
        Attribute, AttributeDefinitionValue, AttributeDefinitions, ElementDefinitionValue,
        ElementDefinitions, ElementProcessorsByTemplateMode, OpenElementTag, TemplateData,
        model::Model,
    };
    use crate::expression::TemplateValue;
    use crate::model::{AttributeValueQuotes, IModel, IProcessableElementTag};
    use crate::templatemode::TemplateMode;
    use crate::templateresource::StringTemplateResource;
    use crate::util::{JavaLocale, Utf16String};
    use crate::{ITemplateEngine, TemplateEngine};

    fn java(value: &str) -> Utf16String {
        Utf16String::from_rust_str(value)
    }

    fn snapshot(handler: &ElementTagStructureHandler) -> String {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            handler.set_body_text,
            handler.set_body_model,
            handler.insert_before_model,
            handler.remove_element,
            handler.iterate_element,
            handler.set_local_variable,
            handler.added_local_variables.len(),
            handler.remove_local_variable,
            handler.removed_local_variable_names.len(),
            handler.set_attribute,
            handler.set_attribute_values.len(),
            handler.remove_attribute,
            handler.remove_attribute_values.len(),
            handler
                .iter_variable_name
                .as_ref()
                .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy),
        )
    }

    /// 建立真实的 HTML 定义仓库，确保测试经过生产属性名解析和不可变标签转换。
    fn definitions() -> (AttributeDefinitions, ElementDefinitions) {
        let processors: ElementProcessorsByTemplateMode = HashMap::new();
        let attribute_definitions =
            AttributeDefinitions::new(processors.clone()).expect("attribute definitions");
        let element_definitions = ElementDefinitions::new(processors).expect("element definitions");
        (attribute_definitions, element_definitions)
    }

    /// 用真实 `OpenElementTag` 建立两个已有属性，避免用 mock 掩盖属性转换语义。
    fn open_tag(
        attribute_definitions: &AttributeDefinitions,
        element_definitions: &ElementDefinitions,
    ) -> Arc<OpenElementTag> {
        let element_definition = ElementDefinitionValue::Html(
            element_definitions
                .for_html_name(Some(&java("element")))
                .expect("element definition"),
        );
        let attributes = ["data-a", "data-b"]
            .into_iter()
            .zip(["one", "two"])
            .map(|(name, value)| {
                let definition = AttributeDefinitionValue::Html(
                    attribute_definitions
                        .for_html_name(Some(&java(name)))
                        .expect("attribute definition"),
                );
                Arc::new(Attribute::new(
                    definition,
                    java(name),
                    None,
                    Some(java(value)),
                    Some(AttributeValueQuotes::DOUBLE),
                    None,
                    -1,
                    -1,
                ))
            })
            .collect();
        Arc::new(OpenElementTag::new(
            TemplateMode::HTML,
            element_definition,
            java("element"),
            Some(crate::engine::Attributes::new(
                Some(attributes),
                // 两个属性各自有一个前置空白；Java 的 `innerWhiteSpaces` 长度必须与
                // 属性数相同（或多一个结尾空白），删除最后一个属性才能保持格式。
                Some(vec![java(" "), java(" ")]),
            )),
            false,
        ))
    }

    fn template_data(name: &str) -> TemplateData {
        TemplateData::new(
            Some(java(name)),
            None,
            Some(Arc::new(
                StringTemplateResource::new(Some(name)).expect("string template resource"),
            )),
            Some(TemplateMode::HTML),
            Some(Arc::new(AlwaysValidCacheEntryValidity::new())),
        )
    }

    #[test]
    fn action_state_matches_java_golden() {
        let mut handler = ElementTagStructureHandler::new();
        let mut actual = vec![("initial", snapshot(&handler))];
        {
            let contract: &mut dyn IElementTagStructureHandler = &mut handler;
            contract.set_local_variable(java("a"), None);
            contract.set_local_variable(java("b"), None);
            contract.remove_local_variable(java("old"));
            contract.set_attribute(java("x"), Some(java("1")), None);
            contract.set_attribute(java("y"), None, Some(AttributeValueQuotes::SINGLE));
            contract.remove_attribute(java("gone"));
            contract.remove_attribute_with_prefix(Some(java("th")), java("each"));
        }
        actual.push(("combined", snapshot(&handler)));
        (&mut handler as &mut dyn IElementTagStructureHandler).remove_element();
        actual.push(("removeElement", snapshot(&handler)));
        (&mut handler as &mut dyn IElementTagStructureHandler)
            .iterate_element(java("item"), None, None)
            .expect("non-empty Java iteration variable");
        actual.push(("iterate", snapshot(&handler)));
        (&mut handler as &mut dyn IElementTagStructureHandler).reset();
        actual.push(("reset", snapshot(&handler)));

        for (key, value) in actual {
            let expected =
                include_str!("../../tests/fixtures/element_tag_structure_handler_golden.txt")
                    .lines()
                    .find_map(|line| line.strip_prefix(&format!("{key}=")))
                    .expect("Java Golden record");
            assert_eq!(value, expected, "Java Golden {key}");
        }
    }

    #[test]
    fn applies_attribute_actions_in_java_remove_replace_set_order() {
        let (attribute_definitions, element_definitions) = definitions();
        let initial = open_tag(&attribute_definitions, &element_definitions);
        let mut handler = ElementTagStructureHandler::new();
        {
            let contract: &mut dyn IElementTagStructureHandler = &mut handler;
            // Java 的三个动作列表分开保存，执行时不能按调用顺序交错：必须先删、再替换、最后设置。
            contract.set_attribute(java("data-c"), Some(java("final")), None);
            contract.remove_attribute(java("data-a"));
            contract.replace_attribute(
                crate::engine::AttributeNameValue::Html(
                    crate::engine::AttributeNames::for_html_name(Some(&java("data-b")))
                        .expect("old attribute name"),
                ),
                java("data-c"),
                Some(java("replacement")),
                Some(AttributeValueQuotes::SINGLE),
            );
            contract.set_attribute(java("data-d"), None, None);
        }

        let result = handler
            .apply_attributes(
                &attribute_definitions,
                initial as Arc<dyn IProcessableElementTag>,
            )
            .expect("production attribute transformation");
        let attributes = result.get_attribute_map();
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes.get(&java("data-c")), Some(&Some(java("final"))));
        assert_eq!(attributes.get(&java("data-d")), Some(&None));
        assert!(attributes.get(&java("data-a")).is_none());
        assert!(attributes.get(&java("data-b")).is_none());
        assert_eq!(
            attributes
                .keys()
                .map(Utf16String::to_string_lossy)
                .collect::<Vec<_>>(),
            vec!["data-c", "data-d"],
            "Java 的替换阶段先于设置阶段，因此最后属性顺序固定"
        );
        let actual = attributes
            .iter()
            .map(|(name, value)| {
                format!(
                    "{}={}",
                    name.to_string_lossy(),
                    value
                        .as_ref()
                        .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let expected =
            include_str!("../../tests/fixtures/element_tag_structure_handler_golden.txt")
                .lines()
                .find_map(|line| line.strip_prefix("attributes="))
                .expect("Java Golden attributes record");
        assert_eq!(actual, expected, "Java Golden attributes");
    }

    #[test]
    fn applies_context_actions_to_a_real_engine_context_in_java_order() {
        let engine = TemplateEngine::new();
        let configuration = engine.get_configuration().expect("engine configuration");
        let old = java("old");
        let new = java("new");
        let mut initial = IndexMap::new();
        initial.insert(
            Some(old.clone()),
            Some(Arc::new(TemplateValue::string(java("root-value")))),
        );
        let context = EngineContext::new(
            configuration,
            template_data("root"),
            None,
            JavaLocale::new(java("en"), java("US")),
            Some(&initial),
        );
        let selection = Arc::new(TemplateValue::string(java("selection")));
        let mut handler = ElementTagStructureHandler::new();
        {
            let contract: &mut dyn IElementTagStructureHandler = &mut handler;
            // 同名 set 后 remove 只有在 Java 固定顺序（setVariables 后 removeVariable）下才会消失。
            contract.set_local_variable(
                old.clone(),
                Some(Arc::new(TemplateValue::string(java("new-old")))),
            );
            contract.set_local_variable(
                new.clone(),
                Some(Arc::new(TemplateValue::string(java("new-value")))),
            );
            contract.remove_local_variable(old.clone());
            contract.set_selection_target(Some(Arc::clone(&selection)));
            contract.set_template_data(Arc::new(template_data("nested")));
        }

        handler.apply_context_modifications(Some(context.as_ref()));
        assert!(context.get_variable(Some(&old)).is_none());
        assert!(matches!(
            context.get_variable(Some(&new)).as_deref(),
            Some(TemplateValue::String(value)) if value.to_string_lossy() == "new-value"
        ));
        assert!(matches!(
            context.get_selection_target().as_deref(),
            Some(TemplateValue::String(value)) if value.to_string_lossy() == "selection"
        ));
        assert_eq!(
            context
                .get_template_data()
                .get_template()
                .expect("nested template")
                .to_string_lossy(),
            "nested"
        );
    }

    #[test]
    fn structural_actions_are_mutually_exclusive_and_keep_model_identity() {
        let engine = TemplateEngine::new();
        let configuration = engine.get_configuration().expect("engine configuration");
        let model: Arc<dyn IModel> = Arc::new(Model::new(configuration, TemplateMode::HTML));
        let mut handler = ElementTagStructureHandler::new();
        {
            let contract: &mut dyn IElementTagStructureHandler = &mut handler;
            // 变量和属性不属于结构互斥组，后续每个结构动作都必须保留它们。
            contract.set_local_variable(java("kept"), None);
            contract.set_attribute(java("data-kept"), Some(java("yes")), None);
            contract.set_body_text(java("text"), true);
        }
        assert!(handler.set_body_text && handler.set_body_text_processable);
        assert!(handler.set_body_model_value.is_none());

        (&mut handler as &mut dyn IElementTagStructureHandler)
            .set_body_model(Arc::clone(&model), false);
        assert!(handler.set_body_model && !handler.set_body_model_processable);
        assert!(Arc::ptr_eq(
            handler.set_body_model_value.as_ref().expect("body model"),
            &model
        ));
        assert!(!handler.set_body_text);

        (&mut handler as &mut dyn IElementTagStructureHandler).insert_before(Arc::clone(&model));
        assert!(handler.insert_before_model);
        assert!(Arc::ptr_eq(
            handler
                .insert_before_model_value
                .as_ref()
                .expect("before model"),
            &model
        ));
        assert!(!handler.set_body_model);

        (&mut handler as &mut dyn IElementTagStructureHandler)
            .insert_immediately_after(Arc::clone(&model), true);
        assert!(
            handler.insert_immediately_after_model
                && handler.insert_immediately_after_model_processable
        );
        assert!(Arc::ptr_eq(
            handler
                .insert_immediately_after_model_value
                .as_ref()
                .expect("after model"),
            &model
        ));
        assert!(!handler.insert_before_model);

        (&mut handler as &mut dyn IElementTagStructureHandler)
            .replace_with_text(java("replacement"), false);
        assert!(handler.replace_with_text && !handler.replace_with_text_processable);
        assert_eq!(
            handler
                .replace_with_text_value
                .as_ref()
                .expect("replacement text")
                .to_string_lossy(),
            "replacement"
        );
        assert!(!handler.insert_immediately_after_model);

        (&mut handler as &mut dyn IElementTagStructureHandler)
            .replace_with_model(Arc::clone(&model), true);
        assert!(handler.replace_with_model && handler.replace_with_model_processable);
        assert!(Arc::ptr_eq(
            handler
                .replace_with_model_value
                .as_ref()
                .expect("replacement model"),
            &model
        ));
        assert!(!handler.replace_with_text);

        for action in [
            IElementTagStructureHandler::remove_tags,
            IElementTagStructureHandler::remove_body,
            IElementTagStructureHandler::remove_all_but_first_child,
            IElementTagStructureHandler::remove_element,
        ] {
            action(&mut handler);
            assert_eq!(
                usize::from(handler.remove_tags)
                    + usize::from(handler.remove_body)
                    + usize::from(handler.remove_all_but_first_child)
                    + usize::from(handler.remove_element),
                1,
                "每次删除动作只能留下它自己"
            );
            assert!(handler.set_local_variable && handler.set_attribute);
        }
    }
}
