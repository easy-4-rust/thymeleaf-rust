use std::error::Error;
use std::fmt::{Display, Formatter};

use super::TemplateEngineException;

/// 表示模板引擎配置无效的异常。
///
/// 对应 Java: `org.thymeleaf.exceptions.ConfigurationException`。
#[derive(Debug)]
pub struct ConfigurationException {
    message: Option<String>,
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl ConfigurationException {
    /// 使用指定消息创建配置异常。
    ///
    /// 对应 Java: `ConfigurationException#ConfigurationException(String)`。
    #[must_use]
    pub fn new(message: Option<String>) -> Self {
        Self {
            message,
            cause: None,
        }
    }

    /// 使用指定消息和原因创建配置异常。
    ///
    /// 对应 Java: `ConfigurationException#ConfigurationException(String, Throwable)`。
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

impl Display for ConfigurationException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message.as_deref().unwrap_or("null"))
    }
}

impl Error for ConfigurationException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|cause| cause as &(dyn Error + 'static))
    }
}

impl TemplateEngineException for ConfigurationException {}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use super::ConfigurationException;

    #[test]
    fn preserves_all_constructor_contracts() {
        let plain = ConfigurationException::new(Some("configuration".to_owned()));
        assert_eq!(plain.get_message(), Some("configuration"));
        assert_eq!(plain.to_string(), "configuration");
        assert!(plain.source().is_none());

        let null_message = ConfigurationException::new(None);
        assert_eq!(null_message.to_string(), "null");

        let caused = ConfigurationException::with_cause(
            Some("configuration".to_owned()),
            io::Error::other("cause"),
        );
        assert_eq!(
            caused.source().map(ToString::to_string),
            Some("cause".to_owned())
        );
    }
}
