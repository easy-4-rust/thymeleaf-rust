use std::pin::Pin;
use std::task::{Context, Poll};

use actix_web::body::{BodySize, MessageBody};
use actix_web::web::Bytes;
use http_body::Body;
use thymeleaf::web::{RenderError, RenderedTemplateBody};

/// 保留 Thymeleaf 流式背压语义的 Actix Web Body。
pub struct ThymeleafBody {
    inner: RenderedTemplateBody,
}

impl ThymeleafBody {
    /// 包装中立渲染 Body。
    ///
    /// # 参数
    /// - `inner`：核心渲染器产生的流式 Body。
    #[must_use]
    pub const fn new(inner: RenderedTemplateBody) -> Self {
        Self { inner }
    }
}

impl MessageBody for ThymeleafBody {
    type Error = RenderError;

    fn size(&self) -> BodySize {
        let hint = self.inner.size_hint();
        hint.exact().map_or(BodySize::Stream, BodySize::Sized)
    }

    fn poll_next(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        loop {
            match Pin::new(&mut self.inner).poll_frame(context) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(Err(error))) => return Poll::Ready(Some(Err(error))),
                Poll::Ready(Some(Ok(frame))) => {
                    if let Ok(bytes) = frame.into_data() {
                        return Poll::Ready(Some(Ok(bytes)));
                    }
                    // 当前核心不会产生 trailer；如果将来新增，继续读取下一个 data frame。
                }
            }
        }
    }
}
