use std::io;

use async_std::io::BufReader;
use futures_util::TryStreamExt;
use http_body_util::BodyExt;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use tide::{Body, Response};

use crate::ThymeleafReader;

/// 可由 Tide endpoint 返回的 Thymeleaf 渲染结果。
///
/// Tide 的公共 Body 是 `AsyncBufRead`，适配器把数据帧转换为按需 Reader，
/// 不预先收集流式输出。
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

    /// 转换为 Tide 原生响应。
    ///
    /// # 返回
    /// 保留状态、Header 和数据流背压的响应。
    #[must_use]
    pub fn into_response(self) -> Response {
        let (status, headers, body) = self.rendered_template.into_parts();
        let mut response = Response::new(status.as_u16());
        for (name, value) in &headers {
            if let Ok(value) = value.to_str() {
                response.append_header(name.as_str(), value);
            }
        }
        match body {
            RenderedTemplateBody::Full(bytes) => response.set_body(bytes.to_vec()),
            RenderedTemplateBody::Stream(stream) => {
                let data = RenderedTemplateBody::Stream(stream)
                    .into_data_stream()
                    .map_err(io::Error::other);
                let reader = BufReader::new(ThymeleafReader::new(data.into_async_read()));
                response.set_body(Body::from_reader(reader, None));
            }
        }
        response
    }
}

impl From<RenderedTemplate> for ThymeleafView {
    fn from(rendered_template: RenderedTemplate) -> Self {
        Self::new(rendered_template)
    }
}

impl From<ThymeleafView> for Response {
    fn from(view: ThymeleafView) -> Self {
        view.into_response()
    }
}
