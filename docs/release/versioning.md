# MSRV 例外登记

依据 `docs/superpowers/specs/2026-07-28-versioning-governance.md` §4：
若某依赖强制更高 MSRV，须在本文件登记批准例外，并单独标注该 crate 的 MSRV。

## 登记项

| 日期 | crate | 例外声明 | 依据 |
| --- | --- | --- | --- |
| 2026-09-17 | `thymeleaf-topcoat` | `rust-version = "1.98"`（显式覆盖 workspace 1.95 继承） | 依赖 topcoat 自 0.7.0 起全线要求 rustc 1.98（0.6.2 及以前为 1.95，无中间版本可选）；exact-pin `=0.8.0` 随 Dependabot #17 合并。该 crate 暂未发布 crates.io，声明自其首发时生效；crates.io 的 `rust_version` 元数据与声明一致 |
