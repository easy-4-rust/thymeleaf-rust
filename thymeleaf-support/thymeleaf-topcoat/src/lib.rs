//! Thymeleaf 对 Topcoat 的完整 Web 集成适配。
//!
//! 两个层次：
//! - [`ThymeleafView`]：渲染结果 → Topcoat `Response`（`IntoResponse`，
//!   带 `Cx` 上下文参数的 topcoat 特色签名）。
//! - `HostWeb*` 四件套：把 Topcoat 请求（`topcoat::router::Request` 即
//!   `http::Request`）适配为 Thymeleaf
//!   `IWebExchange`/`IWebRequest`/`IWebSession`/`IWebApplication`，
//!   语义与 `thymeleaf-hyper`/`thymeleaf-axum` 标杆逐断言对齐。
//!
//! Topcoat 的响应式 view（`topcoat_view` 宏）与 Thymeleaf SSR 互补：
//! route 返回 [`ThymeleafView`] 即整页服务端渲染。

mod host_web_application;
mod host_web_exchange;
mod host_web_request;
mod host_web_session;
mod thymeleaf_view;

pub use host_web_application::HostWebApplication;
pub use host_web_exchange::HostWebExchange;
pub use host_web_request::HostWebRequest;
pub use host_web_session::HostWebSession;
pub use thymeleaf_view::ThymeleafView;
