use std::sync::Arc;

use thymeleaf::web::ThymeleafRenderer;
use tonic::{Request, Status};

/// 从 Tonic Request extensions 读取 Thymeleaf 渲染器的扩展方法。
pub trait TonicRequestExt {
    /// 返回请求携带的共享 Thymeleaf 渲染器。
    ///
    /// # 错误
    /// 未安装 `ThymeleafInterceptor` 时返回 gRPC Internal 状态。
    #[allow(
        clippy::result_large_err,
        reason = "Tonic 扩展遵循框架惯例，直接返回 tonic::Status"
    )]
    fn thymeleaf_renderer(&self) -> Result<Arc<ThymeleafRenderer>, Status>;
}

impl<T> TonicRequestExt for Request<T> {
    #[allow(
        clippy::result_large_err,
        reason = "Tonic 扩展遵循框架惯例，直接返回 tonic::Status"
    )]
    fn thymeleaf_renderer(&self) -> Result<Arc<ThymeleafRenderer>, Status> {
        self.extensions()
            .get::<Arc<ThymeleafRenderer>>()
            .cloned()
            .ok_or_else(|| Status::internal("Thymeleaf renderer is unavailable"))
    }
}
