//! Thymeleaf 作为 Vernal 动态内容渲染框架的协议适配。
//!
//! 两个层次：
//! - [`ThymeleafView`]：渲染结果 → Vernal `HttpResponse`（框架中立协议转换）。
//! - [`VernalWebExchange`] / [`VernalWebRequest`]：把 Vernal 请求上下文
//!   （`RequestContext` + `HttpRequestSnapshot` + `SecurityPrincipal`）适配为
//!   Thymeleaf `IWebExchange`，供 `WebContext` 与 `thymeleaf-sa-token` 安全方言
//!   消费当前请求的认证身份。

mod thymeleaf_view;
mod web_exchange;
mod web_request;

pub use thymeleaf_view::ThymeleafView;
pub use web_exchange::{VernalWebApplication, VernalWebExchange, VernalWebSession};
pub use web_request::VernalWebRequest;
