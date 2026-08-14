//! Thymeleaf 对 Axum 的完整 Web 集成适配。
//!
//! 两个层次：
//! - [`ThymeleafView`]：渲染结果 → Axum `Response`（`IntoResponse`）。
//! - `HostWeb*` 四件套：把 Axum 请求（axum 复用 `http` crate 类型）适配为
//!   Thymeleaf `IWebExchange`/`IWebRequest`/`IWebSession`/`IWebApplication`，
//!   语义与 `thymeleaf-hyper` 标杆逐断言对齐，供 `WebContext` 与
//!   `thymeleaf-sa-token` 安全方言消费当前请求上下文。

mod host_web_request;
mod thymeleaf_error;
mod thymeleaf_view;

pub use host_web_request::HostWebRequest;
pub use thymeleaf_error::ThymeleafError;
pub use thymeleaf_view::ThymeleafView;
