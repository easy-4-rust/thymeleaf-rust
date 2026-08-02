use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::io::AsyncRead;
use sync_wrapper::SyncWrapper;

/// 以独占轮询证明 `Sync` 的 Tide 异步 Body Reader。
///
/// Tide 的公共 `Body::from_reader` 要求 Reader 同时为 `Send + Sync`，而模板帧流
/// 只需要 `Send`。`SyncWrapper` 不加锁，只利用每次轮询都持有独占 `&mut self`
/// 这一事实安全转发读取。
pub struct ThymeleafReader {
    inner: SyncWrapper<Box<dyn AsyncRead + Unpin + Send + 'static>>,
}

impl ThymeleafReader {
    /// 包装只要求 `Send` 的异步 Reader。
    ///
    /// # 参数
    /// - `inner`：由 Thymeleaf 数据帧流构造的 Reader。
    #[must_use]
    pub fn new(inner: impl AsyncRead + Unpin + Send + 'static) -> Self {
        Self {
            inner: SyncWrapper::new(Box::new(inner)),
        }
    }
}

impl AsyncRead for ThymeleafReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.as_mut().get_mut();
        Pin::new(this.inner.get_mut().as_mut()).poll_read(context, buffer)
    }
}
