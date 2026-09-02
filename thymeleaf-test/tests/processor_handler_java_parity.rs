//! ProcessorTemplateHandler Java Golden 差分测试。
//!
//! 通过模板引擎公共 API 覆盖 ProcessorTemplateHandler 的核心路径：
//! 元素处理、属性处理、结构处理、迭代、条件、switch/case、remove、block。

use std::sync::Arc;

use thymeleaf::context::{Context, IContext};
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

fn engine() -> TemplateEngine {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

fn render_err(tmpl: &str, ctx: &dyn IContext) -> Option<String> {
    match engine().process_template(tmpl, ctx) {
        Ok(_) => None,
        Err(error) => Some(error.to_string()),
    }
}

fn render(tmpl: &str, ctx: &dyn IContext) -> String {
    engine()
        .process_template(tmpl, ctx)
        .unwrap()
        .to_string_lossy()
}

fn ctx_var(name: &str, val: &str) -> Context {
    let c = Context::new();
    c.set_variable(
        Some(Utf16String::from_rust_str(name)),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            val,
        )))),
    );
    c
}

fn ctx_bool(name: &str, val: bool) -> Context {
    let c = Context::new();
    c.set_variable(
        Some(Utf16String::from_rust_str(name)),
        Some(Arc::new(TemplateValue::Boolean(val))),
    );
    c
}

fn ctx_num(name: &str, val: i64) -> Context {
    let c = Context::new();
    c.set_variable(
        Some(Utf16String::from_rust_str(name)),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::NumberValue::Long(val),
        ))),
    );
    c
}

fn ctx_list(name: &str, vals: &[&str]) -> Context {
    let c = Context::new();
    let list: Vec<Arc<TemplateValue>> = vals
        .iter()
        .map(|v| Arc::new(TemplateValue::string(Utf16String::from_rust_str(v))))
        .collect();
    c.set_variable(
        Some(Utf16String::from_rust_str(name)),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    c
}

// ===========================================================================
// 1. th:text 元素处理
// ===========================================================================

#[test]
fn th_text_replaces_body() {
    let s = render("<p th:text=\"'hello'\">original</p>", &Context::new());
    assert!(s.contains("hello"));
    assert!(!s.contains("original"));
}

#[test]
fn th_text_escapes_html() {
    let ctx = ctx_var("v", "<b>x</b>");
    let s = render("<p th:text=\"${v}\">x</p>", &ctx);
    assert!(s.contains("&lt;b&gt;"));
    assert!(!s.contains("<b>"));
}

#[test]
fn th_utext_no_escape() {
    let ctx = ctx_var("v", "<b>x</b>");
    let s = render("<p th:utext=\"${v}\">x</p>", &ctx);
    assert!(s.contains("<b>x</b>"));
}

// ===========================================================================
// 2. th:if / th:unless 条件处理
// ===========================================================================

#[test]
fn th_if_true_renders() {
    let s = render(
        "<p th:if=\"${show}\" th:text=\"'ok'\">x</p>",
        &ctx_bool("show", true),
    );
    assert!(s.contains("ok"));
}

#[test]
fn th_if_false_removes() {
    let s = render(
        "<p th:if=\"${show}\">gone</p><span>s</span>",
        &ctx_bool("show", false),
    );
    assert!(!s.contains("gone"));
    assert!(s.contains("s"));
}

#[test]
fn th_if_null_removes() {
    let s = render("<p th:if=\"${m}\">gone</p><span>s</span>", &Context::new());
    assert!(!s.contains("gone"));
}

#[test]
fn th_if_zero_removes() {
    let s = render("<p th:if=\"${n}\">gone</p><span>s</span>", &ctx_num("n", 0));
    assert!(!s.contains("gone"));
}

#[test]
fn th_unless_false_renders() {
    let s = render(
        "<p th:unless=\"${h}\" th:text=\"'ok'\">x</p>",
        &ctx_bool("h", false),
    );
    assert!(s.contains("ok"));
}

#[test]
fn th_unless_true_removes() {
    let s = render(
        "<p th:unless=\"${h}\">gone</p><span>s</span>",
        &ctx_bool("h", true),
    );
    assert!(!s.contains("gone"));
}

// ===========================================================================
// 3. th:each 迭代处理
// ===========================================================================

#[test]
fn th_each_list() {
    let s = render(
        "<ul><li th:each=\"i:${items}\" th:text=\"${i}\">x</li></ul>",
        &ctx_list("items", &["a", "b", "c"]),
    );
    assert!(s.contains("a") && s.contains("b") && s.contains("c"));
}

#[test]
fn th_each_empty() {
    let s = render(
        "<ul><li th:each=\"i:${items}\" th:text=\"${i}\">x</li></ul>",
        &ctx_list("items", &[]),
    );
    assert!(!s.contains("<li"));
}

#[test]
fn th_each_single() {
    let s = render(
        "<ul><li th:each=\"i:${items}\" th:text=\"${i}\">x</li></ul>",
        &ctx_list("items", &["only"]),
    );
    assert!(s.contains("only"));
}

// ===========================================================================
// 4. th:with 变量赋值
// ===========================================================================

#[test]
fn th_with_literal() {
    let s = render(
        "<div th:with=\"x='hi'\" th:text=\"${x}\">x</div>",
        &Context::new(),
    );
    assert!(s.contains("hi"));
}

#[test]
fn th_with_expression() {
    let s = render(
        "<div th:with=\"d=${n*2}\" th:text=\"${d}\">x</div>",
        &ctx_num("n", 5),
    );
    assert!(s.contains("10"));
}

// ===========================================================================
// 5. th:switch / th:case / th:default
// ===========================================================================

#[test]
fn th_switch_match() {
    let s = render(
        "<div th:switch=\"${m}\"><p th:case=\"a\">A</p><p th:case=\"b\">B</p><p th:case=\"*\">D</p></div>",
        &ctx_var("m", "b"),
    );
    assert!(s.contains("B") && !s.contains("D"));
}

#[test]
fn th_switch_default() {
    let s = render(
        "<div th:switch=\"${m}\"><p th:case=\"a\">A</p><p th:case=\"*\">D</p></div>",
        &ctx_var("m", "z"),
    );
    assert!(s.contains("D") && !s.contains("A"));
}

// ===========================================================================
// 6. th:remove 结构处理
// ===========================================================================

#[test]
fn th_remove_all() {
    let s = render(
        "<div th:remove=\"all\"><p>x</p></div><span>k</span>",
        &Context::new(),
    );
    assert!(!s.contains("<p>x</p>") && s.contains("k"));
}

#[test]
fn th_remove_body() {
    let s = render(
        "<div th:remove=\"body\"><p>x</p></div><span>k</span>",
        &Context::new(),
    );
    assert!(!s.contains("<p>x</p>") && s.contains("k"));
}

#[test]
fn th_remove_tag() {
    let s = render(
        "<div th:remove=\"tag\"><p>x</p></div><span>k</span>",
        &Context::new(),
    );
    assert!(s.contains("<p>x</p>"));
}

#[test]
fn th_remove_none() {
    let s = render("<div th:remove=\"none\"><p>x</p></div>", &Context::new());
    assert!(s.contains("<p>x</p>"));
}

// ===========================================================================
// 7. th:block 容器
// ===========================================================================

#[test]
fn th_block_removed() {
    let s = render("<th:block><p>x</p></th:block>", &Context::new());
    assert!(s.contains("<p>x</p>") && !s.contains("th:block"));
}

#[test]
fn th_block_conditional() {
    let s = render(
        "<th:block th:if=\"${s}\"><p>x</p></th:block>",
        &ctx_bool("s", true),
    );
    assert!(s.contains("<p>x</p>"));
}

#[test]
fn th_block_conditional_false() {
    let s = render(
        "<th:block th:if=\"${s}\"><p>x</p></th:block><span>k</span>",
        &ctx_bool("s", false),
    );
    assert!(!s.contains("<p>x</p>") && s.contains("k"));
}

// ===========================================================================
// 8. th:attr 属性处理
// ===========================================================================

#[test]
fn th_attr_sets() {
    let s = render(
        "<a th:attr=\"href=${u}\">link</a>",
        &ctx_var("u", "http://x.com"),
    );
    assert!(s.contains("http://x.com"));
}

#[test]
fn th_attr_multiple() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("u")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "http://x.com",
        )))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("t")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "MyLink",
        )))),
    );
    let s = render("<a th:attr=\"href=${u},title=${t}\">link</a>", &ctx);
    assert!(s.contains("http://x.com") && s.contains("MyLink"));
}

#[test]
fn th_attrappend() {
    let s = render(
        "<div class=\"a\" th:attrappend=\"class=${e}\">x</div>",
        &ctx_var("e", " b"),
    );
    assert!(s.contains("a") && s.contains("b"));
}

#[test]
fn th_attrprepend() {
    let s = render(
        "<div class=\"b\" th:attrprepend=\"class=${p}\">x</div>",
        &ctx_var("p", "a-"),
    );
    assert!(s.contains("a-"));
}

// ===========================================================================
// 9. 多属性组合
// ===========================================================================

#[test]
fn multiple_th_attrs() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("s")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("t")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "Hello",
        )))),
    );
    let s = render("<p th:if=\"${s}\" th:text=\"${t}\">x</p>", &ctx);
    assert!(s.contains("Hello") && !s.contains(">x<"));
}

// ===========================================================================
// 10. th:inline
// ===========================================================================

#[test]
fn th_inline_none() {
    let ctx = ctx_var("n", "test");
    let s = render("<script th:inline=\"none\">var x='${n}';</script>", &ctx);
    assert!(s.contains("${n}"));
}

// ===========================================================================
// 11. 模板模式覆盖
// ===========================================================================

#[test]
fn xml_mode() {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::XML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let input = "<?xml version=\"1.0\"?>\n<root><item>data</item></root>";
    assert_eq!(
        e.process_template(input, &Context::new())
            .unwrap()
            .to_string_lossy(),
        input
    );
}

#[test]
fn text_mode() {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::TEXT);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let input = "Hello\nWorld";
    assert_eq!(
        e.process_template(input, &Context::new())
            .unwrap()
            .to_string_lossy(),
        input
    );
}

#[test]
fn raw_mode() {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::RAW);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let input = "<html th:text=\"'x'\">raw</html>";
    assert_eq!(
        e.process_template(input, &Context::new())
            .unwrap()
            .to_string_lossy(),
        input
    );
}

// ===========================================================================
// 12. Unicode
// ===========================================================================

#[test]
fn unicode_template() {
    let input = "<p>日本語テスト</p>";
    assert_eq!(render(input, &Context::new()), input);
}

#[test]
fn unicode_variable() {
    let s = render("<p th:text=\"${m}\">x</p>", &ctx_var("m", "こんにちは"));
    assert!(s.contains("こんにちは"));
}

// ===========================================================================
// 13. 大模板
// ===========================================================================

#[test]
fn large_template() {
    let mut input = String::from("<html><body>");
    for i in 0..200 {
        input.push_str(&format!("<p>item {i}</p>"));
    }
    input.push_str("</body></html>");
    let s = render(&input, &Context::new());
    assert!(s.contains("item 0") && s.contains("item 199"));
}

// ===========================================================================
// 14. 缓存行为
// ===========================================================================

#[test]
fn cache_hit() {
    let e = engine();
    let c = Context::new();
    let o1 = e
        .process_template("<p>c</p>", &c)
        .unwrap()
        .to_string_lossy();
    let o2 = e
        .process_template("<p>c</p>", &c)
        .unwrap()
        .to_string_lossy();
    assert_eq!(o1, o2);
}

// ===========================================================================
// 15. 表达式对象
// ===========================================================================

#[test]
fn strings_is_empty() {
    let ctx = ctx_var("v", "");
    assert!(render("<p th:text=\"${#strings.isEmpty(v)}\">x</p>", &ctx).contains("true"));
}

#[test]
fn strings_contains() {
    let ctx = ctx_var("v", "hello world");
    assert!(render("<p th:text=\"${#strings.contains(v,'world')}\">x</p>", &ctx).contains("true"));
}

#[test]
fn bools_is_true() {
    let ctx = ctx_bool("v", true);
    assert!(render("<p th:text=\"${#bools.isTrue(v)}\">x</p>", &ctx).contains("true"));
}

#[test]
fn lists_size() {
    let ctx = ctx_list("v", &["a", "b"]);
    assert!(render("<p th:text=\"${#lists.size(v)}\">x</p>", &ctx).contains("2"));
}

#[test]
fn maps_size() {
    let ctx = Context::new();
    let map = vec![(
        Arc::new(TemplateValue::string(Utf16String::from_rust_str("k"))),
        Arc::new(TemplateValue::string(Utf16String::from_rust_str("v"))),
    )];
    ctx.set_variable(
        Some(Utf16String::from_rust_str("m")),
        Some(Arc::new(TemplateValue::Map(Arc::new(map)))),
    );
    assert!(render("<p th:text=\"${#maps.size(m)}\">x</p>", &ctx).contains("1"));
}

// ===========================================================================
// 16. 算术/比较/逻辑
// ===========================================================================

#[test]
fn arithmetic() {
    assert!(render("<p th:text=\"${1+2}\">x</p>", &Context::new()).contains("3"));
    assert!(render("<p th:text=\"${10-3}\">x</p>", &Context::new()).contains("7"));
    assert!(render("<p th:text=\"${4*5}\">x</p>", &Context::new()).contains("20"));
}

#[test]
fn comparison() {
    let ctx = ctx_num("x", 10);
    assert!(render("<p th:text=\"${x>5}\">x</p>", &ctx).contains("true"));
    assert!(render("<p th:text=\"${x<5}\">x</p>", &ctx).contains("false"));
}

#[test]
fn logical() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(Utf16String::from_rust_str("a")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    ctx.set_variable(
        Some(Utf16String::from_rust_str("b")),
        Some(Arc::new(TemplateValue::Boolean(false))),
    );
    assert!(render("<p th:text=\"${a and a}\">x</p>", &ctx).contains("true"));
    assert!(render("<p th:text=\"${a or b}\">x</p>", &ctx).contains("true"));
    assert!(render("<p th:text=\"${!b}\">x</p>", &ctx).contains("true"));
}

#[test]
fn ternary() {
    let ctx = ctx_num("x", 10);
    assert!(render("<p th:text=\"${x>5?'big':'small'}\">x</p>", &ctx).contains("big"));
}

#[test]
fn elvis() {
    // Java 3.1.5 parity：内部 Elvis（含无空格形式）由 OGNL 3.3.4 拒绝。
    assert!(
        render_err("<p th:text=\"${m?:'default'}\">x</p>", &Context::new()).is_some(),
        "内部 Elvis 应解析失败（Java OGNL 语义）"
    );
}
