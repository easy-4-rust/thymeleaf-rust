//! `TemplateEngine` 全处理链 Java Golden 差分测试。
//!
//! 覆盖：初始化状态机、方言配置、模板解析器、模板处理全流程、
//! 多模板模式（HTML/XML/TEXT/RAW）、错误处理、缓存行为和配置冻结。

use std::sync::Arc;

use thymeleaf::context::Context;
use thymeleaf::dialect::IDialect;
use thymeleaf::expression::TemplateValue;
use thymeleaf::standard::StandardDialect;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::{
    ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode, TemplateResolutionAttributes,
    TemplateSpec,
};

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

fn create_engine_with_resolver() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver set before init");
    engine
}

fn create_engine_with_mode(mode: TemplateMode) -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(mode);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver set before init");
    engine
}

fn ctx_with_var(name: &str, value: &str) -> Context {
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str(name)),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            value,
        )))),
    );
    ctx
}

fn ctx_with_bool(name: &str, value: bool) -> Context {
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str(name)),
        Some(Arc::new(TemplateValue::Boolean(value))),
    );
    ctx
}

fn ctx_with_string_list(name: &str, values: &[&str]) -> Context {
    let ctx = Context::new();
    let list: Vec<Arc<TemplateValue>> = values
        .iter()
        .map(|v| Arc::new(TemplateValue::string(Utf16String::from_rust_str(v))))
        .collect();
    ctx.set_variable(
        Some(Utf16String::from_rust_str(name)),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    ctx
}

// ===========================================================================
// 1. 初始化状态机
// ===========================================================================

#[test]
fn engine_starts_uninitialized() {
    let engine = TemplateEngine::new();
    assert!(!engine.is_initialized());
}

#[test]
fn engine_initializes_on_first_process() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let _ = engine.process_template("hello", &ctx);
    assert!(engine.is_initialized());
}

#[test]
fn engine_rejects_config_after_initialization() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let _ = engine.process_template("init", &ctx);

    let dummy: Arc<dyn IDialect> = Arc::new(StandardDialect::new());
    assert!(engine.set_dialect(dummy.clone()).is_err());
    assert!(engine.clear_dialects().is_err());
    assert!(engine.set_template_resolvers(vec![]).is_err());
}

#[test]
fn engine_default_dialect_is_standard() {
    let engine = TemplateEngine::new();
    let dialects = engine.get_dialects();
    assert_eq!(dialects.len(), 1);
    assert_eq!(dialects[0].get_name(), Some("Standard"));
}

#[test]
fn engine_default_prefix_is_none() {
    let engine = TemplateEngine::new();
    let by_prefix = engine.get_dialects_by_prefix();
    // 默认前缀为 None（标准方言使用默认前缀 "th"，但在配置中以 None 注册）
    assert!(
        by_prefix.contains_key(&None),
        "expected None prefix in dialect map"
    );
}

#[test]
fn engine_clear_dialects_before_init() {
    let engine = TemplateEngine::new();
    engine.clear_dialects().expect("clear before init");
    assert!(engine.get_dialects().is_empty());
}

#[test]
fn engine_set_dialects_deduplicates_by_identity() {
    let engine = TemplateEngine::new();
    let d: Arc<dyn IDialect> = Arc::new(StandardDialect::new());
    engine
        .set_dialects(vec![d.clone(), d.clone()])
        .expect("set dialects");
    assert_eq!(engine.get_dialects().len(), 1);
}

#[test]
fn engine_add_dialect_with_prefix() {
    let engine = TemplateEngine::new();
    let d: Arc<dyn IDialect> = Arc::new(StandardDialect::new());
    engine
        .add_dialect_with_prefix(Some("custom"), d)
        .expect("add dialect");
    let by_prefix = engine.get_dialects_by_prefix();
    assert!(by_prefix.contains_key(&Some(Utf16String::from_rust_str("custom"))));
}

#[test]
fn engine_set_additional_dialects() {
    let engine = TemplateEngine::new();
    let d: Arc<dyn IDialect> = Arc::new(StandardDialect::new());
    engine
        .set_additional_dialects(vec![d])
        .expect("set additional");
    // 应该有默认的 th 方言 + 新增的
    assert!(engine.get_dialects().len() >= 2);
}

// ===========================================================================
// 2. 空模板与纯文本
// ===========================================================================

#[test]
fn empty_template_produces_empty_output() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let output = engine.process_template("", &ctx).expect("empty");
    assert_eq!(output.to_string_lossy(), "");
}

#[test]
fn plain_text_passthrough() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let output = engine
        .process_template("Hello, World!", &ctx)
        .expect("plain text");
    assert_eq!(output.to_string_lossy(), "Hello, World!");
}

#[test]
fn whitespace_only_template_preserved() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "   \n\t  \n  ";
    let output = engine.process_template(input, &ctx).expect("whitespace");
    assert_eq!(output.to_string_lossy(), input);
}

// ===========================================================================
// 3. HTML 模板基础
// ===========================================================================

#[test]
fn html_doctype_preserved() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "<!DOCTYPE html>\n<html>\n<body>Hi</body>\n</html>";
    let output = engine.process_template(input, &ctx).expect("html doctype");
    assert_eq!(output.to_string_lossy(), input);
}

#[test]
fn html_self_closing_tags_preserved() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "<br/><hr/>";
    let output = engine.process_template(input, &ctx).expect("self-closing");
    assert_eq!(output.to_string_lossy(), input);
}

#[test]
fn html_comments_preserved() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "<!-- comment --><p>text</p><!-- another -->";
    let output = engine.process_template(input, &ctx).expect("comments");
    assert_eq!(output.to_string_lossy(), input);
}

// ===========================================================================
// 4. th:text 基础表达式
// ===========================================================================

#[test]
fn th_text_with_variable_expression() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_var("name", "World");
    let input = "<span th:text=\"${name}\">placeholder</span>";
    let output = engine.process_template(input, &ctx).expect("th:text");
    assert!(output.to_string_lossy().contains("World"));
    assert!(!output.to_string_lossy().contains("placeholder"));
}

#[test]
fn th_text_escapes_html_entities() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_var("content", "<script>alert('xss')</script>");
    let input = "<p th:text=\"${content}\">safe</p>";
    let output = engine
        .process_template(input, &ctx)
        .expect("th:text escape");
    let s = output.to_string_lossy();
    assert!(s.contains("&lt;script&gt;"));
    assert!(!s.contains("<script>"));
}

#[test]
fn th_utext_does_not_escape() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_var("html", "<b>bold</b>");
    let input = "<p th:utext=\"${html}\">safe</p>";
    let output = engine.process_template(input, &ctx).expect("th:utext");
    assert!(output.to_string_lossy().contains("<b>bold</b>"));
}

#[test]
fn th_text_with_literal_expression() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "<span th:text=\"'hello world'\">x</span>";
    let output = engine
        .process_template(input, &ctx)
        .expect("literal expression");
    assert!(output.to_string_lossy().contains("hello world"));
}

// ===========================================================================
// 5. th:if / th:unless 条件渲染
// ===========================================================================

#[test]
fn th_if_renders_when_true() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_bool("show", true);
    let input = "<p th:if=\"${show}\" th:text=\"'visible'\">hidden</p>";
    let output = engine.process_template(input, &ctx).expect("th:if true");
    assert!(output.to_string_lossy().contains("visible"));
}

#[test]
fn th_if_removes_when_false() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_bool("show", false);
    let input = "<p th:if=\"${show}\">should not appear</p><span>after</span>";
    let output = engine.process_template(input, &ctx).expect("th:if false");
    let s = output.to_string_lossy();
    assert!(!s.contains("should not appear"));
    assert!(s.contains("after"));
}

#[test]
fn th_unless_renders_when_false() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_bool("hide", false);
    let input = "<p th:unless=\"${hide}\" th:text=\"'shown'\">x</p>";
    let output = engine
        .process_template(input, &ctx)
        .expect("th:unless false");
    assert!(output.to_string_lossy().contains("shown"));
}

#[test]
fn th_unless_removes_when_true() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_bool("hide", true);
    let input = "<p th:unless=\"${hide}\">should not appear</p><span>ok</span>";
    let output = engine
        .process_template(input, &ctx)
        .expect("th:unless true");
    let s = output.to_string_lossy();
    assert!(!s.contains("should not appear"));
    assert!(s.contains("ok"));
}

// ===========================================================================
// 6. th:each 迭代
// ===========================================================================

#[test]
fn th_each_over_list() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_string_list("items", &["a", "b", "c"]);
    let input = "<ul><li th:each=\"item : ${items}\" th:text=\"${item}\">x</li></ul>";
    let output = engine.process_template(input, &ctx).expect("th:each");
    let s = output.to_string_lossy();
    assert!(s.contains("a"));
    assert!(s.contains("b"));
    assert!(s.contains("c"));
}

#[test]
fn th_each_empty_list_produces_no_iterations() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_string_list("items", &[]);
    let input = "<ul><li th:each=\"item : ${items}\" th:text=\"${item}\">x</li></ul>";
    let output = engine.process_template(input, &ctx).expect("th:each empty");
    let s = output.to_string_lossy();
    assert!(!s.contains("<li"));
}

// ===========================================================================
// 7. th:with 变量赋值
// ===========================================================================

#[test]
fn th_with_sets_local_variable() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "<div th:with=\"greeting='Hello'\" th:text=\"${greeting}\">x</div>";
    let output = engine.process_template(input, &ctx).expect("th:with");
    assert!(output.to_string_lossy().contains("Hello"));
}

// ===========================================================================
// 8. th:attr 属性设置
// ===========================================================================

#[test]
fn th_attr_sets_dynamic_attribute() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_var("url", "https://example.com");
    let input = "<a th:attr=\"href=${url}\">link</a>";
    let output = engine.process_template(input, &ctx).expect("th:attr");
    assert!(output.to_string_lossy().contains("https://example.com"));
}

// ===========================================================================
// 9. th:block 容器
// ===========================================================================

#[test]
fn th_block_is_removed_from_output() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_bool("show", true);
    let input = "<th:block th:if=\"${show}\"><p>content</p></th:block>";
    let output = engine.process_template(input, &ctx).expect("th:block");
    let s = output.to_string_lossy();
    assert!(s.contains("<p>content</p>"));
    assert!(!s.contains("<th:block"));
}

// ===========================================================================
// 10. TemplateSpec 处理入口
// ===========================================================================

#[test]
fn process_with_template_spec() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let spec = TemplateSpec::with_template_mode(Some("hello"), None).expect("valid spec");
    let output = engine.process(&spec, &ctx).expect("process spec");
    assert_eq!(output.to_string_lossy(), "hello");
}

#[test]
fn process_with_template_spec_and_mode() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let spec = TemplateSpec::with_template_mode(Some("<p>test</p>"), Some(TemplateMode::HTML))
        .expect("spec with mode");
    let output = engine.process(&spec, &ctx).expect("process with mode");
    assert!(output.to_string_lossy().contains("<p>test</p>"));
}

// ===========================================================================
// 11. 错误处理
// ===========================================================================

#[test]
fn empty_template_spec_accepted() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let spec = TemplateSpec::with_template_mode(Some(""), None).expect("empty spec accepted");
    let output = engine
        .process(&spec, &ctx)
        .expect("Java StringTemplateResolver accepts an empty template");
    assert_eq!(output.to_string_lossy(), "");
}

// ===========================================================================
// 12. 多模板模式
// ===========================================================================

#[test]
fn xml_template_mode_preserves_content() {
    let engine = create_engine_with_mode(TemplateMode::XML);
    let ctx = Context::new();
    let input = "<?xml version=\"1.0\"?>\n<root><item>data</item></root>";
    let output = engine.process_template(input, &ctx).expect("xml mode");
    assert_eq!(output.to_string_lossy(), input);
}

#[test]
fn text_template_mode_preserves_content() {
    let engine = create_engine_with_mode(TemplateMode::TEXT);
    let ctx = Context::new();
    let input = "Hello World\nLine 2\nLine 3";
    let output = engine.process_template(input, &ctx).expect("text mode");
    assert_eq!(output.to_string_lossy(), input);
}

#[test]
fn raw_template_mode_preserves_content() {
    let engine = create_engine_with_mode(TemplateMode::RAW);
    let ctx = Context::new();
    let input = "<html th:text=\"'ignored'\">raw content</html>";
    let output = engine.process_template(input, &ctx).expect("raw mode");
    // RAW 模式不做任何处理
    assert_eq!(output.to_string_lossy(), input);
}

// ===========================================================================
// 13. 缓存行为
// ===========================================================================

#[test]
fn repeated_process_same_template_succeeds() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "<p>cached</p>";

    let out1 = engine.process_template(input, &ctx).expect("first");
    let out2 = engine.process_template(input, &ctx).expect("second");
    assert_eq!(out1.to_string_lossy(), out2.to_string_lossy());
}

#[test]
fn different_templates_render_independently() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();

    let out1 = engine
        .process_template("<p>first</p>", &ctx)
        .expect("first");
    let out2 = engine
        .process_template("<div>second</div>", &ctx)
        .expect("second");
    assert!(out1.to_string_lossy().contains("first"));
    assert!(out2.to_string_lossy().contains("second"));
}

// ===========================================================================
// 14. Context 变量
// ===========================================================================

#[test]
fn nested_variable_access() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("greeting")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "Hello",
        )))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("user")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "Alice",
        )))),
    );
    let input = "<p th:text=\"${greeting + ', ' + user + '!'}\">x</p>";
    let output = engine.process_template(input, &ctx).expect("nested var");
    assert!(output.to_string_lossy().contains("Hello, Alice!"));
}

#[test]
fn null_variable_renders_empty() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    // 不设置 missing 变量
    let input = "<p th:text=\"${missing}\">fallback</p>";
    let output = engine.process_template(input, &ctx).expect("null var");
    // null 变量应渲染为空字符串，不保留 fallback
    let s = output.to_string_lossy();
    assert!(!s.contains("fallback"));
}

// ===========================================================================
// 15. Unicode 与特殊字符
// ===========================================================================

#[test]
fn unicode_content_preserved() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "<p>日本語テスト émojis</p>";
    let output = engine.process_template(input, &ctx).expect("unicode");
    assert_eq!(output.to_string_lossy(), input);
}

#[test]
fn unicode_in_variable_expression() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_var("msg", "こんにちは世界");
    let input = "<p th:text=\"${msg}\">x</p>";
    let output = engine.process_template(input, &ctx).expect("unicode var");
    assert!(output.to_string_lossy().contains("こんにちは世界"));
}

// ===========================================================================
// 16. TemplateResolutionAttributes
// ===========================================================================

#[test]
fn template_resolution_attributes_equality() {
    use thymeleaf::TemplateResolutionAttributeValue;
    let mut attrs1 = TemplateResolutionAttributes::new();
    attrs1.insert(
        Some("key1".to_owned()),
        TemplateResolutionAttributeValue::new("value1".to_owned()),
    );
    let mut attrs2 = TemplateResolutionAttributes::new();
    attrs2.insert(
        Some("key1".to_owned()),
        TemplateResolutionAttributeValue::new("value1".to_owned()),
    );
    assert_eq!(attrs1, attrs2);
}

#[test]
fn template_resolution_attributes_different_values() {
    use thymeleaf::TemplateResolutionAttributeValue;
    let mut attrs1 = TemplateResolutionAttributes::new();
    attrs1.insert(
        Some("key1".to_owned()),
        TemplateResolutionAttributeValue::new("value1".to_owned()),
    );
    let mut attrs2 = TemplateResolutionAttributes::new();
    attrs2.insert(
        Some("key1".to_owned()),
        TemplateResolutionAttributeValue::new("value2".to_owned()),
    );
    assert_ne!(attrs1, attrs2);
}

// ===========================================================================
// 17. TemplateMode 语义
// ===========================================================================

#[test]
fn template_mode_is_markup_flags() {
    assert!(TemplateMode::HTML.is_markup());
    assert!(TemplateMode::XML.is_markup());
    assert!(!TemplateMode::TEXT.is_markup());
    assert!(!TemplateMode::JAVASCRIPT.is_markup());
    assert!(!TemplateMode::CSS.is_markup());
    assert!(!TemplateMode::RAW.is_markup());
}

#[test]
fn template_mode_is_text_flags() {
    assert!(!TemplateMode::HTML.is_text());
    assert!(!TemplateMode::XML.is_text());
    assert!(TemplateMode::TEXT.is_text());
    assert!(TemplateMode::JAVASCRIPT.is_text());
    assert!(TemplateMode::CSS.is_text());
    assert!(!TemplateMode::RAW.is_text());
}

#[test]
fn template_mode_case_sensitivity() {
    assert!(!TemplateMode::HTML.is_case_sensitive());
    assert!(TemplateMode::XML.is_case_sensitive());
    assert!(TemplateMode::TEXT.is_case_sensitive());
}

#[test]
fn template_mode_parse_valid() {
    assert_eq!(
        TemplateMode::parse(Some("HTML")).unwrap(),
        TemplateMode::HTML
    );
    assert_eq!(TemplateMode::parse(Some("xml")).unwrap(), TemplateMode::XML);
    assert_eq!(
        TemplateMode::parse(Some("text")).unwrap(),
        TemplateMode::TEXT
    );
}

#[test]
fn template_mode_parse_invalid() {
    // 空字符串和 None 返回错误
    assert!(TemplateMode::parse(Some("")).is_err());
    assert!(TemplateMode::parse(None).is_err());
    // 未知模式默认回退到 HTML（与 Java 行为一致）
    assert_eq!(
        TemplateMode::parse(Some("MARKDOWN")).unwrap(),
        TemplateMode::HTML
    );
}

#[test]
fn template_mode_display() {
    assert_eq!(TemplateMode::HTML.to_string(), "HTML");
    assert_eq!(TemplateMode::XML.to_string(), "XML");
    assert_eq!(TemplateMode::TEXT.to_string(), "TEXT");
    assert_eq!(TemplateMode::JAVASCRIPT.to_string(), "JAVASCRIPT");
    assert_eq!(TemplateMode::CSS.to_string(), "CSS");
    assert_eq!(TemplateMode::RAW.to_string(), "RAW");
}

// ===========================================================================
// 18. TemplateEngine 常量
// ===========================================================================

#[test]
fn timer_logger_name() {
    assert_eq!(
        TemplateEngine::TIMER_LOGGER_NAME,
        "org.thymeleaf.TemplateEngine.TIMER"
    );
}

// ===========================================================================
// 19. 复杂表达式
// ===========================================================================

#[test]
fn string_concatenation_expression() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("a")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "Hello",
        )))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("b")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "World",
        )))),
    );
    let input = "<p th:text=\"${a + ' ' + b}\">x</p>";
    let output = engine.process_template(input, &ctx).expect("concat");
    assert!(output.to_string_lossy().contains("Hello World"));
}

#[test]
fn numeric_literal_expression() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "<p th:text=\"42\">x</p>";
    let output = engine.process_template(input, &ctx).expect("numeric");
    assert!(output.to_string_lossy().contains("42"));
}

#[test]
fn boolean_literal_expression() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "<p th:text=\"true\">x</p>";
    let output = engine.process_template(input, &ctx).expect("bool true");
    assert!(output.to_string_lossy().contains("true"));
}

#[test]
fn null_literal_expression() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "<p th:text=\"null\">x</p>";
    let output = engine.process_template(input, &ctx).expect("null literal");
    // null 应渲染为空
    let s = output.to_string_lossy();
    assert!(!s.contains("x"));
}

// ===========================================================================
// 20. th:remove 语义
// ===========================================================================

#[test]
fn th_remove_all_removes_element_and_body() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let input = "<div th:remove=\"all\"><p>removed</p></div><span>kept</span>";
    let output = engine.process_template(input, &ctx).expect("th:remove all");
    let s = output.to_string_lossy();
    assert!(!s.contains("removed"));
    assert!(s.contains("kept"));
}

// ===========================================================================
// 21. th:switch / th:case / th:default
// ===========================================================================

#[test]
fn th_switch_case_rendering() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_var("mode", "edit");
    let input = "<div th:switch=\"${mode}\">\
                   <p th:case=\"view\">Viewing</p>\
                   <p th:case=\"edit\">Editing</p>\
                   <p th:case=\"*\">Unknown</p>\
                 </div>";
    let output = engine.process_template(input, &ctx).expect("th:switch");
    let s = output.to_string_lossy();
    assert!(s.contains("Editing"));
    // default case 不应渲染
    assert!(!s.contains("Unknown"));
}

// ===========================================================================
// 22. th:with 嵌套作用域
// ===========================================================================

#[test]
fn th_with_sets_selection_target() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_var("product", "Widget");
    let input = "<div th:with=\"name=${product}\"><p th:text=\"${name}\">x</p></div>";
    let output = engine
        .process_template(input, &ctx)
        .expect("th:with nested");
    assert!(output.to_string_lossy().contains("Widget"));
}

// ===========================================================================
// 23. 多元素组合
// ===========================================================================

#[test]
fn multiple_th_attributes_on_same_element() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("show")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("text")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "Hello",
        )))),
    );
    let input = "<p th:if=\"${show}\" th:text=\"${text}\">x</p>";
    let output = engine
        .process_template(input, &ctx)
        .expect("multiple attrs");
    let s = output.to_string_lossy();
    assert!(s.contains("Hello"));
    assert!(!s.contains(">x<"));
}

#[test]
fn nested_th_elements_with_different_attributes() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("outer")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "Outer",
        )))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("inner")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "Inner",
        )))),
    );
    // th:text 替换元素体，所以嵌套元素用 th:attr 或不同属性
    let input = "<div><p th:text=\"${inner}\">x</p><span th:text=\"${outer}\">y</span></div>";
    let output = engine.process_template(input, &ctx).expect("nested");
    let s = output.to_string_lossy();
    assert!(s.contains("Outer"));
    assert!(s.contains("Inner"));
}

// ===========================================================================
// 24. 大模板压力测试
// ===========================================================================

#[test]
fn large_template_with_many_elements() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();

    let mut input = String::from("<html><body>");
    for i in 0..100 {
        input.push_str(&format!("<p>paragraph {i}</p>"));
    }
    input.push_str("</body></html>");

    let output = engine
        .process_template(&input, &ctx)
        .expect("large template");
    let s = output.to_string_lossy();
    assert!(s.contains("paragraph 0"));
    assert!(s.contains("paragraph 99"));
}

// ===========================================================================
// 25. 混合静态与动态内容
// ===========================================================================

#[test]
fn mixed_static_and_dynamic_content() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("title")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "My Page",
        )))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("content")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "Welcome!",
        )))),
    );
    let input = "<html><head><title th:text=\"${title}\">Default</title></head>\
                 <body><p th:text=\"${content}\">placeholder</p>\
                 <footer>static footer</footer></body></html>";
    let output = engine.process_template(input, &ctx).expect("mixed");
    let s = output.to_string_lossy();
    assert!(s.contains("My Page"));
    assert!(s.contains("Welcome!"));
    assert!(s.contains("static footer"));
}

// ===========================================================================
// 26. th:inline 内联模式
// ===========================================================================

#[test]
fn th_inline_none_disables_expression_processing() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_var("name", "test");
    let input = "<script th:inline=\"none\">var x = '${name}';</script>";
    let output = engine.process_template(input, &ctx).expect("inline none");
    let s = output.to_string_lossy();
    // th:inline=none 应保持原始内容不处理表达式
    assert!(s.contains("${name}"));
}

// ===========================================================================
// 27. th:attrappend / th:attrprepend
// ===========================================================================

#[test]
fn th_attrappend_adds_to_existing_attribute() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_var("extra", " extra-class");
    let input = "<div class=\"base\" th:attrappend=\"class=${extra}\">content</div>";
    let output = engine.process_template(input, &ctx).expect("th:attrappend");
    let s = output.to_string_lossy();
    assert!(s.contains("base"));
    assert!(s.contains("extra-class"));
}

#[test]
fn th_attrprepend_prepends_to_attribute() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_var("prefix", "prefix-");
    let input = "<div class=\"base\" th:attrprepend=\"class=${prefix}\">content</div>";
    let output = engine
        .process_template(input, &ctx)
        .expect("th:attrprepend");
    let s = output.to_string_lossy();
    assert!(s.contains("prefix-"));
}

// ===========================================================================
// 28. 配置冻结完整性
// ===========================================================================

#[test]
fn engine_configuration_accessible_after_init() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let _ = engine.process_template("init", &ctx);

    let config = engine.get_configuration().expect("config after init");
    // 配置应包含标准方言
    assert!(!config.get_dialects().is_empty());
}

// ===========================================================================
// 29. TemplateSpec 完整性
// ===========================================================================

#[test]
fn template_spec_with_selector() {
    let selectors = {
        let mut s = std::collections::BTreeSet::new();
        s.insert(Some("fragment1".to_owned()));
        s
    };
    let spec = TemplateSpec::with_selectors_and_template_mode(
        Some("template"),
        Some(&selectors),
        Some(TemplateMode::HTML),
        None,
    )
    .expect("spec with selector");
    assert_eq!(spec.get_template(), "template");
    let sel = spec.get_template_selectors();
    assert!(sel.is_some());
    assert_eq!(sel.unwrap().len(), 1);
}

// ===========================================================================
// 30. TemplateEngine 模板缓存管理
// ===========================================================================

#[test]
fn clear_template_cache_succeeds_after_init() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let _ = engine.process_template("<p>test</p>", &ctx);
    engine
        .clear_template_cache()
        .expect("clear cache after init");
}

#[test]
fn clear_template_cache_for_specific_template() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    let _ = engine.process_template("<p>test</p>", &ctx);
    engine
        .clear_template_cache_for(&Utf16String::from_rust_str("<p>test</p>"))
        .expect("clear specific cache");
}

// ===========================================================================
// 31. 解析器管理
// ===========================================================================

#[test]
fn set_template_resolvers_deduplicates() {
    let engine = TemplateEngine::new();
    let r: Arc<dyn ITemplateResolver> = Arc::new(StringTemplateResolver::new());
    engine
        .set_template_resolvers(vec![r.clone(), r.clone()])
        .expect("set resolvers");
    assert_eq!(engine.get_template_resolvers().len(), 1);
}

#[test]
fn add_template_resolver() {
    let engine = TemplateEngine::new();
    let r: Arc<dyn ITemplateResolver> = Arc::new(StringTemplateResolver::new());
    engine.add_template_resolver(r).expect("add resolver");
    assert_eq!(engine.get_template_resolvers().len(), 1);
}

// ===========================================================================
// 32. 消息解析器管理
// ===========================================================================

#[test]
fn default_message_resolver_exists() {
    let engine = TemplateEngine::new();
    let resolvers = engine.get_message_resolvers();
    assert_eq!(resolvers.len(), 1);
}

// ===========================================================================
// 33. 链接构建器管理
// ===========================================================================

#[test]
fn default_link_builder_exists() {
    let engine = TemplateEngine::new();
    let builders = engine.get_link_builders();
    assert_eq!(builders.len(), 1);
}

// ===========================================================================
// 34. 缓存管理器
// ===========================================================================

#[test]
fn default_cache_manager_exists() {
    let engine = TemplateEngine::new();
    let cm = engine.get_cache_manager();
    assert!(cm.is_some());
}

// ===========================================================================
// 35. 引擎上下文工厂
// ===========================================================================

#[test]
fn default_engine_context_factory_exists() {
    let engine = TemplateEngine::new();
    let _factory = engine.get_engine_context_factory();
    // 默认工厂应存在
}

// ===========================================================================
// 36. 解耦模板逻辑解析器
// ===========================================================================

#[test]
fn default_decoupled_template_logic_resolver_exists() {
    let engine = TemplateEngine::new();
    let _resolver = engine.get_decoupled_template_logic_resolver();
    // 默认解析器应存在
}

// ===========================================================================
// 37. th:text 数字变量
// ===========================================================================

#[test]
fn th_text_with_number_variable() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("count")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(42),
        ))),
    );
    let input = "<p th:text=\"${count}\">x</p>";
    let output = engine.process_template(input, &ctx).expect("number var");
    assert!(output.to_string_lossy().contains("42"));
}

// ===========================================================================
// 38. th:text Boolean 变量
// ===========================================================================

#[test]
fn th_text_with_boolean_variable() {
    let engine = create_engine_with_resolver();
    let ctx = ctx_with_bool("active", true);
    let input = "<p th:text=\"${active}\">x</p>";
    let output = engine.process_template(input, &ctx).expect("bool var");
    assert!(output.to_string_lossy().contains("true"));
}

// ===========================================================================
// 39. th:if with null
// ===========================================================================

#[test]
fn th_if_with_null_is_false() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    // 不设置 missing 变量，应为 null → false
    let input = "<p th:if=\"${missing}\">should not appear</p><span>ok</span>";
    let output = engine.process_template(input, &ctx).expect("th:if null");
    let s = output.to_string_lossy();
    assert!(!s.contains("should not appear"));
    assert!(s.contains("ok"));
}

// ===========================================================================
// 40. th:if with zero number
// ===========================================================================

#[test]
fn th_if_with_zero_number_is_false() {
    let engine = create_engine_with_resolver();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("count")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(0),
        ))),
    );
    let input = "<p th:if=\"${count}\">should not appear</p><span>ok</span>";
    let output = engine.process_template(input, &ctx).expect("th:if zero");
    let s = output.to_string_lossy();
    assert!(!s.contains("should not appear"));
    assert!(s.contains("ok"));
}
