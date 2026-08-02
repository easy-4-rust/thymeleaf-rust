use std::error::Error;
use std::rc::Rc;
use std::task::{Context, Poll};

use http_body::Body;
use ntex::http::body::{BodySize, MessageBody};
use ntex::util::Bytes;
use thymeleaf::web::RenderedTemplateBody;

/// 保留 Thymeleaf 数据流背压语义的 Ntex Body。
///
/// Ntex 2 的 `MessageBody` 只公开字节块，不公开 Trailer；当前 Thymeleaf 核心
/// 不产生 Trailer，数据帧和渲染错误会按原轮询顺序转发。
pub struct ThymeleafBody {
    inner: RenderedTemplateBody,
}

impl ThymeleafBody {
    /// 包装中立渲染 Body。
    ///
    /// # 参数
    /// - `inner`：核心渲染器产生的有限或流式 Body。
    #[must_use]
    pub const fn new(inner: RenderedTemplateBody) -> Self {
        Self { inner }
    }
}

impl MessageBody for ThymeleafBody {
    fn size(&self) -> BodySize {
        self.inner
            .size_hint()
            .exact()
            .map_or(BodySize::Stream, BodySize::Sized)
    }

    fn poll_next_chunk(
        &mut self,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Rc<dyn Error>>>> {
        loop {
            match std::pin::Pin::new(&mut self.inner).poll_frame(context) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(Err(error))) => {
                    return Poll::Ready(Some(Err(Rc::new(error))));
                }
                Poll::Ready(Some(Ok(frame))) => {
                    if let Ok(bytes) = frame.into_data() {
                        return Poll::Ready(Some(Ok(Bytes::copy_from_slice(bytes.as_ref()))));
                    }
                    // Ntex 2 的 MessageBody 没有 Trailer 表示，继续寻找数据帧。
                }
            }
        }
    }
}
