use std::sync::{Arc, OnceLock, Weak};

use crate::IEngineConfiguration;
use crate::expression::{ExpressionObjects, IExpressionObjects, TemplateValue};
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
    self_weak: Weak<ExpressionContext>,
    expression_objects: OnceLock<ExpressionObjects>,
}

impl ExpressionContext {
    /// 使用默认 Locale 和空变量创建表达式上下文。
    ///
    /// 对应 Java: `ExpressionContext#ExpressionContext(IEngineConfiguration)`。
    ///
    /// # 参数
    ///
    /// - `configuration`：所属模板引擎配置；`None` 对应 Java `null`。
    ///
    /// # 返回值
    ///
    /// 返回共享的具体 `ExpressionContext`。共享身份用于保证表达式对象工厂收到的
    /// Context 与调用方持有的对象完全相同。
    ///
    /// # 错误
    ///
    /// 配置为空时返回 `Configuration cannot be null`。
    pub fn new(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
    ) -> Result<Arc<Self>, ValidateError> {
        let base = AbstractExpressionContext::new(configuration)?;
        Ok(Self::from_base(base))
    }

    /// 使用指定 Locale 和空变量创建表达式上下文。
    ///
    /// 对应 Java:
    /// `ExpressionContext#ExpressionContext(IEngineConfiguration, Locale)`。
    ///
    /// # 参数
    ///
    /// - `configuration`：所属模板引擎配置。
    /// - `locale`：可空 Locale；为空时在构造瞬间取得进程默认 Locale。
    ///
    /// # 返回值
    ///
    /// 返回保持具体类型身份的共享 Context。
    pub fn with_locale(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        locale: Option<JavaLocale>,
    ) -> Result<Arc<Self>, ValidateError> {
        let base = AbstractExpressionContext::with_locale(configuration, locale)?;
        Ok(Self::from_base(base))
    }

    /// 使用配置、Locale 和变量快照创建表达式上下文。
    ///
    /// 对应 Java:
    /// `ExpressionContext#ExpressionContext(IEngineConfiguration, Locale, Map)`。
    ///
    /// # 参数
    ///
    /// - `configuration`：所属模板引擎配置。
    /// - `locale`：可空 Locale。
    /// - `variables`：可空、有序变量 Map 快照。
    ///
    /// # 返回值
    ///
    /// 返回保持输入变量插入顺序及值身份的共享 Context。
    pub fn with_locale_and_variables(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        locale: Option<JavaLocale>,
        variables: ContextVariableEntries<'_>,
    ) -> Result<Arc<Self>, ValidateError> {
        let base =
            AbstractExpressionContext::with_locale_and_variables(configuration, locale, variables)?;
        Ok(Self::from_base(base))
    }

    fn from_base(base: Arc<AbstractExpressionContext>) -> Arc<Self> {
        Arc::new_cyclic(|self_weak| Self {
            base,
            self_weak: self_weak.clone(),
            expression_objects: OnceLock::new(),
        })
    }

    /// 修改模板处理 Locale。
    ///
    /// # 参数
    ///
    /// - `locale`：新的非空 Locale。
    ///
    /// # 错误
    ///
    /// `locale` 为空时返回 Java `IllegalArgumentException` 等价错误。
    /// 对应 Java 语义：Java 接口/超类方法 `setLocale()` 的 Rust 移植（`ExpressionContext` 继承路径）。
    pub fn set_locale(&self, locale: Option<JavaLocale>) -> Result<(), ValidateError> {
        self.base.set_locale(locale)
    }

    /// 新增或替换单个变量。
    ///
    /// # 参数
    ///
    /// - `name`：可空变量名。
    /// - `value`：可空变量值；空值保存为显式 Java `null`。
    /// 对应 Java 语义：Java 接口/超类方法 `setVariable()` 的 Rust 移植（`ExpressionContext` 继承路径）。
    pub fn set_variable(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        self.base.set_variable(name, value);
    }

    /// 按输入迭代顺序批量新增或替换变量。
    ///
    /// # 参数
    ///
    /// - `variables`：可空变量 Map；为空时不执行任何操作。
    /// 对应 Java 语义：Java 接口/超类方法 `setVariables()` 的 Rust 移植（`ExpressionContext` 继承路径）。
    pub fn set_variables(&self, variables: ContextVariableEntries<'_>) {
        self.base.set_variables(variables);
    }

    /// 删除指定变量。
    ///
    /// # 参数
    ///
    /// - `name`：待删除的可空变量名。
    /// 对应 Java 语义：Java 接口/超类方法 `removeVariable()` 的 Rust 移植（`ExpressionContext` 继承路径）。
    pub fn remove_variable(&self, name: Option<&JavaString>) {
        self.base.remove_variable(name);
    }

    /// 删除全部变量。
    ///
    /// 已取得的变量名视图会立即观察到清空结果。
    /// 对应 Java 语义：Java 接口/超类方法 `clearVariables()` 的 Rust 移植（`ExpressionContext` 继承路径）。
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

    fn get_variable_names(&self) -> Arc<dyn IContextVariableNames + '_> {
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
        self.expression_objects.get_or_init(|| {
            // Java 把具体 `this` 传给工厂；不能把组合使用的 AbstractExpressionContext
            // 暴露出去，否则自定义工厂的身份比较和具体类型判断会失真。
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
