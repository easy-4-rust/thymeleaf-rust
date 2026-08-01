//! `MarkupSelectorEngine` Java Golden 差分测试。
//!
//! 通过 `TemplateSpec` selectors + `th:fragment` 端到端覆盖：
//! id 简写、class 简写、元素名、属性选择器、组合选择器、
//! fragment 引用与 sibling 语义。

use std::sync::Arc;

use thymeleaf::context::Context;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::{
    ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode, TemplateSelectorSet,
    TemplateSpec,
};

fn engine() -> TemplateEngine {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

fn render_selector(html: &str, selector: &str) -> String {
    let mut selectors = TemplateSelectorSet::new();
    selectors.insert(Some(selector.to_owned()));
    let spec = TemplateSpec::with_selectors_and_template_mode(
        Some(html),
        Some(&selectors),
        Some(TemplateMode::HTML),
        None,
    )
    .expect("spec");
    engine()
        .process(&spec, &Context::new())
        .unwrap()
        .to_string_lossy()
}

fn render_fragment_selector(html: &str, selector: &str) -> String {
    // th:fragment 命名片段 + 选择器提取
    let mut selectors = TemplateSelectorSet::new();
    selectors.insert(Some(selector.to_owned()));
    let spec = TemplateSpec::with_selectors_and_template_mode(
        Some(html),
        Some(&selectors),
        Some(TemplateMode::HTML),
        None,
    )
    .expect("spec");
    engine()
        .process(&spec, &Context::new())
        .unwrap()
        .to_string_lossy()
}

// ===========================================================================
// 1. id 选择器（#id）
// ===========================================================================

#[test]
fn selector_by_id() {
    let html = "<div id=\"main\"><p>content</p></div><div id=\"other\">other</div>";
    let s = render_selector(html, "#main");
    assert!(s.contains("content"), "id selector: {s}");
    assert!(!s.contains("other"), "non-matching must be excluded: {s}");
}

// ===========================================================================
// 2. class 选择器（.class）
// ===========================================================================

#[test]
fn selector_by_class() {
    let html = "<div class=\"box\"><p>a</p></div><div class=\"other\"><p>b</p></div>";
    let s = render_selector(html, ".box");
    assert!(s.contains("<div class=\"box\">"), "class selector: {s}");
    assert!(!s.contains("other"), "non-matching class excluded: {s}");
}

// ===========================================================================
// 3. 元素名选择器
// ===========================================================================

#[test]
fn selector_by_element_name() {
    let html = "<section><p>para</p></section><div>div</div>";
    let s = render_selector(html, "section");
    assert!(s.contains("para"), "element selector: {s}");
    assert!(!s.contains("<div>"), "only section extracted: {s}");
}

// ===========================================================================
// 4. 属性选择器
// ===========================================================================

#[test]
fn selector_by_attribute() {
    let html = "<a href=\"/x\">link</a><a href=\"/y\">link2</a>";
    let s = render_selector(html, "a[href='/x']");
    assert!(s.contains("/x"), "attr selector: {s}");
    assert!(!s.contains("/y"), "non-matching attr excluded: {s}");
}

// ===========================================================================
// 5. 组合选择器（后代）
// ===========================================================================

#[test]
fn selector_descendant() {
    // Thymeleaf fragment selector 语法：// 表示任意层级
    let html = "<div><p><span>deep</span></p></div><p><span>shallow</span></p>";
    let s = render_selector(html, "//div//span");
    assert!(s.contains("deep"), "descendant: {s}");
    assert!(!s.contains("shallow"), "only div descendant: {s}");
}

// ===========================================================================
// 6. 子选择器（>）
// ===========================================================================

#[test]
fn selector_child() {
    // Thymeleaf 语法：/ 表示直接子级
    let html = "<div><p>direct</p><section><p>nested</p></section></div>";
    let s = render_selector(html, "//div/p");
    assert!(s.contains("direct"), "child selector: {s}");
    assert!(!s.contains("nested"), "only direct child: {s}");
}

// ===========================================================================
// 7. th:fragment 命名片段选择
// ===========================================================================

#[test]
fn fragment_selector_by_name() {
    let html = "<div th:fragment=\"header\"><h1>Header</h1></div><p>other</p>";
    let s = render_fragment_selector(html, "header");
    assert!(s.contains("Header"), "fragment name: {s}");
    assert!(!s.contains("other"), "non-fragment excluded: {s}");
}

// ===========================================================================
// 8. 不匹配选择器
// ===========================================================================

#[test]
fn selector_no_match_produces_empty() {
    let html = "<p>text</p>";
    let s = render_selector(html, "#nonexistent");
    // 无匹配 → 空输出
    assert!(!s.contains("text"), "no match must be empty: {s}");
}

// ===========================================================================
// 9. 多选择器
// ===========================================================================

#[test]
fn selector_multiple_via_set() {
    let html = "<div id=\"a\">A</div><div id=\"b\">B</div><div id=\"c\">C</div>";
    let mut selectors = TemplateSelectorSet::new();
    selectors.insert(Some("#a".to_owned()));
    selectors.insert(Some("#c".to_owned()));
    let spec = TemplateSpec::with_selectors_and_template_mode(
        Some(html),
        Some(&selectors),
        Some(TemplateMode::HTML),
        None,
    )
    .expect("spec");
    let s = engine()
        .process(&spec, &Context::new())
        .unwrap()
        .to_string_lossy();
    assert!(s.contains("A"), "first selector: {s}");
    assert!(s.contains("C"), "second selector: {s}");
    assert!(!s.contains("B"), "non-selected excluded: {s}");
}
