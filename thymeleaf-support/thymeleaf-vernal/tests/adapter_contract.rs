//! Vernal 中立 HTTP 桥对状态、Header、Body 与 Trailer 的契约测试。

use futures_executor::block_on;
use futures_util::stream;
use http::header::HeaderValue;
use http_body::Frame;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use thymeleaf_vernal::ThymeleafView;

#[test]
fn response_preserves_status_headers_data_and_trailers() {
    let mut trailers = http::HeaderMap::new();
    trailers.insert("x-render-complete", HeaderValue::from_static("true"));
    let frames = stream::iter(vec![
        Ok(Frame::data(b"vernal".as_slice().to_vec().into())),
        Ok(Frame::trailers(trailers.clone())),
    ]);
    let mut rendered = RenderedTemplate::new(
        Default::default(),
        RenderedTemplateBody::Stream(Box::pin(frames)),
    )
    .with_status(http::StatusCode::ACCEPTED);
    rendered
        .get_headers_mut()
        .insert("x-template", "thymeleaf".parse().expect("valid header"));

    let response = ThymeleafView::new(rendered).into_http_response();
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, http::StatusCode::ACCEPTED);
    assert_eq!(parts.headers["x-template"], "thymeleaf");
    let collected = block_on(body.collect_limited(64)).expect("Vernal body");
    assert_eq!(collected.bytes().as_ref(), b"vernal");
    assert_eq!(collected.trailers(), Some(&trailers));
}
