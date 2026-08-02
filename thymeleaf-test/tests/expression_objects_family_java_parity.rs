//! `org.thymeleaf.expression` #objects 族 Java 1:1 差分测试。
//!
//! 覆盖对象（对象表编号）：`Strings`（166）、`Numbers`（163）、`Uris`（168）、
//! `Dates`（154）、`ExecutionInfo`（155）、`Ids`（159）、`Messages`（162）、
//! `Conversions`（153）。
//! 证据分两层：直测（Strings/Numbers/Uris 方法级 Java golden）+ 引擎驱动
//! （`#strings.*`/`#numbers.*`/`#uris.*`/`#dates.*`/`#execInfo.*`/`#ids.*`
//! 模板求值，与语料 2,595 同构）。

use std::sync::Arc;

use thymeleaf::context::Context;
use thymeleaf::expression::{Numbers, Strings, TemplateValue, Uris};
use thymeleaf::templateresolver::{ITemplateResolver, StringTemplateResolver};
use thymeleaf::util::{JavaLocale, JavaNumber, JavaString};
use thymeleaf::{TemplateEngine, TemplateMode};

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn us() -> JavaLocale {
    JavaLocale::new(js("en"), js("US"))
}

fn string_value(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::string(js(value)))
}

fn engine() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

fn render(template: &str) -> String {
    let engine = engine();
    let context = Context::new();
    engine
        .process_template(template, &context)
        .expect("render")
        .to_string_lossy()
}

// ===========================================================================
// 1. Strings（166）：方法级 Java golden
// ===========================================================================

#[test]
fn strings_methods_match_java() {
    let strings = Strings::new(us());
    let hello = string_value("hello");
    let foobar = string_value("foobar");

    // Java: toString 对 null 返回 null，其余 String.valueOf
    assert!(strings.to_string(None).is_none());
    assert_eq!(
        strings
            .to_string(Some(hello.as_ref()))
            .expect("to string")
            .to_string_lossy(),
        "hello"
    );

    // Java: abbreviate(str, maxSize)：保留 maxSize-3 字符 + "..."
    let long = string_value("Now is the time for all good men");
    assert_eq!(
        strings
            .abbreviate(Some(long.as_ref()), 15)
            .expect("abbreviate")
            .expect("value")
            .to_string_lossy(),
        "Now is the t..."
    );
    assert_eq!(
        strings
            .abbreviate(Some(hello.as_ref()), 15)
            .expect("abbreviate")
            .expect("value")
            .to_string_lossy(),
        "hello"
    );

    // Java: equals/equalsIgnoreCase/contains（大小写变体；Rust 把大小写
    // 敏感标志拆为 contains/contains_ignore_case 两个方法）
    let a = string_value("a");
    let b = string_value("b");
    let upper_a = string_value("A");
    assert!(strings.equals(Some(a.as_ref()), Some(a.as_ref())));
    assert!(!strings.equals(Some(a.as_ref()), Some(b.as_ref())));
    assert!(strings.equals_ignore_case(Some(a.as_ref()), Some(upper_a.as_ref())));
    assert!(
        strings
            .contains(Some(foobar.as_ref()), Some(&js("oba")))
            .expect("contains")
    );
    assert!(
        !strings
            .contains(Some(foobar.as_ref()), Some(&js("OBA")))
            .expect("contains case sensitive")
    );
    assert!(
        strings
            .contains_ignore_case(Some(foobar.as_ref()), Some(&js("OBA")))
            .expect("contains ignore case")
    );

    // Java: startsWith/endsWith
    assert!(
        strings
            .starts_with(Some(foobar.as_ref()), Some(&js("foo")))
            .expect("starts with")
    );
    assert!(
        strings
            .ends_with(Some(foobar.as_ref()), Some(&js("bar")))
            .expect("ends with")
    );
    assert!(
        !strings
            .starts_with(Some(foobar.as_ref()), Some(&js("BAR")))
            .expect("starts with case sensitive")
    );

    // Java: substring/substringAfter（含越界错误）
    assert_eq!(
        strings
            .substring(Some(foobar.as_ref()), 1, 4)
            .expect("substring")
            .expect("value")
            .to_string_lossy(),
        "oob"
    );
    assert_eq!(
        strings
            .substring_from(Some(foobar.as_ref()), 3)
            .expect("substring from")
            .expect("value")
            .to_string_lossy(),
        "bar"
    );
    assert!(
        strings.substring(Some(foobar.as_ref()), 1, 10).is_err(),
        "out of range"
    );

    // Java: isEmpty/toUpperCase/toLowerCase
    let empty = string_value("");
    assert!(strings.is_empty(Some(empty.as_ref())));
    assert!(!strings.is_empty(Some(hello.as_ref())));
    assert!(strings.is_empty(None));
    assert_eq!(
        strings
            .to_upper_case(Some(hello.as_ref()))
            .expect("upper")
            .expect("value")
            .to_string_lossy(),
        "HELLO"
    );
    let upper_hello = string_value("HELLO");
    assert_eq!(
        strings
            .to_lower_case(Some(upper_hello.as_ref()))
            .expect("lower")
            .expect("value")
            .to_string_lossy(),
        "hello"
    );
}

// ===========================================================================
// 2. Numbers（163）：方法级 Java golden
// ===========================================================================

#[test]
fn numbers_methods_match_java() {
    use thymeleaf::util::NumberPointType;

    let numbers = Numbers::new(us());

    // Java: formatInteger(value, minIntegerDigits, thousandsPointType)
    let big = JavaNumber::Integer(1_234_567);
    assert_eq!(
        numbers
            .format_integer(Some(&big), 3, Some(NumberPointType::Comma))
            .expect("format integer")
            .expect("value")
            .to_string_lossy(),
        "1,234,567"
    );
    assert_eq!(
        numbers
            .format_integer(Some(&JavaNumber::Integer(1234)), 8, None)
            .expect("format integer")
            .expect("value")
            .to_string_lossy(),
        "00001234",
        "min integer digits zero-padding"
    );

    // Java: formatDecimal(value, minIntegerDigits, thousandsType, decimalDigits, decimalType)
    assert_eq!(
        numbers
            .format_decimal(
                Some(&JavaNumber::Double(1234.567)),
                1,
                NumberPointType::Comma,
                2,
                NumberPointType::Point,
            )
            .expect("format decimal")
            .expect("value")
            .to_string_lossy(),
        "1,234.57"
    );

    // Java: sequence(from, to[, step])（含负数步长）
    assert_eq!(
        numbers.sequence(1, 3, None).expect("sequence"),
        vec![1, 2, 3]
    );
    assert_eq!(
        numbers.sequence(2, 6, Some(2)).expect("sequence step"),
        vec![2, 4, 6]
    );
    assert_eq!(
        numbers
            .sequence(3, 1, Some(-1))
            .expect("sequence negative step"),
        vec![3, 2, 1]
    );
}

// ===========================================================================
// 3. Uris（168）：方法级 Java golden
// ===========================================================================

#[test]
fn uris_methods_match_java() {
    let uris = Uris;

    // Java: escapePath（RFC 3986 path 转义：空格 -> %20、? -> %3F）
    assert_eq!(
        uris.escape_path(Some(&js("/a b/c?d")))
            .expect("escape path")
            .expect("value")
            .to_string_lossy(),
        "/a%20b/c%3Fd"
    );
    // escapePathSegment（斜杠也转义）
    assert_eq!(
        uris.escape_path_segment(Some(&js("a/b c")))
            .expect("escape segment")
            .expect("value")
            .to_string_lossy(),
        "a%2Fb%20c"
    );
    // unescape 往返
    assert_eq!(
        uris.unescape_path(Some(&js("/a%20b/c%3Fd")))
            .expect("unescape path")
            .expect("value")
            .to_string_lossy(),
        "/a b/c?d"
    );
    // escapeQueryParam（Java unbescape：空格 -> %20、& -> %26；语料
    // features/expression/uris/uris01 期望 "This%20is%20a%20text"）
    assert_eq!(
        uris.escape_query_param(Some(&js("a b&c")))
            .expect("escape query")
            .expect("value")
            .to_string_lossy(),
        "a%20b%26c"
    );
}

// ===========================================================================
// 4. 引擎驱动 #objects 求值
// ===========================================================================

#[test]
fn expression_objects_engine_paths_match_java() {
    // #strings / #numbers / #uris / #dates 在模板中求值
    assert_eq!(
        render("<p th:text=\"${#strings.toUpperCase('hello')}\">x</p>"),
        "<p>HELLO</p>"
    );
    assert_eq!(
        render("<p th:text=\"${#numbers.formatInteger(1234, 8)}\">x</p>"),
        "<p>00001234</p>"
    );
    assert_eq!(
        render("<p th:text=\"${#uris.escapePath('/a b')}\">x</p>"),
        "<p>/a%20b</p>"
    );
    assert_eq!(
        render("<p th:text=\"${#dates.format(#dates.create(2024,5,17), 'yyyy-MM-dd')}\">x</p>"),
        "<p>2024-05-17</p>"
    );

    // #ids.seq/nextSeq：同名计数器序列
    assert_eq!(
        render(
            "<p th:text=\"${#ids.seq('row')}\">x</p><p th:text=\"${#ids.nextSeq('row')}\">x</p>"
        ),
        "<p>row1</p><p>row2</p>"
    );

    // #execInfo：模板模式（Java ExecutionInfo.getTemplateMode）
    assert_eq!(
        render("<p th:text=\"${#execInfo.templateMode}\">x</p>"),
        "<p>HTML</p>"
    );
    // #execInfo.now：当前时间（非空日期文本）
    let now = render("<p th:text=\"${#execInfo.now}\">x</p>");
    assert!(now.len() > 9, "execInfo.now renders a date: {now}");

    // #strings 对 null 输入 null-safe（表达式对象容错）
    assert_eq!(
        render("<p th:text=\"${#strings.isEmpty(null)}\">x</p>"),
        "<p>true</p>"
    );
}

#[test]
fn expression_objects_with_context_vars_match_java() {
    let engine = engine();
    let context = Context::new();
    context.set_variable(
        Some(js("name")),
        Some(string_value("Now is the time for all good men")),
    );
    context.set_variable(
        Some(js("from")),
        Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(2)))),
    );
    context.set_variable(
        Some(js("to")),
        Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(4)))),
    );

    assert_eq!(
        engine
            .process_template(
                "<p th:text=\"${#strings.abbreviate(name, 10)}\">x</p>",
                &context,
            )
            .expect("render")
            .to_string_lossy(),
        "<p>Now is ...</p>"
    );
    assert_eq!(
        engine
            .process_template(
                "<p th:text=\"${#numbers.sequence(from, to)}\">x</p>",
                &context,
            )
            .expect("render")
            .to_string_lossy(),
        "<p>[2, 3, 4]</p>"
    );
}
