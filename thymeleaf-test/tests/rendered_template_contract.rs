//! Rust 中立 Web 响应模型的结构、Frame 与 Trailer 契约测试。

use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use bytes::Bytes;
use futures_util::stream;
use http::header::{CONTENT_TYPE, HeaderValue};
use http::{HeaderMap, StatusCode};
use http_body::{Body, Frame};
use thymeleaf::web::{RenderError, RenderedTemplate, RenderedTemplateBody};

fn poll_frame(body: &mut RenderedTemplateBody) -> Poll<Option<Result<Frame<Bytes>, RenderError>>> {
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    Pin::new(body).poll_frame(&mut context)
}

fn ready<T>(poll: Poll<T>) -> T {
    match poll {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("in-memory body must be immediately ready"),
    }
}

#[test]
fn full_body_reports_exact_size_and_emits_bytes_once() {
    let mut body = RenderedTemplateBody::Full(Bytes::from_static(b"rendered"));

    assert_eq!(body.size_hint().exact(), Some(8));
    assert!(!body.is_end_stream());
    let frame = ready(poll_frame(&mut body))
        .expect("full body must emit one frame")
        .expect("full body frame must succeed");
    assert_eq!(
        frame.into_data().expect("full body emits a data frame"),
        Bytes::from_static(b"rendered")
    );
    assert!(ready(poll_frame(&mut body)).is_none());
    assert!(body.is_end_stream());
    assert_eq!(body.size_hint().exact(), Some(0));
}

#[test]
fn stream_body_preserves_data_and_trailer_frames_in_order() {
    let mut trailers = HeaderMap::new();
    trailers.insert("x-render-complete", HeaderValue::from_static("true"));
    let frames: Vec<Result<Frame<Bytes>, RenderError>> = vec![
        Ok(Frame::data(Bytes::from_static(b"chunk"))),
        Ok(Frame::trailers(trailers.clone())),
    ];
    let mut body = RenderedTemplateBody::Stream(Box::pin(stream::iter(frames)));

    let data = ready(poll_frame(&mut body))
        .expect("stream data frame")
        .expect("stream data succeeds")
        .into_data()
        .expect("first frame is data");
    assert_eq!(data, Bytes::from_static(b"chunk"));

    let actual_trailers = ready(poll_frame(&mut body))
        .expect("stream trailer frame")
        .expect("stream trailer succeeds")
        .into_trailers()
        .expect("second frame is trailers");
    assert_eq!(actual_trailers, trailers);
    assert!(ready(poll_frame(&mut body)).is_none());
}

#[test]
fn http_response_conversion_preserves_status_headers_and_body() {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html;charset=UTF-8"),
    );
    let response = RenderedTemplate::new(
        headers.clone(),
        RenderedTemplateBody::Full(Bytes::from_static(b"<p>ok</p>")),
    )
    .with_status(StatusCode::CREATED)
    .into_http_response();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers(), &headers);
    assert_eq!(response.body().size_hint().exact(), Some(9));
}
