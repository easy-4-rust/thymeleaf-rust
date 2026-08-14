//! Thymeleaf 对 Tide 的独立响应适配。
//!
//! > **实验性（experimental）**：Tide 0.17.0-beta 维护停滞。本适配器为尽力维护，
//! > 不推荐新项目采用；测试位于 `full` feature 之后（日常开发不跑，
//! > CI `--all-features` 仍全量看护）。

mod thymeleaf_reader;
mod thymeleaf_view;

pub use thymeleaf_reader::ThymeleafReader;
pub use thymeleaf_view::ThymeleafView;
