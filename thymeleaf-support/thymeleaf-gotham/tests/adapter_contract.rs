//! Gotham 适配器对状态、Header 与 Body 的中立响应契约测试。

use futures_executor::block_on;
use gotham::handler::IntoResponse;
use gotham::state::State;
use http_body_util::BodyExt;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use thymeleaf_gotham::ThymeleafView;

#[test]
fn response_preserves_status_headers_and_full_payload() {
    State::with_new(|state| {
        let mut rendered = RenderedTemplate::new(
            Default::default(),
            RenderedTemplateBody::Full(b"gotham".as_slice().to_vec().into()),
        )
        .with_status(http::StatusCode::ACCEPTED);
        rendered
            .get_headers_mut()
            .insert("x-template", "thymeleaf".parse().expect("valid header"));

        let response = ThymeleafView::new(rendered).into_response(state);
        assert_eq!(response.status(), http::StatusCode::ACCEPTED);
        assert_eq!(response.headers()["x-template"], "thymeleaf");
        let bytes = block_on(response.into_body().collect())
            .expect("Gotham body")
            .to_bytes();
        assert_eq!(bytes.as_ref(), b"gotham");
    });
}
