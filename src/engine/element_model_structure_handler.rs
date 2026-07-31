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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use indexmap::IndexMap;

    use super::ElementModelStructureHandler;
    use crate::cache::AlwaysValidCacheEntryValidity;
    use crate::context::{EngineContext, IContext, ITemplateContext};
    use crate::element::IElementModelStructureHandler;
    use crate::engine::TemplateData;
    use crate::expression::TemplateValue;
    use crate::templatemode::TemplateMode;
    use crate::templateresource::StringTemplateResource;
    use crate::util::{JavaLocale, JavaString};
    use crate::{ITemplateEngine, TemplateEngine};

    fn java(value: &str) -> JavaString {
        JavaString::from_rust_str(value)
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

    fn snapshot(handler: &ElementModelStructureHandler) -> String {
        format!(
            "{},{},{},{},{},{},{}",
            handler.set_local_variable,
            handler.added_local_variables.len(),
            handler.remove_local_variable,
            handler.removed_local_variable_names.len(),
            handler.set_selection_target,
            handler.set_inliner,
            handler.set_template_data,
        )
    }

    fn golden(key: &str) -> &str {
        include_str!("../../tests/fixtures/element_model_structure_handler_golden.txt")
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))
            .expect("Java Golden record")
    }

    #[test]
    fn state_and_context_effects_match_java_golden() {
        let engine = TemplateEngine::new();
        let configuration = engine.get_configuration().expect("engine configuration");
        let old = java("old");
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
        let mut handler = ElementModelStructureHandler::new();
        assert_eq!(snapshot(&handler), golden("initial"));
        {
            let contract: &mut dyn IElementModelStructureHandler = &mut handler;
            contract.set_local_variable(java("a"), None);
            contract.set_local_variable(
                java("b"),
                Some(Arc::new(TemplateValue::string(java("value")))),
            );
            // 相同名称在真实 Context 中可以观察固定顺序：先批量设置，再删除。
            contract.set_local_variable(
                old.clone(),
                Some(Arc::new(TemplateValue::string(java("new-old")))),
            );
            contract.remove_local_variable(old.clone());
            contract.set_selection_target(Some(Arc::clone(&selection)));
            contract.set_inliner(None);
            contract.set_template_data(Arc::new(template_data("nested")));
        }
        assert_eq!(snapshot(&handler), golden("combined"));
        handler.apply_context_modifications(Some(context.as_ref()));
        assert!(context.get_variable(Some(&old)).is_none());
        assert!(matches!(
            context.get_selection_target().as_deref(),
            Some(TemplateValue::String(value)) if value.to_string_lossy() == "selection"
        ));
        assert_eq!(
            context
                .get_template_data()
                .get_template()
                .expect("nested")
                .to_string_lossy(),
            "nested"
        );

        (&mut handler as &mut dyn IElementModelStructureHandler).reset();
        assert_eq!(snapshot(&handler), golden("reset"));
    }
}
