use std::sync::Arc;
use std::task::{Context, Poll};

use http::Request;
use thymeleaf::web::ThymeleafRenderer;
use tower::Service;

/// 在调用下游 Service 前注入共享 Thymeleaf 渲染器。
#[derive(Clone)]
pub struct ThymeleafService<S> {
    inner: S,
    renderer: Arc<ThymeleafRenderer>,
}

impl<S> ThymeleafService<S> {
    /// 创建渲染器注入服务。
    ///
    /// # 参数
    /// - `inner`：下游 Tower Service；
    /// - `renderer`：写入每个请求 extensions 的共享渲染器。
    #[must_use]
    pub const fn new(inner: S, renderer: Arc<ThymeleafRenderer>) -> Self {
        Self { inner, renderer }
    }

    /// 返回下游 Service 的只读引用。
    ///
    /// # 返回
    /// 当前包装的下游服务。
    #[must_use]
    pub const fn get_inner(&self) -> &S {
        &self.inner
    }
}

impl<S, B> Service<Request<B>> for ThymeleafService<S>
where
    S: Service<Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(context)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        request.extensions_mut().insert(Arc::clone(&self.renderer));
        self.inner.call(request)
    }
}
