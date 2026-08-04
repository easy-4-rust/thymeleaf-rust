use std::sync::{Arc, OnceLock, Weak};

use crate::IEngineConfiguration;
use crate::expression::{ExpressionObjects, IExpressionObjects, TemplateValue};
use crate::util::{Locale, Utf16String, ValidateError};
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
    ///
    /// 对应 Java:
    /// `AbstractExpressionContext#AbstractExpressionContext(IEngineConfiguration)`。
    ///
    /// # 参数
    ///
    /// - `configuration`：所属引擎配置。
    ///
    /// # 返回值
    ///
    /// 返回共享基础 Context。
    ///
    /// # 错误
    ///
    /// 配置为空时返回 `Configuration cannot be null`。
    pub fn new(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
    ) -> Result<Arc<Self>, ValidateError> {
        Self::with_locale_variables_and_web_exchange(configuration, None, None, None)
    }

    /// 使用指定可空 Locale 和空变量创建基础表达式上下文。
    ///
    /// 对应 Java:
    /// `AbstractExpressionContext#AbstractExpressionContext(IEngineConfiguration, Locale)`。
    ///
    /// # 参数
    ///
    /// - `configuration`：所属引擎配置。
    /// - `locale`：可空 Locale。
    ///
    /// # 返回值
    ///
    /// 返回共享基础 Context。
    pub fn with_locale(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        locale: Option<Locale>,
    ) -> Result<Arc<Self>, ValidateError> {
        Self::with_locale_variables_and_web_exchange(configuration, locale, None, None)
    }

    /// 使用配置、可空 Locale 和变量快照创建基础表达式上下文。
    ///
    /// 对应 Java:
    /// `AbstractExpressionContext#AbstractExpressionContext(IEngineConfiguration, Locale, Map)`。
    ///
    /// # 参数
    ///
    /// - `configuration`：所属引擎配置。
    /// - `locale`：可空 Locale。
    /// - `variables`：可空、有序变量 Map 快照。
    ///
    /// # 返回值
    ///
    /// 返回延迟创建表达式对象的共享基础 Context。
    ///
    /// # 错误
    ///
    /// 配置为空时返回与 Java 校验一致的参数错误。
    pub fn with_locale_and_variables(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        locale: Option<Locale>,
        variables: ContextVariableEntries<'_>,
    ) -> Result<Arc<Self>, ValidateError> {
        Self::with_locale_variables_and_web_exchange(configuration, locale, variables, None)
    }

    /// 对应 Java 语义：`AbstractExpressionContext` 的 `with_locale_variables_and_web_exchange` 行为（Rust 侧辅助/私有路径）。
    pub(super) fn with_locale_variables_and_web_exchange(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        locale: Option<Locale>,
        variables: ContextVariableEntries<'_>,
        web_exchange: Option<Arc<dyn IWebExchange>>,
    ) -> Result<Arc<Self>, ValidateError> {
        // Java 先执行 super(locale, variables)，再校验 configuration。这里也先复制
        // Context 输入，从而保留构造阶段的原始求值与异常顺序。
        let base = AbstractContext::new(locale, variables);
        let configuration = configuration.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Configuration cannot be null".to_owned()),
        })?;
        Ok(Arc::new_cyclic(move |self_weak| Self {
            base,
            configuration,
            web_exchange,
            self_weak: self_weak.clone(),
            expression_objects: OnceLock::new(),
        }))
    }

    /// 修改模板处理 Locale。
    ///
    /// # 参数
    ///
    /// - `locale`：新的非空 Locale。
    ///
    /// # 错误
    ///
    /// Locale 为空时返回 `Locale cannot be null`。
    /// 对应 Java 语义：Java 接口/超类方法 `setLocale()` 的 Rust 移植（`AbstractExpressionContext` 继承路径）。
    pub fn set_locale(&self, locale: Option<Locale>) -> Result<(), ValidateError> {
        self.base.set_locale(locale)
    }

    /// 新增或替换单个变量。
    ///
    /// # 参数
    ///
    /// - `name`：可空变量名。
    /// - `value`：可空变量值。
    ///
    /// 对应 Java 语义：Java 接口/超类方法 `setVariable()` 的 Rust 移植（`AbstractExpressionContext` 继承路径）。
    pub fn set_variable(&self, name: Option<Utf16String>, value: Option<Arc<TemplateValue>>) {
        self.base.set_variable(name, value);
    }

    /// 按输入迭代顺序批量新增或替换变量。
    ///
    /// # 参数
    ///
    /// - `variables`：可空变量 Map；为空时无副作用。
    ///
    /// 对应 Java 语义：Java 接口/超类方法 `setVariables()` 的 Rust 移植（`AbstractExpressionContext` 继承路径）。
    pub fn set_variables(&self, variables: ContextVariableEntries<'_>) {
        self.base.set_variables(variables);
    }

    /// 删除指定变量。
    ///
    /// # 参数
    ///
    /// - `name`：待删除的可空变量名。
    ///
    /// 对应 Java 语义：Java 接口/超类方法 `removeVariable()` 的 Rust 移植（`AbstractExpressionContext` 继承路径）。
    pub fn remove_variable(&self, name: Option<&Utf16String>) {
        self.base.remove_variable(name);
    }

    /// 删除全部变量。
    ///
    /// 已发布的变量名实时视图同步观察该修改。
    /// 对应 Java 语义：Java 接口/超类方法 `clearVariables()` 的 Rust 移植（`AbstractExpressionContext` 继承路径）。
    pub fn clear_variables(&self) {
        self.base.clear_variables();
    }
}

impl IContext for AbstractExpressionContext {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_locale(&self) -> Locale {
        self.base.get_locale()
    }

    fn contains_variable(&self, name: Option<&Utf16String>) -> bool {
        self.base.contains_variable(name)
    }

    fn get_variable_names(&self) -> Arc<dyn IContextVariableNames + '_> {
        self.base.get_variable_names()
    }

    fn get_variable(&self, name: Option<&Utf16String>) -> Option<Arc<TemplateValue>> {
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
