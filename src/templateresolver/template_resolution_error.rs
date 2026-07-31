use thiserror::Error;

/// 创建模板解析结果时的参数校验错误。
///
/// 对应 Java: `org.thymeleaf.util.Validate` 在
/// `org.thymeleaf.templateresolver.TemplateResolution` 构造器中抛出的
/// `IllegalArgumentException`。
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TemplateResolutionError {
    /// 必填参数为 Java `null`。
    #[error("{0}")]
    InvalidArgument(&'static str),
}
