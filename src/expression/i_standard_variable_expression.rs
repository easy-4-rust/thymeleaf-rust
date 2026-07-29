use crate::util::JavaString;

/// `${...}` 与 `*{...}` 变量表达式的共同合同。
///
/// 对应 Java: `org.thymeleaf.standard.expression.IStandardVariableExpression`。
pub trait IStandardVariableExpression {
    /// 返回定界符内部的表达式文本。
    fn get_expression(&self) -> Option<&JavaString>;
    /// 返回是否以 selection target 为求值根。
    fn get_use_selection_as_root(&self) -> bool;
}
