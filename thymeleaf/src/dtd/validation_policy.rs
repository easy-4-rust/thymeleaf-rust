//! DTD 验证策略（feature gate `dtd-validation`，默认 `Disabled` 零影响）。

/// XML 模式 DTD 验证策略。
#[cfg(feature = "dtd-validation")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ValidationPolicy {
    /// 不验证（零开销）。
    #[default]
    Disabled,
    /// 验证失败写 warn 日志，继续解析。
    Warn,
    /// 验证失败返回 `TemplateParserError`。
    Strict,
}
