use std::io;

use futures_util::TryStreamExt;
use http_body_util::BodyExt;
use poem::{Body, IntoResponse, Response};
use thymeleaf::web::RenderedTemplate;

/// 可直接从 Poem endpoint 返回的 Thymeleaf 渲染结果。
///
/// Poem 3 的公开 Body 流入口只接受数据帧，因此当前核心产生的数据帧会保持背压
/// 转发；若核心未来产生 HTTP Trailer，需要随 Poem 公开 API 的演进补充 Trailer 映射。
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

impl IntoResponse for ThymeleafView {
    fn into_response(self) -> Response {
        let (status, headers, body) = self.rendered_template.into_parts();
        let stream = body.into_data_stream().map_err(io::Error::other);
        let mut response = Response::from(Body::from_bytes_stream(stream));
        response.set_status(status);
        *response.headers_mut() = headers;
        response
    }
}
