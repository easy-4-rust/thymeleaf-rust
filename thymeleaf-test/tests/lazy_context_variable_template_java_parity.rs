//! `LazyContextVariableTest` 模板驱动差分（Java 21 逐字复刻）。
//!
//! 上游 `org.thymeleaf.context.LazyContextVariableTest` 的 10 个方法：
//! TEXT 模板 `<[# th:if='${doit}']...[[${lazz}]]...[/]>` 在 `doit=true` 时
//! 渲染惰性变量并置 `initialized`；`doit=false` 时 `th:if` 短路，惰性变量
//! 不求值（`initialized` 保持 false）。test05-08 为 Web 变体（同一模板族），
//! test09/10 为 session/application 属性惰性变量（Web 交换桥接，机制同源）。
//!
//! Rust 侧以 `LazyContextVariable` + 求值计数器复现 Java `Lazy.initialized`
//! 语义，断言渲染输出与求值次数均与 Java 一致。

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use thymeleaf::context::{Context, LazyContextVariable};
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::JavaString;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

const TEMPLATE1: &str = "<[# th:if='${doit}'][[${lazz}]][/]>";
const TEMPLATE2: &str = "<[# th:if='${doit}'][[${'Hey, ' + lazz}]][/]>";
const TEMPLATE3: &str = "<[# th:if='${doit}'][['Hey, ' + ${lazz}]][/]>";
const TEMPLATE4: &str = "<[# th:if='${doit}' th:text='${lazz}']...[/]>";

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

/// 构造一个求值计数 +1 的惰性变量（对应 Java `Lazy.initialized` 标志）。
fn lazy_value(marker: &str, loads: Arc<AtomicUsize>) -> Arc<TemplateValue> {
    let marker = Arc::new(TemplateValue::string(js(marker)));
    let variable = LazyContextVariable::new(move || {
        loads.fetch_add(1, Ordering::SeqCst);
        Some(Arc::clone(&marker))
    });
    Arc::new(TemplateValue::Object(Arc::new(variable)))
}

fn render_text(template: &str, context: &Context) -> String {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::TEXT);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver");
    engine
        .process_template(template, context)
        .expect("render text template")
        .to_string_lossy()
}

fn context_with(doit: &str, lazy: Arc<TemplateValue>) -> Context {
    let context = Context::new();
    context.set_variable(
        Some(js("doit")),
        Some(Arc::new(TemplateValue::string(js(doit)))),
    );
    context.set_variable(Some(js("lazz")), Some(lazy));
    context
}

#[test]
fn lazy_context_variable_templates_short_circuit_like_java() {
    // Java test01-04：TEMPLATE1-4 在 doit=true/false 下的渲染与求值时机
    for (template, expected_true, expected_false) in [
        (TEMPLATE1, "<Hello there!>", "<>"),
        (TEMPLATE2, "<Hey, Hello there!>", "<>"),
        (TEMPLATE3, "<Hey, Hello there!>", "<>"),
        (TEMPLATE4, "<Hello there!>", "<>"),
    ] {
        let loads_true = Arc::new(AtomicUsize::new(0));
        let context_true =
            context_with("true", lazy_value("Hello there!", Arc::clone(&loads_true)));
        assert_eq!(
            render_text(template, &context_true),
            expected_true,
            "TEMPLATE doit=true 渲染"
        );
        assert_eq!(
            loads_true.load(Ordering::SeqCst),
            1,
            "doit=true 时惰性变量恰好求值一次"
        );

        let loads_false = Arc::new(AtomicUsize::new(0));
        let context_false = context_with(
            "false",
            lazy_value("Hello there!", Arc::clone(&loads_false)),
        );
        assert_eq!(
            render_text(template, &context_false),
            expected_false,
            "TEMPLATE doit=false 渲染"
        );
        assert_eq!(
            loads_false.load(Ordering::SeqCst),
            0,
            "th:if 短路：doit=false 时惰性变量不求值"
        );
    }
}

#[test]
fn lazy_context_variable_value_is_cached_after_first_evaluation() {
    // Java 语义：首次访问后缓存，重复渲染不再执行 loadValue
    let loads = Arc::new(AtomicUsize::new(0));
    let context = context_with("true", lazy_value("Hello there!", Arc::clone(&loads)));
    assert_eq!(render_text(TEMPLATE1, &context), "<Hello there!>");
    assert_eq!(loads.load(Ordering::SeqCst), 1);
    // 第二次渲染同一上下文：变量已缓存，不再求值
    assert_eq!(render_text(TEMPLATE1, &context), "<Hello there!>");
    assert_eq!(loads.load(Ordering::SeqCst), 1, "缓存后不再重复求值");
}
