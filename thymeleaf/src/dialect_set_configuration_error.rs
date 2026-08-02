use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::exceptions::ConfigurationException;
use crate::util::ValidateError;

/// 构建 `DialectSetConfiguration` 时可能出现的 Java 异常联合。
///
/// Java 的静态 `build` 入口既可能由 `Validate.notNull` 抛出
/// `IllegalArgumentException`，也可能在聚合方言贡献时抛出
/// `ConfigurationException`。Rust 使用显式联合保留异常类别。
#[derive(Debug)]
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
pub enum DialectSetConfigurationError {
    /// Java 参数前置条件失败。
    IllegalArgument(ValidateError),
    /// 方言贡献内容冲突或非法。
    Configuration(ConfigurationException),
}

impl DialectSetConfigurationError {
    /// 返回对应 Java 异常的完整类名。
    ///
    /// 返回 `IllegalArgumentException` 或 Thymeleaf `ConfigurationException` 的限定名。
    #[must_use]
    pub fn java_class_name(&self) -> &'static str {
        match self {
            Self::IllegalArgument(error) => error.java_class_name(),
            Self::Configuration(_) => "org.thymeleaf.exceptions.ConfigurationException",
        }
    }

    /// 转换为 EngineConfiguration 内部统一使用的配置异常。
    ///
    /// 正常内部调用始终传入非空集合；该转换仍保留防御性错误消息。
    ///
    /// 返回保留原配置异常或以前置条件异常为 cause 的 `ConfigurationException`。
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn into_configuration_exception(self) -> ConfigurationException {
        match self {
            Self::Configuration(error) => error,
            Self::IllegalArgument(error) => {
                ConfigurationException::with_cause(Some(error.to_string()), error)
            }
        }
    }
}

impl Display for DialectSetConfigurationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalArgument(error) => Display::fmt(error, formatter),
            Self::Configuration(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for DialectSetConfigurationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IllegalArgument(error) => Some(error),
            Self::Configuration(error) => Some(error),
        }
    }
}

impl From<ValidateError> for DialectSetConfigurationError {
    fn from(error: ValidateError) -> Self {
        Self::IllegalArgument(error)
    }
}

impl From<ConfigurationException> for DialectSetConfigurationError {
    fn from(error: ConfigurationException) -> Self {
        Self::Configuration(error)
    }
}
