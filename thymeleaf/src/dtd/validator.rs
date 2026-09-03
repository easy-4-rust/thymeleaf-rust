//! DTD 验证器封装——对外简单接口，内部驱动 oxixml-dtd 的 push 接口。
//!
//! `DtdValidator` 负责按 SYSTEM 标识符解析 XHTML DTD（内嵌解析器 +
//! 实体展开预算）；文档事件校验由有状态的 `Validator` 在 `parse_xml`
//! 主循环中驱动：`start_element` / `characters` / `reference_data` /
//! `markup` / `end_element`，最后 `finish` 汇总 IDREF 悬空等收尾违反。

#[cfg(feature = "dtd-validation")]
use oxixml_dtd::{Dtd, DtdParser, ValidationOptions};

#[cfg(feature = "dtd-validation")]
pub use oxixml_dtd::{Validator, ValidityError};

#[cfg(feature = "dtd-validation")]
use super::embedded_dtd::build_xhtml_resolver;
#[cfg(feature = "dtd-validation")]
use super::entity_budget::default_budget;

/// 封装已解析的 DTD（对应 W3C XHTML 四族 SYSTEM 标识符）。
///
/// 未知 SYSTEM 标识符或 DTD 解析/实体展开超限 → `DtdValidator::new`
/// 返回 `None`（调用方按策略处理：Strict 报错、Warn 降级为不验证）。
///
/// 对应 Java 语义：Rust 侧扩展封装（Java 上游无内建 DTD 验证器）。
#[cfg(feature = "dtd-validation")]
pub struct DtdValidator {
    dtd: Dtd,
}

#[cfg(feature = "dtd-validation")]
impl DtdValidator {
    /// 从 DOCTYPE 声明主体构建 DTD 验证器。
    ///
    /// # 参数
    /// - `declaration`：`<!DOCTYPE` 与匹配 `>` 之间的声明主体
    ///   （如 `"html SYSTEM \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd\""`）。
    ///   外部标识符（含 W3C XHTML 四族 SYSTEM 标识符）由内嵌 resolver 解析；
    ///   仅含内部子集的声明按内部子集 DTD 验证。
    ///
    /// # 返回
    /// 解析成功（含实体展开预算内）→ `Some(Self)`；否则 → `None`。
    ///
    /// 对应 Java 语义：Rust 侧扩展构造路径（Java 无直接对应）。
    #[must_use]
    pub fn new(declaration: &str) -> Option<Self> {
        let resolver = build_xhtml_resolver();
        let dtd = DtdParser::new()
            .with_resolver(Box::new(resolver))
            .with_limits(default_budget())
            .parse_doctype(declaration)
            .ok()?;
        Some(Self { dtd })
    }

    /// 创建驱动文档事件的有状态验证器。
    ///
    /// 返回值借用内部 DTD，因此 `DtdValidator` 本体在验证期间不可移动。
    /// 对应 Java 语义：Rust 侧扩展方法（Java 无直接对应）。
    #[must_use]
    pub fn validator(&self) -> Validator<'_> {
        Validator::new(&self.dtd, ValidationOptions::default())
    }
}
