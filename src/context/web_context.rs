use std::sync::Arc;

use crate::expression::TemplateValue;
use crate::util::{JavaLocale, JavaString, ValidateError};
use crate::web::IWebExchange;

use super::ContextVariableEntries;
use super::{AbstractContext, IContext, IContextVariableNames, IWebContext};

/// 基础 Web 模板上下文。
///
/// 对应 Java: `org.thymeleaf.context.WebContext`。
///
/// 对象保留宿主 exchange 的共享身份，并组合 `AbstractContext` 的 Locale、变量及
/// 实时变量名视图语义。
pub struct WebContext {
    base: AbstractContext,
    web_exchange: Arc<dyn IWebExchange>,
}

impl WebContext {
    /// 使用默认 Locale 和空变量创建 Web Context。
    pub fn new(web_exchange: Option<Arc<dyn IWebExchange>>) -> Result<Self, ValidateError> {
        Self::with_locale_and_variables(web_exchange, None, None)
    }

    /// 使用指定 Locale 和空变量创建 Web Context。
    pub fn with_locale(
        web_exchange: Option<Arc<dyn IWebExchange>>,
        locale: Option<JavaLocale>,
    ) -> Result<Self, ValidateError> {
        Self::with_locale_and_variables(web_exchange, locale, None)
    }

    /// 使用 exchange、Locale 与变量快照创建 Web Context。
    pub fn with_locale_and_variables(
        web_exchange: Option<Arc<dyn IWebExchange>>,
        locale: Option<JavaLocale>,
        variables: ContextVariableEntries<'_>,
    ) -> Result<Self, ValidateError> {
        let web_exchange = web_exchange.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Web exchange cannot be null in web context".to_owned()),
        })?;
        Ok(Self {
            base: AbstractContext::new(locale, variables),
            web_exchange,
        })
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

impl IContext for WebContext {
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
    fn get_web_exchange_arc(&self) -> Option<Arc<dyn IWebExchange>> {
        Some(Arc::clone(&self.web_exchange))
    }
    fn as_web_context(&self) -> Option<&dyn IWebContext> {
        Some(self)
    }
}

impl IWebContext for WebContext {
    fn get_exchange(&self) -> &dyn IWebExchange {
        self.web_exchange.as_ref()
    }
}
