//! thymeleaf-examples —— Java `examples/core`（GTVG 示例）的 Rust 移植。
//!
//! 对应关系：
//! - `templates/`：8 个 GTVG 模板 + i18n properties + css/images 1:1 复制自上游
//!   `thymeleaf-examples-gtvg-jakarta` 的 webapp（字节级一致，`diff -r` 验证）
//! - `src/business/`：实体 / 仓库（含全部种子数据）/ 服务 / CalendarUtil 1:1 移植
//! - `src/controllers/`：8 个控制器 + URL 映射（对应 `IGTVGController` +
//!   `ControllerMappings`，渲染函数形态 `process(exchange, engine, now)`）
//! - `src/web/`：`GTVGFilter` 的宿主角色（请求 / 会话 / 应用 / exchange）
//! - `examples/gtvg.rs`：引擎装配 + 过滤器流程模拟，渲染全部 7 个 URL
//!
//! 引擎侧无需定制：`StandardMessageResolver` 默认按模板名读取并列的
//! `.properties`（对应 Java 默认消息解析），`StandardLinkBuilder` 处理
//! `@{...}` 链接表达式。
//!
//! 本 crate 不发布（`publish = false`），不绑定任何 web 集成框架。

#![forbid(unsafe_code)]

pub mod business;
pub mod controllers;
pub mod web;
