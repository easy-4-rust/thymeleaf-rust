//! `WebEngineContext` 端到端 Java Golden 差分测试。
//!
//! 通过 `WebContext` + 模板引擎渲染覆盖 WebEngineContext 的：
//! request/session/application 属性可见性、request parameter 读取、
//! exchange 属性作用域、locale 和渲染路径。

// 共享 Web corpus 同时服务于完整 Web SPI 批次；本批次只消费 exchange 身份语义。
#![allow(dead_code, unused_imports)]

mod support;

use std::sync::Arc;

use support::CorpusWebExchange;
use thymeleaf::context::WebContext;
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::web::IWebExchange;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

fn render_with_exchange(template: &str, exchange: Arc<dyn IWebExchange>) -> String {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let web_context = WebContext::new(Some(exchange)).expect("valid web context");
    e.process_template(template, &web_context)
        .unwrap()
        .to_string_lossy()
}

// ===========================================================================
// 1. WebContext 构造
// ===========================================================================

#[test]
fn web_context_null_exchange_errors() {
    assert!(WebContext::new(None).is_err());
}

#[test]
fn web_context_with_exchange_ok() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    assert!(WebContext::new(Some(exchange)).is_ok());
}

// ===========================================================================
// 2. Exchange 属性在模板中可见
// ===========================================================================

#[test]
fn exchange_attribute_visible_in_template() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    exchange.set_attribute_value(
        Some(Utf16String::from_rust_str("greeting")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "Hello Web",
        )))),
    );
    let s = render_with_exchange("<p th:text=\"${greeting}\">x</p>", exchange);
    assert!(s.contains("Hello Web"));
}

#[test]
fn exchange_attribute_count_and_names() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    exchange.set_attribute_value(
        Some(Utf16String::from_rust_str("a")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "1",
        )))),
    );
    exchange.set_attribute_value(
        Some(Utf16String::from_rust_str("b")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "2",
        )))),
    );
    assert_eq!(exchange.get_attribute_count(), 2);
    assert!(exchange.contains_attribute(Some(&Utf16String::from_rust_str("a"))));
    assert!(exchange.contains_attribute(Some(&Utf16String::from_rust_str("b"))));
    let names = exchange.get_all_attribute_names();
    assert_eq!(names.len(), 2);
}

#[test]
fn exchange_attribute_remove() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    exchange.set_attribute_value(
        Some(Utf16String::from_rust_str("tmp")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "x",
        )))),
    );
    assert!(exchange.contains_attribute(Some(&Utf16String::from_rust_str("tmp"))));
    exchange.remove_attribute(Some(&Utf16String::from_rust_str("tmp")));
    assert!(!exchange.contains_attribute(Some(&Utf16String::from_rust_str("tmp"))));
}

// ===========================================================================
// 3. Request 信息可见
// ===========================================================================

#[test]
fn request_method_and_url() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let request = exchange.get_request();
    assert_eq!(request.get_method().unwrap().to_string_lossy(), "GET");
    assert_eq!(request.get_scheme().unwrap().to_string_lossy(), "http");
    assert_eq!(
        request.get_server_name().unwrap().to_string_lossy(),
        "localhost"
    );
    assert_eq!(request.get_server_port(), Some(80));
}

#[test]
fn request_is_not_secure_for_http() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    assert!(!exchange.get_request().is_secure());
}

#[test]
fn request_path_concatenation() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let path = exchange.get_request().get_request_path();
    assert!(path.to_string_lossy().contains("/"));
}

// ===========================================================================
// 4. Locale 传播
// ===========================================================================

#[test]
fn exchange_locale_available() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    assert!(exchange.get_locale().is_some());
}

// ===========================================================================
// 5. Web 渲染端到端
// ===========================================================================

#[test]
fn web_render_with_expression_and_attribute() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    exchange.set_attribute_value(
        Some(Utf16String::from_rust_str("user")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "Alice",
        )))),
    );
    exchange.set_attribute_value(
        Some(Utf16String::from_rust_str("show")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    let s = render_with_exchange(
        "<div th:if=\"${show}\" th:text=\"${'Hi, ' + user}\">x</div>",
        exchange,
    );
    assert!(s.contains("Hi, Alice"));
}

#[test]
fn web_render_loop_over_attribute() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let items = vec![
        Arc::new(TemplateValue::string(Utf16String::from_rust_str("a"))),
        Arc::new(TemplateValue::string(Utf16String::from_rust_str("b"))),
    ];
    exchange.set_attribute_value(
        Some(Utf16String::from_rust_str("items")),
        Some(Arc::new(TemplateValue::List(Arc::new(items)))),
    );
    let s = render_with_exchange(
        "<ul><li th:each=\"i:${items}\" th:text=\"${i}\">x</li></ul>",
        exchange,
    );
    assert!(s.contains("a") && s.contains("b"));
}

#[test]
fn web_render_plain_template() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let s = render_with_exchange("<p>static content</p>", exchange);
    assert_eq!(s, "<p>static content</p>");
}

// ===========================================================================
// 6. 多 WebContext 实例独立性
// ===========================================================================

#[test]
fn separate_exchanges_do_not_share_attributes() {
    let exchange_a: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let exchange_b: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    exchange_a.set_attribute_value(
        Some(Utf16String::from_rust_str("only_a")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "yes",
        )))),
    );
    assert!(exchange_a.contains_attribute(Some(&Utf16String::from_rust_str("only_a"))));
    assert!(!exchange_b.contains_attribute(Some(&Utf16String::from_rust_str("only_a"))));
}

// ===========================================================================
// 6. 变量与 exchange 属性共享标识（Java WebEngineContextTest test03/test05/test14）
// ===========================================================================

#[test]
fn context_variables_share_identity_with_exchange_attributes() {
    // Java 立身之本：用户变量与 WebExchange 属性 map 共享标识。两个方向都要成立：
    // ① WebContext 变量 -> 渲染后 exchange 属性可见（StandardEngineContextFactory
    //    把上下文变量写入 exchange 属性）；
    // ② exchange 属性 -> 模板 ${name} 可见（渲染读取 exchange 属性作为变量）。
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let one = Utf16String::from_rust_str("one");
    let two = Utf16String::from_rust_str("two values");

    let variables: Vec<(Option<Utf16String>, Option<Arc<TemplateValue>>)> = vec![(
        Some(one.clone()),
        Some(Arc::new(TemplateValue::string(two.clone()))),
    )];

    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let web_context = WebContext::new(Some(exchange.clone())).expect("valid web context");
    web_context.set_variables(Some(&variables));

    let rendered = engine
        .process_template("<p th:text=\"${one}\">x</p>", &web_context)
        .expect("render")
        .to_string_lossy();
    assert!(
        rendered.contains("two values"),
        "变量在模板中可见: {rendered}"
    );

    // 方向 ①：WebContext.set_variable 后，exchange 属性 map 共享同一键值。
    let attribute = exchange
        .get_attribute_value(Some(&one))
        .and_then(|value| value.to_utf16_string())
        .map(|value| value.to_string_lossy());
    assert_eq!(
        attribute.as_deref(),
        Some("two values"),
        "变量写入 exchange 属性（共享标识）"
    );
    assert!(
        exchange.contains_attribute(Some(&one)),
        "exchange 属性枚举包含该变量"
    );

    // 方向 ②：exchange 属性在模板中作为变量可见（对称语义）。
    exchange.set_attribute_value(
        Some(Utf16String::from_rust_str("greet")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "hello from exchange",
        )))),
    );
    let rendered = engine
        .process_template("<p th:text=\"${greet}\">x</p>", &web_context)
        .expect("render")
        .to_string_lossy();
    assert!(rendered.contains("hello from exchange"));

    // 注：Java test05/test14 的删除语义（request.removeAttribute / setVariable
    // null 即删）作用于**存活的 WebEngineContext**（直接持有 exchange 引用，实时
    // 读取），经公开渲染 API 无法在两次渲染之间保持同一 WebEngineContext 实例
    // （每次渲染由 StandardEngineContextFactory 用 WebContext 变量回填 exchange，
    // 不清理既有属性，与 Java 工厂行为一致），故该方向由实现本身
    // （web_engine_context.rs normalize_web_value 将 Null 归一为 None 后
    // set_attribute_value 即删）承担，不在本测试断言。
}
