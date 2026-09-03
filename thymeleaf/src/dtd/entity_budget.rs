//! 实体展开预算管理（anti-bomb 防护）。
//!
//! 封装 oxixml-dtd 的 `ExpansionLimits`，为 DTD 解析提供
//! 安全的实体展开预算（防止实体展开炸弹 / expansion bomb）。

#[cfg(feature = "dtd-validation")]
use oxixml_dtd::ExpansionLimits;

/// 返回保守的默认展开限制（喂给 `DtdParser::with_limits`）。
///
/// 参数选择（保守值，低于库默认）：
/// - `max_depth=10`：实体嵌套深度（含嵌套 `<!ENTITY %` 引用）
/// - `max_expansions=1000`：展开总次数
/// - `max_expanded_size=1MB`：展开后总字节数
///
/// 超限返回 `DtdError::LimitExceeded`（oxixml-dtd 的错误分类）。
#[cfg(feature = "dtd-validation")]
#[must_use]
pub fn default_budget() -> ExpansionLimits {
    ExpansionLimits {
        max_depth: 10,
        max_expansions: 1000,
        max_expanded_size: 1_048_576,
    }
}
