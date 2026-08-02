use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};

use indexmap::IndexMap;

use crate::engine::TemplateData;
use crate::exceptions::TemplateProcessingException;
use crate::expression::{
    IExpressionObjects, TemplateObject, TemplateObjectMethodError, TemplateObjectPropertyError,
    TemplateValue,
};
use crate::inline::IInliner;
use crate::model::{IModelFactory, IProcessableElementTag};
use crate::util::{JavaLocale, JavaNumber, JavaString};
use crate::web::IWebExchange;
use crate::{IEngineConfiguration, TemplateMode, TemplateResolutionAttributes};

use super::{
    EngineContext, IContext, IContextVariableNames, IEngineContext, IExpressionContext,
    ITemplateContext, IWebContext, IdentifierSequences,
};

const PARAM_VARIABLE_NAME: &str = "param";
const SESSION_VARIABLE_NAME: &str = "session";
const APPLICATION_VARIABLE_NAME: &str = "application";

/// 把模板执行直接绑定到 Web exchange 属性作用域的引擎上下文。
///
/// 根变量与 exchange 属性保持同一事实来源；局部层修改立即反映到 exchange，并在
/// 降层时仅当属性仍保持本层写入的对象身份才回滚，从而不覆盖宿主在处理期间直接
/// 写入的新值。`param`、`session`、`application` 由只读动态 Map 对象提供。
///
/// 对应 Java: `org.thymeleaf.context.WebEngineContext`。
pub struct WebEngineContext {
    core: EngineContext,
    web_exchange: Arc<dyn IWebExchange>,
    local_changes: RwLock<Vec<WebLevelChanges>>,
    request_parameter_map: Arc<RequestParameterMap>,
    session_attribute_map: Arc<SessionAttributeMap>,
    application_attribute_map: Arc<ApplicationAttributeMap>,
    self_reference: Weak<WebEngineContext>,
}

/// Java 内部 exchange 属性映射在 Rust 中与 Web 上下文本体合并。
///
/// 对应 Java: `WebEngineContext.ExchangeAttributeMap`。别名保留对象级名称，同时由
/// [`WebEngineContext`] 直接承担变量分层、exchange 写入和身份安全回滚语义。
type ExchangeAttributeMap = WebEngineContext;

struct WebLevelChanges {
    level: i32,
    changes: IndexMap<Option<JavaString>, WebVariableChange>,
}

struct WebVariableChange {
    old_value: Option<Arc<TemplateValue>>,
    new_value: Option<Arc<TemplateValue>>,
}

/// 区分“未设置 selection target”和“显式设置为空”的包装值。
///
/// 对应 Java: `WebEngineContext.ExchangeAttributeMap.SelectionTarget`。
struct SelectionTarget {
    selection_target: Option<Arc<TemplateValue>>,
}

/// Java 只读空操作 Map 基类的 Rust 合同。
///
/// 对应 Java: `WebEngineContext.NoOpMapImpl`。Rust 适配对象不暴露任何写方法，
/// 因而在类型层面实现 Java 对修改操作抛出异常的不可变约束。
#[expect(
    dead_code,
    reason = "保留 Java WebEngineContext.NoOpMapImpl 的对象级合同"
)]
trait NoOpMapImpl: TemplateObject {}

impl WebEngineContext {
    /// 创建 Web 引擎上下文并把初始变量写入 exchange。
    ///
    /// # 参数
    ///
    /// - `configuration`：当前引擎配置。
    /// - `template_data`：根模板数据。
    /// - `template_resolution_attributes`：可空解析属性。
    /// - `web_exchange`：不可空 Web exchange。
    /// - `locale`：模板处理 Locale。
    /// - `variables`：待写入 exchange 的用户变量。
    ///
    /// 对应 Java: `WebEngineContext#WebEngineContext`。
    #[must_use]
    pub fn new(
        configuration: Arc<dyn IEngineConfiguration>,
        template_data: TemplateData,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
        web_exchange: Arc<dyn IWebExchange>,
        locale: JavaLocale,
        variables: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    ) -> Arc<ExchangeAttributeMap> {
        let core_variables = variables.map(|variables| {
            variables
                .iter()
                .filter_map(|(name, value)| {
                    normalize_web_value(value.clone()).map(|value| (name.clone(), Some(value)))
                })
                .collect::<IndexMap<_, _>>()
        });
        if let Some(variables) = variables {
            for (name, value) in variables {
                web_exchange.set_attribute_value(name.clone(), normalize_web_value(value.clone()));
            }
        }
        Arc::new_cyclic(|weak: &Weak<Self>| {
            let expression_context: Weak<dyn IExpressionContext> = weak.clone();
            Self {
                core: EngineContext::new_with_expression_context(
                    configuration,
                    template_data,
                    template_resolution_attributes,
                    locale,
                    // 内部 core 同步保留每层变量记录，仅用于严格复现 Java 的
                    // `getStringRepresentationByLevel()`；变量读取仍以 exchange 为准。
                    core_variables.as_ref(),
                    expression_context,
                    None,
                ),
                request_parameter_map: Arc::new(RequestParameterMap::new(Arc::clone(
                    &web_exchange,
                ))),
                session_attribute_map: Arc::new(SessionAttributeMap::new(Arc::clone(
                    &web_exchange,
                ))),
                application_attribute_map: Arc::new(ApplicationAttributeMap::new(Arc::clone(
                    &web_exchange,
                ))),
                web_exchange,
                local_changes: RwLock::new(Vec::new()),
                self_reference: weak.clone(),
            }
        })
    }

    /// 返回按层级展开的 Web 属性诊断表示。
    ///
    /// 对应 Java: `WebEngineContext#getStringRepresentationByLevel()`。
    #[must_use]
    pub fn get_string_representation_by_level(&self) -> String {
        // Java 委托 `ExchangeAttributeMap#getStringRepresentationByLevel`：逐层
        // 重推导，丢弃被 request 直写覆盖的条目（newValue 与当前 exchange 值不再
        // 同一），基座从活 exchange + 恢复值构建。
        let mut representation = self.core.get_string_representation_by_level();
        let mut old_values_sum: IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>> =
            IndexMap::new();
        // Java 从最深层向基座循环；core 骨架保留各层 selection/inliner/template。
        for level in read_changes(&self.local_changes).iter().rev() {
            let rendered = format_web_level_changes_exchange_aware(
                &level.changes,
                &mut old_values_sum,
                self.web_exchange.as_ref(),
            );
            replace_level_variable_map(&mut representation, level.level, &rendered);
        }
        let base = format_web_base_exchange_aware(self.web_exchange.as_ref(), &mut old_values_sum);
        replace_level_variable_map(&mut representation, 0, &base);
        representation
    }

    fn set_web_variable(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        let value = normalize_web_value(value);
        let level = self.core.level();
        if level > 0 {
            let mut levels = write_changes(&self.local_changes);
            if levels.last().is_none_or(|entry| entry.level != level) {
                levels.push(WebLevelChanges {
                    level,
                    changes: IndexMap::new(),
                });
            }
            let entry = levels.last_mut().expect("local level was inserted");
            if let Some(change) = entry.changes.get_mut(&name) {
                change.new_value = value.clone();
            } else {
                entry.changes.insert(
                    name.clone(),
                    WebVariableChange {
                        old_value: self.web_exchange.get_attribute_value(name.as_ref()),
                        new_value: value.clone(),
                    },
                );
            }
        }
        // Exchange 是运行时事实来源，而 core 保留与 Java ExchangeAttributeMap 同步的
        // 诊断层级记录。两者必须一起更新，才能在嵌套层展示 old/new 变量状态。
        if let Some(value) = value.as_ref() {
            self.core
                .set_variable(name.clone(), Some(Arc::clone(value)));
        } else {
            self.core.remove_variable(name.as_ref());
        }
        self.web_exchange.set_attribute_value(name, value);
    }

    fn assert_not_reserved(name: Option<&JavaString>, operation: &str) {
        if is_reserved(name) {
            let name = name.map_or_else(|| "null".to_owned(), JavaString::to_string_lossy);
            panic!(
                "Cannot {operation} variable called '{name}' {} web variables map: such name is a \
                 reserved word",
                if operation == "set" { "into" } else { "in" }
            );
        }
    }
}

impl IContext for WebEngineContext {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_locale(&self) -> JavaLocale {
        self.core.get_locale()
    }

    fn contains_variable(&self, name: Option<&JavaString>) -> bool {
        if is_named(name, SESSION_VARIABLE_NAME) {
            return true;
        }
        if is_named(name, PARAM_VARIABLE_NAME) || is_named(name, APPLICATION_VARIABLE_NAME) {
            return true;
        }
        self.web_exchange.contains_attribute(name)
    }

    fn get_variable_names(&self) -> Arc<dyn IContextVariableNames + '_> {
        Arc::new(WebEngineVariableNames { context: self })
    }

    fn get_variable(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>> {
        if is_named(name, SESSION_VARIABLE_NAME) {
            return Some(Arc::new(TemplateValue::Object(
                self.session_attribute_map.clone(),
            )));
        }
        if is_named(name, PARAM_VARIABLE_NAME) {
            return Some(Arc::new(TemplateValue::Object(
                self.request_parameter_map.clone(),
            )));
        }
        if is_named(name, APPLICATION_VARIABLE_NAME) {
            return Some(Arc::new(TemplateValue::Object(
                self.application_attribute_map.clone(),
            )));
        }
        self.web_exchange
            .get_attribute_value(name)
            .and_then(resolve_lazy)
    }

    fn get_web_exchange(&self) -> Option<&dyn IWebExchange> {
        Some(self.web_exchange.as_ref())
    }

    fn get_web_exchange_arc(&self) -> Option<Arc<dyn IWebExchange>> {
        Some(Arc::clone(&self.web_exchange))
    }

    fn as_web_context(&self) -> Option<&dyn IWebContext> {
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

    fn as_template_context(&self) -> Option<&dyn ITemplateContext> {
        Some(self)
    }
}

impl IWebContext for WebEngineContext {
    fn get_exchange(&self) -> &dyn IWebExchange {
        self.web_exchange.as_ref()
    }

    fn get_exchange_arc(&self) -> Arc<dyn IWebExchange> {
        Arc::clone(&self.web_exchange)
    }
}

impl IExpressionContext for WebEngineContext {
    fn get_configuration(&self) -> &dyn IEngineConfiguration {
        self.core.get_configuration()
    }

    fn get_configuration_arc(&self) -> Arc<dyn IEngineConfiguration> {
        self.core.get_configuration_arc()
    }

    fn get_expression_objects(&self) -> &dyn IExpressionObjects {
        self.core.get_expression_objects()
    }
}

impl ITemplateContext for WebEngineContext {
    fn get_template_data(&self) -> Arc<TemplateData> {
        self.core.get_template_data()
    }

    fn get_template_mode(&self) -> TemplateMode {
        self.core.get_template_mode()
    }

    fn get_template_stack(&self) -> Vec<Arc<TemplateData>> {
        self.core.get_template_stack()
    }

    fn get_element_stack(&self) -> Vec<Arc<dyn IProcessableElementTag>> {
        self.core.get_element_stack()
    }

    fn get_template_resolution_attributes(&self) -> Option<&TemplateResolutionAttributes> {
        self.core.get_template_resolution_attributes()
    }

    fn get_model_factory(&self) -> &dyn IModelFactory {
        self.core.get_model_factory()
    }

    fn has_selection_target(&self) -> bool {
        self.core.has_selection_target()
    }

    fn get_selection_target(&self) -> Option<Arc<TemplateValue>> {
        self.core.get_selection_target()
    }

    fn get_inliner(&self) -> Option<Arc<dyn IInliner>> {
        self.core.get_inliner()
    }

    fn get_message(
        &self,
        origin: Option<TypeId>,
        key: &JavaString,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
        use_absent_message_representation: bool,
    ) -> crate::messageresolver::MessageResolutionResult<Option<JavaString>> {
        self.core.get_message(
            origin,
            key,
            message_parameters,
            use_absent_message_representation,
        )
    }

    fn build_link(
        &self,
        base: Option<&JavaString>,
        parameters: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    ) -> Result<JavaString, TemplateProcessingException> {
        for link_builder in self.get_configuration().get_link_builders() {
            if let Some(link) = link_builder.build_link(self, base, parameters)? {
                return Ok(link);
            }
        }
        let base = base.map_or_else(|| "null".to_owned(), JavaString::to_string_lossy);
        Err(TemplateProcessingException::new(Some(format!(
            "No configured link builder instance was able to build link with base \"{base}\" and \
             parameters {}",
            format_link_parameters(parameters)
        ))))
    }

    fn get_identifier_sequences(&self) -> &IdentifierSequences {
        self.core.get_identifier_sequences()
    }
}

impl IEngineContext for WebEngineContext {
    fn set_variable(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        Self::assert_not_reserved(name.as_ref(), "set");
        self.set_web_variable(name, value);
    }

    fn set_variables(&self, variables: &IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>) {
        for name in variables.keys() {
            Self::assert_not_reserved(name.as_ref(), "set");
        }
        for (name, value) in variables {
            self.set_web_variable(name.clone(), value.clone());
        }
    }

    fn remove_variable(&self, name: Option<&JavaString>) {
        Self::assert_not_reserved(name, "remove");
        self.set_web_variable(name.cloned(), None);
    }

    fn set_selection_target(&self, selection_target: Option<Arc<TemplateValue>>) {
        let selection_target = SelectionTarget { selection_target };
        self.core
            .set_selection_target(selection_target.selection_target);
    }

    fn set_inliner(&self, inliner: Option<Arc<dyn IInliner>>) {
        self.core.set_inliner(inliner);
    }

    fn set_template_data(&self, template_data: Arc<TemplateData>) {
        self.core.set_template_data(template_data);
    }

    fn set_element_tag(&self, element_tag: Option<Arc<dyn IProcessableElementTag>>) {
        self.core.set_element_tag(element_tag);
    }

    fn get_element_stack_above(&self, context_level: i32) -> Vec<Arc<dyn IProcessableElementTag>> {
        self.core.get_element_stack_above(context_level)
    }

    fn is_variable_local(&self, name: Option<&JavaString>) -> bool {
        read_changes(&self.local_changes)
            .iter()
            .rev()
            .find_map(|level| level.changes.get(&name.cloned()))
            .is_some_and(|change| change.new_value.is_some())
    }

    fn increase_level(&self) {
        self.core.increase_level();
    }

    fn decrease_level(&self) {
        let current_level = self.core.level();
        assert!(
            current_level > 0,
            "Cannot decrease variable map level below 0"
        );
        let changes = {
            let mut levels = write_changes(&self.local_changes);
            if levels
                .last()
                .is_some_and(|entry| entry.level == current_level)
            {
                levels.pop()
            } else {
                None
            }
        };
        if let Some(changes) = changes {
            for (name, change) in changes.changes.into_iter().rev() {
                let current = self.web_exchange.get_attribute_value(name.as_ref());
                if same_identity(current.as_ref(), change.new_value.as_ref()) {
                    self.web_exchange
                        .set_attribute_value(name, change.old_value);
                }
            }
        }
        self.core.decrease_level();
    }

    fn level(&self) -> i32 {
        self.core.level()
    }
}

impl Display for WebEngineContext {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&format_attribute_map(
            &self.web_exchange.get_attribute_map(),
        ))?;
        if self.has_selection_target() {
            formatter.write_str("<")?;
            formatter.write_str(
                &self
                    .get_selection_target()
                    .as_deref()
                    .and_then(TemplateValue::to_java_string)
                    .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy()),
            )?;
            formatter.write_str(">")?;
        }
        if let Some(inliner) = self.get_inliner() {
            write!(formatter, "[{}]", inliner.get_name().to_string_lossy())?;
        }
        write!(
            formatter,
            "({})",
            self.get_template_data()
                .get_template()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy)
        )
    }
}

struct WebEngineVariableNames<'a> {
    context: &'a WebEngineContext,
}

impl IContextVariableNames for WebEngineVariableNames<'_> {
    fn len(&self) -> usize {
        self.context.web_exchange.get_all_attribute_names().len()
    }

    fn contains(&self, name: Option<&JavaString>) -> bool {
        self.context.web_exchange.contains_attribute(name)
    }

    fn snapshot(&self) -> Vec<Option<JavaString>> {
        self.context.web_exchange.get_all_attribute_names()
    }

    fn remove(&self, name: Option<&JavaString>) -> bool {
        let existed = self.context.web_exchange.contains_attribute(name);
        if existed {
            self.context.remove_variable(name);
        }
        existed
    }
}

/// 请求参数的只读动态 Map。
///
/// 对应 Java: `WebEngineContext.RequestParameterMap`。
struct RequestParameterMap {
    web_exchange: Arc<dyn IWebExchange>,
}

impl RequestParameterMap {
    fn new(web_exchange: Arc<dyn IWebExchange>) -> Self {
        Self { web_exchange }
    }
}

impl TemplateObject for RequestParameterMap {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.context.WebEngineContext$RequestParameterMap"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str(&format_parameter_map(
            &self.web_exchange.get_request().get_parameter_map(),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        let values = self
            .web_exchange
            .get_request()
            .get_parameter_values(Some(property_name));
        Some(Ok(values.map(|values| {
            Arc::new(TemplateValue::Object(Arc::new(
                RequestParameterValues::new(values),
            )))
        })))
    }

    fn java_invoke_method(
        &self,
        method_name: &JavaString,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        if method_name.to_string_lossy() == "size" && arguments.is_empty() {
            return Some(Ok(Some(integer_value(
                self.web_exchange.get_request().get_parameter_count(),
            ))));
        }
        None
    }
}

impl NoOpMapImpl for RequestParameterMap {}

/// 会话属性的只读动态 Map。
///
/// 对应 Java: `WebEngineContext.SessionAttributeMap`。
struct SessionAttributeMap {
    web_exchange: Arc<dyn IWebExchange>,
}

impl SessionAttributeMap {
    fn new(web_exchange: Arc<dyn IWebExchange>) -> Self {
        Self { web_exchange }
    }
}

impl TemplateObject for SessionAttributeMap {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.context.WebEngineContext$SessionAttributeMap"
    }

    fn to_java_string(&self) -> JavaString {
        let attributes = self
            .web_exchange
            .get_session()
            .map(|session| session.get_attribute_map())
            .unwrap_or_default();
        JavaString::from_rust_str(&format_attribute_map(&attributes))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        Some(Ok(self
            .web_exchange
            .get_session()
            .and_then(|session| session.get_attribute_value(Some(property_name)))
            .and_then(resolve_lazy)))
    }

    fn java_invoke_method(
        &self,
        method_name: &JavaString,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        if method_name.to_string_lossy() == "size" && arguments.is_empty() {
            let size = self
                .web_exchange
                .get_session()
                .map_or(0, |session| session.get_attribute_count());
            return Some(Ok(Some(integer_value(size))));
        }
        None
    }
}

impl NoOpMapImpl for SessionAttributeMap {}

/// 应用属性的只读动态 Map。
///
/// 对应 Java: `WebEngineContext.ApplicationAttributeMap`。
struct ApplicationAttributeMap {
    web_exchange: Arc<dyn IWebExchange>,
}

impl ApplicationAttributeMap {
    fn new(web_exchange: Arc<dyn IWebExchange>) -> Self {
        Self { web_exchange }
    }
}

impl TemplateObject for ApplicationAttributeMap {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.context.WebEngineContext$ApplicationAttributeMap"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str(&format_attribute_map(
            &self.web_exchange.get_application().get_attribute_map(),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        Some(Ok(self
            .web_exchange
            .get_application()
            .get_attribute_value(Some(property_name))
            .and_then(resolve_lazy)))
    }

    fn java_invoke_method(
        &self,
        method_name: &JavaString,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        if method_name.to_string_lossy() == "size" && arguments.is_empty() {
            return Some(Ok(Some(integer_value(
                self.web_exchange.get_application().get_attribute_count(),
            ))));
        }
        None
    }
}

impl NoOpMapImpl for ApplicationAttributeMap {}

fn integer_value(value: i32) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Number(JavaNumber::Integer(value)))
}

/// 请求参数值的只读 List 视图。
///
/// 单值参数直接格式化为唯一元素，多值参数使用 Java List 风格。对应 Java:
/// `WebEngineContext.RequestParameterValues`。
pub struct RequestParameterValues {
    parameter_values: Vec<Option<JavaString>>,
    /// 与 Java 公共 `length` 字段一致的元素数量。
    pub length: i32,
}

impl RequestParameterValues {
    /// 从请求返回的参数数组创建只读视图。
    ///
    /// 对应 Java: `RequestParameterValues#RequestParameterValues(String[])`。
    #[must_use]
    pub fn new(parameter_values: Vec<Option<JavaString>>) -> Self {
        let length = i32::try_from(parameter_values.len()).unwrap_or(i32::MAX);
        Self {
            parameter_values,
            length,
        }
    }

    /// 返回参数数量。
    ///
    /// 对应 Java: `RequestParameterValues#size()`。
    #[must_use]
    pub fn size(&self) -> i32 {
        self.length
    }

    /// 返回指定下标的可空参数。
    ///
    /// 对应 Java: `RequestParameterValues#get(int)`。
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&JavaString> {
        self.parameter_values.get(index).and_then(Option::as_ref)
    }

    /// 返回目标值第一次出现的下标。
    ///
    /// 对应 Java: `RequestParameterValues#indexOf(Object)`。
    #[must_use]
    pub fn index_of(&self, value: Option<&JavaString>) -> i32 {
        self.parameter_values
            .iter()
            .position(|candidate| candidate.as_ref() == value)
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1)
    }

    /// 判断参数数组是否包含目标值。
    ///
    /// 对应 Java: `RequestParameterValues#contains(Object)`。
    #[must_use]
    pub fn contains(&self, value: Option<&JavaString>) -> bool {
        self.index_of(value) >= 0
    }
}

impl TemplateObject for RequestParameterValues {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.context.WebEngineContext$RequestParameterValues"
    }

    fn to_java_string(&self) -> JavaString {
        if self.parameter_values.is_empty() {
            return JavaString::from_rust_str("");
        }
        if self.parameter_values.len() == 1 {
            return self.parameter_values[0]
                .clone()
                .unwrap_or_else(|| JavaString::from_rust_str("null"));
        }
        JavaString::from_rust_str(&format_parameter_values(&self.parameter_values))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_iterable_values(&self) -> Option<Vec<Arc<TemplateValue>>> {
        Some(
            self.parameter_values
                .iter()
                .map(|value| {
                    Arc::new(
                        value
                            .clone()
                            .map_or(TemplateValue::Null, TemplateValue::string),
                    )
                })
                .collect(),
        )
    }
}

fn is_named(name: Option<&JavaString>, expected: &str) -> bool {
    name.is_some_and(|name| name.to_string_lossy() == expected)
}

fn is_reserved(name: Option<&JavaString>) -> bool {
    is_named(name, PARAM_VARIABLE_NAME)
        || is_named(name, SESSION_VARIABLE_NAME)
        || is_named(name, APPLICATION_VARIABLE_NAME)
}

fn normalize_web_value(value: Option<Arc<TemplateValue>>) -> Option<Arc<TemplateValue>> {
    value.filter(|value| !matches!(value.as_ref(), TemplateValue::Null))
}

fn resolve_lazy(value: Arc<TemplateValue>) -> Option<Arc<TemplateValue>> {
    if let TemplateValue::Object(object) = value.as_ref()
        && let Some(resolved) = object.resolve_lazy_context_variable()
    {
        return resolved;
    }
    Some(value)
}

fn same_identity(left: Option<&Arc<TemplateValue>>, right: Option<&Arc<TemplateValue>>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        _ => false,
    }
}

fn read_changes(lock: &RwLock<Vec<WebLevelChanges>>) -> RwLockReadGuard<'_, Vec<WebLevelChanges>> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_changes(
    lock: &RwLock<Vec<WebLevelChanges>>,
) -> RwLockWriteGuard<'_, Vec<WebLevelChanges>> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn format_attribute_map(
    attributes: &IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>,
) -> String {
    let mut output = String::from("{");
    for (index, (name, value)) in attributes.iter().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        output.push_str(
            &name
                .as_ref()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
        );
        output.push('=');
        output.push_str(
            &value
                .as_deref()
                .and_then(TemplateValue::to_java_string)
                .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy()),
        );
    }
    output.push('}');
    output
}

fn format_parameter_map(
    parameters: &IndexMap<Option<JavaString>, Option<Vec<Option<JavaString>>>>,
) -> String {
    let mut output = String::from("{");
    for (index, (name, values)) in parameters.iter().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        output.push_str(
            &name
                .as_ref()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
        );
        output.push('=');
        output.push_str(&values.as_ref().map_or_else(
            || "null".to_owned(),
            |values| format_parameter_values(values),
        ));
    }
    output.push('}');
    output
}

fn format_parameter_values(values: &[Option<JavaString>]) -> String {
    let mut output = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        output.push_str(
            &value
                .as_ref()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
        );
    }
    output.push(']');
    output
}

fn format_link_parameters(
    parameters: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
) -> String {
    let Some(parameters) = parameters else {
        return "null".to_owned();
    };
    let mut result = String::from("{");
    for (index, (name, value)) in parameters.iter().enumerate() {
        if index != 0 {
            result.push_str(", ");
        }
        result.push_str(
            &name
                .as_ref()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
        );
        result.push('=');
        result.push_str(
            &value
                .as_deref()
                .and_then(TemplateValue::to_java_string)
                .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy()),
        );
    }
    result.push('}');
    result
}

/// Java `ExchangeAttributeMap` 单层重推导：丢弃被 exchange 直写覆盖的条目。
fn format_web_level_changes_exchange_aware(
    changes: &IndexMap<Option<JavaString>, WebVariableChange>,
    old_values_sum: &mut IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>,
    exchange: &dyn crate::web::IWebExchange,
) -> String {
    let mut output = String::from("{");
    let mut written = 0_usize;
    for (name, change) in changes {
        if same_identity(change.new_value.as_ref(), change.old_value.as_ref()) {
            // Java：newValue == oldValue 的空操作，跳过。
            continue;
        }
        let exchange_value = name
            .as_ref()
            .and_then(|name| exchange.get_attribute_value(Some(name)));
        let discard = if !old_values_sum.contains_key(name) {
            // 该名未被更深层改动：若当前 exchange 值已被 request 直写替换，
            // newValue 与 exchange 值不再同一 -> 丢弃。
            !same_identity(change.new_value.as_ref(), exchange_value.as_ref())
        } else {
            // 该名已在恢复值表：newValue 需与记录的旧值同一，否则丢弃。
            !same_identity(
                change.new_value.as_ref(),
                old_values_sum.get(name).and_then(Option::as_ref),
            )
        };
        if discard {
            continue;
        }
        if written != 0 {
            output.push_str(", ");
        }
        output.push_str(
            &name
                .as_ref()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
        );
        output.push('=');
        output.push_str(
            &change
                .new_value
                .as_deref()
                .and_then(TemplateValue::to_java_string)
                .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy()),
        );
        written += 1;
        old_values_sum.insert(name.clone(), change.old_value.clone());
    }
    output.push('}');
    output
}

/// Java 基座（level 0）：从活 exchange 属性 + 恢复值表构建。
fn format_web_base_exchange_aware(
    exchange: &dyn crate::web::IWebExchange,
    old_values_sum: &mut IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>,
) -> String {
    let mut base: IndexMap<JavaString, Option<Arc<TemplateValue>>> = IndexMap::new();
    for name in exchange.get_all_attribute_names() {
        let Some(name) = name else {
            continue;
        };
        if let Some(old) = old_values_sum.shift_remove(&Some(name.clone())) {
            if old.is_some() {
                base.insert(name, old);
            }
        } else {
            base.insert(name.clone(), exchange.get_attribute_value(Some(&name)));
        }
    }
    for (name, old) in old_values_sum.iter() {
        let Some(name) = name else {
            continue;
        };
        if !base.contains_key(name) && old.is_some() {
            base.insert(name.clone(), old.clone());
        }
    }
    let mut output = String::from("{");
    for (index, (name, value)) in base.iter().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        output.push_str(&name.to_string_lossy());
        output.push('=');
        output.push_str(
            &value
                .as_deref()
                .and_then(TemplateValue::to_java_string)
                .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy()),
        );
    }
    output.push('}');
    output
}

fn replace_level_variable_map(representation: &mut String, level: i32, replacement: &str) {
    let marker = format!("{level}:{{");
    let Some(marker_start) = representation.find(&marker) else {
        return;
    };
    let map_start = marker_start + marker.len() - 1;
    let Some(map_end_relative) = representation[map_start..].find('}') else {
        return;
    };
    let map_end = map_start + map_end_relative;
    representation.replace_range(map_start..=map_end, replacement);
}
