use std::any::Any;
use std::sync::Arc;

use crate::expression::TemplateValue;
use crate::util::{JavaLocale, JavaString};

/// `IContext#getVariableNames()` 返回的可变 Set 视图合同。
///
/// Java `Map#keySet()` 是由原 Map 支撑的实时视图，移除名称也会删除变量。该
/// Rust 合同保留实时查询与删除能力，同时用快照方法支持安全迭代。
pub trait IContextVariableNames {
    /// 返回当前变量名数量。
    fn len(&self) -> usize;

    /// 判断当前名称集合是否为空。
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 判断集合是否包含可空名称。
    fn contains(&self, name: Option<&JavaString>) -> bool;

    /// 返回当前迭代顺序的独立名称快照。
    fn snapshot(&self) -> Vec<Option<JavaString>>;

    /// 从支撑 Context 删除名称及其变量。
    ///
    /// # 返回
    ///
    /// 名称原先存在时返回 `true`。
    fn remove(&self, name: Option<&JavaString>) -> bool;
}

/// 模板执行所需 Locale 与变量的基础上下文合同。
///
/// 对应 Java: `org.thymeleaf.context.IContext`。
///
/// Context 刻意不继承 Map，表达式访问必须经过引擎控制层，从而避免自定义 Map
/// 实现绕过安全限制。
pub trait IContext: Any {
    /// 返回 `Any` 视图，供 `Contexts` 复现 Java `instanceof`/强制转换。
    fn as_any(&self) -> &dyn Any;

    /// 返回模板处理使用的 Locale 快照。
    fn get_locale(&self) -> JavaLocale;

    /// 判断指定可空变量名是否已存在。
    fn contains_variable(&self, name: Option<&JavaString>) -> bool;

    /// 返回由 Context 变量 Map 支撑的实时名称视图。
    fn get_variable_names(&self) -> Box<dyn IContextVariableNames + '_>;

    /// 返回指定变量的可空值。
    ///
    /// `None` 表示变量不存在；显式 Java null 返回
    /// `Some(TemplateValue::Null)`，最终 Java API 边界可重新折叠两者。
    fn get_variable(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>>;
}
