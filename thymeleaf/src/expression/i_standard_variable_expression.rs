use std::any::Any;
use std::sync::Arc;

use crate::util::JavaString;

/// `${...}` 与 `*{...}` 变量表达式的共同合同。
///
/// 对应 Java: `org.thymeleaf.standard.expression.IStandardVariableExpression`。
pub trait IStandardVariableExpression {
    /// 返回定界符内部的表达式文本。
    fn get_expression(&self) -> Option<&JavaString>;
    /// 返回是否以 selection target 为求值根。
    fn get_use_selection_as_root(&self) -> bool;
    /// 返回是否启用双括号字符串转换。
    fn get_convert_to_string(&self) -> bool;
    /// 返回求值器缓存的已解析内部表达式。
    fn get_cached_expression(&self) -> Option<Arc<dyn Any + Send + Sync>>;
    /// 替换求值器缓存的已解析内部表达式。
    ///
    /// # 参数
    /// - `cached_expression`：Java volatile 缓存字段的新值；`None` 对应 null。
    fn set_cached_expression(&self, cached_expression: Option<Arc<dyn Any + Send + Sync>>);
}
