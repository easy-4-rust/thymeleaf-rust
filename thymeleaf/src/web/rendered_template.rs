use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_core::Stream;
use http::{HeaderMap, Response, StatusCode};
use http_body::{Body, Frame, SizeHint};

use super::RenderError;

/// 中立渲染 Body 的异步帧流。
pub type RenderedTemplateStream =
    Pin<Box<dyn Stream<Item = Result<Frame<Bytes>, RenderError>> + Send + 'static>>;

/// Thymeleaf 完整输出或背压流式输出。
///
/// 这是 Rust Web 整合扩展，不对应额外 Java 对象。所有框架适配 crate 只需把该
/// 中立表示转换成各自的 Body，不得重新实现模板处理。
pub enum RenderedTemplateBody {
    /// 已按响应字符集编码的完整缓冲输出。
    Full(Bytes),
    /// 按 `http-body` 数据帧输出的背压流。
    Stream(RenderedTemplateStream),
}

impl Body for RenderedTemplateBody {
    type Data = Bytes;
    type Error = RenderError;

    fn poll_frame(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            Self::Full(bytes) if bytes.is_empty() => Poll::Ready(None),
            Self::Full(bytes) => Poll::Ready(Some(Ok(Frame::data(std::mem::take(bytes))))),
            Self::Stream(stream) => stream.as_mut().poll_next(context),
        }
    }

    fn is_end_stream(&self) -> bool {
        matches!(self, Self::Full(bytes) if bytes.is_empty())
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Full(bytes) => SizeHint::with_exact(bytes.len() as u64),
            Self::Stream(_) => SizeHint::default(),
        }
    }
}

/// 可直接转换成 HTTP 响应的框架中立模板结果。
///
/// 状态码、Header 和 Body 使用 Rust HTTP 生态的中立类型，因此核心不依赖
/// Actix Web、Axum、Hyper、Poem、Rocket、Salvo 等任一宿主框架。
pub struct RenderedTemplate {
    status: StatusCode,
    headers: HeaderMap,
    body: RenderedTemplateBody,
}

impl RenderedTemplate {
    /// 创建状态为 `200 OK` 的中立渲染结果。
    #[must_use]
    pub fn new(headers: HeaderMap, body: RenderedTemplateBody) -> Self {
        Self {
            status: StatusCode::OK,
            headers,
            body,
        }
    }

    /// 返回 HTTP 状态码。
    #[must_use]
    pub const fn get_status(&self) -> StatusCode {
        self.status
    }

    /// 修改 HTTP 状态码并返回自身。
    #[must_use]
    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// 返回响应 Header。
    #[must_use]
    pub const fn get_headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// 返回可修改的响应 Header。
    #[must_use]
    pub fn get_headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// 返回渲染 Body。
    #[must_use]
    pub const fn get_body(&self) -> &RenderedTemplateBody {
        &self.body
    }

    /// 消费结果并返回状态码、Header 与 Body。
    #[must_use]
    pub fn into_parts(self) -> (StatusCode, HeaderMap, RenderedTemplateBody) {
        (self.status, self.headers, self.body)
    }

    /// 转换为 Hyper、Tower、Axum 等生态可直接消费的标准 HTTP 响应。
    #[must_use]
    pub fn into_http_response(self) -> Response<RenderedTemplateBody> {
        let mut response = Response::new(self.body);
        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;
        response
    }
}
