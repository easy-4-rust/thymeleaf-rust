//! Thymeleaf 对 Actix Web 的完整 Web 集成适配。
//!
//! 两个层次：
//! - [`ThymeleafView`]：渲染结果 → Actix `HttpResponse`（`Responder`），
//!   [`ThymeleafBody`] 保留流式背压语义。
//! - `HostWeb*` 四件套：把 Actix 请求适配为 Thymeleaf
//!   `IWebExchange`/`IWebRequest`/`IWebSession`/`IWebApplication`，
//!   语义与 `thymeleaf-hyper` 标杆逐断言对齐，供 `WebContext` 与
//!   `thymeleaf-sa-token` 安全方言消费当前请求上下文。

mod host_web_application;
mod host_web_exchange;
mod host_web_request;
mod host_web_session;
mod thymeleaf_body;
mod thymeleaf_view;

pub use host_web_application::HostWebApplication;
pub use host_web_exchange::HostWebExchange;
pub use host_web_request::HostWebRequest;
pub use host_web_session::HostWebSession;
pub use thymeleaf_body::ThymeleafBody;
pub use thymeleaf_view::ThymeleafView;
