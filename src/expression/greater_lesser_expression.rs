use crate::util::JavaString;

use super::{ComplexExpression, IStandardExpression};

/// 大小比较表达式的共同抽象合同。
///
/// 对应 Java: `org.thymeleaf.standard.expression.GreaterLesserExpression`。
pub trait GreaterLesserExpression: ComplexExpression {
    /// `>`。
    fn greater_than_operator() -> JavaString {
        JavaString::from_rust_str(">")
    }
    /// `>=`。
    fn greater_or_equal_to_operator() -> JavaString {
        JavaString::from_rust_str(">=")
    }
    /// `<`。
    fn less_than_operator() -> JavaString {
        JavaString::from_rust_str("<")
    }
    /// `<=`。
    fn less_or_equal_to_operator() -> JavaString {
        JavaString::from_rust_str("<=")
    }
    /// 判断左操作数解析约束。
    fn is_left_allowed(left: Option<&dyn IStandardExpression>) -> bool {
        operand_allowed(left)
    }
    /// 判断右操作数解析约束。
    fn is_right_allowed(right: Option<&dyn IStandardExpression>) -> bool {
        operand_allowed(right)
    }
}

fn operand_allowed(expression: Option<&dyn IStandardExpression>) -> bool {
    expression.is_some_and(|expression| {
        !expression.is_token_expression()
            || expression.is_number_token_expression()
            || expression.is_generic_token_expression()
    })
}
