//! Poem 适配器对状态、Header 与 Body 的中立响应契约测试。

use futures_executor::block_on;
use poem::IntoResponse;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use thymeleaf_poem::ThymeleafView;

#[test]
fn response_preserves_status_headers_and_full_payload() {
    let mut rendered = RenderedTemplate::new(
        Default::default(),
        RenderedTemplateBody::Full(b"poem".as_slice().to_vec().into()),
    )
    .with_status(poem::http::StatusCode::ACCEPTED);
    rendered
        .get_headers_mut()
        .insert("x-template", "thymeleaf".parse().expect("valid header"));

    let response = ThymeleafView::new(rendered).into_response();
    assert_eq!(response.status(), poem::http::StatusCode::ACCEPTED);
    assert_eq!(response.headers()["x-template"], "thymeleaf");
    let bytes = block_on(response.into_body().into_bytes()).expect("Poem body");
    assert_eq!(bytes.as_ref(), b"poem");
}
