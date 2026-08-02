//! Axum 适配器对状态、Header 与 Body 的中立响应契约测试。

use axum::response::IntoResponse;
use futures_executor::block_on;
use http_body_util::BodyExt;
use thymeleaf::web::{RenderError, RenderedTemplate, RenderedTemplateBody};
use thymeleaf_axum::{ThymeleafError, ThymeleafView};

#[test]
fn response_preserves_status_headers_and_full_payload() {
    let mut rendered = RenderedTemplate::new(
        Default::default(),
        RenderedTemplateBody::Full(b"axum".as_slice().to_vec().into()),
    )
    .with_status(axum::http::StatusCode::ACCEPTED);
    rendered
        .get_headers_mut()
        .insert("x-template", "thymeleaf".parse().expect("valid header"));

    let response = ThymeleafView::new(rendered).into_response();
    assert_eq!(response.status(), axum::http::StatusCode::ACCEPTED);
    assert_eq!(response.headers()["x-template"], "thymeleaf");
    let bytes = block_on(response.into_body().collect())
        .expect("Axum body collection")
        .to_bytes();
    assert_eq!(bytes.as_ref(), b"axum");
}

#[test]
fn error_response_is_generic_while_cause_remains_diagnostic() {
    let error = ThymeleafError::from(RenderError::new("secret/template.html: expression failed"));
    assert_eq!(
        error.get_cause().get_message(),
        "secret/template.html: expression failed"
    );

    let response = error.into_response();
    assert_eq!(
        response.status(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    );
    let bytes = block_on(response.into_body().collect())
        .expect("Axum error body collection")
        .to_bytes();
    assert_eq!(bytes.as_ref(), b"Template rendering failed");
}
