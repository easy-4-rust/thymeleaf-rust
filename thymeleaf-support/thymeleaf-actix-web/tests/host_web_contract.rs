//! Actix Web 宿主 Web SPI 对 Java Servlet 语义的中立契约测试。
//! 与 thymeleaf-hyper 契约逐断言对齐（宿主请求构造为 Actix TestRequest）。

use std::io::Read;
use std::sync::Arc;

use actix_web::test::TestRequest;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::{Locale, Utf16String};
use thymeleaf::web::{IWebApplication, IWebExchange, IWebRequest, IWebSession};
use thymeleaf_actix_web::{HostWebApplication, HostWebExchange, HostWebRequest, HostWebSession};

fn text(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn template_text(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::string(text(value)))
}

fn value_text(value: &Arc<TemplateValue>) -> String {
    value
        .to_utf16_string()
        .expect("string template value")
        .to_string_lossy()
}

fn contract_request() -> TestRequest {
    TestRequest::post()
        .uri("https://example.test:8443/shop/items?tag=rust&tag=web&q=a+b&escaped=%2F")
        .append_header(("x-trace", "first"))
        .append_header(("x-trace", "second"))
        .append_header((actix_web::http::header::COOKIE, "sid=one; theme=dark"))
        .append_header((actix_web::http::header::COOKIE, "sid=two"))
}

#[actix_web::test]
async fn request_preserves_uri_header_parameter_and_cookie_semantics() {
    let request = contract_request().to_http_request();
    let web_request = HostWebRequest::from_request(&request, "/shop/");

    assert_eq!(
        web_request.get_method().map(|v| v.to_string_lossy()),
        Some("POST".into())
    );
    assert_eq!(
        web_request.get_scheme().map(|v| v.to_string_lossy()),
        Some("https".into())
    );
    assert_eq!(
        web_request.get_server_name().map(|v| v.to_string_lossy()),
        Some("example.test".into())
    );
    assert_eq!(web_request.get_server_port(), Some(8443));
    assert_eq!(
        web_request
            .get_application_path()
            .map(|v| v.to_string_lossy()),
        Some("/shop".into())
    );
    assert_eq!(
        web_request
            .get_path_within_application()
            .map(|v| v.to_string_lossy()),
        Some("/items".into())
    );
    assert_eq!(
        web_request.get_query_string().map(|v| v.to_string_lossy()),
        Some("tag=rust&tag=web&q=a+b&escaped=%2F".into())
    );

    assert!(web_request.contains_header(Some(&text("x-trace"))));
    assert_eq!(
        web_request
            .get_header_values(Some(&text("X-TRACE")))
            .expect("case-insensitive header")
            .into_iter()
            .map(|value| value.expect("header value").to_string_lossy())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert_eq!(web_request.get_parameter_count(), 3);
    assert_eq!(
        web_request
            .get_parameter_values(Some(&text("tag")))
            .expect("tag parameter")
            .into_iter()
            .map(|value| value.expect("parameter value").to_string_lossy())
            .collect::<Vec<_>>(),
        ["rust", "web"]
    );
    assert_eq!(
        web_request
            .get_parameter_values(Some(&text("q")))
            .expect("q parameter")[0]
            .as_ref()
            .expect("q value")
            .to_string_lossy(),
        "a b"
    );
    assert_eq!(
        web_request
            .get_parameter_values(Some(&text("escaped")))
            .expect("escaped parameter")[0]
            .as_ref()
            .expect("escaped value")
            .to_string_lossy(),
        "/"
    );
    assert_eq!(web_request.get_cookie_count(), 3);
    assert_eq!(
        web_request
            .get_cookie_values(Some(&text("sid")))
            .expect("sid cookie")
            .into_iter()
            .map(|value| value.expect("cookie value").to_string_lossy())
            .collect::<Vec<_>>(),
        ["one", "two"]
    );
}

#[actix_web::test]
async fn request_uses_scheme_default_port_and_root_application_path() {
    let request = TestRequest::get()
        .uri("http://example.test/root")
        .to_http_request();
    let web_request = HostWebRequest::from_request(&request, "/");

    assert_eq!(web_request.get_server_port(), Some(80));
    assert_eq!(
        web_request
            .get_application_path()
            .expect("application path")
            .to_string_lossy(),
        ""
    );
    assert_eq!(
        web_request
            .get_path_within_application()
            .expect("path within application")
            .to_string_lossy(),
        "/root"
    );
}

#[test]
fn session_is_created_lazily_and_preserves_attribute_order_and_identity() {
    let session = HostWebSession::new();
    let first_name = text("first");
    let second_name = text("second");
    let first_value = template_text("one");
    let second_value = template_text("two");

    assert!(!session.exists());
    session.set_attribute_value(Some(first_name.clone()), None);
    assert!(!session.exists());
    session.set_attribute_value(Some(first_name.clone()), Some(Arc::clone(&first_value)));
    session.set_attribute_value(Some(second_name.clone()), Some(Arc::clone(&second_value)));

    assert!(session.exists());
    assert_eq!(session.get_attribute_count(), 2);
    assert_eq!(
        session
            .get_all_attribute_names()
            .into_iter()
            .map(|name| name.expect("attribute name").to_string_lossy())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert!(Arc::ptr_eq(
        &session
            .get_attribute_value(Some(&first_name))
            .expect("first attribute"),
        &first_value
    ));
    assert_eq!(
        value_text(
            &session
                .get_attribute_map()
                .get(&Some(second_name.clone()))
                .expect("second entry")
                .clone()
                .expect("second value")
        ),
        "two"
    );

    session.remove_attribute(Some(&first_name));
    session.set_attribute_value(Some(second_name.clone()), None);
    assert_eq!(session.get_attribute_count(), 0);
    assert!(session.exists());
    assert!(HostWebSession::existing().exists());
}

#[test]
fn application_preserves_attributes_and_reads_resources_from_ordered_roots() {
    let first_root = tempfile::tempdir().expect("first resource root");
    let second_root = tempfile::tempdir().expect("second resource root");
    std::fs::write(second_root.path().join("template.html"), "from-second")
        .expect("write resource");
    let application = HostWebApplication::new(vec![
        first_root.path().to_path_buf(),
        second_root.path().to_path_buf(),
    ]);
    let name = text("shared");
    let value = template_text("application");

    application.set_attribute_value(Some(name.clone()), Some(Arc::clone(&value)));
    assert!(application.contains_attribute(Some(&name)));
    assert_eq!(application.get_attribute_count(), 1);
    assert!(Arc::ptr_eq(
        &application
            .get_attribute_value(Some(&name))
            .expect("application attribute"),
        &value
    ));

    let resource = text("/template.html");
    assert!(application.resource_exists(Some(&resource)));
    let mut contents = String::new();
    application
        .get_resource_as_stream(Some(&resource))
        .expect("resource stream")
        .read_to_string(&mut contents)
        .expect("read resource");
    assert_eq!(contents, "from-second");
    assert!(!application.resource_exists(Some(&text("/missing.html"))));
    application.set_attribute_value(Some(name.clone()), None);
    assert!(!application.contains_attribute(Some(&name)));
}

#[test]
#[should_panic(expected = "Name cannot be null")]
fn request_rejects_null_lookup_names_like_servlet_validate() {
    let request = TestRequest::get().uri("/").to_http_request();
    let _ = HostWebRequest::from_request(&request, "").get_cookie_values(None);
}

#[test]
#[should_panic(expected = "Name cannot be null")]
fn session_rejects_null_attribute_names_like_servlet_validate() {
    HostWebSession::new().contains_attribute(None);
}

#[test]
#[should_panic(expected = "Path cannot be null")]
fn application_rejects_null_resource_paths_like_servlet_validate() {
    HostWebApplication::new(Vec::new()).get_resource_as_stream(None);
}

#[test]
#[should_panic(expected = "Name cannot be null")]
fn exchange_rejects_null_attribute_names_like_servlet_validate() {
    let request = Arc::new(HostWebRequest::from_request(
        &TestRequest::get().uri("/").to_http_request(),
        "",
    ));
    let application = Arc::new(HostWebApplication::new(Vec::new()));
    HostWebExchange::new(request, None, application, None, None).remove_attribute(None);
}

#[test]
fn exchange_preserves_scope_identity_metadata_attributes_and_url_transform() {
    let request = Arc::new(HostWebRequest::from_request(
        &TestRequest::get()
            .uri("https://example.test/view")
            .to_http_request(),
        "",
    ));
    let session = Arc::new(HostWebSession::existing());
    let application = Arc::new(HostWebApplication::new(Vec::new()));
    let principal = template_text("alice");
    let locale = Locale::new(text("zh-CN"), text("CN"));
    let exchange = HostWebExchange::new(
        Arc::clone(&request),
        Some(Arc::clone(&session)),
        application.clone(),
        Some(Arc::clone(&principal)),
        Some(locale.clone()),
    );

    assert_eq!(
        exchange.get_request() as *const dyn IWebRequest as *const (),
        Arc::as_ptr(&request).cast::<()>()
    );
    assert_eq!(
        exchange.get_session().expect("session") as *const dyn IWebSession as *const (),
        Arc::as_ptr(&session).cast::<()>()
    );
    assert_eq!(
        exchange.get_application() as *const dyn IWebApplication as *const (),
        Arc::as_ptr(&application).cast::<()>()
    );
    assert!(exchange.has_session());
    assert!(Arc::ptr_eq(
        &exchange.get_principal().expect("principal"),
        &principal
    ));
    assert_eq!(exchange.get_locale(), Some(locale));

    exchange.set_content_type(Some(text("text/html")));
    exchange.set_character_encoding(Some(text("UTF-8")));
    assert_eq!(
        exchange
            .get_content_type()
            .expect("content type")
            .to_string_lossy(),
        "text/html"
    );
    assert_eq!(
        exchange
            .get_character_encoding()
            .expect("encoding")
            .to_string_lossy(),
        "UTF-8"
    );

    let name = text("request-id");
    let value = template_text("42");
    exchange.set_attribute_value(Some(name.clone()), Some(Arc::clone(&value)));
    assert!(exchange.contains_attribute(Some(&name)));
    assert_eq!(exchange.get_attribute_count(), 1);
    assert_eq!(
        exchange
            .get_all_attribute_names()
            .first()
            .and_then(Option::as_ref)
            .expect("attribute name")
            .to_string_lossy(),
        "request-id"
    );
    assert!(Arc::ptr_eq(
        &exchange
            .get_attribute_value(Some(&name))
            .expect("exchange attribute"),
        &value
    ));
    assert_eq!(
        exchange
            .transform_url(Some(&text("/next")))
            .expect("transformed URL")
            .to_string_lossy(),
        "/next"
    );
    assert!(exchange.transform_url(None).is_none());

    exchange.remove_attribute(Some(&name));
    assert_eq!(exchange.get_attribute_count(), 0);
}

/// Actix 适配器端到端：exchange 属性通过 WebContext 渲染为模板变量。
/// 与 thymeleaf-hyper 契约逐断言对齐（宿主为 Actix 请求）。
#[test]
fn exchange_renders_template_with_request_context() {
    let request = Arc::new(HostWebRequest::from_request(
        &TestRequest::get()
            .uri("https://example.test/shop/items?tag=rust&tag=web")
            .to_http_request(),
        "/shop",
    ));
    let exchange: Arc<dyn IWebExchange> = Arc::new(HostWebExchange::new(
        Arc::clone(&request),
        None,
        Arc::new(HostWebApplication::new(Vec::new())),
        None,
        None,
    ));
    exchange.set_attribute_value(Some(text("greeting")), Some(template_text("hello-web")));

    let request = exchange.get_request();
    assert_eq!(
        request.get_application_path().map(|v| v.to_string_lossy()),
        Some("/shop".into())
    );
    assert_eq!(
        request
            .get_path_within_application()
            .map(|v| v.to_string_lossy()),
        Some("/items".into())
    );
    assert_eq!(
        request.get_query_string().map(|v| v.to_string_lossy()),
        Some("tag=rust&tag=web".into())
    );

    let mut resolver = thymeleaf::templateresolver::StringTemplateResolver::new();
    resolver.set_template_mode(thymeleaf::TemplateMode::HTML);
    let engine = thymeleaf::TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn thymeleaf::ITemplateResolver>)
        .expect("resolver");
    let web_context =
        thymeleaf::context::WebContext::new(Some(exchange)).expect("valid web context");
    let rendered = engine
        .process_template("<p th:text=\"${greeting}\">x</p>", &web_context)
        .expect("render")
        .to_string_lossy();
    assert!(
        rendered.contains("hello-web"),
        "exchange 属性在模板中可见: {rendered}"
    );
}
