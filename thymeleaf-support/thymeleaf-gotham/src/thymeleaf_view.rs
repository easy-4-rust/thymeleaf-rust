use std::io;

use gotham::handler::IntoResponse;
use gotham::helpers::http::Body;
use gotham::state::State;
use http_body_util::BodyExt;
use thymeleaf::web::RenderedTemplate;

/// 可作为 Gotham handler 返回值的 Thymeleaf 渲染结果。
///
/// Gotham 0.8 使用 HTTP 1.x `UnsyncBoxBody`，适配过程保留 Frame、Trailer、
/// SizeHint 和下游背压，仅把渲染错误包装为标准 I/O 错误。
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
    fn into_response(self, _state: &State) -> http::Response<Body> {
        let response = self.rendered_template.into_http_response();
        response.map(|body| body.map_err(io::Error::other).boxed_unsync())
    }
}
