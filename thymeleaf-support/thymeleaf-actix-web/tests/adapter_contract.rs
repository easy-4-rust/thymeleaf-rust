//! Actix Web 适配器对中立 Body 的完整输出契约测试。

use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use actix_web::body::{BodySize, MessageBody};
use actix_web::{Responder, test as actix_test};
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use thymeleaf_actix_web::{ThymeleafBody, ThymeleafView};

#[test]
fn full_body_preserves_exact_length_payload_and_completion() {
    let mut body = ThymeleafBody::new(RenderedTemplateBody::Full(
        b"actix".as_slice().to_vec().into(),
    ));
    assert_eq!(body.size(), BodySize::Sized(5));

    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    let chunk = match Pin::new(&mut body).poll_next(&mut context) {
        Poll::Ready(Some(Ok(chunk))) => chunk,
        _ => panic!("Actix body must synchronously emit the full payload"),
    };
    assert_eq!(chunk.as_ref(), b"actix");
    assert!(matches!(
        Pin::new(&mut body).poll_next(&mut context),
        Poll::Ready(None)
    ));
}

#[test]
fn responder_preserves_status_headers_and_body() {
    let mut rendered = RenderedTemplate::new(
        Default::default(),
        RenderedTemplateBody::Full(b"view".as_slice().to_vec().into()),
    )
    .with_status(http::StatusCode::ACCEPTED);
    rendered
        .get_headers_mut()
        .insert("x-template", "thymeleaf".parse().expect("valid header"));

    let request = actix_test::TestRequest::default().to_http_request();
    let response = ThymeleafView::from(rendered).respond_to(&request);
    assert_eq!(response.status(), actix_web::http::StatusCode::ACCEPTED);
    assert_eq!(
        response
            .headers()
            .get("x-template")
            .expect("x-template header"),
        "thymeleaf"
    );

    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    let mut body = response.into_body();
    let chunk = match Pin::new(&mut body).poll_next(&mut context) {
        Poll::Ready(Some(Ok(chunk))) => chunk,
        _ => panic!("Actix view body must synchronously emit its full payload"),
    };
    assert_eq!(chunk.as_ref(), b"view");
}
