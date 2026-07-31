use std::fmt::{Display, Formatter};

use crate::TemplateInputException;

/// 模板资源操作的错误类别。
///
/// 对应 Java: `ITemplateResource` 各实现可能抛出的 `IllegalArgumentException`、
/// `MalformedURLException` 和 `TemplateInputException`。这是 Rust 类型化错误扩展，
/// 不计入 Thymeleaf Java 对象迁移分子。
#[derive(Debug)]
pub enum TemplateResourceError {
    /// 参数违反具体资源实现的前置条件。
    InvalidArgument(String),
    /// URL 文本无法构造成资源地址。
    MalformedUrl {
        /// 无法解析的原始 URL 位置。
        location: String,
        /// Rust URL 解析器报告的底层原因。
        source: url::ParseError,
    },
    /// 模板输入资源无法创建或定位。
    Input(TemplateInputException),
}

impl Display for TemplateResourceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument(message) => formatter.write_str(message),
            Self::MalformedUrl { location, source } => {
                write!(formatter, "Malformed URL \"{location}\": {source}")
            }
            Self::Input(error) => Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for TemplateResourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidArgument(_) => None,
            Self::MalformedUrl { source, .. } => Some(source),
            Self::Input(error) => error.source(),
        }
    }
}
