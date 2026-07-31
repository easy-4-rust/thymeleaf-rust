//! Tide 适配器对状态、Header 与 Body 的中立响应契约测试。

use futures_util::stream;
use http_body::Frame;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use thymeleaf_tide::ThymeleafView;

#[test]
fn response_preserves_status_headers_and_full_payload() {
    async_std::task::block_on(async {
        let mut rendered = RenderedTemplate::new(
            Default::default(),
            RenderedTemplateBody::Full(b"tide".as_slice().to_vec().into()),
        )
        .with_status(http::StatusCode::ACCEPTED);
        rendered
            .get_headers_mut()
            .insert("x-template", "thymeleaf".parse().expect("valid header"));

        let mut response = ThymeleafView::new(rendered).into_response();
        assert_eq!(response.status(), tide::StatusCode::Accepted);
        assert_eq!(response["x-template"], "thymeleaf");
        let bytes = response.take_body().into_bytes().await.expect("Tide body");
        assert_eq!(bytes, b"tide");
    });
}

#[test]
fn response_streams_data_frames_without_precollecting() {
    async_std::task::block_on(async {
        let frames = stream::iter(vec![
            Ok(Frame::data(b"first".as_slice().to_vec().into())),
            Ok(Frame::data(b"-second".as_slice().to_vec().into())),
        ]);
        let rendered = RenderedTemplate::new(
            Default::default(),
            RenderedTemplateBody::Stream(Box::pin(frames)),
        );

        let mut response = ThymeleafView::from(rendered).into_response();
        let bytes = response
            .take_body()
            .into_bytes()
            .await
            .expect("Tide streaming body");
        assert_eq!(bytes, b"first-second");
    });
}
