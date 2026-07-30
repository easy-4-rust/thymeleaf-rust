use futures_util::StreamExt;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use vernal_http::{HttpBody, HttpBodyError, HttpResponse};

/// 把中立 Thymeleaf 渲染结果转换为 Vernal HTTP 协议响应。
///
/// 此对象不绑定任何具体 Vernal Web 运行时；`vernal-actix-web`、
/// `vernal-axum`、`vernal-gotham` 等均可继续消费同一个 `HttpResponse`。
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

    /// 转换为 Vernal 的框架中立 HTTP 响应。
    ///
    /// # 返回
    /// 状态码、Header、Frame、Trailer 与背压语义均得到保留的响应。
    #[must_use]
    pub fn into_http_response(self) -> HttpResponse {
        let (status, headers, body) = self.rendered_template.into_parts();
        let body = match body {
            RenderedTemplateBody::Full(bytes) => HttpBody::full(bytes),
            RenderedTemplateBody::Stream(stream) => {
                HttpBody::from_stream(stream.map(|frame| frame.map_err(HttpBodyError::transport)))
            }
        };
        let mut response = http::Response::new(body);
        *response.status_mut() = status;
        *response.headers_mut() = headers;
        HttpResponse::new(response)
    }
}

impl From<RenderedTemplate> for ThymeleafView {
    fn from(rendered_template: RenderedTemplate) -> Self {
        Self::new(rendered_template)
    }
}

impl From<ThymeleafView> for HttpResponse {
    fn from(view: ThymeleafView) -> Self {
        view.into_http_response()
    }
}
