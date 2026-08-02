use std::sync::{Arc, OnceLock, Weak};

use crate::IEngineConfiguration;
use crate::expression::{ExpressionObjects, IExpressionObjects, TemplateValue};
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
    self_weak: Weak<WebExpressionContext>,
    expression_objects: OnceLock<ExpressionObjects>,
}

impl WebExpressionContext {
    /// 使用默认 Locale 和空变量创建 Web 表达式上下文。
    ///
    /// 对应 Java:
    /// `WebExpressionContext#WebExpressionContext(IEngineConfiguration, IWebExchange)`。
    ///
    /// # 参数
    ///
    /// - `configuration`：所属模板引擎配置。
    /// - `web_exchange`：当前模板执行使用的 Web exchange。
    ///
    /// # 返回值
    ///
    /// 返回保持具体 Web Context 身份的共享对象。
    ///
    /// # 错误
    ///
    /// 按 Java 构造顺序先报告空 configuration，再报告空 exchange。
    pub fn new(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        web_exchange: Option<Arc<dyn IWebExchange>>,
    ) -> Result<Arc<Self>, ValidateError> {
        Self::with_locale_and_variables(configuration, web_exchange, None, None)
    }

    /// 使用指定 Locale 和空变量创建 Web 表达式上下文。
    ///
    /// 对应 Java:
    /// `WebExpressionContext#WebExpressionContext(IEngineConfiguration, IWebExchange, Locale)`。
    ///
    /// # 参数
    ///
    /// - `configuration`：所属模板引擎配置。
    /// - `web_exchange`：当前模板执行使用的 Web exchange。
    /// - `locale`：可空 Locale；为空时在基础构造阶段读取默认值。
    ///
    /// # 返回值
    ///
    /// 返回同时实现 `IExpressionContext` 与 `IWebContext` 的共享具体对象。
    pub fn with_locale(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        web_exchange: Option<Arc<dyn IWebExchange>>,
        locale: Option<JavaLocale>,
    ) -> Result<Arc<Self>, ValidateError> {
        Self::with_locale_and_variables(configuration, web_exchange, locale, None)
    }

    /// 使用配置、exchange、Locale 与变量快照创建 Web 表达式上下文。
    ///
    /// 对应 Java:
    /// `WebExpressionContext#WebExpressionContext(IEngineConfiguration, IWebExchange, Locale, Map)`。
    ///
    /// Java 先完整执行 `AbstractExpressionContext` 构造，再校验 exchange。本实现先创建
    /// 基础表达式 Context，随后校验 exchange，最后用 `Arc::new_cyclic` 绑定具体
    /// `WebExpressionContext` 身份。
    ///
    /// # 参数
    ///
    /// - `configuration`：所属模板引擎配置。
    /// - `web_exchange`：当前模板执行使用的 Web exchange。
    /// - `locale`：可空 Locale。
    /// - `variables`：可空、有序变量 Map 快照。
    ///
    /// # 返回值
    ///
    /// 返回让表达式对象工厂观察到具体 Web Context 与 exchange capability 的共享对象。
    ///
    /// # 错误
    ///
    /// 保留 Java configuration → exchange 的首错顺序和完整消息。
    pub fn with_locale_and_variables(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        web_exchange: Option<Arc<dyn IWebExchange>>,
        locale: Option<JavaLocale>,
        variables: ContextVariableEntries<'_>,
    ) -> Result<Arc<Self>, ValidateError> {
        let base =
            AbstractExpressionContext::with_locale_and_variables(configuration, locale, variables)?;
        let web_exchange = web_exchange.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Web exchange cannot be null in web context".to_owned()),
        })?;
        Ok(Arc::new_cyclic(move |self_weak| Self {
            base,
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
    pub fn set_locale(&self, locale: Option<JavaLocale>) -> Result<(), ValidateError> {
        self.base.set_locale(locale)
    }
    /// 新增或替换单个变量。
    ///
    /// # 参数
    ///
    /// - `name`：可空变量名。
    /// - `value`：可空变量值；空值保存为显式 Java null。
    pub fn set_variable(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        self.base.set_variable(name, value);
    }
    /// 按输入迭代顺序批量新增或替换变量。
    ///
    /// # 参数
    ///
    /// - `variables`：可空变量 Map；为空时无副作用。
    pub fn set_variables(&self, variables: ContextVariableEntries<'_>) {
        self.base.set_variables(variables);
    }
    /// 删除指定变量。
    ///
    /// # 参数
    ///
    /// - `name`：待删除的可空变量名。
    pub fn remove_variable(&self, name: Option<&JavaString>) {
        self.base.remove_variable(name);
    }
    /// 删除全部变量。
    ///
    /// 已取得的变量名实时视图立即观察到清空结果。
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
    fn get_variable_names(&self) -> Arc<dyn IContextVariableNames + '_> {
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

impl IExpressionContext for WebExpressionContext {
    fn get_configuration(&self) -> &dyn IEngineConfiguration {
        self.base.get_configuration()
    }
    fn get_configuration_arc(&self) -> Arc<dyn IEngineConfiguration> {
        self.base.get_configuration_arc()
    }
    fn get_expression_objects(&self) -> &dyn IExpressionObjects {
        self.expression_objects.get_or_init(|| {
            // Java 的 `new ExpressionObjects(this, factory)` 传入同时具备具体类型和
            // IWebContext 能力的 `this`，不能暴露内部组合基础对象。
            let weak: Weak<dyn IExpressionContext> = self.self_weak.clone();
            ExpressionObjects::new(
                Some(weak),
                Some(
                    self.base
                        .get_configuration()
                        .get_expression_object_factory(),
                ),
            )
            .expect("validated expression-object dependencies")
        })
    }
}

impl IWebContext for WebExpressionContext {
    /// 返回构造时传入的同一 Web exchange。
    fn get_exchange(&self) -> &dyn IWebExchange {
        self.web_exchange.as_ref()
    }

    /// 返回与 `get_exchange()` 指向同一分配的共享 exchange。
    fn get_exchange_arc(&self) -> Arc<dyn IWebExchange> {
        Arc::clone(&self.web_exchange)
    }
}
