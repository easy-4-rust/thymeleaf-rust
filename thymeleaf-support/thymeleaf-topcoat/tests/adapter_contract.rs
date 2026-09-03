//! Topcoat IntoResponse 对中立 HTTP 响应的无损转换契约测试。

use futures_executor::block_on;
use http_body_util::BodyExt;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use thymeleaf_topcoat::ThymeleafView;
use topcoat::context::CxTestBuilder;
use topcoat::router::response::IntoResponse;

#[test]
fn response_preserves_status_headers_and_full_payload() {
    let mut rendered = RenderedTemplate::new(
        Default::default(),
        RenderedTemplateBody::Full(b"topcoat".as_slice().to_vec().into()),
    )
    .with_status(http::StatusCode::ACCEPTED);
    rendered
        .get_headers_mut()
        .insert("x-template", "thymeleaf".parse().expect("valid header"));
    let context = CxTestBuilder::new().build();

    let response = ThymeleafView::new(rendered)
        .into_response(&context)
        .expect("Topcoat response");
    assert_eq!(response.status(), http::StatusCode::ACCEPTED);
    assert_eq!(response.headers()["x-template"], "thymeleaf");
    let bytes = block_on(response.into_body().collect())
        .expect("Topcoat body")
        .to_bytes();
    assert_eq!(bytes.as_ref(), b"topcoat");
}

#[tokio::test]
async fn render_async_renders_template_on_blocking_pool() {
    use std::sync::Arc;
    use thymeleaf::context::Context;
    use thymeleaf::expression::TemplateValue;
    use thymeleaf::templateresolver::StringTemplateResolver;
    use thymeleaf::util::Utf16String;
    use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver");
    let context = Context::new();
    context.set_variable(
        Some(Utf16String::from_rust_str("name")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "async-topcoat",
        )))),
    );

    let view = thymeleaf_topcoat::ThymeleafView::render_async(
        Arc::new(engine),
        "<p th:text=\"${name}\">x</p>",
        Arc::new(context),
    )
    .await
    .unwrap_or_else(|error| panic!("async render: {error}"));
    let cx = topcoat::context::CxTestBuilder::new().build();
    let response = view.into_response(&cx).expect("into response");
    let bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .expect("collect")
        .to_bytes();
    assert!(String::from_utf8_lossy(&bytes).contains("async-topcoat"));
}
