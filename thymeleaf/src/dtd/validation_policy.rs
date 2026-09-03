//! DTD 验证策略（feature gate `dtd-validation`，默认 `Disabled` 零影响）。

/// XML 模式 DTD 验证策略。
/// 对应 Java 语义：Rust 侧扩展配置项（Java 上游无内建 DTD 验证，由
/// 容器/JAXP 外部承担；Rust 侧经 feature gate 以三策略显式建模）。
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
