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
    ///
    /// 对应 Java: `WebContext#WebContext(IWebExchange)`。
    ///
    /// # 参数
    ///
    /// - `web_exchange`：当前模板执行使用的非空 Web exchange。
    ///
    /// # 返回值
    ///
    /// 返回在构造瞬间冻结默认 Locale 的空变量 Web Context。
    ///
    /// # 错误
    ///
    /// exchange 为空时返回 `Web exchange cannot be null in web context`。
    pub fn new(web_exchange: Option<Arc<dyn IWebExchange>>) -> Result<Self, ValidateError> {
        Self::with_locale_and_variables(web_exchange, None, None)
    }

    /// 使用指定 Locale 和空变量创建 Web Context。
    ///
    /// 对应 Java: `WebContext#WebContext(IWebExchange, Locale)`。
    ///
    /// # 参数
    ///
    /// - `web_exchange`：当前模板执行使用的非空 Web exchange。
    /// - `locale`：可空 Locale；为空时在基础 Context 构造阶段读取进程默认值。
    ///
    /// # 返回值
    ///
    /// 返回保留同一 exchange 身份的 Web Context。
    pub fn with_locale(
        web_exchange: Option<Arc<dyn IWebExchange>>,
        locale: Option<JavaLocale>,
    ) -> Result<Self, ValidateError> {
        Self::with_locale_and_variables(web_exchange, locale, None)
    }

    /// 使用 exchange、Locale 与变量快照创建 Web Context。
    ///
    /// 对应 Java: `WebContext#WebContext(IWebExchange, Locale, Map)`。
    ///
    /// Java 先执行 `super(locale, variables)`，再校验 exchange；因此本实现也先完成
    /// Locale 快照和变量浅复制，保持构造阶段的求值与失败顺序。
    ///
    /// # 参数
    ///
    /// - `web_exchange`：当前模板执行使用的非空 Web exchange。
    /// - `locale`：可空 Locale。
    /// - `variables`：可空、有序变量 Map 快照。
    ///
    /// # 返回值
    ///
    /// 返回共享变量值身份、但与输入条目容器独立的 Web Context。
    ///
    /// # 错误
    ///
    /// exchange 为空时返回 Java `IllegalArgumentException` 等价错误。
    pub fn with_locale_and_variables(
        web_exchange: Option<Arc<dyn IWebExchange>>,
        locale: Option<JavaLocale>,
        variables: ContextVariableEntries<'_>,
    ) -> Result<Self, ValidateError> {
        let base = AbstractContext::new(locale, variables);
        let web_exchange = web_exchange.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Web exchange cannot be null in web context".to_owned()),
        })?;
        Ok(Self { base, web_exchange })
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
    /// 对应 Java 语义：Java 接口/超类方法 `setLocale()` 的 Rust 移植（`WebContext` 继承路径）。
    pub fn set_locale(&self, locale: Option<JavaLocale>) -> Result<(), ValidateError> {
        self.base.set_locale(locale)
    }
    /// 新增或替换单个变量。
    ///
    /// # 参数
    ///
    /// - `name`：可空变量名。
    /// - `value`：可空变量值；空值保存为显式 Java null。
    /// 对应 Java 语义：Java 接口/超类方法 `setVariable()` 的 Rust 移植（`WebContext` 继承路径）。
    pub fn set_variable(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        self.base.set_variable(name, value);
    }
    /// 按输入迭代顺序批量新增或替换变量。
    ///
    /// # 参数
    ///
    /// - `variables`：可空变量 Map；为空时无副作用。
    /// 对应 Java 语义：Java 接口/超类方法 `setVariables()` 的 Rust 移植（`WebContext` 继承路径）。
    pub fn set_variables(&self, variables: ContextVariableEntries<'_>) {
        self.base.set_variables(variables);
    }
    /// 删除指定变量。
    ///
    /// # 参数
    ///
    /// - `name`：待删除的可空变量名。
    /// 对应 Java 语义：Java 接口/超类方法 `removeVariable()` 的 Rust 移植（`WebContext` 继承路径）。
    pub fn remove_variable(&self, name: Option<&JavaString>) {
        self.base.remove_variable(name);
    }
    /// 删除全部变量。
    ///
    /// 已取得的变量名实时视图立即观察到清空结果。
    /// 对应 Java 语义：Java 接口/超类方法 `clearVariables()` 的 Rust 移植（`WebContext` 继承路径）。
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

impl IWebContext for WebContext {
    /// 返回构造时传入的同一 Web exchange。
    fn get_exchange(&self) -> &dyn IWebExchange {
        self.web_exchange.as_ref()
    }

    /// 返回与 `get_exchange()` 指向同一分配的共享 exchange。
    fn get_exchange_arc(&self) -> Arc<dyn IWebExchange> {
        Arc::clone(&self.web_exchange)
    }
}
