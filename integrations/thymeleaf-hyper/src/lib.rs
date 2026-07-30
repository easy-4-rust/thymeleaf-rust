//! Thymeleaf 对 Hyper 的独立薄适配。

mod host_web_application;
mod host_web_exchange;
mod host_web_request;
mod host_web_session;

use hyper::Response;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};

pub use host_web_application::HostWebApplication;
pub use host_web_exchange::HostWebExchange;
pub use host_web_request::HostWebRequest;
pub use host_web_session::HostWebSession;

/// 将中立模板结果转换为 Hyper 原生响应。
///
/// 状态码、Header、完整 Body 和流式 Body 均原样保留；背压与取消仍由
/// `RenderedTemplateBody` 的 `http_body::Body` 实现负责。
#[must_use]
pub fn into_response(rendered_template: RenderedTemplate) -> Response<RenderedTemplateBody> {
    rendered_template.into_http_response()
}
