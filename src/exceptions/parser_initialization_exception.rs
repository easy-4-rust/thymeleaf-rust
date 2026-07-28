use std::error::Error;
use std::fmt::{Display, Formatter};

use super::TemplateEngineException;

/// 表示模板解析器初始化失败的异常。
///
/// 对应 Java: `org.thymeleaf.exceptions.ParserInitializationException`。
#[derive(Debug)]
pub struct ParserInitializationException {
    message: Option<String>,
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl ParserInitializationException {
    /// 使用指定消息创建解析器初始化异常。
    ///
    /// 对应 Java:
    /// `ParserInitializationException#ParserInitializationException(String)`。
    #[must_use]
    pub fn new(message: Option<String>) -> Self {
        Self {
            message,
            cause: None,
        }
    }

    /// 使用指定消息和原因创建解析器初始化异常。
    ///
    /// 对应 Java:
    /// `ParserInitializationException#ParserInitializationException(String, Throwable)`。
    pub fn with_cause<E>(message: Option<String>, cause: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            message,
            cause: Some(Box::new(cause)),
        }
    }

    /// 返回原始异常消息；`None` 对应 Java `null`。
    #[must_use]
    pub fn get_message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl Display for ParserInitializationException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message.as_deref().unwrap_or("null"))
    }
}

impl Error for ParserInitializationException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|cause| cause as &(dyn Error + 'static))
    }
}

impl TemplateEngineException for ParserInitializationException {}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use super::ParserInitializationException;

    #[test]
    fn preserves_all_constructor_contracts() {
        let plain = ParserInitializationException::new(Some("parser".to_owned()));
        assert_eq!(plain.get_message(), Some("parser"));
        assert_eq!(plain.to_string(), "parser");
        assert!(plain.source().is_none());

        let null_message = ParserInitializationException::new(None);
        assert_eq!(null_message.to_string(), "null");

        let caused = ParserInitializationException::with_cause(
            Some("parser".to_owned()),
            io::Error::other("cause"),
        );
        assert_eq!(
            caused.source().map(ToString::to_string),
            Some("cause".to_owned())
        );
    }
}
