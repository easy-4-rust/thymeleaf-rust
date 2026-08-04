use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};

use indexmap::{IndexMap, IndexSet};

use crate::engine::TemplateData;
use crate::exceptions::TemplateProcessingException;
use crate::expression::{IExpressionObjects, TemplateValue};
use crate::inline::IInliner;
use crate::model::{IModelFactory, IProcessableElementTag};
use crate::util::{JavaLocale, Utf16String};
use crate::{IEngineConfiguration, TemplateMode, TemplateResolutionAttributes};

use super::{
    AbstractEngineContext, IContext, IContextVariableNames, IEngineContext, IExpressionContext,
    ITemplateContext, IdentifierSequences,
};

/// 非 Web 模板处理使用的分层引擎上下文。
///
/// 每个发生状态修改的处理层拥有独立条目；读取从最新条目向根层查找，降低层级时
/// 整体丢弃该层变量、selection、inliner 与模板数据。对象通过内部锁复现 Java
/// 可变引用语义，并使表达式对象能够安全持有指回当前上下文的弱引用。
///
/// 对应 Java: `org.thymeleaf.context.EngineContext`。
pub struct EngineContext {
    base: AbstractEngineContext,
    state: RwLock<EngineContextState>,
    self_reference: Weak<EngineContext>,
}

struct EngineContextState {
    level: i32,
    entries: Vec<LevelEntry>,
    element_tags: Vec<Option<Arc<dyn IProcessableElementTag>>>,
}

struct LevelEntry {
    level: i32,
    variables: IndexMap<Option<Utf16String>, ScopedVariable>,
    selection_target: Option<SelectionTarget>,
    inliner: Option<InlinerSetting>,
    template_data: Option<Arc<TemplateData>>,
}

enum ScopedVariable {
    Removed,
    Value(Arc<TemplateValue>),
}

struct SelectionTarget {
    value: Option<Arc<TemplateValue>>,
}

enum InlinerSetting {
    Disabled,
    Value(Arc<dyn IInliner>),
}

impl EngineContext {
    /// 创建以给定模板和用户变量为根层的引擎上下文。
    ///
    /// # 参数
    ///
    /// - `configuration`：当前引擎配置。
    /// - `template_data`：根模板数据。
    /// - `template_resolution_attributes`：可空解析属性。
    /// - `locale`：模板处理 Locale。
    /// - `variables`：用户上下文变量快照。
    ///
    /// 对应 Java: `EngineContext#EngineContext`。
    #[must_use]
    pub fn new(
        configuration: Arc<dyn IEngineConfiguration>,
        template_data: TemplateData,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
        locale: JavaLocale,
        variables: Option<&IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>,
    ) -> Arc<Self> {
        Arc::new_cyclic(|weak: &Weak<Self>| {
            let expression_context: Weak<dyn IExpressionContext> = weak.clone();
            Self::new_with_expression_context(
                configuration,
                template_data,
                template_resolution_attributes,
                locale,
                variables,
                expression_context,
                Some(weak.clone()),
            )
        })
    }

    /// 使用外层上下文作为表达式求值目标创建核心状态。
    ///
    /// Web 引擎上下文用该入口确保表达式对象读取的是包含 request/session/application
    /// 能力的最终对象。对应 Java: `WebEngineContext.ExchangeAttributeMap` 的组合职责。
    pub(crate) fn new_with_expression_context(
        configuration: Arc<dyn IEngineConfiguration>,
        template_data: TemplateData,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
        locale: JavaLocale,
        variables: Option<&IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>,
        expression_context: Weak<dyn IExpressionContext>,
        self_reference: Option<Weak<EngineContext>>,
    ) -> Self {
        let base = AbstractEngineContext::new(
            configuration,
            template_resolution_attributes,
            locale,
            expression_context,
        )
        .expect("non-null engine context dependencies");
        let mut root_variables = IndexMap::new();
        if let Some(variables) = variables {
            root_variables.reserve(variables.len());
            for (name, value) in variables {
                root_variables.insert(
                    name.clone(),
                    ScopedVariable::Value(value.clone().unwrap_or_else(null_value)),
                );
            }
        }
        Self {
            base,
            self_reference: self_reference.unwrap_or_default(),
            state: RwLock::new(EngineContextState {
                level: 0,
                entries: vec![LevelEntry {
                    level: 0,
                    variables: root_variables,
                    selection_target: None,
                    inliner: None,
                    template_data: Some(Arc::new(template_data)),
                }],
                element_tags: Vec::with_capacity(20),
            }),
        }
    }

    /// 返回按层级展开的诊断表示。
    ///
    /// 变量名称在每层内排序，删除标记只在确实遮蔽较早变量时显示。对应 Java:
    /// `EngineContext#getStringRepresentationByLevel()`。
    #[must_use]
    pub fn get_string_representation_by_level(&self) -> String {
        let state = read_state(&self.state);
        let mut output = String::from("{");
        for (entry_index, entry) in state.entries.iter().enumerate().rev() {
            let visible = diagnostic_variables(&state.entries, entry_index);
            if entry_index == 0
                || !visible.is_empty()
                || entry.selection_target.is_some()
                || entry.inliner.is_some()
                || entry.template_data.is_some()
            {
                if output.len() > 1 {
                    output.push(',');
                }
                output.push_str(&entry.level.to_string());
                output.push(':');
                if entry_index == 0 || !visible.is_empty() {
                    output.push_str(&format_variable_map(&visible));
                }
                if let Some(selection_target) = &entry.selection_target {
                    output.push('<');
                    output.push_str(&format_optional_value(selection_target.value.as_deref()));
                    output.push('>');
                }
                if let Some(inliner) = &entry.inliner {
                    output.push('[');
                    match inliner {
                        InlinerSetting::Disabled => output.push_str("NOOP"),
                        InlinerSetting::Value(value) => {
                            output.push_str(&value.get_name().to_string_lossy());
                        }
                    }
                    output.push(']');
                }
                if let Some(template_data) = &entry.template_data {
                    output.push('(');
                    output.push_str(
                        &template_data
                            .get_template()
                            .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy),
                    );
                    output.push(')');
                }
            }
        }
        output.push_str("}[");
        output.push_str(&state.level.to_string());
        output.push(']');
        output
    }

    fn ensure_level_initialized(state: &mut EngineContextState) -> &mut LevelEntry {
        if state
            .entries
            .last()
            .is_none_or(|entry| entry.level != state.level)
        {
            state.entries.push(LevelEntry {
                level: state.level,
                variables: IndexMap::new(),
                selection_target: None,
                inliner: None,
                template_data: None,
            });
        }
        state.entries.last_mut().expect("root entry always exists")
    }
}

impl IContext for EngineContext {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_locale(&self) -> JavaLocale {
        self.base.get_locale()
    }

    fn contains_variable(&self, name: Option<&Utf16String>) -> bool {
        let state = read_state(&self.state);
        find_scoped_variable(&state.entries, name)
            .is_some_and(|variable| !matches!(variable, ScopedVariable::Removed))
    }

    fn get_variable_names(&self) -> Arc<dyn IContextVariableNames + '_> {
        Arc::new(EngineContextVariableNames { context: self })
    }

    fn get_variable(&self, name: Option<&Utf16String>) -> Option<Arc<TemplateValue>> {
        let state = read_state(&self.state);
        match find_scoped_variable(&state.entries, name) {
            Some(ScopedVariable::Value(value)) => resolve_lazy(value),
            Some(ScopedVariable::Removed) | None => None,
        }
    }

    fn as_template_context(&self) -> Option<&dyn ITemplateContext> {
        Some(self)
    }

    fn as_engine_context(&self) -> Option<&dyn IEngineContext> {
        Some(self)
    }

    fn get_engine_context_arc(&self) -> Option<Arc<dyn IEngineContext>> {
        self.self_reference
            .upgrade()
            .map(|context| context as Arc<dyn IEngineContext>)
    }
}

impl IExpressionContext for EngineContext {
    fn get_configuration(&self) -> &dyn IEngineConfiguration {
        self.base.get_configuration()
    }

    fn get_configuration_arc(&self) -> Arc<dyn IEngineConfiguration> {
        self.base.get_configuration_arc()
    }

    fn get_expression_objects(&self) -> &dyn IExpressionObjects {
        self.base.get_expression_objects()
    }
}

impl ITemplateContext for EngineContext {
    fn get_template_data(&self) -> Arc<TemplateData> {
        read_state(&self.state)
            .entries
            .iter()
            .rev()
            .find_map(|entry| entry.template_data.clone())
            .expect("root template data is always present")
    }

    fn get_template_mode(&self) -> TemplateMode {
        self.get_template_data()
            .get_template_mode()
            .expect("engine template data requires a template mode")
    }

    fn get_template_stack(&self) -> Vec<Arc<TemplateData>> {
        read_state(&self.state)
            .entries
            .iter()
            .filter_map(|entry| entry.template_data.clone())
            .collect()
    }

    fn get_element_stack(&self) -> Vec<Arc<dyn IProcessableElementTag>> {
        read_state(&self.state)
            .element_tags
            .iter()
            .filter_map(Clone::clone)
            .collect()
    }

    fn get_template_resolution_attributes(&self) -> Option<&TemplateResolutionAttributes> {
        self.base.get_template_resolution_attributes()
    }

    fn get_model_factory(&self) -> &dyn IModelFactory {
        self.base.get_model_factory(self)
    }

    fn has_selection_target(&self) -> bool {
        read_state(&self.state)
            .entries
            .iter()
            .rev()
            .any(|entry| entry.selection_target.is_some())
    }

    fn get_selection_target(&self) -> Option<Arc<TemplateValue>> {
        read_state(&self.state)
            .entries
            .iter()
            .rev()
            // `SelectionTarget` 包装器区分“未设置”与“显式设置为 null”。一旦找到
            // 最近包装器，即使其值为 null 也必须停止向父层回退。
            .find_map(|entry| entry.selection_target.as_ref())
            .and_then(|target| target.value.clone())
    }

    fn get_inliner(&self) -> Option<Arc<dyn IInliner>> {
        read_state(&self.state)
            .entries
            .iter()
            .rev()
            .find_map(|entry| entry.inliner.as_ref())
            .and_then(|inliner| match inliner {
                InlinerSetting::Disabled => None,
                InlinerSetting::Value(value) => Some(Arc::clone(value)),
            })
    }

    fn get_message(
        &self,
        origin: Option<TypeId>,
        key: &Utf16String,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
        use_absent_message_representation: bool,
    ) -> crate::messageresolver::MessageResolutionResult<Option<Utf16String>> {
        self.base.get_message(
            self,
            origin,
            key,
            message_parameters,
            use_absent_message_representation,
        )
    }

    fn build_link(
        &self,
        base: Option<&Utf16String>,
        parameters: Option<&IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>,
    ) -> Result<Utf16String, TemplateProcessingException> {
        self.base.build_link(self, base, parameters)
    }

    fn get_identifier_sequences(&self) -> &IdentifierSequences {
        self.base.get_identifier_sequences()
    }
}

impl IEngineContext for EngineContext {
    fn set_variable(&self, name: Option<Utf16String>, value: Option<Arc<TemplateValue>>) {
        let mut state = write_state(&self.state);
        let level = state.level;
        let entry = Self::ensure_level_initialized(&mut state);
        entry.variables.insert(
            name,
            ScopedVariable::Value(value.unwrap_or_else(null_value)),
        );
        debug_assert_eq!(entry.level, level);
    }

    fn set_variables(&self, variables: &IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>) {
        if variables.is_empty() {
            return;
        }
        let mut state = write_state(&self.state);
        let entry = Self::ensure_level_initialized(&mut state);
        entry.variables.reserve(variables.len());
        for (name, value) in variables {
            entry.variables.insert(
                name.clone(),
                ScopedVariable::Value(value.clone().unwrap_or_else(null_value)),
            );
        }
    }

    fn remove_variable(&self, name: Option<&Utf16String>) {
        if !self.contains_variable(name) {
            return;
        }
        let mut state = write_state(&self.state);
        let level = state.level;
        let entry = Self::ensure_level_initialized(&mut state);
        if level == 0 {
            entry.variables.shift_remove(&name.cloned());
        } else {
            entry
                .variables
                .insert(name.cloned(), ScopedVariable::Removed);
        }
    }

    fn set_selection_target(&self, selection_target: Option<Arc<TemplateValue>>) {
        let mut state = write_state(&self.state);
        Self::ensure_level_initialized(&mut state).selection_target = Some(SelectionTarget {
            value: selection_target,
        });
    }

    fn set_inliner(&self, inliner: Option<Arc<dyn IInliner>>) {
        let mut state = write_state(&self.state);
        Self::ensure_level_initialized(&mut state).inliner =
            Some(inliner.map_or(InlinerSetting::Disabled, InlinerSetting::Value));
    }

    fn set_template_data(&self, template_data: Arc<TemplateData>) {
        let mut state = write_state(&self.state);
        Self::ensure_level_initialized(&mut state).template_data = Some(template_data);
    }

    fn set_element_tag(&self, element_tag: Option<Arc<dyn IProcessableElementTag>>) {
        let mut state = write_state(&self.state);
        let index = usize::try_from(state.level).expect("context level is non-negative");
        if state.element_tags.len() <= index {
            state.element_tags.resize_with(index + 1, || None);
        }
        state.element_tags[index] = element_tag;
    }

    fn get_element_stack_above(&self, context_level: i32) -> Vec<Arc<dyn IProcessableElementTag>> {
        read_state(&self.state)
            .element_tags
            .iter()
            .enumerate()
            .filter(|(level, _)| i32::try_from(*level).is_ok_and(|level| level > context_level))
            .filter_map(|(_, tag)| tag.clone())
            .collect()
    }

    fn is_variable_local(&self, name: Option<&Utf16String>) -> bool {
        let state = read_state(&self.state);
        for entry in state.entries.iter().rev().filter(|entry| entry.level > 0) {
            if let Some(value) = entry.variables.get(&name.cloned()) {
                return matches!(value, ScopedVariable::Value(_));
            }
        }
        false
    }

    fn increase_level(&self) {
        let mut state = write_state(&self.state);
        state.level = state.level.wrapping_add(1);
    }

    fn decrease_level(&self) {
        let mut state = write_state(&self.state);
        assert!(
            state.level > 0,
            "Cannot decrease variable map level below 0"
        );
        let current_level = state.level;
        if state
            .entries
            .last()
            .is_some_and(|entry| entry.level == current_level)
        {
            state.entries.pop();
        }
        if let Ok(index) = usize::try_from(current_level)
            && index < state.element_tags.len()
        {
            state.element_tags[index] = None;
        }
        state.level -= 1;
    }

    fn level(&self) -> i32 {
        read_state(&self.state).level
    }
}

impl Display for EngineContext {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let state = read_state(&self.state);
        let mut variables: IndexMap<Option<Utf16String>, Arc<TemplateValue>> = IndexMap::new();
        for entry in &state.entries {
            for (name, value) in &entry.variables {
                match value {
                    ScopedVariable::Removed => {
                        variables.shift_remove(name);
                    }
                    ScopedVariable::Value(value) => {
                        variables.insert(name.clone(), Arc::clone(value));
                    }
                }
            }
        }
        formatter.write_str(&format_variable_map(&variables))?;
        if let Some(target) = state
            .entries
            .iter()
            .rev()
            .find_map(|entry| entry.selection_target.as_ref())
        {
            write!(
                formatter,
                "<{}>",
                format_optional_value(target.value.as_deref())
            )?;
        }
        if let Some(inliner) = state
            .entries
            .iter()
            .rev()
            .find_map(|entry| entry.inliner.as_ref())
            && let InlinerSetting::Value(inliner) = inliner
        {
            write!(formatter, "[{}]", inliner.get_name().to_string_lossy())?;
        }
        let template_data = state
            .entries
            .iter()
            .rev()
            .find_map(|entry| entry.template_data.as_ref())
            .expect("root template data is always present");
        write!(
            formatter,
            "({})",
            template_data
                .get_template()
                .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy)
        )
    }
}

struct EngineContextVariableNames<'a> {
    context: &'a EngineContext,
}

impl IContextVariableNames for EngineContextVariableNames<'_> {
    fn len(&self) -> usize {
        visible_variable_names(&read_state(&self.context.state)).len()
    }

    fn contains(&self, name: Option<&Utf16String>) -> bool {
        self.context.contains_variable(name)
    }

    fn snapshot(&self) -> Vec<Option<Utf16String>> {
        visible_variable_names(&read_state(&self.context.state))
            .into_iter()
            .collect()
    }

    fn remove(&self, name: Option<&Utf16String>) -> bool {
        let existed = self.context.contains_variable(name);
        self.context.remove_variable(name);
        existed
    }
}

fn find_scoped_variable<'a>(
    entries: &'a [LevelEntry],
    name: Option<&Utf16String>,
) -> Option<&'a ScopedVariable> {
    let key = name.cloned();
    entries
        .iter()
        .rev()
        .find_map(|entry| entry.variables.get(&key))
}

fn visible_variable_names(state: &EngineContextState) -> IndexSet<Option<Utf16String>> {
    let mut names = IndexSet::new();
    for entry in &state.entries {
        for (name, value) in &entry.variables {
            match value {
                ScopedVariable::Removed => {
                    names.shift_remove(name);
                }
                ScopedVariable::Value(_) => {
                    names.insert(name.clone());
                }
            }
        }
    }
    names
}

fn resolve_lazy(value: &Arc<TemplateValue>) -> Option<Arc<TemplateValue>> {
    if let TemplateValue::Object(object) = value.as_ref()
        && let Some(resolved) = object.resolve_lazy_context_variable()
    {
        return resolved;
    }
    Some(Arc::clone(value))
}

fn null_value() -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Null)
}

fn read_state(lock: &RwLock<EngineContextState>) -> RwLockReadGuard<'_, EngineContextState> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_state(lock: &RwLock<EngineContextState>) -> RwLockWriteGuard<'_, EngineContextState> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn diagnostic_variables(
    entries: &[LevelEntry],
    entry_index: usize,
) -> IndexMap<Option<Utf16String>, Arc<TemplateValue>> {
    let mut result = IndexMap::new();
    let entry = &entries[entry_index];
    let mut names: Vec<_> = entry.variables.keys().cloned().collect();
    names.sort_by(|left, right| {
        left.as_ref()
            .map(Utf16String::as_utf16)
            .cmp(&right.as_ref().map(Utf16String::as_utf16))
    });
    for name in names {
        match entry.variables.get(&name) {
            Some(ScopedVariable::Value(value)) => {
                result.insert(name, Arc::clone(value));
            }
            Some(ScopedVariable::Removed) => {
                let removes_existing = entries[..entry_index].iter().rev().any(|earlier| {
                    matches!(earlier.variables.get(&name), Some(ScopedVariable::Value(_)))
                });
                if removes_existing {
                    result.insert(
                        name,
                        Arc::new(TemplateValue::string(Utf16String::from_rust_str(
                            "(*removed*)",
                        ))),
                    );
                }
            }
            None => {}
        }
    }
    result
}

fn format_variable_map(variables: &IndexMap<Option<Utf16String>, Arc<TemplateValue>>) -> String {
    let mut output = String::from("{");
    for (index, (name, value)) in variables.iter().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        output.push_str(
            &name
                .as_ref()
                .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy),
        );
        output.push('=');
        output.push_str(&format_optional_value(Some(value)));
    }
    output.push('}');
    output
}

fn format_optional_value(value: Option<&TemplateValue>) -> String {
    value
        .and_then(TemplateValue::to_utf16_string)
        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy())
}
