use futures_util::StreamExt;
use salvo::Scribe;
use salvo::http::body::{BytesFrame, ResBody};
use salvo::prelude::Response;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};

/// 可由 Salvo handler 直接写入响应的 Thymeleaf 渲染结果。
///
/// 流式变体逐帧转交 Salvo `ResBody`，保留 Trailer、背压和渲染错误。
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

impl Scribe for ThymeleafView {
    fn render(self, response: &mut Response) {
        let (status, headers, body) = self.rendered_template.into_parts();
        response.status_code(status);
        *response.headers_mut() = headers;
        *response.body_mut() = match body {
            RenderedTemplateBody::Full(bytes) => ResBody::Once(bytes),
            RenderedTemplateBody::Stream(stream) => {
                ResBody::stream(stream.map(|frame| frame.map(BytesFrame)))
            }
        };
    }
}
