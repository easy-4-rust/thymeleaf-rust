# MSRV 例外登记

依据 `docs/superpowers/specs/2026-07-28-versioning-governance.md` §4：
若某依赖强制更高 MSRV，须在本文件登记批准例外，并单独标注该 crate 的 MSRV。

## 登记项

| 日期 | crate | 例外声明 | 依据 |
| --- | --- | --- | --- |
| 2026-09-17 | `thymeleaf-topcoat` | `rust-version = "1.98"`（显式覆盖 workspace 1.95 继承） | 依赖 topcoat 自 0.7.0 起全线要求 rustc 1.98（0.6.2 及以前为 1.95，无中间版本可选）；exact-pin `=0.8.0` 随 Dependabot #17 合并。该 crate 暂未发布 crates.io，声明自其首发时生效；crates.io 的 `rust_version` 元数据与声明一致 |
| 2026-11-18 | `thymeleaf-topcoat` | `rust-version = "1.99"`（替代本 crate 的历史 1.98 声明） | 升级至 `topcoat =0.12.1`，其正式包与上游 `v0.12.1/Cargo.toml` 均声明 Rust 1.99；契约测试采用公开 `Cx::new()`。核心及其他适配器仍保持各自原有版本声明 |
