//! ProcessorTemplateHandler 深入处理 Java Golden 差分测试。
//!
//! 覆盖：th:each 迭代状态变量、th:switch 复杂流、片段参数传递、
//! th:object 选择上下文、th:with 嵌套、th:insert 带参数片段、
//! 文本内联与 th:inline 组合。

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

fn list_var(name: &str, values: &[&str]) -> Context {
    let ctx = Context::new();
    let list = values
        .iter()
        .map(|v| Arc::new(TemplateValue::string(js(v))))
        .collect();
    ctx.set_variable(
        Some(js(name)),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    ctx
}

// ===========================================================================
// 1. th:each 迭代状态变量（iterStat）
// ===========================================================================

#[test]
fn th_each_iteration_status_index() {
    let ctx = list_var("items", &["a", "b", "c"]);
    let s = render(
        "<ul><li th:each=\"item, stat : ${items}\" th:text=\"${stat.index} + ':' + ${item}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("0:a"));
    assert!(s.contains("1:b"));
    assert!(s.contains("2:c"));
}

#[test]
fn th_each_iteration_status_count() {
    let ctx = list_var("items", &["a", "b"]);
    let s = render(
        "<ul><li th:each=\"item, stat : ${items}\" th:text=\"${stat.count}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("1"));
    assert!(s.contains("2"));
}

#[test]
fn th_each_iteration_status_odd_even() {
    let ctx = list_var("items", &["a", "b", "c"]);
    let s = render(
        "<ul><li th:each=\"item, stat : ${items}\" th:text=\"${stat.odd} + '/' + ${stat.even}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("true/false"));
    assert!(s.contains("false/true"));
}

#[test]
fn th_each_iteration_status_first_last() {
    let ctx = list_var("items", &["a", "b", "c"]);
    let s = render(
        "<ul><li th:each=\"item, stat : ${items}\" th:text=\"${stat.first} + '/' + ${stat.last}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("true/false"));
    assert!(s.contains("false/true"));
}

#[test]
fn th_each_iteration_status_size() {
    let ctx = list_var("items", &["a", "b", "c"]);
    let s = render(
        "<ul><li th:each=\"item, stat : ${items}\" th:text=\"${stat.size}\">x</li></ul>",
        &ctx,
    );
    assert!(s.contains("3"));
}

// ===========================================================================
// 2. th:switch 复杂流
// ===========================================================================

#[test]
fn th_switch_multiple_cases_single_match() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("m")),
        Some(Arc::new(TemplateValue::string(js("b")))),
    );
    let s = render(
        "<div th:switch=\"${m}\">\
           <p th:case=\"a\">A</p>\
           <p th:case=\"b\">B</p>\
           <p th:case=\"b\">B2</p>\
           <p th:case=\"*\">D</p>\
         </div>",
        &ctx,
    );
    assert!(s.contains("B"));
    assert!(!s.contains("A"));
    assert!(!s.contains("D"));
}

#[test]
fn th_switch_with_expression_cases() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("m")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(2),
        ))),
    );
    let s = render(
        "<div th:switch=\"${m}\">\
           <p th:case=\"1\">one</p>\
           <p th:case=\"2\">two</p>\
         </div>",
        &ctx,
    );
    assert!(s.contains("two"));
}

#[test]
fn th_switch_numeric_default() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("m")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(9),
        ))),
    );
    let s = render(
        "<div th:switch=\"${m}\">\
           <p th:case=\"1\">one</p>\
           <p th:case=\"*\">default</p>\
         </div>",
        &ctx,
    );
    assert!(s.contains("default"));
}

// ===========================================================================
// 3. th:object 选择上下文
// ===========================================================================

#[test]
fn th_object_selection_with_star_expression() {
    let ctx = Context::new();
    // selection target 为 Map：*{...} 从当前选择对象读取属性
    let map = vec![(
        Arc::new(TemplateValue::string(js("name"))),
        Arc::new(TemplateValue::string(js("Alice"))),
    )];
    ctx.set_variable(
        Some(js("user")),
        Some(Arc::new(TemplateValue::Map(Arc::new(map)))),
    );
    let s = render(
        "<div th:object=\"${user}\" th:text=\"*{name}\">x</div>",
        &ctx,
    );
    assert!(
        s.contains("Alice"),
        "selection expression must read from object: {s}"
    );
}

// ===========================================================================
// 4. th:with 嵌套作用域
// ===========================================================================

#[test]
fn th_with_nested_scope_shadows() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("x")),
        Some(Arc::new(TemplateValue::string(js("outer")))),
    );
    let s = render(
        "<div th:with=\"x='inner'\"><p th:text=\"${x}\">x</p></div><p th:text=\"${x}\">x</p>",
        &ctx,
    );
    assert!(s.contains("inner"));
    assert!(s.contains("outer"));
}

#[test]
fn th_with_multiple_variables() {
    let ctx = Context::new();
    let s = render(
        "<div th:with=\"a='x', b='y'\" th:text=\"${a + b}\">x</div>",
        &ctx,
    );
    assert!(s.contains("xy"));
}

// ===========================================================================
// 5. th:insert 带参数片段
// ===========================================================================

#[test]
fn th_insert_fragment_with_parameters() {
    let ctx = Context::new();
    let template = "<div th:fragment=\"greet(name)\"><p th:text=\"${'Hi, ' + name}\">x</p></div>\
                    <div th:insert=\"~{this :: greet('Bob')}\">x</div>";
    let s = render(template, &ctx);
    assert!(s.contains("Hi, Bob"));
}

// ===========================================================================
// 6. 文本事件与内联组合
// ===========================================================================

#[test]
fn text_with_conditional_inline() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("show")),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    let s = render("<p th:inline=\"text\">[(${show} ? 'yes' : 'no')]</p>", &ctx);
    assert!(s.contains("yes"));
}

#[test]
fn th_inline_javascript_mode() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("count")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(7),
        ))),
    );
    let s = render(
        "<script th:inline=\"javascript\">var n = [[${count}]];</script>",
        &ctx,
    );
    assert!(s.contains("7"));
}

// ===========================================================================
// 7. th:attr 多值组合
// ===========================================================================

#[test]
fn th_attr_multiple_attributes_on_element() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("url")),
        Some(Arc::new(TemplateValue::string(js("http://x.com")))),
    );
    ctx.set_variable(
        Some(js("title")),
        Some(Arc::new(TemplateValue::string(js("Link Title")))),
    );
    ctx.set_variable(
        Some(js("cls")),
        Some(Arc::new(TemplateValue::string(js("btn")))),
    );
    let s = render(
        "<a th:attr=\"href=${url},title=${title},class=${cls}\">link</a>",
        &ctx,
    );
    assert!(s.contains("http://x.com"));
    assert!(s.contains("Link Title"));
    assert!(s.contains("btn"));
}

// ===========================================================================
// 8. 组合：each + if + text
// ===========================================================================

#[test]
fn each_with_conditional_inner() {
    let ctx = list_var("items", &["a", "b", "c", "d"]);
    let s = render(
        "<ul>\
           <li th:each=\"item : ${items}\" th:if=\"${item != 'b'}\" th:text=\"${item}\">x</li>\
         </ul>",
        &ctx,
    );
    assert!(s.contains("a"));
    assert!(!s.contains("b"));
    assert!(s.contains("c"));
    assert!(s.contains("d"));
}

#[test]
fn each_with_class_alternation() {
    let ctx = list_var("items", &["a", "b", "c"]);
    let s = render(
        "<ul>\
           <li th:each=\"item, stat : ${items}\" th:classappend=\"${stat.odd} ? 'odd' : 'even'\" th:text=\"${item}\">x</li>\
         </ul>",
        &ctx,
    );
    assert!(s.contains("odd"));
    assert!(s.contains("even"));
}

// ===========================================================================
// 9. 属性值中的表达式
// ===========================================================================

#[test]
fn attribute_value_expression() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("w")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(50),
        ))),
    );
    let s = render(
        "<div th:attr=\"style=${'width:' + w + 'px'}\">x</div>",
        &ctx,
    );
    assert!(s.contains("width:50px"));
}

// ===========================================================================
// 10. th:remove 变体
// ===========================================================================

#[test]
fn th_remove_none_keeps_all() {
    let ctx = Context::new();
    let s = render("<div th:remove=\"none\"><p>keep</p></div>", &ctx);
    assert!(s.contains("keep"));
}

#[test]
fn th_remove_all_but_first() {
    let ctx = Context::new();
    let s = render(
        "<div th:remove=\"all-but-first\"><p>keep</p><p>gone1</p><p>gone2</p></div>",
        &ctx,
    );
    assert!(s.contains("keep"));
    assert!(!s.contains("gone1"));
    assert!(!s.contains("gone2"));
}

// ===========================================================================
// 11. 嵌套模板结构
// ===========================================================================

#[test]
fn deeply_nested_conditional_blocks() {
    let ctx = Context::new();
    ctx.set_variable(Some(js("a")), Some(Arc::new(TemplateValue::Boolean(true))));
    ctx.set_variable(Some(js("b")), Some(Arc::new(TemplateValue::Boolean(false))));
    let s = render(
        "<div th:if=\"${a}\">\
           <span th:if=\"${b}\">gone</span>\
           <span th:unless=\"${b}\">kept</span>\
         </div>",
        &ctx,
    );
    assert!(!s.contains("gone"));
    assert!(s.contains("kept"));
}

#[test]
fn multiple_sibling_conditional_elements() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("role")),
        Some(Arc::new(TemplateValue::string(js("admin")))),
    );
    let s = render(
        "<div>\
           <p th:if=\"${role == 'admin'}\">admin panel</p>\
           <p th:if=\"${role == 'user'}\">user panel</p>\
           <p th:unless=\"${role != 'guest'}\">guest panel</p>\
         </div>",
        &ctx,
    );
    assert!(s.contains("admin panel"));
    assert!(!s.contains("user panel"));
}
