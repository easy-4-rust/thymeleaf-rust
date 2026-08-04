use std::sync::Arc;

use crate::expression::TemplateValue;
use crate::util::{JavaLocale, Utf16String, ValidateError};

use super::ContextVariableEntries;
use super::{AbstractContext, IContext, IContextVariableNames};

/// 适用于非 Web 场景的基础模板 Context。
///
/// 对应 Java: `org.thymeleaf.context.Context`。
///
/// 该 final 类通过组合复用 `AbstractContext` 的 Locale、插入有序变量与实时名称
/// Set 视图语义。
///
/// Thymeleaf 1.0 起曾存在同名类，3.0 将其重写为多数非 Web 场景使用的基础实现。
pub struct Context {
    base: AbstractContext,
}

impl Context {
    /// 使用当前进程默认 Locale 和空变量 Map 创建 Context。
    ///
    /// 对应 Java: `Context#Context()`。
    ///
    /// # 返回值
    ///
    /// 返回独立的空 Context；默认 Locale 在构造瞬间冻结。
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: AbstractContext::new(None, None),
        }
    }

    /// 使用可空 Locale 和空变量 Map 创建 Context。
    ///
    /// null Locale 在构造时替换为当前默认 Locale。
    ///
    /// 对应 Java: `Context#Context(Locale)`。
    ///
    /// # 参数
    ///
    /// - `locale`：可空 Locale。
    ///
    /// # 返回值
    ///
    /// 返回空变量 Context。
    #[must_use]
    pub fn with_locale(locale: Option<JavaLocale>) -> Self {
        Self {
            base: AbstractContext::new(locale, None),
        }
    }

    /// 使用可空 Locale 和变量 Map 快照创建 Context。
    ///
    /// 对应 Java: `Context#Context(Locale, Map)`。
    ///
    /// # 参数
    ///
    /// - `locale`：可空 Locale。
    /// - `variables`：可空、有序变量条目。
    ///
    /// # 返回值
    ///
    /// 返回与输入条目容器独立的浅复制 Context。
    #[must_use]
    pub fn with_locale_and_variables(
        locale: Option<JavaLocale>,
        variables: ContextVariableEntries<'_>,
    ) -> Self {
        Self {
            base: AbstractContext::new(locale, variables),
        }
    }

    /// 修改模板处理 Locale。
    ///
    /// 对应 Java: `AbstractContext#setLocale(Locale)`。
    ///
    /// # 参数
    ///
    /// - `locale`：新的非空 Locale。
    ///
    /// # 错误
    ///
    /// null Locale 返回 Java `IllegalArgumentException` 对应错误。
    pub fn set_locale(&self, locale: Option<JavaLocale>) -> Result<(), ValidateError> {
        self.base.set_locale(locale)
    }

    /// 新增或替换单个变量。
    ///
    /// # 参数
    ///
    /// - `name`：可空变量名。
    /// - `value`：可空变量值。
    ///
    /// 对应 Java 语义：Java 接口/超类方法 `setVariable()` 的 Rust 移植（`Context` 继承路径）。
    pub fn set_variable(&self, name: Option<Utf16String>, value: Option<Arc<TemplateValue>>) {
        self.base.set_variable(name, value);
    }

    /// 按迭代顺序批量新增或替换变量；null Map 不执行操作。
    ///
    /// # 参数
    ///
    /// - `variables`：可空变量 Map 快照。
    ///
    /// 对应 Java 语义：Java 接口/超类方法 `setVariables()` 的 Rust 移植（`Context` 继承路径）。
    pub fn set_variables(&self, variables: ContextVariableEntries<'_>) {
        self.base.set_variables(variables);
    }

    /// 删除指定变量。
    ///
    /// # 参数
    ///
    /// - `name`：待删除的可空变量名。
    ///
    /// 对应 Java 语义：Java 接口/超类方法 `removeVariable()` 的 Rust 移植（`Context` 继承路径）。
    pub fn remove_variable(&self, name: Option<&Utf16String>) {
        self.base.remove_variable(name);
    }

    /// 删除全部变量。
    ///
    /// 已取得的变量名实时视图会同步观察到空集合。
    /// 对应 Java 语义：Java 接口/超类方法 `clearVariables()` 的 Rust 移植（`Context` 继承路径）。
    pub fn clear_variables(&self) {
        self.base.clear_variables();
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl IContext for Context {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_locale(&self) -> JavaLocale {
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
}
