//! Tonic 拦截器对共享渲染器注入和缺失错误的契约测试。

use std::sync::Arc;

use thymeleaf::web::ThymeleafRenderer;
use thymeleaf::{ITemplateEngine, TemplateEngine};
use thymeleaf_tonic::{ThymeleafInterceptor, TonicRequestExt};
use tonic::Request;
use tonic::service::Interceptor;

#[test]
fn interceptor_injects_same_renderer_and_extension_reports_missing_state() {
    let missing = Request::new(());
    let status = match missing.thymeleaf_renderer() {
        Ok(_) => panic!("missing interceptor must be observable"),
        Err(status) => status,
    };
    assert_eq!(status.code(), tonic::Code::Internal);

    let engine = Arc::new(TemplateEngine::new()) as Arc<dyn ITemplateEngine>;
    let renderer = Arc::new(ThymeleafRenderer::new(engine));
    let mut interceptor = ThymeleafInterceptor::new(Arc::clone(&renderer));
    let request = interceptor
        .call(Request::new(()))
        .expect("interceptor cannot reject a valid request");
    let injected = request
        .thymeleaf_renderer()
        .expect("renderer must be available after interception");
    assert!(Arc::ptr_eq(&renderer, &injected));
}
