use axum::body::Body;
use axum::http::Response;
use axum::response::IntoResponse;
use thymeleaf::web::RenderedTemplate;

/// 可作为 Axum handler 返回值的 Thymeleaf 渲染结果。
///
/// 对应 Thymeleaf Web 集成中的视图响应职责。
pub struct ThymeleafView {
    rendered_template: RenderedTemplate,
}

impl ThymeleafView {
    /// 包装框架中立渲染结果。
    ///
    /// # 参数
    /// - `rendered_template`：核心渲染器产生的状态、Header 与 Body。
    #[must_use]
    pub const fn new(rendered_template: RenderedTemplate) -> Self {
        Self { rendered_template }
    }

    /// 消费适配器并返回中立结果。
    ///
    /// # 返回
    /// 未丢失状态、Header 或流式 Body 的 `RenderedTemplate`。
    #[must_use]
    pub fn into_rendered_template(self) -> RenderedTemplate {
        self.rendered_template
    }
}

impl From<RenderedTemplate> for ThymeleafView {
    fn from(rendered_template: RenderedTemplate) -> Self {
        Self::new(rendered_template)
    }
}

impl IntoResponse for ThymeleafView {
    fn into_response(self) -> Response<Body> {
        let (status, headers, body) = self.rendered_template.into_parts();
        let mut response = Response::new(Body::new(body));
        *response.status_mut() = status;
        *response.headers_mut() = headers;
        response
    }
}
