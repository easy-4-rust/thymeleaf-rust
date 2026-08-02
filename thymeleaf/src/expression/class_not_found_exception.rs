use std::fmt::{Display, Formatter};

/// OGNL 类型解析无法找到 Java 类时保留的异常身份。
///
/// 对应 Java: `java.lang.ClassNotFoundException`。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassNotFoundException {
    class_name: String,
}

impl ClassNotFoundException {
    /// 创建指定类型名的类未找到异常。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn new(class_name: String) -> Self {
        Self { class_name }
    }
}

impl Display for ClassNotFoundException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.class_name)
    }
}

impl std::error::Error for ClassNotFoundException {}
