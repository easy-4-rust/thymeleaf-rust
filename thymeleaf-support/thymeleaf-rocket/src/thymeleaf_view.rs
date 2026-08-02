use std::io;

use futures_util::TryStreamExt;
use http_body_util::BodyExt;
use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::{Request, response};
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use tokio_util::io::StreamReader;

/// 可直接从 Rocket route 返回的 Thymeleaf 渲染结果。
///
/// Rocket 的响应 Body 使用异步 Reader；流式数据帧通过 `StreamReader` 按需读取，
/// 不会预先收集模板输出。
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

impl<'r> Responder<'r, 'static> for ThymeleafView {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        let (status, headers, body) = self.rendered_template.into_parts();
        let mut builder = Response::build();
        builder.status(Status::new(status.as_u16()));
        for (name, value) in &headers {
            let Ok(value) = value.to_str() else {
                return Err(Status::InternalServerError);
            };
            builder.raw_header_adjoin(name.as_str().to_owned(), value.to_owned());
        }
        match body {
            RenderedTemplateBody::Full(bytes) => {
                let length = bytes.len();
                builder.sized_body(length, std::io::Cursor::new(bytes));
            }
            RenderedTemplateBody::Stream(stream) => {
                let body = RenderedTemplateBody::Stream(stream)
                    .into_data_stream()
                    .map_err(io::Error::other);
                builder.streamed_body(StreamReader::new(body));
            }
        }
        builder.ok()
    }
}
