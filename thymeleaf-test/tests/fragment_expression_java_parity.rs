//! `FragmentExpression` Java Golden 差分测试。
//!
//! 覆盖：`~{}` 语法解析、模板名/选择器/参数提取、synthetic 参数、
//! empty 表达式和无效输入。

use thymeleaf::expression::FragmentExpression;
use thymeleaf::util::Utf16String;

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
}

// ===========================================================================
// 1. 基本语法解析
// ===========================================================================

#[test]
fn parse_named_fragment() {
    // ~{templatename::selector}
    let expr = FragmentExpression::parse_fragment_expression(Some(&js("~{mytemplate::fragment1}")))
        .expect("valid fragment expression");
    assert!(expr.get_template_name().is_some());
    assert!(expr.get_fragment_selector().is_some());
    assert!(!expr.has_parameters());
    assert!(!expr.has_synthetic_parameters());
}

#[test]
fn parse_template_name_only() {
    // ~{templatename}
    let expr = FragmentExpression::parse_fragment_expression(Some(&js("~{mytemplate}")))
        .expect("valid fragment expression");
    assert!(expr.get_template_name().is_some());
    assert!(!expr.has_fragment_selector());
    assert!(!expr.has_parameters());
}

#[test]
fn parse_selector_only() {
    // ~{::selector} 只有选择器
    let expr = FragmentExpression::parse_fragment_expression(Some(&js("~{::frag}")))
        .expect("valid fragment expression");
    assert!(expr.get_fragment_selector().is_some());
}

#[test]
fn parse_empty_expression() {
    // ~{} 为空表达式
    let expr = FragmentExpression::parse_fragment_expression(Some(&js("~{}")));
    assert!(expr.is_some());
    let expr = expr.unwrap();
    assert!(expr.get_template_name().is_none());
    assert!(expr.get_fragment_selector().is_none());
}

#[test]
fn parse_with_whitespace() {
    let expr = FragmentExpression::parse_fragment_expression(Some(&js("  ~{ tpl :: frag }  ")))
        .expect("whitespace tolerated");
    assert!(expr.get_template_name().is_some());
    assert!(expr.get_fragment_selector().is_some());
}

// ===========================================================================
// 2. 参数解析
// ===========================================================================

#[test]
fn parse_with_parameters() {
    // ~{tpl::frag(param1='value1',param2='value2')}
    let expr =
        FragmentExpression::parse_fragment_expression(Some(&js("~{tpl::frag(a='1',b='2')}")))
            .expect("valid with parameters");
    assert!(expr.has_parameters());
    assert!(!expr.has_synthetic_parameters());
}

#[test]
fn parse_synthetic_parameters() {
    // ~{tpl::frag('value1','value2')} 无名参数 → synthetic
    let expr = FragmentExpression::parse_fragment_expression(Some(&js("~{tpl::frag('v1','v2')}")))
        .expect("valid with synthetic parameters");
    assert!(expr.has_synthetic_parameters());
}

// ===========================================================================
// 3. 无效输入
// ===========================================================================

#[test]
fn parse_null_input() {
    assert!(FragmentExpression::parse_fragment_expression(None).is_none());
}

#[test]
fn parse_not_fragment_expression() {
    // 不是 ~{} 前缀
    assert!(FragmentExpression::parse_fragment_expression(Some(&js("${other}"))).is_none());
}

#[test]
fn parse_too_short() {
    assert!(FragmentExpression::parse_fragment_expression(Some(&js("~{}"))).is_some());
    assert!(FragmentExpression::parse_fragment_expression(Some(&js("~"))).is_none());
    assert!(FragmentExpression::parse_fragment_expression(Some(&js(""))).is_none());
}

#[test]
fn parse_unclosed_brace() {
    assert!(FragmentExpression::parse_fragment_expression(Some(&js("~{tpl::frag"))).is_none());
}

#[test]
fn parse_wrong_prefix() {
    assert!(FragmentExpression::parse_fragment_expression(Some(&js("{tpl::frag}"))).is_none());
}

// ===========================================================================
// 4. 特殊语法
// ===========================================================================

#[test]
fn parse_this_reference() {
    // ~{::frag} 表示当前模板
    let expr = FragmentExpression::parse_fragment_expression(Some(&js("~{::frag}")))
        .expect("this template fragment");
    assert!(expr.get_fragment_selector().is_some());
}

#[test]
fn parse_template_only_with_colon_returns_none() {
    // ~{tpl::} 空选择器触发参数回退路径，按实现返回 None
    assert!(FragmentExpression::parse_fragment_expression(Some(&js("~{tpl::}"))).is_none());
}

#[test]
fn parse_selector_with_quotes() {
    let expr = FragmentExpression::parse_fragment_expression(Some(&js("~{'quoted'::frag}")))
        .expect("quoted template name");
    assert!(expr.get_template_name().is_some());
}

// ===========================================================================
// 5. 名称/选择器表达式语义
// ===========================================================================

#[test]
fn template_name_is_expression() {
    let expr =
        FragmentExpression::parse_fragment_expression(Some(&js("~{base/head}"))).expect("valid");
    // template 名被解析为默认字面量表达式
    assert!(expr.get_template_name().is_some());
}

#[test]
fn selector_is_expression() {
    let expr =
        FragmentExpression::parse_fragment_expression(Some(&js("~{tpl::content}"))).expect("valid");
    assert!(expr.get_fragment_selector().is_some());
}

// ===========================================================================
// 6. 通过模板引擎端到端（th:insert / th:replace）
// ===========================================================================

#[test]
fn th_insert_with_fragment_expression() {
    use std::sync::Arc;
    use thymeleaf::context::Context;
    use thymeleaf::templateresolver::StringTemplateResolver;
    use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let ctx = Context::new();

    // 同名模板内联片段
    let template =
        "<div th:fragment=\"frag\">FragContent</div><p th:insert=\"~{this :: frag}\">x</p>";
    let out = e
        .process_template(template, &ctx)
        .unwrap()
        .to_string_lossy();
    assert!(out.contains("FragContent"));
}

#[test]
fn th_replace_with_fragment_expression() {
    use std::sync::Arc;
    use thymeleaf::context::Context;
    use thymeleaf::templateresolver::StringTemplateResolver;
    use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let ctx = Context::new();

    let template =
        "<div th:fragment=\"frag\">FragContent</div><p th:replace=\"~{this :: frag}\">x</p>";
    let out = e
        .process_template(template, &ctx)
        .unwrap()
        .to_string_lossy();
    assert!(out.contains("FragContent"));
    // th:replace 替换整个元素
    assert!(!out.contains(">x</p>"));
}
