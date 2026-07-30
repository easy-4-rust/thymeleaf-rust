use std::sync::Arc;

use thymeleaf::web::ThymeleafRenderer;
use tower::Layer;

use crate::ThymeleafService;

/// 把共享 `ThymeleafRenderer` 注入请求 extensions 的 Tower Layer。
#[derive(Clone)]
pub struct ThymeleafLayer {
    renderer: Arc<ThymeleafRenderer>,
}

impl ThymeleafLayer {
    /// 创建渲染器注入层。
    ///
    /// # 参数
    /// - `renderer`：跨请求共享的中立渲染器。
    #[must_use]
    pub const fn new(renderer: Arc<ThymeleafRenderer>) -> Self {
        Self { renderer }
    }

    /// 返回此 Layer 持有的共享渲染器。
    ///
    /// # 返回
    /// 保持相同对象身份的 `Arc`。
    #[must_use]
    pub fn get_renderer(&self) -> Arc<ThymeleafRenderer> {
        Arc::clone(&self.renderer)
    }
}

impl<S> Layer<S> for ThymeleafLayer {
    type Service = ThymeleafService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ThymeleafService::new(inner, Arc::clone(&self.renderer))
    }
}
