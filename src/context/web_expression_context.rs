use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::expression::{IExpressionObjects, TemplateValue};
use crate::util::{JavaLocale, JavaString, ValidateError};
use crate::web::IWebExchange;

use super::ContextVariableEntries;
use super::{
    AbstractExpressionContext, IContext, IContextVariableNames, IExpressionContext, IWebContext,
};

/// Web 场景的标准表达式上下文。
///
/// 对应 Java: `org.thymeleaf.context.WebExpressionContext`。
///
/// 对象同时暴露表达式配置和框架中立 Web exchange。内部表达式对象工厂通过
/// `IContext#get_web_exchange` capability 仍能识别 Web 上下文，不会因 Rust 组合
/// 代替 Java 继承而丢失 `instanceof IWebContext` 的语义。
pub struct WebExpressionContext {
    base: Arc<AbstractExpressionContext>,
    web_exchange: Arc<dyn IWebExchange>,
}

impl WebExpressionContext {
    /// 使用默认 Locale 和空变量创建 Web 表达式上下文。
    pub fn new(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        web_exchange: Option<Arc<dyn IWebExchange>>,
    ) -> Result<Self, ValidateError> {
        Self::with_locale_and_variables(configuration, web_exchange, None, None)
    }

    /// 使用指定 Locale 和空变量创建 Web 表达式上下文。
    pub fn with_locale(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        web_exchange: Option<Arc<dyn IWebExchange>>,
        locale: Option<JavaLocale>,
    ) -> Result<Self, ValidateError> {
        Self::with_locale_and_variables(configuration, web_exchange, locale, None)
    }

    /// 使用配置、exchange、Locale 与变量快照创建 Web 表达式上下文。
    pub fn with_locale_and_variables(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        web_exchange: Option<Arc<dyn IWebExchange>>,
        locale: Option<JavaLocale>,
        variables: ContextVariableEntries<'_>,
    ) -> Result<Self, ValidateError> {
        // Java 先调用 super(configuration)，再校验 webExchange，保持首错顺序。
        let configuration = configuration.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Configuration cannot be null".to_owned()),
        })?;
        let web_exchange = web_exchange.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Web exchange cannot be null in web context".to_owned()),
        })?;
        let base = AbstractExpressionContext::with_locale_variables_and_web_exchange(
            Some(configuration),
            locale,
            variables,
            Some(Arc::clone(&web_exchange)),
        )?;
        Ok(Self { base, web_exchange })
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

impl IContext for WebExpressionContext {
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
        Some(self.web_exchange.as_ref())
    }
}

impl IExpressionContext for WebExpressionContext {
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

impl IWebContext for WebExpressionContext {
    fn get_exchange(&self) -> &dyn IWebExchange {
        self.web_exchange.as_ref()
    }
}
