use std::fmt::{Display, Formatter};
use std::ptr;

static NO_OP_TOKEN_VALUE: NoOpToken = NoOpToken { _identity: 0 };

/// Thymeleaf 标准表达式的 NO-OP（不执行操作）标记值。
///
/// 本对象只有一个公开静态实例，常用于处理器判断表达式结果是否要求保持原值。
/// Java 未覆写 `equals`/`hashCode`，因此 Rust 相等比较保留引用身份而非文本值。
///
/// 对应 Java: `org.thymeleaf.standard.expression.NoOpToken`。
#[derive(Debug)]
pub struct NoOpToken {
    // 非零尺寸确保静态对象拥有稳定且唯一的可观察地址。
    _identity: u8,
}

impl NoOpToken {
    /// NO-OP 值的唯一公开实例。对应 Java: `NoOpToken.VALUE`。
    pub const VALUE: &'static Self = &NO_OP_TOKEN_VALUE;
}

impl PartialEq for NoOpToken {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Eq for NoOpToken {}

impl Display for NoOpToken {
    /// 输出固定的 NO-OP 表达式文本 `_`。对应 Java: `NoOpToken#toString()`。
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("_")
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::NoOpToken;

    #[test]
    fn exposes_singleton_identity_and_fixed_text() {
        assert!(ptr::eq(NoOpToken::VALUE, NoOpToken::VALUE));
        assert!(NoOpToken::VALUE.eq(NoOpToken::VALUE));
        assert_eq!(NoOpToken::VALUE.to_string(), "_");
    }
}
