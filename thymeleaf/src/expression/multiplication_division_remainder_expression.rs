use crate::util::JavaString;

use super::{ComplexExpression, IStandardExpression};

/// 乘法、除法与余数表达式的共同抽象合同。
///
/// 对应 Java:
/// `org.thymeleaf.standard.expression.MultiplicationDivisionRemainderExpression`。
pub trait MultiplicationDivisionRemainderExpression: ComplexExpression {
    /// 乘法操作符。
    fn multiplication_operator() -> JavaString {
        JavaString::from_rust_str("*")
    }
    /// 除法符号操作符。
    fn division_operator() -> JavaString {
        JavaString::from_rust_str("/")
    }
    /// 除法关键字操作符。
    fn division_operator_2() -> JavaString {
        JavaString::from_rust_str("div")
    }
    /// 余数符号操作符。
    fn remainder_operator() -> JavaString {
        JavaString::from_rust_str("%")
    }
    /// 余数关键字操作符。
    fn remainder_operator_2() -> JavaString {
        JavaString::from_rust_str("mod")
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
        (!expression.is_token_expression() || expression.is_number_token_expression())
            && !expression.is_text_literal_expression()
    })
}
