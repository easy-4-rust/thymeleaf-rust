//! 实体展开预算管理（anti-bomb 防护）。
//!
//! 封装 oxixml-dtd 的 `ExpansionLimits`，为 DTD 解析提供
//! 安全的实体展开预算（防止实体展开炸弹 / expansion bomb）。

#[cfg(feature = "dtd-validation")]
use oxixml_dtd::ExpansionLimits;

/// 返回默认展开限制（喂给 `DtdParser::with_limits`）。
///
/// 参数取库默认档（`ExpansionLimits::DEFAULT`）：
/// - `max_depth=40`：实体嵌套深度
/// - `max_expansions=10_000`：展开总次数
/// - `max_expanded_size=10MiB`：展开后总字节数
///
/// 预算只作用于内嵌可信 DTD 集（resolver 对未知 SYSTEM ID 返回
/// 未找到，解析随即失败，外部 DTD 根本进不来），因此取值以
/// "最大族 xhtml11 完整展开"为准下限——实测 xhtml11 单体在
/// xhtml-table.mod 处即超过 1,000 次展开，1MB 预算会误杀。
///
/// 超限返回 `DtdError::LimitExceeded`（oxixml-dtd 的错误分类）。
///
/// 对应 Java 语义：Rust 侧扩展（Java 容器默认不限制实体展开；
/// Rust 侧按安全基线显式设防，见 docs/release/security.md）。
#[cfg(feature = "dtd-validation")]
#[must_use]
pub const fn default_budget() -> ExpansionLimits {
    ExpansionLimits::DEFAULT
}
