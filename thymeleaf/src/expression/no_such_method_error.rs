use std::fmt::{Display, Formatter};

/// OGNL 动态调用找不到目标方法时保留的异常身份。
///
/// 对应 Java: `java.lang.NoSuchMethodException`。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoSuchMethodError {
    message: String,
}

impl NoSuchMethodError {
    /// 创建包含目标类与方法信息的异常。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Display for NoSuchMethodError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for NoSuchMethodError {}
