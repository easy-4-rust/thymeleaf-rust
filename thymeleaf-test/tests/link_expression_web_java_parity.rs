//! `LinkExpression`（@{}）在 Web 上下文下的 Java Golden 差分测试。
//!
//! 覆盖：context-relative 链接、查询参数、多参数、th:attr 组合。
// 共享 Web corpus 服务于完整 Web SPI 批次；本批次只消费 exchange 身份语义。
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

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
}

fn web_engine() -> TemplateEngine {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

fn render_web(tmpl: &str, exchange: Arc<dyn IWebExchange>) -> String {
    let ctx = WebContext::new(Some(exchange)).expect("web context");
    web_engine()
        .process_template(tmpl, &ctx)
        .unwrap()
        .to_string_lossy()
}

// ===========================================================================
// 1. @{} 链接表达式（Web 上下文）
// ===========================================================================

#[test]
fn link_basic_url() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let s = render_web("<a th:href=\"@{/products}\">link</a>", exchange);
    assert!(s.contains("/products"), "link href: {s}");
}

#[test]
fn link_with_literal_param() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let s = render_web("<a th:href=\"@{/product(id='42')}\">link</a>", exchange);
    assert!(s.contains("id=42"), "link param: {s}");
}

#[test]
fn link_with_variable_param() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    exchange.set_attribute_value(
        Some(js("product_id")),
        Some(Arc::new(TemplateValue::string(js("99")))),
    );
    let s = render_web(
        "<a th:href=\"@{/product(id=${product_id})}\">link</a>",
        exchange,
    );
    assert!(s.contains("id=99"), "variable param: {s}");
}

#[test]
fn link_multiple_params() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let s = render_web(
        "<a th:href=\"@{/search(q='rust',page=2)}\">link</a>",
        exchange,
    );
    assert!(s.contains("q=rust"), "first param: {s}");
    assert!(s.contains("page=2"), "second param: {s}");
}

#[test]
fn link_in_th_attr() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let s = render_web("<a th:attr=\"href=@{/products}\">x</a>", exchange);
    assert!(s.contains("/products"), "th:attr link: {s}");
}

#[test]
fn link_absolute_url() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let s = render_web("<a th:href=\"@{https://example.com/x}\">x</a>", exchange);
    assert!(s.contains("https://example.com/x"), "absolute url: {s}");
}

// ===========================================================================
// 2. @{} 在非 Web 上下文失败（与 Java 一致）
// ===========================================================================

#[test]
fn link_without_web_exchange_fails() {
    use thymeleaf::context::Context;
    let ctx = Context::new();
    let result = web_engine().process_template("<a th:href=\"@{/products}\">x</a>", &ctx);
    assert!(
        result.is_err(),
        "link expression without web exchange must fail like Java"
    );
}
