use http::HeaderMap;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use warp::Reply;
use warp::http::StatusCode;
use warp::reply::Response;

use crate::ThymeleafReplyError;

/// Warp `Reply` 可表示的有限 Thymeleaf 渲染结果。
///
/// 构造过程显式区分有限与流式 Body，绝不为了满足 Warp 私有 Body 类型而收集流。
pub struct ThymeleafReply {
    status: StatusCode,
    headers: HeaderMap,
    body: bytes::Bytes,
}

impl TryFrom<RenderedTemplate> for ThymeleafReply {
    type Error = ThymeleafReplyError;

    fn try_from(rendered_template: RenderedTemplate) -> Result<Self, Self::Error> {
        let (status, headers, body) = rendered_template.into_parts();
        let RenderedTemplateBody::Full(body) = body else {
            return Err(ThymeleafReplyError::StreamingUnsupported);
        };
        Ok(Self {
            status,
            headers,
            body,
        })
    }
}

impl Reply for ThymeleafReply {
    fn into_response(self) -> Response {
        let mut response = Response::new(self.body.into());
        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;
        response
    }
}
