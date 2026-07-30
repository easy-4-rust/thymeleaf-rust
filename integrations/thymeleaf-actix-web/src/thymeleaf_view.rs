use actix_web::http::StatusCode;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{HttpRequest, HttpResponse, Responder};
use thymeleaf::web::RenderedTemplate;

use crate::ThymeleafBody;

/// 可直接从 Actix Web handler 返回的 Thymeleaf 视图。
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
    type Body = ThymeleafBody;

    fn respond_to(self, _request: &HttpRequest) -> HttpResponse<Self::Body> {
        let (status, headers, body) = self.rendered_template.into_parts();
        let status =
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = HttpResponse::with_body(status, ThymeleafBody::new(body));
        for (name, value) in &headers {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(name.as_str().as_bytes()),
                HeaderValue::from_bytes(value.as_bytes()),
            ) {
                response.headers_mut().append(name, value);
            }
        }
        response
    }
}
