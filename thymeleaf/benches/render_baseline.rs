//! 性能基线（criterion）——S11「性能」组件基建。
//!
//! 运行：`cargo bench -p thymeleaf`
//! 结果登记 `docs/release/benchmarks.md`（drift gate：后续版本对比防回归）。
//!
//! 三条基准对应渲染稳态（引擎与 Context 复用，缓存命中路径）：
//! 1. `render_simple_variable` —— 单变量插值（解析缓存命中 + 单点求值）
//! 2. `render_each_100` —— th:each 100 行 + 表达式链
//! 3. `render_full_document` —— 多处理器混合文档（if/each/text/utext/attr）
//!
//! API 构造模式与 `thymeleaf-test/tests/expression_context_java_parity.rs`
//! 逐一核对（StringTemplateResolver + Context::set_variable + process_template）。

use std::sync::Arc;

use criterion::{Criterion, criterion_group, criterion_main};
use thymeleaf::context::Context;
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
}

fn make_engine() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver set");
    engine
}

fn set_str(ctx: &Context, name: &str, value: &str) {
    ctx.set_variable(
        Some(js(name)),
        Some(Arc::new(TemplateValue::string(js(value)))),
    );
}

fn set_num(ctx: &Context, name: &str, value: i64) {
    use thymeleaf::util::NumberValue;
    ctx.set_variable(
        Some(js(name)),
        Some(Arc::new(TemplateValue::Number(NumberValue::Long(value)))),
    );
}

fn set_list(ctx: &Context, name: &str, items: &[&str]) {
    let list: Vec<Arc<TemplateValue>> = items
        .iter()
        .map(|s| Arc::new(TemplateValue::string(js(s))))
        .collect();
    ctx.set_variable(
        Some(js(name)),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
}

fn bench_render_simple_variable(c: &mut Criterion) {
    let engine = make_engine();
    let ctx = Context::new();
    set_str(&ctx, "name", "bench");
    let tpl = "<p th:text=\"${name}\">n</p>";

    let mut group = c.benchmark_group("render_simple_variable");
    group.throughput(criterion::Throughput::Bytes(tpl.len() as u64));
    group.bench_function("single_interpolation", |b| {
        b.iter(|| {
            let _ = engine.process_template(tpl, &ctx).expect("render");
        })
    });
    group.finish();
}

fn bench_render_each_100(c: &mut Criterion) {
    let engine = make_engine();
    let ctx = Context::new();
    let items: Vec<&'static str> = (0..100)
        .map(|i| {
            let s: &'static str = Box::leak(format!("item-{i}").into_boxed_str());
            s
        })
        .collect();
    set_list(&ctx, "items", &items);
    let tpl = r#"<ul><li th:each="i : ${items}" th:text="${i}">row</li></ul>"#;

    let mut group = c.benchmark_group("render_each_100");
    group.throughput(criterion::Throughput::Bytes(tpl.len() as u64));
    group.bench_function("list_iteration", |b| {
        b.iter(|| {
            let _ = engine.process_template(tpl, &ctx).expect("render");
        })
    });
    group.finish();
}

fn bench_render_full_document(c: &mut Criterion) {
    let engine = make_engine();
    let ctx = Context::new();
    set_str(&ctx, "title", "Bench Document");
    set_str(&ctx, "user", "alice");
    set_num(&ctx, "count", 42);
    let items: Vec<&'static str> = (0..50)
        .map(|i| {
            let s: &'static str = Box::leak(format!("row-{i}").into_boxed_str());
            s
        })
        .collect();
    set_list(&ctx, "rows", &items);
    let tpl = r#"<html>
  <head><title th:text="${title}">t</title></head>
  <body>
    <p th:text="'user: ' + ${user}">u</p>
    <p th:if="${count > 10}" th:text="${count}">c</p>
    <table>
      <tr th:each="r : ${rows}">
        <td th:text="${r}">cell</td>
        <td th:text="${count} + 1">n</td>
      </tr>
    </table>
    <div th:utext="${title}">raw</div>
  </body>
</html>"#;

    let mut group = c.benchmark_group("render_full_document");
    group.throughput(criterion::Throughput::Bytes(tpl.len() as u64));
    group.bench_function("mixed_processors", |b| {
        b.iter(|| {
            let _ = engine.process_template(tpl, &ctx).expect("render");
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_render_simple_variable,
    bench_render_each_100,
    bench_render_full_document
);
criterion_main!(benches);
