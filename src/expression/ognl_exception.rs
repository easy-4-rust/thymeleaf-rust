use std::fmt::{Display, Formatter};

/// 安全 OGNL 兼容层对外保留的求值异常。
///
/// 该对象只表达 Thymeleaf 可观察的 OGNL 求值失败，不开放 Java 反射能力。
/// 对应 Java: `ognl.OgnlException`。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OgnlException {
    message: String,
}

impl OgnlException {
    /// 创建包含 OGNL detail message 的异常。
    ///
    /// # 参数
    /// - `message`：与 Java OGNL 失败对应的详细消息。
    #[must_use]
    pub fn new(message: String) -> Self {
        Self { message }
    }

    /// 返回 Java 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        "ognl.OgnlException"
    }
}

impl Display for OgnlException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for OgnlException {}
