//! Thymeleaf 对 Warp 的独立 Reply 适配。
//!
//! > **实验性（experimental）**：Warp 被 axum 取代中。本适配器为尽力维护，
//! > 不推荐新项目采用；测试位于 `full` feature 之后（日常开发不跑，
//! > CI `--all-features` 仍全量看护）。

mod thymeleaf_reply;
mod thymeleaf_reply_error;

pub use thymeleaf_reply::ThymeleafReply;
pub use thymeleaf_reply_error::ThymeleafReplyError;
