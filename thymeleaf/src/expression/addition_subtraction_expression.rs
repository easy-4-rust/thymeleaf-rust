use crate::util::Utf16String;

use super::{ComplexExpression, IStandardExpression};

/// 加法与减法表达式的共同抽象合同。
///
/// 对应 Java:
/// `org.thymeleaf.standard.expression.AdditionSubtractionExpression`。
pub trait AdditionSubtractionExpression: ComplexExpression {
    /// 加法操作符。
    fn addition_operator() -> Utf16String {
        Utf16String::from_rust_str("+")
    }

    /// 减法操作符。
    fn subtraction_operator() -> Utf16String {
        Utf16String::from_rust_str("-")
    }

    /// 判断左操作数是否符合上游解析约束。
    fn is_left_allowed(left: Option<&dyn IStandardExpression>) -> bool {
        operand_allowed(left)
    }

    /// 判断右操作数是否符合上游解析约束。
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
