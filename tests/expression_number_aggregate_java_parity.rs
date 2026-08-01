//! `#numbers`/`#aggregates`/`#dates`/`#calendars` 表达式对象 Java Golden 差分测试。
//!
//! 覆盖 StandardExpressionObjectInvoker 的数值、聚合、日期/日历方法分派。
#![allow(clippy::approx_constant)]

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

fn render(tmpl: &str, ctx: &dyn IContext) -> String {
    engine()
        .process_template(tmpl, ctx)
        .unwrap()
        .to_string_lossy()
}

fn num_list_var(name: &str, values: &[i64]) -> Context {
    let ctx = Context::new();
    let list = values
        .iter()
        .map(|v| Arc::new(TemplateValue::Number(thymeleaf::util::JavaNumber::Long(*v))))
        .collect();
    ctx.set_variable(
        Some(js(name)),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    ctx
}

// ===========================================================================
// 1. #aggregates 聚合方法族
// ===========================================================================

#[test]
fn aggregates_sum() {
    let ctx = num_list_var("nums", &[1, 2, 3]);
    let s = render("<p th:text=\"${#aggregates.sum(nums)}\">x</p>", &ctx);
    assert!(s.contains("6"), "sum: {s}");
}

#[test]
fn aggregates_avg() {
    let ctx = num_list_var("nums", &[2, 4, 6]);
    let s = render("<p th:text=\"${#aggregates.avg(nums)}\">x</p>", &ctx);
    assert!(s.contains("4"), "avg: {s}");
}

// ===========================================================================
// 2. #numbers 数值方法族
// ===========================================================================

#[test]
fn numbers_format_integer_min_digits() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("n")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(5),
        ))),
    );
    // formatInteger(target, minIntegerDigits)：补零到最少位数
    let s = render("<p th:text=\"${#numbers.formatInteger(n, 3)}\">x</p>", &ctx);
    assert!(s.contains("005"), "formatInteger min digits: {s}");
}

#[test]
fn numbers_format_integer_with_thousands() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("n")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(12345),
        ))),
    );
    // formatInteger(target, min, thousandsPointType)：COMMA 千位分隔
    let s = render(
        "<p th:text=\"${#numbers.formatInteger(n, 3, 'COMMA')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("12,345"), "formatInteger thousands: {s}");
}

#[test]
fn numbers_format_decimal() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("n")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Double(3.14159),
        ))),
    );
    let s = render(
        "<p th:text=\"${#numbers.formatDecimal(n, 1, 2)}\">x</p>",
        &ctx,
    );
    assert!(s.contains("3.14"), "formatDecimal: {s}");
}

#[test]
fn numbers_integer_sequence() {
    let ctx = Context::new();
    // #numbers.sequence(from, to)：生成范围序列
    let s = render(
        "<ul><li th:each=\"n : ${#numbers.sequence(1, 3)}\" th:text=\"${n}\">x</li></ul>",
        &ctx,
    );
    assert!(
        s.contains("1") && s.contains("2") && s.contains("3"),
        "sequence: {s}"
    );
}

// ===========================================================================
// 3. #dates 日期方法族
// ===========================================================================

#[test]
fn dates_format() {
    let ctx = Context::new();
    // #dates.create(year, month, day) → format(yyyy)
    let s = render(
        "<p th:text=\"${#dates.format(#dates.create(2024, 5, 17), 'yyyy')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("2024"), "dates.format: {s}");
}

#[test]
fn dates_components() {
    let ctx = Context::new();
    let s = render(
        "<p th:text=\"${#dates.year(#dates.create(2024, 5, 17))}\">x</p>",
        &ctx,
    );
    assert!(s.contains("2024"), "dates.year: {s}");
}

// ===========================================================================
// 4. #calendars 日历方法族
// ===========================================================================

#[test]
fn calendars_format() {
    let ctx = Context::new();
    let s = render(
        "<p th:text=\"${#calendars.format(#calendars.create(2024, 5, 17), 'yyyy-MM')}\">x</p>",
        &ctx,
    );
    assert!(s.contains("2024-05"), "calendars.format: {s}");
}

#[test]
fn calendars_year_month() {
    let ctx = Context::new();
    let s = render(
        "<p th:text=\"${#calendars.year(#calendars.create(2023, 12, 25))}\">x</p>",
        &ctx,
    );
    assert!(s.contains("2023"), "calendars.year: {s}");
    let s2 = render(
        "<p th:text=\"${#calendars.month(#calendars.create(2023, 12, 25))}\">x</p>",
        &ctx,
    );
    assert!(s2.contains("12"), "calendars.month: {s2}");
}

// ===========================================================================
// 5. 组合表达式
// ===========================================================================

#[test]
fn aggregates_in_condition() {
    let ctx = num_list_var("nums", &[10, 20, 30]);
    let s = render(
        "<p th:if=\"${#aggregates.sum(nums) > 50}\" th:text=\"'big'\">x</p>",
        &ctx,
    );
    assert!(s.contains("big"));
}

#[test]
fn numbers_and_aggregates_composition() {
    let ctx = num_list_var("nums", &[1, 2, 3]);
    let s = render(
        "<p th:text=\"${#numbers.formatInteger(#aggregates.sum(nums), 1)}\">x</p>",
        &ctx,
    );
    assert!(s.contains("6"), "composed: {s}");
}

// ===========================================================================
// 6. 空集合与 null
// ===========================================================================

#[test]
fn aggregates_empty_list_returns_null() {
    let ctx = num_list_var("nums", &[]);
    // Java AggregateUtils.sum 对空 Iterable 返回 null → 渲染为空
    let s = render("<p th:text=\"${#aggregates.sum(nums)}\">x</p>", &ctx);
    assert!(!s.contains("0"), "empty sum must be null: {s}");
    assert!(!s.contains("1"), "must render empty: {s}");
}

#[test]
fn aggregates_null_target_errors() {
    let ctx = Context::new();
    let result =
        engine().process_template("<p th:text=\"${#aggregates.sum(missing)}\">x</p>", &ctx);
    assert!(result.is_err(), "null aggregate must fail like Java");
}
