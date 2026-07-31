//! Standard Expression 对象 (`#strings`, `#bools`, `#arrays`, `#lists`, `#sets`, `#maps`, `#objects`)
//! 通过模板引擎公共 API 的 Java Golden 差分测试。
//!
//! 覆盖 `standard_expression_object_invoker.rs` 的核心路径。

use std::sync::Arc;

use thymeleaf::context::{Context, IContext};
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::JavaString;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

fn create_engine() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    engine
}

fn render(template: &str, ctx: &dyn IContext) -> String {
    create_engine()
        .process_template(template, ctx)
        .unwrap()
        .to_string_lossy()
}

// ===========================================================================
// 1. #strings 表达式对象
// ===========================================================================

#[test]
fn strings_is_empty_with_empty_string() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("val")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "",
        )))),
    );
    let s = render("<p th:text=\"${#strings.isEmpty(val)}\">x</p>", &ctx);
    assert!(s.contains("true"));
}

#[test]
fn strings_is_empty_with_non_empty() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("val")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "hello",
        )))),
    );
    let s = render("<p th:text=\"${#strings.isEmpty(val)}\">x</p>", &ctx);
    assert!(s.contains("false"));
}

#[test]
fn strings_contains() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("val")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "hello world",
        )))),
    );
    let s = render(
        "<p th:text=\"${#strings.contains(val, 'world')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("true"));
}

#[test]
fn strings_starts_with() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("val")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "hello world",
        )))),
    );
    let s = render(
        "<p th:text=\"${#strings.startsWith(val, 'hello')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("true"));
}

#[test]
fn strings_ends_with() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("val")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "hello world",
        )))),
    );
    let s = render(
        "<p th:text=\"${#strings.endsWith(val, 'world')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("true"));
}

// ===========================================================================
// 2. #bools 表达式对象
// ===========================================================================

#[test]
fn bools_is_true() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("val")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    let s = render("<p th:text=\"${#bools.isTrue(val)}\">x</p>", &ctx);
    assert!(s.contains("true"));
}

#[test]
fn bools_is_false() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("val")),
        Some(Arc::new(TemplateValue::Boolean(false))),
    );
    let s = render("<p th:text=\"${#bools.isFalse(val)}\">x</p>", &ctx);
    assert!(s.contains("true"));
}

// ===========================================================================
// 3. #arrays 表达式对象
// ===========================================================================

#[test]
fn arrays_length() {
    let ctx = Context::new();
    let arr = vec![
        Arc::new(TemplateValue::string(JavaString::from_rust_str("a"))),
        Arc::new(TemplateValue::string(JavaString::from_rust_str("b"))),
        Arc::new(TemplateValue::string(JavaString::from_rust_str("c"))),
    ];
    ctx.set_variable(
        Some(JavaString::from_rust_str("arr")),
        Some(Arc::new(TemplateValue::List(Arc::new(arr)))),
    );
    let s = render("<p th:text=\"${#arrays.length(arr)}\">x</p>", &ctx);
    assert!(s.contains("3"));
}

// ===========================================================================
// 4. #lists 表达式对象
// ===========================================================================

#[test]
fn lists_size() {
    let ctx = Context::new();
    let list = vec![
        Arc::new(TemplateValue::string(JavaString::from_rust_str("a"))),
        Arc::new(TemplateValue::string(JavaString::from_rust_str("b"))),
    ];
    ctx.set_variable(
        Some(JavaString::from_rust_str("list")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    let s = render("<p th:text=\"${#lists.size(list)}\">x</p>", &ctx);
    assert!(s.contains("2"));
}

#[test]
fn lists_contains() {
    let ctx = Context::new();
    let list = vec![
        Arc::new(TemplateValue::string(JavaString::from_rust_str("a"))),
        Arc::new(TemplateValue::string(JavaString::from_rust_str("b"))),
    ];
    ctx.set_variable(
        Some(JavaString::from_rust_str("list")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    ctx.set_variable(
        Some(JavaString::from_rust_str("elem")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "a",
        )))),
    );
    let s = render("<p th:text=\"${#lists.contains(list, elem)}\">x</p>", &ctx);
    assert!(s.contains("true"));
}

// ===========================================================================
// 5. #sets 表达式对象
// ===========================================================================

#[test]
fn sets_size() {
    let ctx = Context::new();
    let set = vec![
        Arc::new(TemplateValue::string(JavaString::from_rust_str("x"))),
        Arc::new(TemplateValue::string(JavaString::from_rust_str("y"))),
        Arc::new(TemplateValue::string(JavaString::from_rust_str("z"))),
    ];
    ctx.set_variable(
        Some(JavaString::from_rust_str("set")),
        Some(Arc::new(TemplateValue::List(Arc::new(set)))),
    );
    let s = render("<p th:text=\"${#sets.size(set)}\">x</p>", &ctx);
    assert!(s.contains("3"));
}

// ===========================================================================
// 6. #maps 表达式对象
// ===========================================================================

#[test]
fn maps_size() {
    let ctx = Context::new();
    let map = vec![
        (
            Arc::new(TemplateValue::string(JavaString::from_rust_str("k1"))),
            Arc::new(TemplateValue::string(JavaString::from_rust_str("v1"))),
        ),
        (
            Arc::new(TemplateValue::string(JavaString::from_rust_str("k2"))),
            Arc::new(TemplateValue::string(JavaString::from_rust_str("v2"))),
        ),
    ];
    ctx.set_variable(
        Some(JavaString::from_rust_str("map")),
        Some(Arc::new(TemplateValue::Map(Arc::new(map)))),
    );
    let s = render("<p th:text=\"${#maps.size(map)}\">x</p>", &ctx);
    assert!(s.contains("2"));
}

// ===========================================================================
// 7. #aggregates 表达式对象
// ===========================================================================

#[test]
fn aggregates_sum() {
    let ctx = Context::new();
    let list = vec![
        Arc::new(TemplateValue::Number(thymeleaf::util::JavaNumber::Integer(
            1,
        ))),
        Arc::new(TemplateValue::Number(thymeleaf::util::JavaNumber::Integer(
            2,
        ))),
        Arc::new(TemplateValue::Number(thymeleaf::util::JavaNumber::Integer(
            3,
        ))),
    ];
    ctx.set_variable(
        Some(JavaString::from_rust_str("nums")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    let s = render("<p th:text=\"${#aggregates.sum(nums)}\">x</p>", &ctx);
    assert!(s.contains("6"));
}

// ===========================================================================
// 8. 条件渲染与表达式对象组合
// ===========================================================================

#[test]
fn th_if_with_strings_is_empty() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("val")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "",
        )))),
    );
    let s = render(
        "<p th:if=\"${#strings.isEmpty(val)}\" th:text=\"'empty'\">x</p>",
        &ctx,
    );
    assert!(s.contains("empty"));
}

#[test]
fn th_if_with_strings_is_not_empty() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("val")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "hello",
        )))),
    );
    let s = render(
        "<p th:if=\"${!#strings.isEmpty(val)}\" th:text=\"'not empty'\">x</p>",
        &ctx,
    );
    assert!(s.contains("not empty"));
}

// ===========================================================================
// 9. 迭代与表达式对象组合
// ===========================================================================

#[test]
fn th_each_with_lists() {
    let ctx = Context::new();
    let list = vec![
        Arc::new(TemplateValue::string(JavaString::from_rust_str("a"))),
        Arc::new(TemplateValue::string(JavaString::from_rust_str("b"))),
        Arc::new(TemplateValue::string(JavaString::from_rust_str("c"))),
    ];
    ctx.set_variable(
        Some(JavaString::from_rust_str("items")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    let s = render(
        "<ul><li th:each=\"item : ${items}\" th:text=\"${item}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("a"));
    assert!(s.contains("b"));
    assert!(s.contains("c"));
}

// ===========================================================================
// 10. 复杂表达式组合
// ===========================================================================

#[test]
fn complex_expression_with_ternary_and_strings() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(JavaString::from_rust_str("name")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "Alice",
        )))),
    );
    let s = render(
        "<p th:text=\"${#strings.isEmpty(name) ? 'Anonymous' : name}\">x</p>",
        &ctx,
    );
    assert!(s.contains("Alice"));
    assert!(!s.contains("Anonymous"));
}

#[test]
fn complex_expression_with_elvis_and_strings() {
    let ctx = Context::new();
    // name is null
    let s = render(
        "<p th:text=\"${#strings.isEmpty(name) ? 'default' : name}\">x</p>",
        &ctx,
    );
    assert!(s.contains("default"));
}
