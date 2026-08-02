//! Topcoat IntoResponse 对中立 HTTP 响应的无损转换契约测试。

use futures_executor::block_on;
use http_body_util::BodyExt;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use thymeleaf_topcoat::ThymeleafView;
use topcoat::context::CxTestBuilder;
use topcoat::router::IntoResponse;

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
