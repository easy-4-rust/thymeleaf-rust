use std::sync::Arc;

use crate::expression::TemplateValue;
use crate::util::{JavaLocale, JavaString, ValidateError};

use super::{AbstractContext, IContext, IContextVariableNames};

/// 适用于非 Web 场景的基础模板 Context。
///
/// 对应 Java: `org.thymeleaf.context.Context`。
///
/// 该 final 类通过组合复用 `AbstractContext` 的 Locale、插入有序变量与实时名称
/// Set 视图语义。
pub struct Context {
    base: AbstractContext,
}

impl Context {
    /// 使用当前进程默认 Locale 和空变量 Map 创建 Context。
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: AbstractContext::new(None, None),
        }
    }

    /// 使用可空 Locale 和空变量 Map 创建 Context。
    ///
    /// null Locale 在构造时替换为当前默认 Locale。
    #[must_use]
    pub fn with_locale(locale: Option<JavaLocale>) -> Self {
        Self {
            base: AbstractContext::new(locale, None),
        }
    }

    /// 使用可空 Locale 和变量 Map 快照创建 Context。
    #[must_use]
    pub fn with_locale_and_variables(
        locale: Option<JavaLocale>,
        variables: Option<&[(Option<JavaString>, Option<Arc<TemplateValue>>)]>,
    ) -> Self {
        Self {
            base: AbstractContext::new(locale, variables),
        }
    }

    /// 修改模板处理 Locale。
    ///
    /// # 错误
    ///
    /// null Locale 返回 Java `IllegalArgumentException` 对应错误。
    pub fn set_locale(&self, locale: Option<JavaLocale>) -> Result<(), ValidateError> {
        self.base.set_locale(locale)
    }

    /// 新增或替换单个变量。
    pub fn set_variable(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        self.base.set_variable(name, value);
    }

    /// 按迭代顺序批量新增或替换变量；null Map 不执行操作。
    pub fn set_variables(
        &self,
        variables: Option<&[(Option<JavaString>, Option<Arc<TemplateValue>>)]>,
    ) {
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
