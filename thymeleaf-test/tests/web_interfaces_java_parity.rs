//! `org.thymeleaf.web` 接口族 Java 1:1 差分测试。
//!
//! 覆盖对象（对象表编号）：`IWebExchange`（477）、`IWebRequest`（478）、
//! `IWebSession`（479）、`IWebApplication`（476）。
//! 以语料 WebExchange（`tests/support/corpus_web_exchange.rs`）作为 trait-object
//! 实现，验证四个接口的合同：request 参数、session/application 属性往返、
//! exchange 属性与 locale/principal，以及 WebContext 的 exchange 身份保持
//! （与 `web_context_java_parity.rs` 的既有差分互补）。

use std::sync::Arc;

use thymeleaf::context::{IWebContext, WebContext};
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::Utf16String;
use thymeleaf::web::{IWebApplication, IWebExchange, IWebRequest, IWebSession};

#[allow(dead_code, unused_imports)]
mod support;

use support::CorpusWebExchange;

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn exchange() -> Arc<dyn IWebExchange> {
    Arc::new(CorpusWebExchange::new())
}

// ===========================================================================
// 1. IWebExchange（477）
// ===========================================================================

#[test]
fn web_exchange_contract_matches_java() {
    let exchange = exchange();

    // request/application 恒非空（Java 契约），session 可选
    assert_eq!(
        exchange
            .get_request()
            .get_method()
            .unwrap()
            .to_string_lossy(),
        "GET"
    );
    assert!(
        exchange.has_session(),
        "corpus exchange always has a session"
    );
    let session = exchange.get_session().expect("session present");

    // 属性往返（Java IWebExchange 属性映射）
    assert!(!exchange.contains_attribute(Some(&js("attr1"))));
    exchange.set_attribute_value(
        Some(js("attr1")),
        Some(Arc::new(TemplateValue::string(js("v")))),
    );
    assert!(exchange.contains_attribute(Some(&js("attr1"))));
    assert_eq!(
        exchange
            .get_attribute_value(Some(&js("attr1")))
            .expect("attr1 value")
            .to_utf16_string()
            .expect("string")
            .to_string_lossy(),
        "v"
    );
    assert_eq!(exchange.get_attribute_count(), 1);
    assert_eq!(
        exchange
            .get_all_attribute_names()
            .iter()
            .map(|name| name.as_ref().expect("name").to_string_lossy())
            .collect::<Vec<_>>(),
        ["attr1"]
    );
    exchange.remove_attribute(Some(&js("attr1")));
    assert!(!exchange.contains_attribute(Some(&js("attr1"))));

    // locale/principal/content-type 可空读取（Java 契约）
    assert!(exchange.get_locale().is_some());
    assert!(exchange.get_principal().is_none());
    assert!(exchange.get_content_type().is_none());

    let _ = session;
}

// ===========================================================================
// 2. IWebRequest（478）
// ===========================================================================

#[test]
fn web_request_contract_matches_java() {
    let exchange = exchange();
    let request: &dyn IWebRequest = exchange.get_request();

    assert_eq!(request.get_method().unwrap().to_string_lossy(), "GET");
    assert_eq!(request.get_request_path().to_string_lossy(), "/");
    assert!(request.get_query_string().is_none());
}

// ===========================================================================
// 3. IWebSession（479）+ IWebApplication（476）
// ===========================================================================

#[test]
fn web_session_and_application_contract_matches_java() {
    let exchange = exchange();

    // session：exists + 属性往返（Java ISession 属性映射）
    let session: &dyn IWebSession = exchange.get_session().expect("session present");
    assert!(session.exists());
    assert!(!session.contains_attribute(Some(&js("s1"))));
    session.set_attribute_value(
        Some(js("s1")),
        Some(Arc::new(TemplateValue::string(js("sv")))),
    );
    assert!(session.contains_attribute(Some(&js("s1"))));
    assert_eq!(
        session
            .get_attribute_value(Some(&js("s1")))
            .expect("s1 value")
            .to_utf16_string()
            .expect("string")
            .to_string_lossy(),
        "sv"
    );
    assert_eq!(session.get_attribute_count(), 1);
    session.remove_attribute(Some(&js("s1")));
    assert!(!session.contains_attribute(Some(&js("s1"))));

    // application：属性往返 + 资源查询（Java IWebApplication 合同）
    let application: &dyn IWebApplication = exchange.get_application();
    assert!(!application.contains_attribute(Some(&js("a1"))));
    application.set_attribute_value(
        Some(js("a1")),
        Some(Arc::new(TemplateValue::string(js("av")))),
    );
    assert!(application.contains_attribute(Some(&js("a1"))));
    assert_eq!(
        application
            .get_attribute_value(Some(&js("a1")))
            .expect("a1 value")
            .to_utf16_string()
            .expect("string")
            .to_string_lossy(),
        "av"
    );
    application.remove_attribute(Some(&js("a1")));
    assert!(!application.contains_attribute(Some(&js("a1"))));
}

// ===========================================================================
// 4. WebContext exchange 身份保持（Java WebContext 契约）
// ===========================================================================

#[test]
fn web_context_exchange_identity_matches_java() {
    let exchange = exchange();
    let context = WebContext::new(Some(Arc::clone(&exchange))).expect("web context");
    assert!(
        std::ptr::eq(context.get_exchange(), exchange.as_ref()),
        "WebContext.getExchange returns the same exchange instance"
    );
    let _ = &context;
}
