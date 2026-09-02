//! 实体展开预算管理（anti-bomb 防护）。
//!
//! 封装 oxixml-dtd 的 `ExpansionLimits` + `Budget`，为 DTD 验证器提供
//! 安全的实体展开预算（防止实体展开炸弹 / expansion bomb）。

#[cfg(feature = "dtd-validation")]
use oxixml_dtd::{Budget, ExpansionLimits};

/// 返回保守的默认展开预算。
///
/// 参数选择（基于设计 spec §6.2）：
/// - `max_entity_depth=10`：实体嵌套深度（含嵌套 `<!ENTITY %` 引用）
/// - `max_entity_expansions=1000`：展开总次数
/// - `max_expanded_bytes=1MB`：展开后总字节数
///
/// 超限返回 `DtdError::LimitExceeded`（oxixml-dtd 的错误分类）。
///
/// # 返回
/// 从默认 limits 创建的运行时 Budget（每 DTD 解析消耗一份）。
#[cfg(feature = "dtd-validation")]
pub fn default_budget() -> Budget {
    ExpansionLimits {
        max_depth: 10,
        max_expansions: 1000,
        max_expanded_size: 1_048_576,
        ..ExpansionLimits::default()
    }
    .budget()
}
