use super::Expression;

/// Thymeleaf Complex Expression 标记合同。
///
/// 对应 Java: `org.thymeleaf.standard.expression.ComplexExpression`。
///
/// 具体二元、一元和条件表达式直接实现执行动态入口，等价替代 Java 中央
/// `instanceof` 分派。
pub trait ComplexExpression: Expression {}
