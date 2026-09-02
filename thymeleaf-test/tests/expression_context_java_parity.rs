//! 表达式求值器与 Context 链路 Java Golden 差分测试。
//!
//! 覆盖：变量表达式、选择表达式、字面量、算术、比较、逻辑、
//! 三元运算、Elvis、方法调用、属性访问、集合操作和空值处理。

use std::sync::Arc;

use thymeleaf::context::{Context, IContext};
use thymeleaf::dialect::IDialect;
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::{ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode};

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

fn create_engine() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver set");
    engine
}

fn ctx_var(name: &str, value: &str) -> Context {
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str(name)),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            value,
        )))),
    );
    ctx
}

fn ctx_num(name: &str, value: i64) -> Context {
    use thymeleaf::util::NumberValue;
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str(name)),
        Some(Arc::new(TemplateValue::Number(NumberValue::Long(value)))),
    );
    ctx
}

fn render_err(engine: &TemplateEngine, template: &str, ctx: &dyn IContext) -> Option<String> {
    match engine.process_template(template, ctx) {
        Ok(_) => None,
        Err(error) => Some(error.to_string()),
    }
}

fn render(engine: &TemplateEngine, template: &str, ctx: &dyn IContext) -> String {
    engine
        .process_template(template, ctx)
        .expect("template renders")
        .to_string_lossy()
}

// ===========================================================================
// 1. 变量表达式 ${}
// ===========================================================================

#[test]
fn variable_string_value() {
    let engine = create_engine();
    let ctx = ctx_var("name", "Alice");
    assert!(render(&engine, "<p th:text=\"${name}\">x</p>", &ctx).contains("Alice"));
}

#[test]
fn variable_null_renders_empty() {
    let engine = create_engine();
    let ctx = Context::new();
    let s = render(&engine, "<p th:text=\"${missing}\">x</p>", &ctx);
    assert!(!s.contains("x"));
}

#[test]
fn variable_boolean_true() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("flag")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    assert!(render(&engine, "<p th:text=\"${flag}\">x</p>", &ctx).contains("true"));
}

#[test]
fn variable_boolean_false() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("flag")),
        Some(Arc::new(TemplateValue::Boolean(false))),
    );
    assert!(render(&engine, "<p th:text=\"${flag}\">x</p>", &ctx).contains("false"));
}

#[test]
fn variable_number_integer() {
    let engine = create_engine();
    let ctx = ctx_num("count", 42);
    assert!(render(&engine, "<p th:text=\"${count}\">x</p>", &ctx).contains("42"));
}

#[test]
fn variable_number_negative() {
    let engine = create_engine();
    let ctx = ctx_num("val", -100);
    assert!(render(&engine, "<p th:text=\"${val}\">x</p>", &ctx).contains("-100"));
}

#[test]
fn variable_number_zero() {
    let engine = create_engine();
    let ctx = ctx_num("val", 0);
    assert!(render(&engine, "<p th:text=\"${val}\">x</p>", &ctx).contains("0"));
}

// ===========================================================================
// 2. 字面量表达式
// ===========================================================================

#[test]
fn string_literal_single_quotes() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(render(&engine, "<p th:text=\"'hello'\">x</p>", &ctx).contains("hello"));
}

#[test]
fn string_literal_double_quotes() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(render(&engine, "<p th:text=\"'world'\">x</p>", &ctx).contains("world"));
}

#[test]
fn numeric_literal() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(render(&engine, "<p th:text=\"123\">x</p>", &ctx).contains("123"));
}

#[test]
fn boolean_literal_true() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(render(&engine, "<p th:text=\"true\">x</p>", &ctx).contains("true"));
}

#[test]
fn boolean_literal_false() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(render(&engine, "<p th:text=\"false\">x</p>", &ctx).contains("false"));
}

#[test]
fn null_literal_renders_empty() {
    let engine = create_engine();
    let ctx = Context::new();
    let s = render(&engine, "<p th:text=\"null\">x</p>", &ctx);
    assert!(!s.contains("x"));
}

// ===========================================================================
// 3. 字符串连接
// ===========================================================================

#[test]
fn string_concat_two_literals() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(
        render(
            &engine,
            "<p th:text=\"'hello' + ' ' + 'world'\">x</p>",
            &ctx
        )
        .contains("hello world")
    );
}

#[test]
fn string_concat_variable_and_literal() {
    let engine = create_engine();
    let ctx = ctx_var("name", "Alice");
    assert!(
        render(
            &engine,
            "<p th:text=\"${'Hello, ' + name + '!'}\">x</p>",
            &ctx
        )
        .contains("Hello, Alice!")
    );
}

#[test]
fn string_concat_two_variables() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("a")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "foo",
        )))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("b")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "bar",
        )))),
    );
    assert!(render(&engine, "<p th:text=\"${a + b}\">x</p>", &ctx).contains("foobar"));
}

// ===========================================================================
// 4. 算术运算
// ===========================================================================

#[test]
fn arithmetic_addition() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(render(&engine, "<p th:text=\"1 + 2\">x</p>", &ctx).contains("3"));
}

#[test]
fn arithmetic_subtraction() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(render(&engine, "<p th:text=\"10 - 3\">x</p>", &ctx).contains("7"));
}

#[test]
fn arithmetic_multiplication() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(render(&engine, "<p th:text=\"4 * 5\">x</p>", &ctx).contains("20"));
}

#[test]
fn arithmetic_division() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(render(&engine, "<p th:text=\"10 / 2\">x</p>", &ctx).contains("5"));
}

#[test]
fn arithmetic_modulo() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(render(&engine, "<p th:text=\"10 % 3\">x</p>", &ctx).contains("1"));
}

#[test]
fn arithmetic_with_variables() {
    let engine = create_engine();
    let ctx = ctx_num("x", 10);
    assert!(render(&engine, "<p th:text=\"${x + 5}\">x</p>", &ctx).contains("15"));
}

// ===========================================================================
// 5. 比较运算
// ===========================================================================

#[test]
fn comparison_equals() {
    let engine = create_engine();
    let ctx = ctx_var("status", "active");
    assert!(render(&engine, "<p th:text=\"${status == 'active'}\">x</p>", &ctx).contains("true"));
}

#[test]
fn comparison_not_equals() {
    let engine = create_engine();
    let ctx = ctx_var("status", "inactive");
    assert!(render(&engine, "<p th:text=\"${status != 'active'}\">x</p>", &ctx).contains("true"));
}

#[test]
fn comparison_less_than() {
    let engine = create_engine();
    let ctx = ctx_num("x", 5);
    assert!(render(&engine, "<p th:text=\"${x < 10}\">x</p>", &ctx).contains("true"));
}

#[test]
fn comparison_greater_than() {
    let engine = create_engine();
    let ctx = ctx_num("x", 15);
    assert!(render(&engine, "<p th:text=\"${x > 10}\">x</p>", &ctx).contains("true"));
}

#[test]
fn comparison_less_equal() {
    let engine = create_engine();
    let ctx = ctx_num("x", 10);
    assert!(render(&engine, "<p th:text=\"${x <= 10}\">x</p>", &ctx).contains("true"));
}

#[test]
fn comparison_greater_equal() {
    let engine = create_engine();
    let ctx = ctx_num("x", 10);
    assert!(render(&engine, "<p th:text=\"${x >= 10}\">x</p>", &ctx).contains("true"));
}

// ===========================================================================
// 6. 逻辑运算
// ===========================================================================

#[test]
fn logical_and() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("a")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("b")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    assert!(render(&engine, "<p th:text=\"${a and b}\">x</p>", &ctx).contains("true"));
}

#[test]
fn logical_or() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("a")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("b")),
        Some(Arc::new(TemplateValue::Boolean(false))),
    );
    assert!(render(&engine, "<p th:text=\"${a or b}\">x</p>", &ctx).contains("true"));
}

#[test]
fn logical_not() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("a")),
        Some(Arc::new(TemplateValue::Boolean(false))),
    );
    assert!(render(&engine, "<p th:text=\"${!a}\">x</p>", &ctx).contains("true"));
}

#[test]
fn logical_and_false() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("a")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("b")),
        Some(Arc::new(TemplateValue::Boolean(false))),
    );
    assert!(render(&engine, "<p th:text=\"${a and b}\">x</p>", &ctx).contains("false"));
}

// ===========================================================================
// 7. 三元运算符
// ===========================================================================

#[test]
fn ternary_true_branch() {
    let engine = create_engine();
    let ctx = ctx_num("x", 10);
    assert!(
        render(
            &engine,
            "<p th:text=\"${x > 5 ? 'big' : 'small'}\">x</p>",
            &ctx
        )
        .contains("big")
    );
}

#[test]
fn ternary_false_branch() {
    let engine = create_engine();
    let ctx = ctx_num("x", 3);
    assert!(
        render(
            &engine,
            "<p th:text=\"${x > 5 ? 'big' : 'small'}\">x</p>",
            &ctx
        )
        .contains("small")
    );
}

// ===========================================================================
// 8. Elvis 运算符 ?:
// ===========================================================================

#[test]
fn internal_elvis_rejected_like_java_ognl() {
    // Java 3.1.5 parity：`${a ?: b}`（Elvis 简写在 ${} 内部）原样交给 OGNL
    // 3.3.4 求值，而 OGNL 不支持 Elvis 简写 → 渲染期 TemplateInputException。
    // Thymeleaf 的 default expression 只支持 `${a} ?: b` 外部形式。
    // golden 锚点：ognl_evaluation_golden.txt elvis_null_default/elvis_present_value。
    let engine = create_engine();
    let ctx = Context::new();
    assert!(
        render_err(
            &engine,
            "<p th:text=\"${missing ?: 'default'}\">x</p>",
            &ctx
        )
        .is_some(),
        "内部 Elvis 应像 Java OGNL 一样解析失败"
    );
    let ctx = ctx_var("name", "Alice");
    assert!(
        render_err(&engine, "<p th:text=\"${name ?: 'default'}\">x</p>", &ctx).is_some(),
        "内部 Elvis 应像 Java OGNL 一样解析失败"
    );
}

// ===========================================================================
// 9. 条件渲染 th:if / th:unless
// ===========================================================================

#[test]
fn th_if_with_true_renders() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("show")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    let s = render(
        &engine,
        "<p th:if=\"${show}\" th:text=\"'visible'\">x</p>",
        &ctx,
    );
    assert!(s.contains("visible"));
}

#[test]
fn th_if_with_false_removes() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("show")),
        Some(Arc::new(TemplateValue::Boolean(false))),
    );
    let s = render(
        &engine,
        "<p th:if=\"${show}\">gone</p><span>stay</span>",
        &ctx,
    );
    assert!(!s.contains("gone"));
    assert!(s.contains("stay"));
}

#[test]
fn th_if_with_null_removes() {
    let engine = create_engine();
    let ctx = Context::new();
    let s = render(
        &engine,
        "<p th:if=\"${missing}\">gone</p><span>stay</span>",
        &ctx,
    );
    assert!(!s.contains("gone"));
    assert!(s.contains("stay"));
}

#[test]
fn th_unless_with_false_renders() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("hide")),
        Some(Arc::new(TemplateValue::Boolean(false))),
    );
    let s = render(
        &engine,
        "<p th:unless=\"${hide}\" th:text=\"'shown'\">x</p>",
        &ctx,
    );
    assert!(s.contains("shown"));
}

#[test]
fn th_unless_with_true_removes() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("hide")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    let s = render(
        &engine,
        "<p th:unless=\"${hide}\">gone</p><span>stay</span>",
        &ctx,
    );
    assert!(!s.contains("gone"));
    assert!(s.contains("stay"));
}

// ===========================================================================
// 10. th:each 迭代
// ===========================================================================

#[test]
fn th_each_list_items() {
    let engine = create_engine();
    let ctx = Context::new();
    let items: Vec<Arc<TemplateValue>> = ["a", "b", "c"]
        .iter()
        .map(|v| Arc::new(TemplateValue::string(Utf16String::from_rust_str(v))))
        .collect();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("items")),
        Some(Arc::new(TemplateValue::List(Arc::new(items)))),
    );
    let s = render(
        &engine,
        "<ul><li th:each=\"item : ${items}\" th:text=\"${item}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("a"));
    assert!(s.contains("b"));
    assert!(s.contains("c"));
}

#[test]
fn th_each_empty_list() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("items")),
        Some(Arc::new(TemplateValue::List(Arc::new(vec![])))),
    );
    let s = render(
        &engine,
        "<ul><li th:each=\"item : ${items}\" th:text=\"${item}\">x</li></ul>",
        &ctx,
    );
    assert!(!s.contains("<li"));
}

// ===========================================================================
// 11. th:with 局部变量
// ===========================================================================

#[test]
fn th_with_literal_value() {
    let engine = create_engine();
    let ctx = Context::new();
    let s = render(
        &engine,
        "<div th:with=\"greeting='Hello'\" th:text=\"${greeting}\">x</div>",
        &ctx,
    );
    assert!(s.contains("Hello"));
}

#[test]
fn th_with_expression_value() {
    let engine = create_engine();
    let ctx = ctx_num("x", 10);
    let s = render(
        &engine,
        "<div th:with=\"doubled=${x * 2}\" th:text=\"${doubled}\">x</div>",
        &ctx,
    );
    assert!(s.contains("20"));
}

// ===========================================================================
// 12. th:switch / th:case
// ===========================================================================

#[test]
fn th_switch_matching_case() {
    let engine = create_engine();
    let ctx = ctx_var("mode", "edit");
    let s = render(
        &engine,
        "<div th:switch=\"${mode}\">\
           <p th:case=\"view\">Viewing</p>\
           <p th:case=\"edit\">Editing</p>\
           <p th:case=\"*\">Unknown</p>\
         </div>",
        &ctx,
    );
    assert!(s.contains("Editing"));
    assert!(!s.contains("Unknown"));
}

#[test]
fn th_switch_default_case() {
    let engine = create_engine();
    let ctx = ctx_var("mode", "other");
    let s = render(
        &engine,
        "<div th:switch=\"${mode}\">\
           <p th:case=\"view\">Viewing</p>\
           <p th:case=\"edit\">Editing</p>\
           <p th:case=\"*\">Unknown</p>\
         </div>",
        &ctx,
    );
    assert!(s.contains("Unknown"));
    assert!(!s.contains("Editing"));
}

// ===========================================================================
// 13. th:block
// ===========================================================================

#[test]
fn th_block_removed_from_output() {
    let engine = create_engine();
    let ctx = Context::new();
    let s = render(&engine, "<th:block><p>content</p></th:block>", &ctx);
    assert!(s.contains("<p>content</p>"));
    assert!(!s.contains("<th:block"));
}

// ===========================================================================
// 14. th:remove
// ===========================================================================

#[test]
fn th_remove_all() {
    let engine = create_engine();
    let ctx = Context::new();
    let s = render(
        &engine,
        "<div th:remove=\"all\"><p>removed</p></div><span>kept</span>",
        &ctx,
    );
    assert!(!s.contains("removed"));
    assert!(s.contains("kept"));
}

#[test]
fn th_remove_body() {
    let engine = create_engine();
    let ctx = Context::new();
    let s = render(
        &engine,
        "<div th:remove=\"body\"><p>removed</p></div><span>kept</span>",
        &ctx,
    );
    assert!(!s.contains("removed"));
    assert!(s.contains("kept"));
}

// ===========================================================================
// 15. th:attr
// ===========================================================================

#[test]
fn th_attr_sets_attribute() {
    let engine = create_engine();
    let ctx = ctx_var("url", "https://example.com");
    let s = render(&engine, "<a th:attr=\"href=${url}\">link</a>", &ctx);
    assert!(s.contains("https://example.com"));
}

#[test]
fn th_attr_multiple_attributes() {
    let engine = create_engine();
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("url")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "http://test.com",
        )))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("title")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "My Link",
        )))),
    );
    let s = render(
        &engine,
        "<a th:attr=\"href=${url},title=${title}\">link</a>",
        &ctx,
    );
    assert!(s.contains("http://test.com"));
    assert!(s.contains("My Link"));
}

// ===========================================================================
// 16. th:attrappend / th:attrprepend
// ===========================================================================

#[test]
fn th_attrappend() {
    let engine = create_engine();
    let ctx = ctx_var("extra", " extra-class");
    let s = render(
        &engine,
        "<div class=\"base\" th:attrappend=\"class=${extra}\">x</div>",
        &ctx,
    );
    assert!(s.contains("base"));
    assert!(s.contains("extra-class"));
}

#[test]
fn th_attrprepend() {
    let engine = create_engine();
    let ctx = ctx_var("prefix", "prefix-");
    let s = render(
        &engine,
        "<div class=\"base\" th:attrprepend=\"class=${prefix}\">x</div>",
        &ctx,
    );
    assert!(s.contains("prefix-"));
}

// ===========================================================================
// 17. th:inline
// ===========================================================================

#[test]
fn th_inline_none_preserves_expressions() {
    let engine = create_engine();
    let ctx = ctx_var("name", "test");
    let s = render(
        &engine,
        "<script th:inline=\"none\">var x = '${name}';</script>",
        &ctx,
    );
    assert!(s.contains("${name}"));
}

// ===========================================================================
// 18. 多属性组合
// ===========================================================================

#[test]
fn multiple_th_on_same_element() {
    let engine = create_engine();
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
    let s = render(
        &engine,
        "<p th:if=\"${show}\" th:text=\"${text}\">x</p>",
        &ctx,
    );
    assert!(s.contains("Hello"));
    assert!(!s.contains(">x<"));
}

// ===========================================================================
// 19. Unicode 支持
// ===========================================================================

#[test]
fn unicode_in_template() {
    let engine = create_engine();
    let ctx = Context::new();
    let input = "<p>日本語テスト</p>";
    assert_eq!(render(&engine, input, &ctx), input);
}

#[test]
fn unicode_in_variable() {
    let engine = create_engine();
    let ctx = ctx_var("msg", "こんにちは");
    assert!(render(&engine, "<p th:text=\"${msg}\">x</p>", &ctx).contains("こんにちは"));
}

// ===========================================================================
// 20. 大模板
// ===========================================================================

#[test]
fn large_template_100_elements() {
    let engine = create_engine();
    let ctx = Context::new();
    let mut input = String::from("<html><body>");
    for i in 0..100 {
        input.push_str(&format!("<p>item {i}</p>"));
    }
    input.push_str("</body></html>");
    let s = render(&engine, &input, &ctx);
    assert!(s.contains("item 0"));
    assert!(s.contains("item 99"));
}

// ===========================================================================
// 21. TemplateMode 覆盖
// ===========================================================================

#[test]
fn xml_mode_preserves_content() {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::XML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let ctx = Context::new();
    let input = "<?xml version=\"1.0\"?>\n<root><item>data</item></root>";
    assert_eq!(render(&engine, input, &ctx), input);
}

#[test]
fn text_mode_preserves_content() {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::TEXT);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let ctx = Context::new();
    let input = "Hello World\nLine 2";
    assert_eq!(render(&engine, input, &ctx), input);
}

#[test]
fn raw_mode_preserves_content() {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::RAW);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let ctx = Context::new();
    let input = "<html th:text=\"'ignored'\">raw</html>";
    assert_eq!(render(&engine, input, &ctx), input);
}

// ===========================================================================
// 22. 缓存行为
// ===========================================================================

#[test]
fn repeated_render_same_template() {
    let engine = create_engine();
    let ctx = Context::new();
    let out1 = render(&engine, "<p>cached</p>", &ctx);
    let out2 = render(&engine, "<p>cached</p>", &ctx);
    assert_eq!(out1, out2);
}

#[test]
fn different_templates_independent() {
    let engine = create_engine();
    let ctx = Context::new();
    assert!(render(&engine, "<p>first</p>", &ctx).contains("first"));
    assert!(render(&engine, "<div>second</div>", &ctx).contains("second"));
}

// ===========================================================================
// 23. 配置冻结
// ===========================================================================

#[test]
fn config_rejected_after_init() {
    use thymeleaf::standard::StandardDialect;
    let engine = create_engine();
    let ctx = Context::new();
    let _ = engine.process_template("init", &ctx);
    let d: Arc<dyn IDialect> = Arc::new(StandardDialect::new());
    assert!(engine.set_dialect(d).is_err());
    assert!(engine.clear_dialects().is_err());
}

// ===========================================================================
// 24. TemplateSpec 入口
// ===========================================================================

#[test]
fn template_spec_with_mode() {
    use thymeleaf::TemplateSpec;
    let engine = create_engine();
    let ctx = Context::new();
    let spec =
        TemplateSpec::with_template_mode(Some("<p>test</p>"), Some(TemplateMode::HTML)).unwrap();
    let output = engine.process(&spec, &ctx).expect("process spec");
    assert!(output.to_string_lossy().contains("<p>test</p>"));
}

// ===========================================================================
// 25. TemplateMode 语义覆盖
// ===========================================================================

#[test]
fn template_mode_markup_flags() {
    assert!(TemplateMode::HTML.is_markup());
    assert!(TemplateMode::XML.is_markup());
    assert!(!TemplateMode::TEXT.is_markup());
    assert!(!TemplateMode::JAVASCRIPT.is_markup());
    assert!(!TemplateMode::CSS.is_markup());
    assert!(!TemplateMode::RAW.is_markup());
}

#[test]
fn template_mode_text_flags() {
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
fn template_mode_display() {
    assert_eq!(TemplateMode::HTML.to_string(), "HTML");
    assert_eq!(TemplateMode::XML.to_string(), "XML");
    assert_eq!(TemplateMode::TEXT.to_string(), "TEXT");
    assert_eq!(TemplateMode::JAVASCRIPT.to_string(), "JAVASCRIPT");
    assert_eq!(TemplateMode::CSS.to_string(), "CSS");
    assert_eq!(TemplateMode::RAW.to_string(), "RAW");
}

#[test]
fn template_mode_parse_known() {
    assert_eq!(
        TemplateMode::parse(Some("HTML")).unwrap(),
        TemplateMode::HTML
    );
    assert_eq!(TemplateMode::parse(Some("xml")).unwrap(), TemplateMode::XML);
    assert_eq!(
        TemplateMode::parse(Some("text")).unwrap(),
        TemplateMode::TEXT
    );
    assert_eq!(
        TemplateMode::parse(Some("JAVASCRIPT")).unwrap(),
        TemplateMode::JAVASCRIPT
    );
    assert_eq!(TemplateMode::parse(Some("css")).unwrap(), TemplateMode::CSS);
    assert_eq!(TemplateMode::parse(Some("RAW")).unwrap(), TemplateMode::RAW);
}

#[test]
fn template_mode_parse_unknown_defaults_html() {
    assert_eq!(
        TemplateMode::parse(Some("MARKDOWN")).unwrap(),
        TemplateMode::HTML
    );
    assert_eq!(
        TemplateMode::parse(Some("YAML")).unwrap(),
        TemplateMode::HTML
    );
}

#[test]
fn template_mode_parse_empty_errors() {
    assert!(TemplateMode::parse(Some("")).is_err());
    assert!(TemplateMode::parse(None).is_err());
    assert!(TemplateMode::parse(Some("   ")).is_err());
}

// ===========================================================================
// 26. TemplateResolutionAttributes
// ===========================================================================

#[test]
fn resolution_attributes_equality() {
    use thymeleaf::TemplateResolutionAttributeValue;
    use thymeleaf::TemplateResolutionAttributes;
    let mut a1 = TemplateResolutionAttributes::new();
    a1.insert(
        Some("k".to_owned()),
        TemplateResolutionAttributeValue::new("v".to_owned()),
    );
    let mut a2 = TemplateResolutionAttributes::new();
    a2.insert(
        Some("k".to_owned()),
        TemplateResolutionAttributeValue::new("v".to_owned()),
    );
    assert_eq!(a1, a2);
}

#[test]
fn resolution_attributes_inequality() {
    use thymeleaf::TemplateResolutionAttributeValue;
    use thymeleaf::TemplateResolutionAttributes;
    let mut a1 = TemplateResolutionAttributes::new();
    a1.insert(
        Some("k".to_owned()),
        TemplateResolutionAttributeValue::new("v1".to_owned()),
    );
    let mut a2 = TemplateResolutionAttributes::new();
    a2.insert(
        Some("k".to_owned()),
        TemplateResolutionAttributeValue::new("v2".to_owned()),
    );
    assert_ne!(a1, a2);
}

// ===========================================================================
// 27. TemplateEngine 常量和配置
// ===========================================================================

#[test]
fn timer_logger_name() {
    assert_eq!(
        TemplateEngine::TIMER_LOGGER_NAME,
        "org.thymeleaf.TemplateEngine.TIMER"
    );
}

#[test]
fn engine_starts_uninitialized() {
    let engine = TemplateEngine::new();
    assert!(!engine.is_initialized());
}

#[test]
fn engine_initializes_on_first_process() {
    let engine = create_engine();
    let ctx = Context::new();
    let _ = engine.process_template("hello", &ctx);
    assert!(engine.is_initialized());
}

#[test]
fn default_dialect_is_standard() {
    let engine = TemplateEngine::new();
    let dialects = engine.get_dialects();
    assert_eq!(dialects.len(), 1);
    assert_eq!(dialects[0].get_name(), Some("Standard"));
}

#[test]
fn clear_dialects_before_init() {
    let engine = TemplateEngine::new();
    engine.clear_dialects().unwrap();
    assert!(engine.get_dialects().is_empty());
}

#[test]
fn set_dialects_deduplicates() {
    use thymeleaf::dialect::IDialect;
    use thymeleaf::standard::StandardDialect;
    let engine = TemplateEngine::new();
    let d: Arc<dyn IDialect> = Arc::new(StandardDialect::new());
    engine.set_dialects(vec![d.clone(), d.clone()]).unwrap();
    assert_eq!(engine.get_dialects().len(), 1);
}

#[test]
fn default_message_resolver() {
    let engine = TemplateEngine::new();
    assert_eq!(engine.get_message_resolvers().len(), 1);
}

#[test]
fn default_link_builder() {
    let engine = TemplateEngine::new();
    assert_eq!(engine.get_link_builders().len(), 1);
}

#[test]
fn default_cache_manager() {
    let engine = TemplateEngine::new();
    assert!(engine.get_cache_manager().is_some());
}

#[test]
fn set_template_resolvers_deduplicates() {
    let engine = TemplateEngine::new();
    let r: Arc<dyn ITemplateResolver> = Arc::new(StringTemplateResolver::new());
    engine
        .set_template_resolvers(vec![r.clone(), r.clone()])
        .unwrap();
    assert_eq!(engine.get_template_resolvers().len(), 1);
}

#[test]
fn add_template_resolver() {
    let engine = TemplateEngine::new();
    let r: Arc<dyn ITemplateResolver> = Arc::new(StringTemplateResolver::new());
    engine.add_template_resolver(r).unwrap();
    assert_eq!(engine.get_template_resolvers().len(), 1);
}

// ===========================================================================
// 28. 混合静态与动态
// ===========================================================================

#[test]
fn mixed_static_dynamic() {
    let engine = create_engine();
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
        &engine,
        "<html><head><title th:text=\"${title}\">D</title></head><body><p th:text=\"${body}\">x</p><footer>static</footer></body></html>",
        &ctx,
    );
    assert!(s.contains("My Page"));
    assert!(s.contains("Welcome"));
    assert!(s.contains("static"));
}
