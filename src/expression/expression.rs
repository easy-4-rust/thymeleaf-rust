use super::IStandardExpression;

/// Thymeleaf 内建 Standard Expression 的共同抽象层。
///
/// 对应 Java: `org.thymeleaf.standard.expression.Expression`。
///
/// Java 使用抽象类的 final `execute` 完成 Simple/Complex 分派；Rust 将同一分派
/// 下沉到各具体对象的 `IStandardExpression` 动态方法，因此本 trait 保留类型层级，
/// 而不会再维护一份基于 `instanceof` 的中央分派表。
pub trait Expression: IStandardExpression {
    /// 嵌套表达式起始定界符。
    const NESTING_START_CHAR: u16 = b'(' as u16;
    /// 嵌套表达式结束定界符。
    const NESTING_END_CHAR: u16 = b')' as u16;
}

impl<T: IStandardExpression + ?Sized> Expression for T {}
