//! Rocket route 对状态、Header 与 Body 的端到端响应契约测试。

use rocket::local::blocking::Client;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use thymeleaf_rocket::ThymeleafView;

#[rocket::get("/")]
fn rendered() -> ThymeleafView {
    let mut rendered = RenderedTemplate::new(
        Default::default(),
        RenderedTemplateBody::Full(b"rocket".as_slice().to_vec().into()),
    )
    .with_status(http::StatusCode::ACCEPTED);
    rendered
        .get_headers_mut()
        .insert("x-template", "thymeleaf".parse().expect("valid header"));
    ThymeleafView::new(rendered)
}

#[test]
fn response_preserves_status_headers_and_full_payload() {
    let client = Client::tracked(rocket::build().mount("/", rocket::routes![rendered]))
        .expect("Rocket client");
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), rocket::http::Status::Accepted);
    assert_eq!(response.headers().get_one("x-template"), Some("thymeleaf"));
    assert_eq!(response.into_bytes().expect("Rocket body"), b"rocket");
}
