//! thymeleaf-rust 全项目验收包。
//!
//! 本包是 `thymeleaf-rust` 的 whole-project acceptance authority（对应
//! rust-java-migration-testing 技能的 `<project>-test` 要求）：
//!
//! - 拥有完整上游源码套件重放与差分验收命令与产物；
//! - 校验固定上游基线、`.thtest` 资产字节级副本（SHA-256）和
//!   逐 case 差分结果 `MATCH`；
//! - `publish = false`，不进入 crates.io 发布面。
//!
//! 生产 crate（`thymeleaf`）内部测试只证明子系统本地行为，不替代本包的
//! 全项目源码套件重放与差分结论。

/// 固定上游基线 SHA（Thymeleaf 3.1.5.RELEASE）。
pub const UPSTREAM_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
