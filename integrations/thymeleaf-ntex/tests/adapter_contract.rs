//! Ntex 适配器对中立 Body 的完整输出契约测试。

use std::task::{Context, Poll, Waker};

use ntex::http::body::{BodySize, MessageBody};
use ntex::web::{Responder, test};
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use thymeleaf_ntex::{ThymeleafBody, ThymeleafView};

#[test]
fn full_body_preserves_exact_length_payload_and_completion() {
    let mut body = ThymeleafBody::new(RenderedTemplateBody::Full(
        b"ntex".as_slice().to_vec().into(),
    ));
    assert_eq!(body.size(), BodySize::Sized(4));

    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    let chunk = match body.poll_next_chunk(&mut context) {
        Poll::Ready(Some(Ok(chunk))) => chunk,
        _ => panic!("Ntex body must synchronously emit the full payload"),
    };
    assert_eq!(chunk.as_ref(), b"ntex");
    assert!(matches!(
        body.poll_next_chunk(&mut context),
        Poll::Ready(None)
    ));
}

#[test]
fn responder_preserves_status_headers_and_body() {
    futures_executor::block_on(async {
        let mut rendered = RenderedTemplate::new(
            Default::default(),
            RenderedTemplateBody::Full(b"view".as_slice().to_vec().into()),
        )
        .with_status(http::StatusCode::ACCEPTED);
        rendered
            .get_headers_mut()
            .insert("x-template", "thymeleaf".parse().expect("valid header"));

        let request = test::TestRequest::default().to_http_request();
        let mut response = ThymeleafView::from(rendered).respond_to(&request).await;
        assert_eq!(response.status().as_u16(), 202);
        assert_eq!(
            response
                .headers()
                .get("x-template")
                .expect("x-template header")
                .as_bytes(),
            b"thymeleaf"
        );

        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let chunk = match response.take_body().poll_next_chunk(&mut context) {
            Poll::Ready(Some(Ok(chunk))) => chunk,
            _ => panic!("Ntex view body must synchronously emit its full payload"),
        };
        assert_eq!(chunk.as_ref(), b"view");
    });
}
