//! Tower Layer 对共享渲染器身份和请求扩展注入的契约测试。

use std::convert::Infallible;
use std::sync::Arc;

use futures_executor::block_on;
use http::Request;
use thymeleaf::web::ThymeleafRenderer;
use thymeleaf::{ITemplateEngine, TemplateEngine};
use thymeleaf_tower::ThymeleafLayer;
use tower::{Layer, ServiceExt, service_fn};

#[test]
fn layer_injects_the_same_renderer_arc_into_each_request() {
    let engine = Arc::new(TemplateEngine::new()) as Arc<dyn ITemplateEngine>;
    let renderer = Arc::new(ThymeleafRenderer::new(engine));
    let expected = Arc::clone(&renderer);
    let layer = ThymeleafLayer::new(renderer);
    let service = layer.layer(service_fn(move |request: Request<()>| {
        let expected = Arc::clone(&expected);
        async move {
            let actual = request
                .extensions()
                .get::<Arc<ThymeleafRenderer>>()
                .expect("renderer extension");
            Ok::<_, Infallible>(Arc::ptr_eq(actual, &expected))
        }
    }));

    let same_identity = block_on(service.oneshot(Request::new(()))).expect("Tower service");
    assert!(same_identity);
}
