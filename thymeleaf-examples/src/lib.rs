//! thymeleaf-examples —— Java `examples/core`（GTVG 示例）的 Rust 移植。
//!
//! 布局对应：
//! - `templates/`：8 个 GTVG 模板 1:1 复制自上游 webapp（含 order/、product/ 子目录与 i18n properties）
//! - `src/`：业务层（实体/仓库/服务）+ 控制器渲染函数 1:1 移植
//! - `examples/gtvg.rs`：引擎级装配，渲染全部 8 个页面
//!
//! 本 crate 不发布（`publish = false`），不绑定任何 web 集成框架。

#![forbid(unsafe_code)]

pub mod business;
pub mod controllers;
