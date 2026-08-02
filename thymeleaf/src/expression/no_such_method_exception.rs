use std::fmt::{Display, Formatter};

/// OGNL 动态调用找不到目标方法时保留的异常身份。
///
/// 对应 Java: `java.lang.NoSuchMethodException`。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoSuchMethodException {
    message: String,
}

impl NoSuchMethodException {
    /// 创建包含目标类与方法信息的异常。
    #[must_use]
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Display for NoSuchMethodException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for NoSuchMethodException {}
