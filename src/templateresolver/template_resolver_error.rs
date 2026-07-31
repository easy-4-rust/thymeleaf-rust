use std::fmt::{Display, Formatter};

use super::TemplateResolutionError;
use crate::TemplateResourceError;

/// 模板解析器在构造资源或解析结果时产生的错误。
///
/// Java Resolver 会直接传播资源构造器的 `IllegalArgumentException`、
/// `TemplateInputException` 等运行时异常。Rust 使用显式错误保持“不适用”
/// 与“解析失败”两种结果互不混淆。
///
/// 对应 Java: `org.thymeleaf.templateresolver.ITemplateResolver#resolveTemplate`。
/// 这是 Rust 类型化错误扩展，不计入 Thymeleaf Java 对象迁移分子。
#[derive(Debug)]
pub enum TemplateResolverError {
    /// Resolver 配置或构造参数违反 Java 前置条件。
    InvalidArgument(String),
    /// 模板资源构造失败。
    Resource(TemplateResourceError),
    /// 模板解析结果构造失败。
    Resolution(TemplateResolutionError),
}

impl Display for TemplateResolverError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument(message) => formatter.write_str(message),
            Self::Resource(error) => Display::fmt(error, formatter),
            Self::Resolution(error) => Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for TemplateResolverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidArgument(_) => None,
            Self::Resource(error) => Some(error),
            Self::Resolution(error) => Some(error),
        }
    }
}

impl From<TemplateResourceError> for TemplateResolverError {
    fn from(error: TemplateResourceError) -> Self {
        Self::Resource(error)
    }
}

impl From<TemplateResolutionError> for TemplateResolverError {
    fn from(error: TemplateResolutionError) -> Self {
        Self::Resolution(error)
    }
}
