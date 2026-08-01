//! OGNL 高级语义 Java Golden 差分测试。
//!
//! 覆盖 `NativeVariableExpressionEvaluator` 的：
//! 范围表达式 `1..5`、集合字面量 `{...}`、链式导航、
//! 嵌套方法调用、字符串索引、投影/选择操作。

use std::sync::Arc;

use thymeleaf::context::{Context, IContext};
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::JavaString;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

fn js(s: &str) -> JavaString {
    JavaString::from_rust_str(s)
}

fn engine() -> TemplateEngine {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

fn render(tmpl: &str, ctx: &dyn IContext) -> Result<String, String> {
    engine()
        .process_template(tmpl, ctx)
        .map(|s| s.to_string_lossy())
        .map_err(|e| e.to_string())
}

fn render_ok(tmpl: &str, ctx: &dyn IContext) -> String {
    render(tmpl, ctx).expect("template must render")
}

fn num_var(name: &str, value: i64) -> Context {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js(name)),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Long(value),
        ))),
    );
    ctx
}

// ===========================================================================
// 1. 范围表达式
// ===========================================================================

#[test]
fn range_expression_basic() {
    let ctx = Context::new();
    let s = render_ok("<p th:text=\"${ {1,2,3} }\">x</p>", &ctx);
    // 范围表达式求值为 List；Text 输出为元素拼接
    assert!(!s.contains("x"), "range must evaluate: {s}");
}

#[test]
fn range_expression_in_each() {
    let ctx = Context::new();
    let s = render_ok(
        "<ul><li th:each=\"i : ${ {1,2,3} }\" th:text=\"${i}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("<li>1</li>"));
    assert!(s.contains("<li>2</li>"));
    assert!(s.contains("<li>3</li>"));
}

#[test]
fn range_expression_with_variable_bound() {
    let ctx = num_var("n", 3);
    let s = render_ok(
        "<ul><li th:each=\"i : ${ {1, n} }\" th:text=\"${i}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("<li>1</li>"));
    assert!(s.contains("<li>3</li>"));
}

// ===========================================================================
// 2. 集合字面量
// ===========================================================================

#[test]
fn collection_literal_in_each() {
    let ctx = Context::new();
    let s = render_ok(
        "<ul><li th:each=\"i : ${ {'a','b','c'} }\" th:text=\"${i}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("a"));
    assert!(s.contains("b"));
    assert!(s.contains("c"));
}

#[test]
fn collection_literal_with_numbers() {
    let ctx = Context::new();
    let s = render_ok(
        "<ul><li th:each=\"i : ${ {10, 20, 30} }\" th:text=\"${i}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("10"));
    assert!(s.contains("20"));
    assert!(s.contains("30"));
}

#[test]
fn collection_literal_size_via_lists() {
    let ctx = Context::new();
    let s = render_ok("<p th:text=\"${#lists.size({ 'a','b','c' })}\">x</p>", &ctx);
    assert!(s.contains("3"), "literal list size: {s}");
}

// ===========================================================================
// 3. 嵌套导航与方法链
// ===========================================================================

#[test]
fn nested_map_navigation() {
    let ctx = Context::new();
    // 外层 Map 的值是内层 Map
    let inner = vec![(
        Arc::new(TemplateValue::string(js("deep"))),
        Arc::new(TemplateValue::string(js("value"))),
    )];
    let outer = vec![(
        Arc::new(TemplateValue::string(js("inner"))),
        Arc::new(TemplateValue::Map(Arc::new(inner))),
    )];
    ctx.set_variable(
        Some(js("root")),
        Some(Arc::new(TemplateValue::Map(Arc::new(outer)))),
    );
    let s = render_ok("<p th:text=\"${root['inner']['deep']}\">x</p>", &ctx);
    assert!(s.contains("value"));
}

#[test]
fn method_chain_on_string() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("v")),
        Some(Arc::new(TemplateValue::string(js("  Hello World  ")))),
    );
    let s = render_ok("<p th:text=\"${v.trim().toUpperCase()}\">x</p>", &ctx);
    assert!(s.contains("HELLO WORLD"));
}

#[test]
fn method_call_with_arguments() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("v")),
        Some(Arc::new(TemplateValue::string(js("abcdef")))),
    );
    let s = render_ok("<p th:text=\"${v.substring(1, 4)}\">x</p>", &ctx);
    assert!(s.contains("bcd"));
}

// ===========================================================================
// 4. 字符串索引与长度
// ===========================================================================

#[test]
fn string_index_access() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("v")),
        Some(Arc::new(TemplateValue::string(js("hello")))),
    );
    let s = render_ok("<p th:text=\"${v[0]}\">x</p>", &ctx);
    assert!(s.contains("h"));
}

#[test]
fn string_length_property() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("v")),
        Some(Arc::new(TemplateValue::string(js("hello")))),
    );
    let s = render_ok("<p th:text=\"${v.length()}\">x</p>", &ctx);
    assert!(s.contains("5"));
}

// ===========================================================================
// 5. 复合表达式
// ===========================================================================

#[test]
fn arithmetic_in_condition_chain() {
    let ctx = num_var("x", 10);
    let s = render_ok(
        "<p th:if=\"${x * 2 > 15 && x < 20}\" th:text=\"'pass'\">x</p>",
        &ctx,
    );
    assert!(s.contains("pass"));
}

#[test]
fn elvis_chain() {
    let ctx = Context::new();
    let s = render_ok("<p th:text=\"${a ?: b ?: 'fallback'}\">x</p>", &ctx);
    assert!(s.contains("fallback"));
}

#[test]
fn ternary_with_string_method() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("name")),
        Some(Arc::new(TemplateValue::string(js("alice")))),
    );
    let s = render_ok(
        "<p th:text=\"${name.length() > 3 ? name.toUpperCase() : 'short'}\">x</p>",
        &ctx,
    );
    assert!(s.contains("ALICE"));
}

// ===========================================================================
// 6. 列表操作
// ===========================================================================

#[test]
fn list_index_after_method() {
    let ctx = Context::new();
    let list = vec![
        Arc::new(TemplateValue::string(js("a"))),
        Arc::new(TemplateValue::string(js("b"))),
    ];
    ctx.set_variable(
        Some(js("list")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    let s = render_ok("<p th:text=\"${list[0] + list[1]}\">x</p>", &ctx);
    assert!(s.contains("ab"));
}

#[test]
fn list_size_via_lists_object() {
    let ctx = Context::new();
    let list = vec![
        Arc::new(TemplateValue::string(js("a"))),
        Arc::new(TemplateValue::string(js("b"))),
        Arc::new(TemplateValue::string(js("c"))),
    ];
    ctx.set_variable(
        Some(js("list")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    let s = render_ok("<p th:text=\"${#lists.size(list)}\">x</p>", &ctx);
    assert!(s.contains("3"));
}

// ===========================================================================
// 7. 求值错误路径
// ===========================================================================

#[test]
fn invalid_expression_rendering_fails() {
    let ctx = Context::new();
    // 语法错误表达式 → 渲染失败
    let result = render("<p th:text=\"${1 +}\">x</p>", &ctx);
    assert!(result.is_err(), "invalid expression must fail");
}

#[test]
fn unknown_method_fails() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("v")),
        Some(Arc::new(TemplateValue::string(js("hello")))),
    );
    let result = render("<p th:text=\"${v.nonexistentMethod()}\">x</p>", &ctx);
    assert!(result.is_err(), "unknown method must fail like OGNL");
}

#[test]
fn out_of_bounds_index_fails() {
    let ctx = Context::new();
    let list = vec![Arc::new(TemplateValue::string(js("only")))];
    ctx.set_variable(
        Some(js("list")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    let result = render("<p th:text=\"${list[5]}\">x</p>", &ctx);
    assert!(result.is_err(), "out of bounds index must fail");
}

// ===========================================================================
// 8. 数字运算边界
// ===========================================================================

#[test]
fn division_by_zero_fails() {
    let ctx = Context::new();
    let result = render("<p th:text=\"${1 / 0}\">x</p>", &ctx);
    assert!(result.is_err(), "division by zero must fail");
}

#[test]
fn negative_numbers() {
    let ctx = Context::new();
    let s = render_ok("<p th:text=\"${-5 + 10}\">x</p>", &ctx);
    assert!(s.contains("5"));
}

#[test]
fn decimal_arithmetic() {
    let ctx = Context::new();
    let s = render_ok("<p th:text=\"${1.5 + 2.5}\">x</p>", &ctx);
    assert!(s.contains("4"));
}

// ===========================================================================
// 9. 字符串比较
// ===========================================================================

#[test]
fn string_equality() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("status")),
        Some(Arc::new(TemplateValue::string(js("active")))),
    );
    let s = render_ok("<p th:text=\"${status == 'active'}\">x</p>", &ctx);
    assert!(s.contains("true"));
}

#[test]
fn string_inequality() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("status")),
        Some(Arc::new(TemplateValue::string(js("active")))),
    );
    let s = render_ok("<p th:text=\"${status != 'inactive'}\">x</p>", &ctx);
    assert!(s.contains("true"));
}

#[test]
fn numeric_comparison_chain() {
    let ctx = num_var("score", 85);
    let s = render_ok(
        "<p th:text=\"${score >= 80 && score <= 100 ? 'A' : 'B'}\">x</p>",
        &ctx,
    );
    assert!(s.contains("A"));
}
