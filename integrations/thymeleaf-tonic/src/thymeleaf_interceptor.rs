use std::sync::Arc;

use thymeleaf::web::ThymeleafRenderer;
use tonic::service::Interceptor;
use tonic::{Request, Status};

/// 把共享 Thymeleaf 渲染器注入 Tonic Request extensions 的拦截器。
///
/// Tonic 是 gRPC 框架而非 HTML 响应框架，因此该集成只提供请求级渲染能力，
/// 不会把 HTML Body 伪装成 gRPC message。
#[derive(Clone)]
pub struct ThymeleafInterceptor {
    renderer: Arc<ThymeleafRenderer>,
}

impl ThymeleafInterceptor {
    /// 创建渲染器注入拦截器。
    ///
    /// # 参数
    /// - `renderer`：跨 RPC 请求共享的中立渲染器。
    #[must_use]
    pub const fn new(renderer: Arc<ThymeleafRenderer>) -> Self {
        Self { renderer }
    }

    /// 返回此拦截器持有的共享渲染器。
    #[must_use]
    pub fn get_renderer(&self) -> Arc<ThymeleafRenderer> {
        Arc::clone(&self.renderer)
    }
}

impl Interceptor for ThymeleafInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request.extensions_mut().insert(Arc::clone(&self.renderer));
        Ok(request)
    }
}
