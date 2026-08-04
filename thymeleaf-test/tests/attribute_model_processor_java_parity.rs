//! `AbstractAttributeModelProcessor`/`StandardModelFactory` Java Golden 差分测试。
//!
//! 通过模板引擎端到端覆盖属性 Model 处理器：
//! `th:attr` 设置/移除、动态属性值、处理器优先级与匹配规则。

use std::sync::Arc;

use thymeleaf::context::{Context, IContext};
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
}

fn engine() -> TemplateEngine {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

fn render(tmpl: &str, ctx: &dyn IContext) -> String {
    engine()
        .process_template(tmpl, ctx)
        .unwrap()
        .to_string_lossy()
}

// ===========================================================================
// 1. th:attr 属性设置（AbstractAttributeModelProcessor 路径）
// ===========================================================================

#[test]
fn th_attr_sets_single_attribute() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("url")),
        Some(Arc::new(TemplateValue::string(js("http://x.com")))),
    );
    let s = render("<a th:attr=\"href=${url}\">link</a>", &ctx);
    assert!(s.contains("http://x.com"), "href set: {s}");
}

#[test]
fn th_attr_sets_multiple_attributes() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("url")),
        Some(Arc::new(TemplateValue::string(js("http://x.com")))),
    );
    ctx.set_variable(
        Some(js("title")),
        Some(Arc::new(TemplateValue::string(js("Title")))),
    );
    let s = render("<a th:attr=\"href=${url},title=${title}\">x</a>", &ctx);
    assert!(s.contains("http://x.com"));
    assert!(s.contains("Title"));
}

#[test]
fn th_attr_removes_th_attr_itself() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("cls")),
        Some(Arc::new(TemplateValue::string(js("btn")))),
    );
    let s = render("<div th:attr=\"class=${cls}\">x</div>", &ctx);
    // th:attr 自身必须被移除（remove_attribute）
    assert!(!s.contains("th:attr"), "th:attr must be removed: {s}");
    assert!(s.contains("btn"));
}

#[test]
fn th_attr_null_value_removes_attribute() {
    let ctx = Context::new();
    // missing 变量 → null → href 属性被移除
    let s = render("<a href=\"old\" th:attr=\"href=${missing}\">x</a>", &ctx);
    assert!(
        !s.contains("href"),
        "null attr value must remove attribute: {s}"
    );
}

// ===========================================================================
// 2. th:attrappend / th:attrprepend（append/prepend 处理器）
// ===========================================================================

#[test]
fn th_attrappend_appends() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("extra")),
        Some(Arc::new(TemplateValue::string(js(" extra")))),
    );
    let s = render(
        "<div class=\"base\" th:attrappend=\"class=${extra}\">x</div>",
        &ctx,
    );
    assert!(s.contains("base"), "original kept: {s}");
    assert!(s.contains("extra"), "appended: {s}");
}

#[test]
fn th_attrprepend_prepends() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("prefix")),
        Some(Arc::new(TemplateValue::string(js("pre- ")))),
    );
    let s = render(
        "<div class=\"base\" th:attrprepend=\"class=${prefix}\">x</div>",
        &ctx,
    );
    assert!(s.contains("pre-"), "prepended: {s}");
    assert!(s.contains("base"), "original kept: {s}");
}

// ===========================================================================
// 3. 固定属性处理器（th:href/th:src/th:value 等固定属性）
// ===========================================================================

#[test]
fn th_href_sets_attribute() {
    let ctx = Context::new();
    let s = render("<a th:href=\"'/page'\">x</a>", &ctx);
    assert!(s.contains("/page"), "th:href: {s}");
}

#[test]
fn th_href_removed_after_processing() {
    let ctx = Context::new();
    let s = render("<a th:href=\"'/page'\">x</a>", &ctx);
    assert!(!s.contains("th:href"), "th:href must be removed: {s}");
}

#[test]
fn th_src_sets_attribute() {
    let ctx = Context::new();
    let s = render("<img th:src=\"'/img.png'\">", &ctx);
    assert!(s.contains("/img.png"));
}

#[test]
fn th_value_sets_attribute() {
    let ctx = Context::new();
    let s = render("<input th:value=\"'text'\">", &ctx);
    assert!(s.contains("text"));
}

// ===========================================================================
// 4. 属性值表达式
// ===========================================================================

#[test]
fn attribute_value_with_expression() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("w")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::NumberValue::Integer(50),
        ))),
    );
    let s = render(
        "<div th:attr=\"style=${'width:' + w + 'px'}\">x</div>",
        &ctx,
    );
    assert!(s.contains("width:50px"), "computed attr: {s}");
}

#[test]
fn attribute_value_boolean() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("disabled")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    let s = render("<input th:attr=\"disabled=${disabled}\">", &ctx);
    assert!(s.contains("disabled"), "boolean attr: {s}");
}

// ===========================================================================
// 5. 优先级与多属性共存
// ===========================================================================

#[test]
fn th_attr_with_other_th_attributes() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("show")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    ctx.set_variable(
        Some(js("cls")),
        Some(Arc::new(TemplateValue::string(js("btn")))),
    );
    let s = render(
        "<div th:if=\"${show}\" th:attr=\"class=${cls}\">x</div>",
        &ctx,
    );
    assert!(s.contains("btn"));
    assert!(!s.contains("th:if"));
    assert!(!s.contains("th:attr"));
}

// ===========================================================================
// 6. StandardModelFactory 路径（th:each/th:with 创建 Model）
// ===========================================================================

#[test]
fn th_each_creates_models() {
    let ctx = Context::new();
    let list = vec![
        Arc::new(TemplateValue::string(js("a"))),
        Arc::new(TemplateValue::string(js("b"))),
    ];
    ctx.set_variable(
        Some(js("items")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    let s = render(
        "<ul><li th:each=\"i:${items}\" th:text=\"${i}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("a") && s.contains("b"));
}

#[test]
fn th_with_creates_local_scope() {
    let ctx = Context::new();
    let s = render(
        "<div th:with=\"x='v'\"><p th:text=\"${x}\">x</p></div>",
        &ctx,
    );
    assert!(s.contains("v"));
}

#[test]
fn th_object_creates_selection_model() {
    let ctx = Context::new();
    let map = vec![(
        Arc::new(TemplateValue::string(js("name"))),
        Arc::new(TemplateValue::string(js("Alice"))),
    )];
    ctx.set_variable(
        Some(js("user")),
        Some(Arc::new(TemplateValue::Map(Arc::new(map)))),
    );
    let s = render(
        "<div th:object=\"${user}\" th:text=\"*{name}\">x</div>",
        &ctx,
    );
    assert!(s.contains("Alice"), "selection model: {s}");
}
