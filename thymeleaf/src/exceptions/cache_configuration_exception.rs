use std::error::Error;
use std::fmt::{Display, Formatter};

use super::{ConfigurationException, TemplateEngineException};

/// 表示模板缓存配置无效的异常。
///
/// 对应 Java: `org.thymeleaf.exceptions.CacheConfigurationException`。
/// Java 类型继承 `ConfigurationException`，Rust 通过相同消息、原因链和
/// `TemplateEngineException` 契约保留可观察语义。
#[derive(Debug)]
pub struct CacheConfigurationException {
    configuration: ConfigurationException,
}

impl CacheConfigurationException {
    /// 使用指定消息创建缓存配置异常。
    ///
    /// 对应 Java:
    /// `CacheConfigurationException#CacheConfigurationException(String)`。
    #[must_use]
    pub fn new(message: Option<String>) -> Self {
        Self {
            configuration: ConfigurationException::new(message),
        }
    }

    /// 使用指定消息和原因创建缓存配置异常。
    ///
    /// 对应 Java:
    /// `CacheConfigurationException#CacheConfigurationException(String, Throwable)`。
    pub fn with_cause<E>(message: Option<String>, cause: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            configuration: ConfigurationException::with_cause(message, cause),
        }
    }

    /// 返回原始异常消息；`None` 对应 Java `null`。
    #[must_use]
    pub fn get_message(&self) -> Option<&str> {
        self.configuration.get_message()
    }
}

impl Display for CacheConfigurationException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.configuration, formatter)
    }
}

impl Error for CacheConfigurationException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.configuration.source()
    }
}

impl TemplateEngineException for CacheConfigurationException {}

impl AsRef<ConfigurationException> for CacheConfigurationException {
    fn as_ref(&self) -> &ConfigurationException {
        &self.configuration
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use super::CacheConfigurationException;

    #[test]
    fn preserves_all_constructor_contracts() {
        let plain = CacheConfigurationException::new(Some("cache".to_owned()));
        assert_eq!(plain.get_message(), Some("cache"));
        assert_eq!(plain.to_string(), "cache");
        assert!(plain.source().is_none());

        let null_message = CacheConfigurationException::new(None);
        assert_eq!(null_message.to_string(), "null");

        let caused = CacheConfigurationException::with_cause(
            Some("cache".to_owned()),
            io::Error::other("cause"),
        );
        assert_eq!(caused.as_ref().get_message(), Some("cache"));
        assert_eq!(
            caused.source().map(ToString::to_string),
            Some("cause".to_owned())
        );
    }
}
