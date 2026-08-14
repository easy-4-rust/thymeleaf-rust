//! Thymeleaf 对 Gotham 的独立响应适配。
//!
//! > **实验性（experimental）**：Gotham 上游 2022 年后停滞维护。本适配器为尽力维护，
//! > 不推荐新项目采用；测试位于 `full` feature 之后（日常开发不跑，
//! > CI `--all-features` 仍全量看护）。

mod thymeleaf_view;

pub use thymeleaf_view::ThymeleafView;
