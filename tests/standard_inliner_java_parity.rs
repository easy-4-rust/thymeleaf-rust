//! `AbstractStandardInliner` / `StandardInlineMode` Java Golden 差分测试。
//!
//! 覆盖：th:inline 模式解析、HTML 内联 `[[...]]`/`[(...)]` 表达式、
//! TEXT 内联、JavaScript/CSS 内联模式、th:inline=none。

use std::sync::Arc;

use thymeleaf::context::{Context, IContext};
use thymeleaf::expression::TemplateValue;
use thymeleaf::inline::StandardInlineMode;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::JavaString;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

fn js(s: &str) -> JavaString {
    JavaString::from_rust_str(s)
}

fn engine_with_mode(mode: TemplateMode) -> TemplateEngine {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(mode);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

fn render(mode: TemplateMode, tmpl: &str, ctx: &dyn IContext) -> String {
    engine_with_mode(mode)
        .process_template(tmpl, ctx)
        .unwrap()
        .to_string_lossy()
}

// ===========================================================================
// 1. StandardInlineMode 解析
// ===========================================================================

#[test]
fn inline_mode_parse_valid() {
    assert_eq!(
        StandardInlineMode::parse(Some(&js("html"))).unwrap(),
        StandardInlineMode::HTML
    );
    assert_eq!(
        StandardInlineMode::parse(Some(&js("text"))).unwrap(),
        StandardInlineMode::TEXT
    );
    assert_eq!(
        StandardInlineMode::parse(Some(&js("javascript"))).unwrap(),
        StandardInlineMode::JAVASCRIPT
    );
    assert_eq!(
        StandardInlineMode::parse(Some(&js("css"))).unwrap(),
        StandardInlineMode::CSS
    );
    assert_eq!(
        StandardInlineMode::parse(Some(&js("none"))).unwrap(),
        StandardInlineMode::NONE
    );
}

#[test]
fn inline_mode_parse_null() {
    assert!(StandardInlineMode::parse(None).is_err());
}

#[test]
fn inline_mode_parse_unknown() {
    assert!(StandardInlineMode::parse(Some(&js("markdown"))).is_err());
}

// ===========================================================================
// 2. HTML 内联 [[...]] 转义输出
// ===========================================================================

#[test]
fn html_inline_double_bracket_escapes() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("name")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "<b>Alice</b>",
        )))),
    );
    // HTML 模式下 th:inline="html" 的 [[...]] 会转义
    let s = render(
        TemplateMode::HTML,
        "<p th:inline=\"html\">[[${name}]]</p>",
        &ctx,
    );
    assert!(s.contains("&lt;b&gt;Alice&lt;/b&gt;"));
}

// ===========================================================================
// 3. th:inline="none" 禁用处理
// ===========================================================================

#[test]
fn inline_none_preserves_expression() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("name")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "x",
        )))),
    );
    let s = render(
        TemplateMode::HTML,
        "<script th:inline=\"none\">var x = '${name}';</script>",
        &ctx,
    );
    assert!(s.contains("${name}"));
}

// ===========================================================================
// 4. TEXT 内联
// ===========================================================================

#[test]
fn text_inline_expression() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("msg")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "Hello",
        )))),
    );
    let s = render(TemplateMode::TEXT, "[(${msg})] world", &ctx);
    assert!(s.contains("Hello world"));
}

#[test]
fn text_inline_escaped() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("msg")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "a<b",
        )))),
    );
    let s = render(TemplateMode::TEXT, "[[${msg}]]", &ctx);
    assert!(s.contains("a&lt;b"));
}

// ===========================================================================
// 5. JavaScript 内联
// ===========================================================================

#[test]
fn javascript_inline_literal() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("num")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(42),
        ))),
    );
    let s = render(TemplateMode::JAVASCRIPT, "var n = [[${num}]];", &ctx);
    assert!(s.contains("42"));
}

#[test]
fn javascript_inline_string() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("name")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "Alice",
        )))),
    );
    let s = render(TemplateMode::JAVASCRIPT, "var n = [[${name}]];", &ctx);
    assert!(s.contains("Alice"));
}

// ===========================================================================
// 6. CSS 内联
// ===========================================================================

#[test]
fn css_inline_number() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("w")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(100),
        ))),
    );
    let s = render(TemplateMode::CSS, "width: [[${w}]]px;", &ctx);
    assert!(s.contains("100"));
}

// ===========================================================================
// 7. th:inline 与 th:each 组合
// ===========================================================================

#[test]
fn inline_with_each() {
    let ctx = Context::new();
    let list = vec![
        Arc::new(TemplateValue::string(JavaString::from_rust_str("a"))),
        Arc::new(TemplateValue::string(JavaString::from_rust_str("b"))),
    ];
    ctx.set_variable(
        Some(JavaString::from_rust_str("items")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    let s = render(
        TemplateMode::HTML,
        "<div th:each=\"i:${items}\" th:inline=\"text\">[(${i})]</div>",
        &ctx,
    );
    assert!(s.contains("a"));
    assert!(s.contains("b"));
}

// ===========================================================================
// 8. 普通 HTML 输出不受影响
// ===========================================================================

#[test]
fn html_default_inlining_enabled() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("name")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "Alice",
        )))),
    );
    // Thymeleaf 3 中 HTML 模式默认启用内联：[[...]] 会被求值
    let s = render(TemplateMode::HTML, "<p>[[${name}]]</p>", &ctx);
    assert!(s.contains("Alice"));
    assert!(!s.contains("[[${name}]]"));
}
