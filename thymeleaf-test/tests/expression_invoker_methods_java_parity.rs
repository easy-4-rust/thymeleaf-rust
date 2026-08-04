//! `StandardExpressionObjectInvoker` 方法分派 Java Golden 差分测试。
//!
//! 通过 `#` 表达式对象端到端覆盖：`#strings` 字符串方法族、
//! `#arrays`/`#lists`/`#sets` 集合方法族、`#maps` 映射方法族、
//! `#objects` 对象方法族、`#bools` 布尔方法族、`#uris` URI 方法族。

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

fn str_var(name: &str, value: &str) -> Context {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js(name)),
        Some(Arc::new(TemplateValue::string(js(value)))),
    );
    ctx
}

fn list_var(name: &str, values: &[&str]) -> Context {
    let ctx = Context::new();
    let list = values
        .iter()
        .map(|v| Arc::new(TemplateValue::string(js(v))))
        .collect();
    ctx.set_variable(
        Some(js(name)),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    ctx
}

fn bool_var(name: &str, value: bool) -> Context {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js(name)),
        Some(Arc::new(TemplateValue::Boolean(value))),
    );
    ctx
}

// ===========================================================================
// 1. #strings 方法族
// ===========================================================================

#[test]
fn strings_abbreviate() {
    let ctx = str_var("v", "hello world this is long");
    let s = render("<p th:text=\"${#strings.abbreviate(v, 10)}\">x</p>", &ctx);
    assert!(s.contains("..."), "abbreviate must add ellipsis: {s}");
}

#[test]
fn strings_capitalize() {
    let ctx = str_var("v", "hello");
    let s = render("<p th:text=\"${#strings.capitalize(v)}\">x</p>", &ctx);
    assert!(s.contains("Hello"));
}

#[test]
fn strings_lower_upper() {
    let ctx = str_var("v", "Hello World");
    let s = render("<p th:text=\"${#strings.toLowerCase(v)}\">x</p>", &ctx);
    assert!(s.contains("hello world"));
}

#[test]
fn strings_trim() {
    let ctx = str_var("v", "  spaced  ");
    let s = render("<p th:text=\"${#strings.trim(v)}\">x</p>", &ctx);
    assert!(s.contains("spaced"));
}

#[test]
fn strings_join() {
    let ctx = list_var("l", &["a", "b", "c"]);
    let s = render("<p th:text=\"${#strings.listJoin(l, ',')}\">x</p>", &ctx);
    assert!(s.contains("a,b,c"));
}

#[test]
fn strings_split() {
    let ctx = str_var("v", "a,b,c");
    let s = render(
        "<p th:text=\"${#strings.listJoin(#strings.listSplit(v, ','), '-')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("a-b-c"));
}

#[test]
fn strings_replace() {
    let ctx = str_var("v", "hello world");
    let s = render(
        "<p th:text=\"${#strings.replace(v, 'world', 'rust')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("hello rust"));
}

#[test]
fn strings_index_of() {
    let ctx = str_var("v", "hello world");
    let s = render("<p th:text=\"${#strings.indexOf(v, 'world')}\">x</p>", &ctx);
    assert!(s.contains("6"));
}

#[test]
fn strings_substring_before_after() {
    let ctx = str_var("v", "hello.world");
    let s = render(
        "<p th:text=\"${#strings.substringBefore(v, '.')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("hello"));
    let s2 = render(
        "<p th:text=\"${#strings.substringAfter(v, '.')}\">x</p>",
        &ctx,
    );
    assert!(s2.contains("world"));
}

#[test]
fn strings_length() {
    let ctx = str_var("v", "hello");
    let s = render("<p th:text=\"${#strings.length(v)}\">x</p>", &ctx);
    assert!(s.contains("5"));
}

// ===========================================================================
// 2. #arrays 方法族
// ===========================================================================

#[test]
fn arrays_length() {
    let ctx = list_var("a", &["x", "y", "z"]);
    let s = render("<p th:text=\"${#arrays.length(a)}\">x</p>", &ctx);
    assert!(s.contains("3"));
}

#[test]
fn arrays_contains() {
    let ctx = list_var("a", &["x", "y"]);
    let s = render("<p th:text=\"${#arrays.contains(a, 'x')}\">x</p>", &ctx);
    assert!(s.contains("true"));
}

// ===========================================================================
// 3. #lists 方法族
// ===========================================================================

#[test]
fn lists_size_contains() {
    let ctx = list_var("l", &["a", "b"]);
    assert!(render("<p th:text=\"${#lists.size(l)}\">x</p>", &ctx).contains("2"));
    assert!(render("<p th:text=\"${#lists.contains(l, 'a')}\">x</p>", &ctx).contains("true"));
}

#[test]
fn lists_contains_all() {
    let ctx = Context::new();
    let list = vec![
        Arc::new(TemplateValue::string(js("a"))),
        Arc::new(TemplateValue::string(js("b"))),
    ];
    ctx.set_variable(
        Some(js("l")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    let s = render(
        "<p th:text=\"${#lists.containsAll(l, {'a','b'})}\">x</p>",
        &ctx,
    );
    assert!(s.contains("true"));
}

#[test]
fn lists_is_empty() {
    let ctx = list_var("l", &[]);
    let s = render("<p th:text=\"${#lists.isEmpty(l)}\">x</p>", &ctx);
    assert!(s.contains("true"));
}

// ===========================================================================
// 4. #sets 方法族
// ===========================================================================

#[test]
fn sets_size() {
    let ctx = list_var("s", &["a", "b", "c"]);
    let s = render("<p th:text=\"${#sets.size(s)}\">x</p>", &ctx);
    assert!(s.contains("3"));
}

#[test]
fn sets_contains() {
    let ctx = list_var("s", &["a", "b"]);
    let s = render("<p th:text=\"${#sets.contains(s, 'a')}\">x</p>", &ctx);
    assert!(s.contains("true"));
}

// ===========================================================================
// 5. #maps 方法族
// ===========================================================================

#[test]
fn maps_size() {
    let ctx = Context::new();
    let map = vec![(
        Arc::new(TemplateValue::string(js("k"))),
        Arc::new(TemplateValue::string(js("v"))),
    )];
    ctx.set_variable(
        Some(js("m")),
        Some(Arc::new(TemplateValue::Map(Arc::new(map)))),
    );
    let s = render("<p th:text=\"${#maps.size(m)}\">x</p>", &ctx);
    assert!(s.contains("1"));
}

#[test]
fn maps_contains_key() {
    let ctx = Context::new();
    let map = vec![(
        Arc::new(TemplateValue::string(js("k1"))),
        Arc::new(TemplateValue::string(js("v1"))),
    )];
    ctx.set_variable(
        Some(js("m")),
        Some(Arc::new(TemplateValue::Map(Arc::new(map)))),
    );
    let s = render("<p th:text=\"${#maps.containsKey(m, 'k1')}\">x</p>", &ctx);
    assert!(s.contains("true"));
}

// ===========================================================================
// 6. #objects 方法族
// ===========================================================================

#[test]
fn objects_null_check() {
    let ctx = Context::new();
    let s = render(
        "<p th:text=\"${#objects.nullSafe(missing, 'default')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("default"));
}

#[test]
fn objects_null_safe_with_list_containing_null() {
    // #objects 表达式对象只暴露 nullSafe；null 出现在列表中时被替换
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("l")),
        Some(Arc::new(TemplateValue::List(Arc::new(vec![
            Arc::new(TemplateValue::Null),
            Arc::new(TemplateValue::string(js("keep"))),
        ])))),
    );
    let s = render(
        "<p th:text=\"${#objects.nullSafe(l, 'default')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("default") && s.contains("keep"));
}

// ===========================================================================
// 7. #bools 方法族
// ===========================================================================

#[test]
fn bools_is_true_false() {
    let ctx = bool_var("t", true);
    let ctx2 = bool_var("f", false);
    assert!(render("<p th:text=\"${#bools.isTrue(t)}\">x</p>", &ctx).contains("true"));
    assert!(render("<p th:text=\"${#bools.isFalse(f)}\">x</p>", &ctx2).contains("true"));
}

#[test]
fn bools_and_or() {
    let ctx = bool_var("a", true);
    let s = render("<p th:text=\"${#bools.listAnd({a, a})}\">x</p>", &ctx);
    assert!(s.contains("true"));
}

// ===========================================================================
// 8. #uris 方法族
// ===========================================================================

#[test]
fn uris_escape() {
    let ctx = str_var("v", "a b&c");
    let s = render("<p th:text=\"${#uris.escapePathSegment(v)}\">x</p>", &ctx);
    assert!(s.contains("a%20b"), "space must be percent-encoded: {s}");
}

// ===========================================================================
// 9. #calendars / #temporals 基础
// ===========================================================================

#[test]
fn temporals_create_and_format() {
    let ctx = Context::new();
    let s = render(
        "<p th:text=\"${#temporals.format(#temporals.create(2024,5,17), 'yyyy-MM-dd')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("2024-05-17"), "temporal format: {s}");
}

// ===========================================================================
// 10. 组合表达式
// ===========================================================================

#[test]
fn strings_and_bools_composition() {
    let ctx = str_var("v", "");
    let s = render(
        "<p th:if=\"${#strings.isEmpty(v)}\" th:text=\"'empty'\">x</p>",
        &ctx,
    );
    assert!(s.contains("empty"));
}

#[test]
fn strings_method_in_ternary() {
    let ctx = str_var("v", "alice");
    let s = render(
        "<p th:text=\"${#strings.length(v) > 3 ? 'long' : 'short'}\">x</p>",
        &ctx,
    );
    assert!(s.contains("long"));
}
