use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::expression::{IExpressionObjects, TemplateValue};
use crate::util::{JavaLocale, JavaString, ValidateError};

use super::ContextVariableEntries;
use super::{AbstractExpressionContext, IContext, IContextVariableNames, IExpressionContext};

/// 非 Web 场景的基础表达式上下文。
///
/// 对应 Java: `org.thymeleaf.context.ExpressionContext`。
///
/// 与 Java 类一致，该类型面向一次模板执行，不应跨不同模板执行复用。
pub struct ExpressionContext {
    base: Arc<AbstractExpressionContext>,
}

impl ExpressionContext {
    /// 使用默认 Locale 和空变量创建表达式上下文。
    pub fn new(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
    ) -> Result<Self, ValidateError> {
        AbstractExpressionContext::new(configuration).map(|base| Self { base })
    }

    /// 使用指定 Locale 和空变量创建表达式上下文。
    pub fn with_locale(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        locale: Option<JavaLocale>,
    ) -> Result<Self, ValidateError> {
        AbstractExpressionContext::with_locale(configuration, locale).map(|base| Self { base })
    }

    /// 使用配置、Locale 和变量快照创建表达式上下文。
    pub fn with_locale_and_variables(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        locale: Option<JavaLocale>,
        variables: ContextVariableEntries<'_>,
    ) -> Result<Self, ValidateError> {
        AbstractExpressionContext::with_locale_and_variables(configuration, locale, variables)
            .map(|base| Self { base })
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

impl IContext for ExpressionContext {
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
}

impl IExpressionContext for ExpressionContext {
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
