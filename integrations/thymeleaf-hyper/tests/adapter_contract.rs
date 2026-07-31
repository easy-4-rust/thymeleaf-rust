//! Hyper 适配器对标准 HTTP 响应的无损转换契约测试。

use http_body::Body;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};

#[test]
fn response_preserves_status_headers_and_exact_body_size() {
    let mut rendered = RenderedTemplate::new(
        Default::default(),
        RenderedTemplateBody::Full(b"hyper".as_slice().to_vec().into()),
    )
    .with_status(hyper::StatusCode::PARTIAL_CONTENT);
    rendered
        .get_headers_mut()
        .insert("x-template", "thymeleaf".parse().expect("valid header"));

    let response = thymeleaf_hyper::into_response(rendered);
    assert_eq!(response.status(), hyper::StatusCode::PARTIAL_CONTENT);
    assert_eq!(response.headers()["x-template"], "thymeleaf");
    assert_eq!(response.body().size_hint().exact(), Some(5));
}
