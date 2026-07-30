use thymeleaf::web::RenderedTemplate;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{Body, IntoResponse, Response};

/// 可直接从 Topcoat route 返回的 Thymeleaf 渲染结果。
///
/// 该适配器使用 Topcoat 的公开 `http_body` 包装入口，保留状态码、Header、
/// 数据帧、背压和取消传播。
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
    fn into_response(self, _context: &Cx) -> Result<Response> {
        let (status, headers, body) = self.rendered_template.into_parts();
        let mut response = Response::new(Body::new(body));
        *response.status_mut() = status;
        *response.headers_mut() = headers;
        Ok(response)
    }
}
