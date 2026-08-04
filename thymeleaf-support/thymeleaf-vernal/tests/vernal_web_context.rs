//! `thymeleaf-vernal` Web 上下文适配测试。
//!
//! 验证：`VernalWebRequest` 从 `HttpRequestSnapshot` 读取方法/URI/Header/参数/Cookie；
//! `VernalWebExchange` 把 `SecurityPrincipal` 暴露为 `get_principal()`（供 sec 方言
//! 消费）；`WebContext` 可基于 Vernal 交换构造并渲染。

use std::sync::Arc;

use http::Request;
use thymeleaf::context::WebContext;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::web::{IWebExchange, IWebRequest};
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};
use tokio_util::sync::CancellationToken;
use vernal_http::HttpRequestSnapshot;
use vernal_web::{RequestContext, RouteMetadata, SecurityPrincipal};

use thymeleaf_vernal::{VernalWebExchange, VernalWebRequest};

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
}

fn snapshot(uri: &str, headers: &[(&str, &str)]) -> Arc<HttpRequestSnapshot> {
    let mut builder = Request::builder().method("GET").uri(uri);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let request = builder.body(()).expect("HTTP request");
    Arc::new(HttpRequestSnapshot::capture(&request))
}

fn request_context() -> Arc<RequestContext> {
    Arc::new(RequestContext::new(
        RouteMetadata::new("test_handler", "invoke", "/greet"),
        CancellationToken::new(),
    ))
}

#[test]
fn web_request_reads_method_uri_headers_parameters_and_cookies() {
    let snapshot = snapshot(
        "/greet?name=world&tag=a&tag=b",
        &[
            ("host", "example.com"),
            ("x-request-id", "req-123"),
            ("cookie", "device=web; tenant=easy-rust"),
        ],
    );
    let request = VernalWebRequest::new(snapshot);

    assert_eq!(request.get_method().unwrap().to_string_lossy(), "GET");
    assert_eq!(
        request
            .get_path_within_application()
            .unwrap()
            .to_string_lossy(),
        "/greet"
    );
    assert_eq!(
        request.get_query_string().unwrap().to_string_lossy(),
        "name=world&tag=a&tag=b"
    );
    assert_eq!(
        request
            .get_header_value(Some(&js("x-request-id")))
            .unwrap()
            .to_string_lossy(),
        "req-123"
    );
    assert_eq!(
        request
            .get_parameter_value(Some(&js("name")))
            .unwrap()
            .to_string_lossy(),
        "world"
    );
    // 重复参数保持多值
    let tags = request.get_parameter_values(Some(&js("tag"))).unwrap();
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].as_ref().unwrap().to_string_lossy(), "a");
    assert_eq!(tags[1].as_ref().unwrap().to_string_lossy(), "b");
    assert_eq!(
        request
            .get_cookie_value(Some(&js("device")))
            .unwrap()
            .to_string_lossy(),
        "web"
    );
    assert_eq!(
        request
            .get_cookie_value(Some(&js("tenant")))
            .unwrap()
            .to_string_lossy(),
        "easy-rust"
    );
}

#[test]
fn web_exchange_renders_with_request_variables() {
    let snapshot = snapshot("/greet", &[("x-request-id", "req-42")]);
    let context = request_context();
    let exchange = VernalWebExchange::new(context, snapshot);
    let web_context = WebContext::new(Some(Arc::new(exchange))).expect("web context");
    web_context.set_variable(
        Some(js("greeting")),
        Some(Arc::new(thymeleaf::expression::TemplateValue::string(js(
            "hello",
        )))),
    );

    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();

    let output = engine
        .process_template("<p th:text=\"${greeting}\">x</p>", &web_context)
        .unwrap()
        .to_string_lossy();
    assert_eq!(output, "<p>hello</p>");
}

#[test]
fn web_exchange_exposes_principal() {
    let snapshot = snapshot("/greet", &[]);
    let context = request_context();
    let exchange = VernalWebExchange::new(context, snapshot);
    // 渲染入口在 async 上下文预读 principal 后同步注入（sec 方言同步求值）
    exchange.set_principal_snapshot(Some(Arc::new(SecurityPrincipal::new(
        "user-42",
        ["ROLE_ADMIN", "ROLE_USER"],
    ))));

    let principal = exchange.get_principal();
    let principal = principal.expect("principal should be present");
    let thymeleaf::expression::TemplateValue::Object(object) = principal.as_ref() else {
        panic!("principal must be a TemplateValue::Object");
    };
    assert_eq!(object.to_utf16_string().to_string_lossy(), "user-42");
}

#[test]
fn vernal_principal_drives_sa_token_sec_dialect() {
    // 端到端联动：Vernal 请求上下文 → VernalWebExchange → WebContext → 注入
    // Sa-Token 安全快照 → thymeleaf-sa-token sec 方言渲染。
    use std::sync::Arc;
    use thymeleaf::expression::TemplateValue;
    use thymeleaf_sa_token::{
        AUTHENTICATION_VARIABLE, SaTokenAuthentication, SaTokenAuthenticationObject, SaTokenDialect,
    };

    let snapshot = snapshot("/orders/42", &[]);
    let context = request_context();
    let exchange = VernalWebExchange::new(context, snapshot);
    exchange.set_principal_snapshot(Some(Arc::new(SecurityPrincipal::new(
        "user-42",
        ["ROLE_ADMIN"],
    ))));

    // 渲染入口：从 Vernal principal 构建 Sa-Token 快照并注入 WebContext
    let web_context = WebContext::new(Some(Arc::new(exchange))).expect("web context");
    let authentication = SaTokenAuthentication::new(
        "user-42".to_owned(),
        Arc::from([Arc::from("ROLE_ADMIN")]),
        Arc::from([Arc::from("orders:*")]),
    );
    web_context.set_variable(
        Some(js(AUTHENTICATION_VARIABLE)),
        Some(Arc::new(SaTokenAuthenticationObject::to_template_value(
            Arc::new(authentication),
        ))),
    );
    web_context.set_variable(
        Some(js("orderId")),
        Some(Arc::new(TemplateValue::string(js("42")))),
    );

    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    engine.add_dialect(Arc::new(SaTokenDialect::new())).unwrap();

    let output = engine
        .process_template(
            "<div sec:authorize=\"hasRole('ROLE_ADMIN')\">admin orders</div>\
             <p th:text=\"${#authentication.name}\">x</p>",
            &web_context,
        )
        .unwrap()
        .to_string_lossy();
    assert!(output.contains("admin orders"), "output: {output}");
    assert!(output.contains("user-42"), "output: {output}");
    assert!(!output.contains("sec:authorize"), "output: {output}");
}
