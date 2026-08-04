//! 模板选择器与处理链 Java Golden 差分测试。

use std::sync::Arc;

use thymeleaf::context::{Context, IContext};
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::{ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode, TemplateSpec};

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

#[test]
fn th_fragment_defines_named_fragment() {
    let ctx = Context::new();
    let s = render(
        "<div th:fragment=\"greeting\">Hello</div><div>Other</div>",
        &ctx,
    );
    assert!(s.contains("Hello"));
    assert!(s.contains("Other"));
}

#[test]
fn th_remove_all() {
    let ctx = Context::new();
    let s = render(
        "<div th:remove=\"all\"><p>gone</p></div><span>stay</span>",
        &ctx,
    );
    assert!(!s.contains("gone"));
    assert!(s.contains("stay"));
}

#[test]
fn th_remove_body() {
    let ctx = Context::new();
    let s = render(
        "<div th:remove=\"body\"><p>gone</p></div><span>stay</span>",
        &ctx,
    );
    assert!(!s.contains("gone"));
    assert!(s.contains("stay"));
}

#[test]
fn th_remove_tag() {
    let ctx = Context::new();
    let s = render(
        "<div th:remove=\"tag\"><p>content</p></div><span>after</span>",
        &ctx,
    );
    assert!(s.contains("<p>content</p>"));
    assert!(s.contains("after"));
}

#[test]
fn th_block_removed() {
    let ctx = Context::new();
    let s = render("<th:block><p>content</p></th:block>", &ctx);
    assert!(s.contains("<p>content</p>"));
    assert!(!s.contains("<th:block"));
}

#[test]
fn th_block_conditional() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("show")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    let s = render(
        "<th:block th:if=\"${show}\"><p>visible</p></th:block>",
        &ctx,
    );
    assert!(s.contains("visible"));
}

#[test]
fn cache_hit_same_template() {
    let engine = create_engine();
    let ctx = Context::new();
    let out1 = engine
        .process_template("<p>cached</p>", &ctx)
        .unwrap()
        .to_string_lossy();
    let out2 = engine
        .process_template("<p>cached</p>", &ctx)
        .unwrap()
        .to_string_lossy();
    assert_eq!(out1, out2);
}

#[test]
fn cache_miss_different_templates() {
    let engine = create_engine();
    let ctx = Context::new();
    let out1 = engine
        .process_template("<p>first</p>", &ctx)
        .unwrap()
        .to_string_lossy();
    let out2 = engine
        .process_template("<div>second</div>", &ctx)
        .unwrap()
        .to_string_lossy();
    assert!(out1.contains("first"));
    assert!(out2.contains("second"));
}

#[test]
fn xml_mode() {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::XML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let ctx = Context::new();
    let input = "<?xml version=\"1.0\"?>\n<root><item>data</item></root>";
    assert_eq!(
        engine
            .process_template(input, &ctx)
            .unwrap()
            .to_string_lossy(),
        input
    );
}

#[test]
fn text_mode() {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::TEXT);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let ctx = Context::new();
    let input = "Hello World\nLine 2";
    assert_eq!(
        engine
            .process_template(input, &ctx)
            .unwrap()
            .to_string_lossy(),
        input
    );
}

#[test]
fn raw_mode() {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::RAW);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let ctx = Context::new();
    let input = "<html th:text=\"'ignored'\">raw</html>";
    assert_eq!(
        engine
            .process_template(input, &ctx)
            .unwrap()
            .to_string_lossy(),
        input
    );
}

#[test]
fn empty_template() {
    let engine = create_engine();
    let ctx = Context::new();
    assert_eq!(
        engine.process_template("", &ctx).unwrap().to_string_lossy(),
        ""
    );
}

#[test]
fn unicode_preserved() {
    let ctx = Context::new();
    let input = "<p>日本語テスト émojis</p>";
    assert_eq!(render(input, &ctx), input);
}

#[test]
fn unicode_variable() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("msg")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "こんにちは",
        )))),
    );
    assert!(render("<p th:text=\"${msg}\">x</p>", &ctx).contains("こんにちは"));
}

#[test]
fn large_template_200_elements() {
    let ctx = Context::new();
    let mut input = String::from("<html><body>");
    for i in 0..200 {
        input.push_str(&format!("<p>item {i}</p>"));
    }
    input.push_str("</body></html>");
    let s = render(&input, &ctx);
    assert!(s.contains("item 0"));
    assert!(s.contains("item 199"));
}

#[test]
fn mixed_static_dynamic() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("title")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "My Page",
        )))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("body")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "Welcome",
        )))),
    );
    let s = render(
        "<html><head><title th:text=\"${title}\">D</title></head><body><p th:text=\"${body}\">x</p><footer>static</footer></body></html>",
        &ctx,
    );
    assert!(s.contains("My Page"));
    assert!(s.contains("Welcome"));
    assert!(s.contains("static"));
}

#[test]
fn template_spec_with_mode() {
    let engine = create_engine();
    let ctx = Context::new();
    let spec =
        TemplateSpec::with_template_mode(Some("<p>test</p>"), Some(TemplateMode::HTML)).unwrap();
    assert!(
        engine
            .process(&spec, &ctx)
            .unwrap()
            .to_string_lossy()
            .contains("<p>test</p>")
    );
}
