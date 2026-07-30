use crate::util::JavaString;

use super::{ComplexExpression, IStandardExpression};

/// 相等与不相等表达式的共同抽象合同。
///
/// 对应 Java: `org.thymeleaf.standard.expression.EqualsNotEqualsExpression`。
pub trait EqualsNotEqualsExpression: ComplexExpression {
    /// `==` 操作符。
    fn equals_operator() -> JavaString {
        JavaString::from_rust_str("==")
    }
    /// `eq` 操作符。
    fn equals_operator_2() -> JavaString {
        JavaString::from_rust_str("eq")
    }
    /// `!=` 操作符。
    fn not_equals_operator() -> JavaString {
        JavaString::from_rust_str("!=")
    }
    /// `neq` 操作符。
    fn not_equals_operator_2() -> JavaString {
        JavaString::from_rust_str("neq")
    }
    /// `ne` 操作符。
    fn not_equals_operator_3() -> JavaString {
        JavaString::from_rust_str("ne")
    }
    /// 上游允许任意左操作数，包括 null。
    fn is_left_allowed(_left: Option<&dyn IStandardExpression>) -> bool {
        true
    }
    /// 上游允许任意右操作数，包括 null。
    fn is_right_allowed(_right: Option<&dyn IStandardExpression>) -> bool {
        true
    }
}
