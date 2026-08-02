use std::error::Error;
use std::fmt::{Display, Formatter};

use super::TemplateEngineException;

/// 表示对象已经初始化、不能再次修改或初始化的异常。
///
/// 对应 Java: `org.thymeleaf.exceptions.AlreadyInitializedException`。
#[derive(Debug)]
pub struct AlreadyInitializedException {
    message: Option<String>,
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl AlreadyInitializedException {
    /// 使用指定消息创建异常。
    ///
    /// 对应 Java: `AlreadyInitializedException#AlreadyInitializedException(String)`。
    ///
    /// # 参数
    /// - `message`：Java 参数 `message`；`None` 对应 Java `null`。
    #[must_use]
    pub fn new(message: Option<String>) -> Self {
        Self {
            message,
            cause: None,
        }
    }

    /// 使用指定消息和原因创建异常。
    ///
    /// 对应 Java:
    /// `AlreadyInitializedException#AlreadyInitializedException(String, Throwable)`。
    ///
    /// # 参数
    /// - `message`：Java 参数 `message`；`None` 对应 Java `null`。
    /// - `cause`：Java 参数 `cause`。
    pub fn with_cause<E>(message: Option<String>, cause: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            message,
            cause: Some(Box::new(cause)),
        }
    }

    /// 返回原始异常消息；`None` 对应 Java `Throwable#getMessage()` 返回 `null`。
    #[must_use]
    pub fn get_message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl Display for AlreadyInitializedException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message.as_deref().unwrap_or("null"))
    }
}

impl Error for AlreadyInitializedException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|cause| cause as &(dyn Error + 'static))
    }
}

impl TemplateEngineException for AlreadyInitializedException {}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use super::AlreadyInitializedException;

    #[test]
    fn preserves_message_and_optional_cause() {
        let without_cause = AlreadyInitializedException::new(Some("initialized".to_owned()));
        assert_eq!(without_cause.get_message(), Some("initialized"));
        assert_eq!(without_cause.to_string(), "initialized");
        assert!(without_cause.source().is_none());

        let null_message = AlreadyInitializedException::new(None);
        assert_eq!(null_message.get_message(), None);
        assert_eq!(null_message.to_string(), "null");

        let with_cause = AlreadyInitializedException::with_cause(
            Some("initialized".to_owned()),
            io::Error::other("cause"),
        );
        assert_eq!(
            with_cause.source().map(ToString::to_string),
            Some("cause".to_owned())
        );
    }
}
