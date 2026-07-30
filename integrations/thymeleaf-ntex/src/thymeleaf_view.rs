use ntex::http::body::Body;
use ntex::http::header::{HeaderName, HeaderValue};
use ntex::http::{Response, StatusCode};
use ntex::web::{HttpRequest, Responder};
use thymeleaf::web::RenderedTemplate;

use crate::ThymeleafBody;

/// 可直接从 Ntex handler 返回的 Thymeleaf 渲染结果。
pub struct ThymeleafView {
    rendered_template: RenderedTemplate,
}

impl ThymeleafView {
    /// 包装框架中立渲染结果。
    ///
    /// # 参数
    /// - `rendered_template`：核心渲染器产生的响应。
    #[must_use]
    pub const fn new(rendered_template: RenderedTemplate) -> Self {
        Self { rendered_template }
    }
}

impl From<RenderedTemplate> for ThymeleafView {
    fn from(rendered_template: RenderedTemplate) -> Self {
        Self::new(rendered_template)
    }
}

impl Responder for ThymeleafView {
    async fn respond_to(self, _request: &HttpRequest) -> Response {
        let (status, headers, body) = self.rendered_template.into_parts();
        let status =
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response =
            Response::with_body(status, Body::from_message(ThymeleafBody::new(body)));
        for (name, value) in &headers {
            if let (Ok(name), Ok(value)) = (
                HeaderName::try_from(name.as_str()),
                HeaderValue::try_from(value.as_bytes()),
            ) {
                response.headers_mut().append(name, value);
            }
        }
        response
    }
}
