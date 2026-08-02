use super::Expression;

/// Thymeleaf Simple Expression 标记合同。
///
/// 对应 Java: `org.thymeleaf.standard.expression.SimpleExpression`。
///
/// Java 的静态 `executeSimple` 类型分派已由具体表达式的 trait 动态分派等价替代。
pub trait SimpleExpression: Expression {
    /// Simple Expression 内容起始字符。
    const EXPRESSION_START_CHAR: u16 = b'{' as u16;
    /// Simple Expression 内容结束字符。
    const EXPRESSION_END_CHAR: u16 = b'}' as u16;
}
