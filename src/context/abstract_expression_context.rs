use std::sync::{Arc, OnceLock, Weak};

use crate::IEngineConfiguration;
use crate::expression::{ExpressionObjects, IExpressionObjects, TemplateValue};
use crate::util::{JavaLocale, JavaString, ValidateError};
use crate::web::IWebExchange;

use super::ContextVariableEntries;
use super::{AbstractContext, IContext, IContextVariableNames, IExpressionContext};

/// 表达式上下文的共享基础实现。
///
/// 对应 Java: `org.thymeleaf.context.AbstractExpressionContext`。
///
/// 该对象组合 `AbstractContext` 的 Locale 与变量语义，并在第一次访问时创建
/// `ExpressionObjects`。配置引用在整个上下文生命周期保持同一共享身份。
pub struct AbstractExpressionContext {
    base: AbstractContext,
    configuration: Arc<dyn IEngineConfiguration>,
    web_exchange: Option<Arc<dyn IWebExchange>>,
    self_weak: Weak<AbstractExpressionContext>,
    expression_objects: OnceLock<ExpressionObjects>,
}

impl AbstractExpressionContext {
    /// 使用默认 Locale 和空变量创建基础表达式上下文。
    pub fn new(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
    ) -> Result<Arc<Self>, ValidateError> {
        Self::with_locale_variables_and_web_exchange(configuration, None, None, None)
    }

    /// 使用指定可空 Locale 和空变量创建基础表达式上下文。
    pub fn with_locale(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        locale: Option<JavaLocale>,
    ) -> Result<Arc<Self>, ValidateError> {
        Self::with_locale_variables_and_web_exchange(configuration, locale, None, None)
    }

    /// 使用配置、可空 Locale 和变量快照创建基础表达式上下文。
    ///
    /// 对应 Java:
    /// `AbstractExpressionContext#AbstractExpressionContext(IEngineConfiguration, Locale, Map)`。
    pub fn with_locale_and_variables(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        locale: Option<JavaLocale>,
        variables: ContextVariableEntries<'_>,
    ) -> Result<Arc<Self>, ValidateError> {
        Self::with_locale_variables_and_web_exchange(configuration, locale, variables, None)
    }

    pub(super) fn with_locale_variables_and_web_exchange(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        locale: Option<JavaLocale>,
        variables: ContextVariableEntries<'_>,
        web_exchange: Option<Arc<dyn IWebExchange>>,
    ) -> Result<Arc<Self>, ValidateError> {
        let configuration = configuration.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Configuration cannot be null".to_owned()),
        })?;
        Ok(Arc::new_cyclic(|self_weak| Self {
            base: AbstractContext::new(locale, variables),
            configuration,
            web_exchange,
            self_weak: self_weak.clone(),
            expression_objects: OnceLock::new(),
        }))
    }

    /// 修改模板处理 Locale。
    pub fn set_locale(&self, locale: Option<JavaLocale>) -> Result<(), ValidateError> {
        self.base.set_locale(locale)
    }

    /// 新增或替换单个变量。
    pub fn set_variable(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        self.base.set_variable(name, value);
    }

    /// 按输入迭代顺序批量新增或替换变量。
    pub fn set_variables(&self, variables: ContextVariableEntries<'_>) {
        self.base.set_variables(variables);
    }

    /// 删除指定变量。
    pub fn remove_variable(&self, name: Option<&JavaString>) {
        self.base.remove_variable(name);
    }

    /// 删除全部变量。
    pub fn clear_variables(&self) {
        self.base.clear_variables();
    }
}

impl IContext for AbstractExpressionContext {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_locale(&self) -> JavaLocale {
        self.base.get_locale()
    }

    fn contains_variable(&self, name: Option<&JavaString>) -> bool {
        self.base.contains_variable(name)
    }

    fn get_variable_names(&self) -> Box<dyn IContextVariableNames + '_> {
        self.base.get_variable_names()
    }

    fn get_variable(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>> {
        self.base.get_variable(name)
    }

    fn get_web_exchange(&self) -> Option<&dyn IWebExchange> {
        self.web_exchange.as_deref()
    }
}

impl IExpressionContext for AbstractExpressionContext {
    fn get_configuration(&self) -> &dyn IEngineConfiguration {
        self.configuration.as_ref()
    }

    fn get_configuration_arc(&self) -> Arc<dyn IEngineConfiguration> {
        Arc::clone(&self.configuration)
    }

    fn get_expression_objects(&self) -> &dyn IExpressionObjects {
        self.expression_objects.get_or_init(|| {
            let weak: Weak<dyn IExpressionContext> = self.self_weak.clone();
            ExpressionObjects::new(
                Some(weak),
                Some(self.configuration.get_expression_object_factory()),
            )
            .expect("validated expression-object dependencies")
        })
    }
}
