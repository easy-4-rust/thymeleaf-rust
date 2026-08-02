//! Warp Reply 的有限 Body 与显式流式拒绝契约测试。

use futures_util::stream;
use http_body::Frame;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use thymeleaf_warp::{ThymeleafReply, ThymeleafReplyError};
use warp::Reply;

#[test]
fn full_reply_preserves_status_and_headers() {
    let mut rendered = RenderedTemplate::new(
        Default::default(),
        RenderedTemplateBody::Full(b"warp".as_slice().to_vec().into()),
    )
    .with_status(warp::http::StatusCode::ACCEPTED);
    rendered
        .get_headers_mut()
        .insert("x-template", "thymeleaf".parse().expect("valid header"));

    let response = ThymeleafReply::try_from(rendered)
        .expect("full rendering is supported")
        .into_response();
    assert_eq!(response.status(), warp::http::StatusCode::ACCEPTED);
    assert_eq!(response.headers()["x-template"], "thymeleaf");
}

#[test]
fn streaming_reply_is_rejected_instead_of_being_silently_collected() {
    let frames = stream::iter(vec![Ok(Frame::data(b"warp".as_slice().to_vec().into()))]);
    let rendered = RenderedTemplate::new(
        Default::default(),
        RenderedTemplateBody::Stream(Box::pin(frames)),
    );

    let error = match ThymeleafReply::try_from(rendered) {
        Ok(_) => panic!("Warp must not silently collect a streaming response"),
        Err(error) => error,
    };
    assert!(matches!(error, ThymeleafReplyError::StreamingUnsupported));
}
