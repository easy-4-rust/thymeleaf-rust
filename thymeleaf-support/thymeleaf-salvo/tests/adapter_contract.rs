//! Salvo Scribe 对状态、Header 与有限 Body 的契约测试。

use salvo::Scribe;
use salvo::http::body::ResBody;
use salvo::prelude::Response;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use thymeleaf_salvo::ThymeleafView;

#[test]
fn scribe_preserves_status_headers_and_full_payload() {
    let mut rendered = RenderedTemplate::new(
        Default::default(),
        RenderedTemplateBody::Full(b"salvo".as_slice().to_vec().into()),
    )
    .with_status(salvo::http::StatusCode::CREATED);
    rendered
        .get_headers_mut()
        .insert("x-template", "thymeleaf".parse().expect("valid header"));
    let mut response = Response::new();

    ThymeleafView::new(rendered).render(&mut response);

    assert_eq!(response.status_code, Some(salvo::http::StatusCode::CREATED));
    assert_eq!(response.headers()["x-template"], "thymeleaf");
    match response.take_body() {
        ResBody::Once(bytes) => assert_eq!(bytes.as_ref(), b"salvo"),
        _ => panic!("full Thymeleaf rendering must remain a Salvo Once body"),
    }
}
