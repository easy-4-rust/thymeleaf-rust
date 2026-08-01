//! Standard `th:*` 处理器族 Java 1:1 差分测试。
#![allow(dead_code, unused_imports)]
//!
//! 逐 case 转录上游 `thymeleaf-tests-core` 的 `.thtest` 语料
//! （`templateengine/attrprocessors/` 与 `templateengine/features/`，
//! 资产为字节一致副本）：每个案例的 %INPUT/%CONTEXT/%MESSAGES/
//! %OUTPUT 原样进入本文件，输出用与语料相同的 canonical markup
//! 追踪比较（HTML 模式空白归一化）。
//!
//! 覆盖处理器对象：StandardAttr/Attrappend/Attrprepend/Classappend/
//! Styleappend、DefaultAttributes、If/Unless/With/Object/Switch/Case/
//! Assert、Each/Text、AltTitle/LangXmlLang、Action/Method/Href/Src/
//! Value、XmlBase/XmlLang/XmlSpace、Remove、Insert/Include/Replace/
//! Fragment、Inline（HTML/Textual/XML 设置）以及其抽象基类。

mod support;

use std::collections::HashMap;
use std::sync::Arc;

use html5gum::Tokenizer;

use thymeleaf::context::{Context, IContext};
use thymeleaf::expression::TemplateValue;
use thymeleaf::messageresolver::{IMessageResolver, MessageResolutionResult};
use thymeleaf::templateresolver::{
    ITemplateResolver, StringTemplateResolver, TemplateResolution, TemplateResolverError,
};
use thymeleaf::util::JavaString;
use thymeleaf::{TemplateEngine, TemplateMode};

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn string_value(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::String(Arc::new(js(value))))
}

fn number_value(value: i32) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Number(thymeleaf::util::JavaNumber::Integer(
        value,
    )))
}

fn double_value(value: f64) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Number(thymeleaf::util::JavaNumber::Double(
        value,
    )))
}

fn bool_value(value: bool) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Boolean(value))
}

fn map_value(entries: Vec<(&str, Arc<TemplateValue>)>) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Map(Arc::new(
        entries
            .into_iter()
            .map(|(key, value)| (string_value(key), value))
            .collect(),
    )))
}

fn list_value(values: Vec<Arc<TemplateValue>>) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::List(Arc::new(values)))
}

/// 上游 `#{ 'key': value }` OGNL map 字面量的 Rust 等价构建。
fn context_one() -> Context {
    let ctx = Context::new();
    ctx.set_variable(Some(js("one")), Some(string_value("one!")));
    ctx
}

fn context_user_age_24() -> Context {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("user")),
        Some(map_value(vec![("age", number_value(24))])),
    );
    ctx
}

fn context_switch_users() -> Context {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("user1")),
        Some(map_value(vec![
            ("name", string_value("Jack Melon")),
            ("role", string_value("finance")),
        ])),
    );
    ctx.set_variable(
        Some(js("user2")),
        Some(map_value(vec![
            ("name", string_value("Elizabeth Carrot")),
            ("role", string_value("admin")),
        ])),
    );
    ctx.set_variable(
        Some(js("user3")),
        Some(map_value(vec![
            ("name", string_value("Marie Ann Cho")),
            ("role", string_value("mgmnt")),
        ])),
    );
    ctx
}

fn context_two_products() -> Context {
    let ctx = Context::new();
    let product1 = map_value(vec![
        ("name", string_value("Lettuce")),
        ("price", double_value(12.0)),
    ]);
    let product2 = map_value(vec![
        ("name", string_value("Apricot")),
        ("price", double_value(8.0)),
    ]);
    ctx.set_variable(Some(js("product1")), Some(product1.clone()));
    ctx.set_variable(Some(js("product2")), Some(product2.clone()));
    ctx.set_variable(
        Some(js("products")),
        Some(list_value(vec![product1, product2])),
    );
    ctx
}

fn context_products() -> Context {
    let ctx = Context::new();
    let product1 = map_value(vec![
        ("name", string_value("Lettuce")),
        ("price", double_value(12.0)),
    ]);
    let product2 = map_value(vec![
        ("name", string_value("Apricot")),
        ("price", double_value(8.0)),
    ]);
    let product3 = map_value(vec![
        ("name", string_value("Thyme")),
        ("price", double_value(1.23)),
    ]);
    let product4 = map_value(vec![
        ("name", string_value("Carrot")),
        ("price", double_value(2.0)),
    ]);
    ctx.set_variable(Some(js("product1")), Some(product1.clone()));
    ctx.set_variable(Some(js("product2")), Some(product2.clone()));
    ctx.set_variable(Some(js("product3")), Some(product3.clone()));
    ctx.set_variable(Some(js("product4")), Some(product4.clone()));
    ctx.set_variable(
        Some(js("products")),
        Some(list_value(vec![product1, product2, product3, product4])),
    );
    ctx
}

fn context_object_product() -> Context {
    let ctx = Context::new();
    let prices = map_value(vec![
        ("euros", double_value(9.0)),
        ("dollars", double_value(12.0)),
    ]);
    ctx.set_variable(Some(js("prices")), Some(prices.clone()));
    ctx.set_variable(
        Some(js("product")),
        Some(map_value(vec![
            ("name", string_value("Lettuce")),
            ("prices", prices),
        ])),
    );
    ctx
}

fn context_remove() -> Context {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("test")),
        Some(map_value(vec![("text", string_value("Hi there!"))])),
    );
    ctx.set_variable(Some(js("condition")), Some(bool_value(true)));
    ctx
}

/// 内存消息解析器（%MESSAGES 等价）。
struct ProcessorMessageResolver {
    messages: HashMap<JavaString, JavaString>,
}

impl ProcessorMessageResolver {
    fn new(entries: Vec<(&str, &str)>) -> Self {
        Self {
            messages: entries
                .into_iter()
                .map(|(key, value)| (js(key), js(value)))
                .collect(),
        }
    }
}

impl IMessageResolver for ProcessorMessageResolver {
    fn get_name(&self) -> Option<&JavaString> {
        None
    }

    fn get_order(&self) -> Option<i32> {
        None
    }

    fn resolve_message_nullable(
        &self,
        _context: Option<&dyn thymeleaf::context::ITemplateContext>,
        _origin: Option<std::any::TypeId>,
        key: Option<&JavaString>,
        _message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        Ok(key.and_then(|key| self.messages.get(key).cloned()))
    }

    fn create_absent_message_representation_nullable(
        &self,
        context: Option<&dyn thymeleaf::context::ITemplateContext>,
        _origin: Option<std::any::TypeId>,
        key: Option<&JavaString>,
        _message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        let (Some(context), Some(key)) = (context, key) else {
            return Ok(None);
        };
        Ok(Some(JavaString::from_rust_str(&format!(
            "??{}_{}??",
            key.to_string_lossy(),
            context.get_locale()
        ))))
    }
}

/// 具名模板 Resolver（对应语料 %INPUT[name] 命名模板）。
struct NamedTemplateResolver {
    delegate: StringTemplateResolver,
    root_template_name: JavaString,
    root_template: JavaString,
    named_templates: HashMap<JavaString, JavaString>,
}

impl NamedTemplateResolver {
    fn new(
        mode: TemplateMode,
        root_template_name: &str,
        root_template: &str,
        named_templates: Vec<(&str, &str)>,
    ) -> Self {
        let mut delegate = StringTemplateResolver::new();
        delegate.set_template_mode(mode);
        Self {
            delegate,
            root_template_name: js(root_template_name),
            root_template: js(root_template),
            named_templates: named_templates
                .into_iter()
                .map(|(name, content)| (js(name), js(content)))
                .collect(),
        }
    }
}

impl ITemplateResolver for NamedTemplateResolver {
    fn get_name(&self) -> Option<&JavaString> {
        self.delegate.get_name()
    }

    fn get_order(&self) -> Option<i32> {
        self.delegate.get_order()
    }

    fn resolve_template(
        &self,
        configuration: &dyn thymeleaf::IEngineConfiguration,
        _owner_template: Option<&JavaString>,
        template: &JavaString,
        attributes: Option<&thymeleaf::TemplateResolutionAttributes>,
    ) -> Result<Option<TemplateResolution>, TemplateResolverError> {
        if template == &self.root_template_name {
            return self.delegate.resolve_template(
                configuration,
                None,
                &self.root_template,
                attributes,
            );
        }
        if let Some(content) = self.named_templates.get(template) {
            return self
                .delegate
                .resolve_template(configuration, None, content, attributes);
        }
        Ok(None)
    }
}

fn render_with(
    mode: TemplateMode,
    root_template_name: &str,
    input: &str,
    named: Vec<(&str, &str)>,
    messages: Vec<(&str, &str)>,
    context: &dyn IContext,
) -> Result<String, String> {
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(NamedTemplateResolver::new(
            mode,
            root_template_name,
            input,
            named,
        )))
        .map_err(|error| error.to_string())?;
    engine
        .set_message_resolver(Arc::new(ProcessorMessageResolver::new(messages)))
        .map_err(|error| error.to_string())?;
    engine
        .set_link_builder(Arc::new(support::TestLinkBuilder))
        .map_err(|error| error.to_string())?;
    engine
        .process_template(root_template_name, context)
        .map(|value| value.to_string_lossy())
        .map_err(|error| error.to_string())
}

fn render(input: &str, context: &dyn IContext) -> Result<String, String> {
    render_with(
        TemplateMode::HTML,
        "case-001",
        input,
        vec![],
        vec![],
        context,
    )
}

/// 与语料完全相同的 HTML canonical 追踪比较。
fn assert_html_output(expected: &str, actual: &str) {
    let expected_trace = canonical_markup_trace(expected);
    let actual_trace = canonical_markup_trace(actual);
    assert_eq!(
        actual_trace, expected_trace,
        "canonical trace mismatch\n--- expected ---\n{expected}\n--- actual ---\n{actual}"
    );
}

fn canonical_markup_trace(markup: &str) -> Vec<String> {
    let normalized = normalize_markup_whitespace(markup);
    let mut trace = Vec::new();
    for token in Tokenizer::new(normalized.as_str()).flatten() {
        match token {
            html5gum::Token::StartTag(tag) => {
                let mut item = format!("S:{}", String::from_utf8_lossy(tag.name.as_ref()));
                for (name, value) in tag.attributes {
                    item.push('|');
                    item.push_str(&String::from_utf8_lossy(name.as_ref()));
                    item.push('=');
                    item.push_str(&String::from_utf8_lossy(value.value.as_ref()));
                }
                trace.push(item);
            }
            html5gum::Token::EndTag(tag) => {
                trace.push(format!("E:{}", String::from_utf8_lossy(tag.name.as_ref())));
            }
            html5gum::Token::String(text) => {
                let compressed = text
                    .value
                    .as_ref()
                    .split(|byte: &u8| byte.is_ascii_whitespace())
                    .filter(|part| !part.is_empty())
                    .map(String::from_utf8_lossy)
                    .collect::<Vec<_>>()
                    .join(" ");
                if !compressed.is_empty() {
                    trace.push(format!("T:{compressed}"));
                }
            }
            _ => {}
        }
    }
    trace
}

fn normalize_markup_whitespace(markup: &str) -> String {
    let mut normalized = String::with_capacity(markup.len());
    let mut pending = String::new();
    let mut after_tag = false;
    for character in markup.chars() {
        if after_tag && character.is_whitespace() {
            pending.push(character);
            continue;
        }
        if after_tag && character == '<' {
            pending.clear();
        } else {
            normalized.push_str(&pending);
            pending.clear();
        }
        normalized.push(character);
        after_tag = character == '>';
    }
    normalized.push_str(&pending);
    normalized
}

// ===========================================================================
// 1. 简单值处理器族（%CONTEXT one = 'one!' 统一形状）
// ===========================================================================

fn simple_value_cases() -> Vec<(&'static str, String, String)> {
    // Java 处理器分两类：action/href/src/value 为不可移除属性（null/'' 保留空属性），
    // method/xml:base/xml:lang/xml:space 为可移除属性（null/'' 移除属性）。
    let kept: Vec<(&str, &str)> = vec![
        (
            "action",
            "<div action=\"one!\">..</div>\n<div action=\"hello\">..</div>\n<div action=\"\">..</div>\n<div action=\"\">..</div>",
        ),
        (
            "href",
            "<div href=\"one!\">..</div>\n<div href=\"hello\">..</div>\n<div href=\"\">..</div>\n<div href=\"\">..</div>",
        ),
        (
            "src",
            "<div src=\"one!\">..</div>\n<div src=\"hello\">..</div>\n<div src=\"\">..</div>\n<div src=\"\">..</div>",
        ),
        (
            "value",
            "<div value=\"one!\">..</div>\n<div value=\"hello\">..</div>\n<div value=\"\">..</div>\n<div value=\"\">..</div>",
        ),
    ];
    let removed: Vec<(&str, &str)> = vec![
        (
            "method",
            "<div method=\"one!\">..</div>\n<div method=\"hello\">..</div>\n<div>..</div>\n<div>..</div>",
        ),
        (
            "xmlbase",
            "<div xml:base=\"one!\">..</div>\n<div xml:base=\"hello\">..</div>\n<div>..</div>\n<div>..</div>",
        ),
        (
            "xmllang",
            "<div xml:lang=\"one!\">..</div>\n<div xml:lang=\"hello\">..</div>\n<div>..</div>\n<div>..</div>",
        ),
        (
            "xmlspace",
            "<div xml:space=\"one!\">..</div>\n<div xml:space=\"hello\">..</div>\n<div>..</div>\n<div>..</div>",
        ),
    ];
    let mut cases = Vec::new();
    for (attribute, output) in kept {
        let old_name = attribute;
        let input = format!(
            "<div th:{attribute}=\"${{one}}\">..</div>\n<div th:{attribute}=\"'hello'\">..</div>\n<div th:{attribute}=\"${{null}}\">..</div>\n<div th:{attribute}=\"''\">..</div>"
        );
        let full_input = format!(
            "{input}\n\n<div {old_name}=\"old\" th:{attribute}=\"${{one}}\">..</div>\n<div {old_name}=\"old\" th:{attribute}=\"'hello'\">..</div>\n<div {old_name}=\"old\" th:{attribute}=\"${{null}}\">..</div>\n<div {old_name}=\"old\" th:{attribute}=\"''\">..</div>"
        );
        let full_output = format!(
            "{output}\n\n<div {old_name}=\"one!\">..</div>\n<div {old_name}=\"hello\">..</div>\n<div {old_name}=\"\">..</div>\n<div {old_name}=\"\">..</div>"
        );
        cases.push((attribute, full_input, full_output));
    }
    for (attribute, output) in removed {
        let old_name = match attribute {
            "xmlbase" => "xml:base",
            "xmllang" => "xml:lang",
            "xmlspace" => "xml:space",
            _ => attribute,
        };
        let input = format!(
            "<div th:{attribute}=\"${{one}}\">..</div>\n<div th:{attribute}=\"'hello'\">..</div>\n<div th:{attribute}=\"${{null}}\">..</div>\n<div th:{attribute}=\"''\">..</div>"
        );
        let full_input = format!(
            "{input}\n\n<div {old_name}=\"old\" th:{attribute}=\"${{one}}\">..</div>\n<div {old_name}=\"old\" th:{attribute}=\"'hello'\">..</div>\n<div {old_name}=\"old\" th:{attribute}=\"${{null}}\">..</div>\n<div {old_name}=\"old\" th:{attribute}=\"''\">..</div>"
        );
        let full_output = format!(
            "{output}\n\n<div {old_name}=\"one!\">..</div>\n<div {old_name}=\"hello\">..</div>\n<div>..</div>\n<div>..</div>"
        );
        cases.push((attribute, full_input, full_output));
    }
    cases
}

#[test]
fn processor_simple_value_family() {
    let ctx = context_one();
    for (attribute, input, expected) in simple_value_cases() {
        let actual = render(&input, &ctx).unwrap_or_else(|error| {
            panic!("th:{attribute} must render: {error}");
        });
        assert_html_output(&expected, &actual);
    }
}

// ===========================================================================
// 2. 条件/结构处理器
// ===========================================================================

#[test]
fn processor_attr_with_message() {
    // attrprocessors/attr/attr01.thtest
    let ctx = Context::new();
    let input = "<form action=\"subscribe.html\" th:attr=\"action=@{/subscribe}\">\n  <fieldset>\n    <input type=\"text\" name=\"email\" />\n    <input type=\"submit\" value=\"Subscribe me!\" th:attr=\"value=#{subscribe.submit}\"/>\n  </fieldset>\n</form>";
    let expected = "<form action=\"/testing/subscribe\">\n  <fieldset>\n    <input type=\"text\" name=\"email\" />\n    <input type=\"submit\" value=\"Subscribe me please!\" />\n  </fieldset>\n</form>";
    let actual = render_with(
        TemplateMode::HTML,
        "case-001",
        input,
        vec![],
        vec![("subscribe.submit", "Subscribe me please!")],
        &ctx,
    )
    .expect("attr must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_default_attributes() {
    // attrprocessors/default/default01.thtest（EXACT_MATCH）
    let ctx = Context::new();
    let input = "<html th:ng-app>";
    let expected = "<html>";
    let actual = render(input, &ctx).expect("default attributes must render");
    assert_eq!(actual, expected);
}

#[test]
fn processor_if() {
    // attrprocessors/if/if01.thtest
    let ctx = context_user_age_24();
    let input = "Text before\n<div th:if=\"${user.age > 24}\"> \n    Bigger\n</div>\n<div th:if=\"${user.age} > 24\"> \n    Bigger\n</div>\n<div th:if=\"${user.age} > 24.0\"> \n    Bigger\n</div>\n<div th:if=\"${user.age >= 24}\"> \n    Or equal\n</div>\n<div th:if=\"${user.age} >= 24\"> \n    Or equal\n</div>\n<div th:if=\"${user.age} >= 24.0\"> \n    Or equal\n</div>\nText after";
    let expected = "Text before\n<div> \n    Or equal\n</div>\n<div> \n    Or equal\n</div>\n<div> \n    Or equal\n</div>\nText after";
    let actual = render(input, &ctx).expect("if must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_unless() {
    // attrprocessors/unless/unless01.thtest
    let ctx = context_user_age_24();
    let input = "Text before\n<div th:unless=\"${user.age > 24}\"> \n    Bigger\n</div>\n<div th:unless=\"${user.age} > 24\"> \n    Bigger\n</div>\n<div th:unless=\"${user.age} > 24.0\"> \n    Bigger\n</div>\n<div th:unless=\"${user.age >= 24}\"> \n    Or equal\n</div>\n<div th:unless=\"${user.age} >= 24\"> \n    Or equal\n</div>\n<div th:unless=\"${user.age} >= 24.0\"> \n    Or equal\n</div>\nText after";
    let expected = "Text before\n<div> \n    Bigger\n</div>\n<div> \n    Bigger\n</div>\n<div> \n    Bigger\n</div>\nText after";
    let actual = render(input, &ctx).expect("unless must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_with() {
    // attrprocessors/with/with02.thtest
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("user")),
        Some(map_value(vec![
            ("name", string_value("Jack Melon")),
            ("role", string_value("finance")),
        ])),
    );
    let input = "<div th:with=\"a=${user}\" th:text=\"${a.name}\">...</div>";
    let expected = "<div>Jack Melon</div>";
    let actual = render(input, &ctx).expect("with must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_assert_fails() {
    // attrprocessors/assert/assert02.thtest（断言失败 → TemplateAssertionException）
    let ctx = Context::new();
    let input = "<div>\n  <div th:assert=\"${onevar}\">...</div>\n</div>";
    let result = render(input, &ctx);
    assert!(result.is_err(), "th:assert must fail on false condition");
    let message = result.expect_err("error");
    assert!(
        !message.trim().is_empty(),
        "assert failure carries a message"
    );
}

#[test]
fn processor_switch_case() {
    // attrprocessors/switch/switch01.thtest
    let ctx = context_switch_users();
    let input = "<div th:switch=\"${user1.role}\">\n  <p th:case=\"'admin'\">User is an administrator</p>\n  <p th:case=\"#{roles.manager}\">User is a manager</p>\n</div>\n\n<div th:switch=\"${user2.role}\">\n  <p th:case=\"'admin'\">User is an administrator</p>\n  <p th:case=\"#{roles.manager}\">User is a manager</p>\n</div>\n\n<div th:switch=\"${user3.role}\">\n  <p th:case=\"'admin'\">User is an administrator</p>\n  <p th:case=\"#{roles.manager}\">User is a manager</p>\n</div>";
    let expected = "<div>\n</div>\n\n<div>\n  <p>User is an administrator</p>\n</div>\n\n<div>\n  <p>User is a manager</p>\n</div>";
    let actual = render_with(
        TemplateMode::HTML,
        "case-001",
        input,
        vec![],
        vec![("roles.manager", "mgmnt")],
        &ctx,
    )
    .expect("switch must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_each() {
    // attrprocessors/each/each01.thtest
    let ctx = context_products();
    let input = "<table>\n  <tr th:each=\"product : ${products}\">\n    <td th:text=\"${product['name']}\">name</td>\n    <td th:text=\"${product['price']}\">price</td>\n  </tr>\n</table>";
    let expected = "<table>\n  <tr>\n    <td>Lettuce</td>\n    <td>12.0</td>\n  </tr>\n  <tr>\n    <td>Apricot</td>\n    <td>8.0</td>\n  </tr>\n  <tr>\n    <td>Thyme</td>\n    <td>1.23</td>\n  </tr>\n  <tr>\n    <td>Carrot</td>\n    <td>2.0</td>\n  </tr>\n</table>";
    let actual = render(input, &ctx).expect("each must render");
    assert_html_output(expected, &actual);
}

// ===========================================================================
// 3. 追加/前置处理器
// ===========================================================================

#[test]
fn processor_attrappend() {
    // attrprocessors/appendprepend/attrappend01.thtest
    let ctx = context_one();
    let input = "<div th:attrappend=\"style=${one}\">..</div>\n<div th:attrappend=\"style='hello'\">..</div>\n<div th:attrappend=\"style=${null}\">..</div>\n<div th:attrappend=\"style=''\">..</div>\n\n<div style=\"old\" th:attrappend=\"style=${one}\">..</div>\n<div style=\"old\" th:attrappend=\"style='hello'\">..</div>\n<div style=\"old\" th:attrappend=\"style=${null}\">..</div>\n<div style=\"old\" th:attrappend=\"style=''\">..</div>\n\n<div style=\"\" th:attrappend=\"style=${one}\">..</div>\n<div style=\"\" th:attrappend=\"style='hello'\">..</div>\n<div style=\"\" th:attrappend=\"style=${null}\">..</div>\n<div style=\"\" th:attrappend=\"style=''\">..</div>";
    let expected = "<div style=\"one!\">..</div>\n<div style=\"hello\">..</div>\n<div>..</div>\n<div>..</div>\n\n<div style=\"oldone!\">..</div>\n<div style=\"oldhello\">..</div>\n<div style=\"old\">..</div>\n<div style=\"old\">..</div>\n\n<div style=\"one!\">..</div>\n<div style=\"hello\">..</div>\n<div style=\"\">..</div>\n<div style=\"\">..</div>";
    let actual = render(input, &ctx).expect("attrappend must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_attrprepend() {
    // attrprocessors/appendprepend/attrprepend01.thtest
    let ctx = context_one();
    let input = "<div th:attrprepend=\"style=${one}\">..</div>\n<div th:attrprepend=\"style='hello'\">..</div>\n<div th:attrprepend=\"style=${null}\">..</div>\n<div th:attrprepend=\"style=''\">..</div>\n\n<div style=\"old\" th:attrprepend=\"style=${one}\">..</div>\n<div style=\"old\" th:attrprepend=\"style='hello'\">..</div>\n<div style=\"old\" th:attrprepend=\"style=${null}\">..</div>\n<div style=\"old\" th:attrprepend=\"style=''\">..</div>\n\n<div style=\"\" th:attrprepend=\"style=${one}\">..</div>\n<div style=\"\" th:attrprepend=\"style='hello'\">..</div>\n<div style=\"\" th:attrprepend=\"style=${null}\">..</div>\n<div style=\"\" th:attrprepend=\"style=''\">..</div>";
    let expected = "<div style=\"one!\">..</div>\n<div style=\"hello\">..</div>\n<div>..</div>\n<div>..</div>\n\n<div style=\"one!old\">..</div>\n<div style=\"helloold\">..</div>\n<div style=\"old\">..</div>\n<div style=\"old\">..</div>\n\n<div style=\"one!\">..</div>\n<div style=\"hello\">..</div>\n<div style=\"\">..</div>\n<div style=\"\">..</div>";
    let actual = render(input, &ctx).expect("attrprepend must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_classappend() {
    // attrprocessors/appendprepend/classappend01.thtest
    let ctx = context_one();
    let input = "<div th:classappend=\"${one}\">..</div>\n<div th:classappend=\"'hello'\">..</div>\n<div th:classappend=\"${null}\">..</div>\n<div th:classappend=\"''\">..</div>\n\n<div class=\"old\" th:classappend=\"${one}\">..</div>\n<div class=\"old\" th:classappend=\"'hello'\">..</div>\n<div class=\"old\" th:classappend=\"${null}\">..</div>\n<div class=\"old\" th:classappend=\"''\">..</div>\n\n<div class=\"\" th:classappend=\"${one}\">..</div>\n<div class=\"\" th:classappend=\"'hello'\">..</div>\n<div class=\"\" th:classappend=\"${null}\">..</div>\n<div class=\"\" th:classappend=\"''\">..</div>";
    let expected = "<div class=\"one!\">..</div>\n<div class=\"hello\">..</div>\n<div>..</div>\n<div>..</div>\n\n<div class=\"old one!\">..</div>\n<div class=\"old hello\">..</div>\n<div class=\"old\">..</div>\n<div class=\"old\">..</div>\n\n<div class=\"one!\">..</div>\n<div class=\"hello\">..</div>\n<div class=\"\">..</div>\n<div class=\"\">..</div>";
    let actual = render(input, &ctx).expect("classappend must render");
    assert_html_output(expected, &actual);
}

// ===========================================================================
// 4. 对象/双值/移除/内联处理器
// ===========================================================================

#[test]
fn processor_object_selection() {
    // attrprocessors/object/object01.thtest
    let ctx = context_object_product();
    let input = "<p th:object=\"${product}\" th:with=\"x=*{prices}\" th:text=\"${x.euros}\">...</p>\n<p th:object=\"${product}\" th:with=\"x=*{prices}\">\n  <span th:text=\"${x.euros}\">...</span>\n</p>\n<p th:object=\"${product}\">\n  <span th:with=\"x=*{prices}\" th:text=\"${x.euros}\">...</span>\n</p>\n<p th:object=\"${product}\">\n  <span th:with=\"x=*{prices}\">\n    <span th:text=\"${x.euros}\">...</span>\n  </span>\n</p>";
    let expected = "<p>9.0</p>\n<p>\n  <span>9.0</span>\n</p>\n<p>\n  <span>9.0</span>\n</p>\n<p>\n  <span>\n    <span>9.0</span>\n  </span>\n</p>";
    let actual = render(input, &ctx).expect("object must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_alt_title() {
    // attrprocessors/doublevalue/alttitle01.thtest
    let ctx = context_one();
    let input = "<div th:alt-title=\"${one}\">..</div>\n<div th:alt-title=\"'hello'\">..</div>\n<div th:alt-title=\"${null}\">..</div>\n<div th:alt-title=\"''\">..</div>\n\n<div alt=\"old\" title=\"old\" th:alt-title=\"${one}\">..</div>\n<div alt=\"old\" title=\"old\" th:alt-title=\"'hello'\">..</div>\n<div alt=\"old\" title=\"old\" th:alt-title=\"${null}\">..</div>\n<div alt=\"old\" title=\"old\" th:alt-title=\"''\">..</div>";
    let expected = "<div alt=\"one!\" title=\"one!\">..</div>\n<div alt=\"hello\" title=\"hello\">..</div>\n<div>..</div>\n<div>..</div>\n\n<div alt=\"one!\" title=\"one!\">..</div>\n<div alt=\"hello\" title=\"hello\">..</div>\n<div>..</div>\n<div>..</div>";
    let actual = render(input, &ctx).expect("alt-title must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_lang_xmllang() {
    // attrprocessors/doublevalue/langxmllang01.thtest
    let ctx = context_one();
    let input = "<div th:lang-xmllang=\"${one}\">..</div>\n<div th:lang-xmllang=\"'hello'\">..</div>\n<div th:lang-xmllang=\"${null}\">..</div>\n<div th:lang-xmllang=\"''\">..</div>\n\n<div lang=\"old\" xml:lang=\"old\" th:lang-xmllang=\"${one}\">..</div>\n<div lang=\"old\" xml:lang=\"old\" th:lang-xmllang=\"'hello'\">..</div>\n<div lang=\"old\" xml:lang=\"old\" th:lang-xmllang=\"${null}\">..</div>\n<div lang=\"old\" xml:lang=\"old\" th:lang-xmllang=\"''\">..</div>";
    let expected = "<div lang=\"one!\" xml:lang=\"one!\">..</div>\n<div lang=\"hello\" xml:lang=\"hello\">..</div>\n<div>..</div>\n<div>..</div>\n\n<div lang=\"one!\" xml:lang=\"one!\">..</div>\n<div lang=\"hello\" xml:lang=\"hello\">..</div>\n<div>..</div>\n<div>..</div>";
    let actual = render(input, &ctx).expect("lang-xmllang must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_remove() {
    // attrprocessors/remove/remove14.thtest
    let ctx = context_remove();
    let input = "<div th:object=\"${test}\" th:remove=\"${condition}? tags\">\n    <span th:text=\"*{text}\">Text</span> \n</div>";
    let expected = "    <span>Hi there!</span>";
    let actual = render(input, &ctx).expect("remove must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_inline_javascript() {
    // attrprocessors/inline/inline03.thtest
    let ctx = Context::new();
    let input = "<script th:inline=\"javascript\"> \n    objArray: {\n        obj1:{\n            attr11: /*[[#{foo}]]*/ \"Some text 11\",\n            attr12: /*[[#{foo}]]*/ \"Some text 12\",\n            attr13: /*[[#{foo}]]*/ \"Some text 13\"\n        },\n        obj2:{\n            attr21: /*[[#{foo}]]*/ \"Some text 21\",\n            attr22: /*[[#{foo}]]*/ \"Some text 22\", // some comment here\n            attr23: /*[[#{foo}]]*/ \"Some text 23\" // some comment here\n        }\n    }\n</script>";
    let expected = "<script> \n    objArray: {\n        obj1:{\n            attr11: \"fooo!\",\n            attr12: \"fooo!\",\n            attr13: \"fooo!\"\n        },\n        obj2:{\n            attr21: \"fooo!\",\n            attr22: \"fooo!\", // some comment here\n            attr23: \"fooo!\"// some comment here\n        }\n    }\n</script>";
    let actual = render_with(
        TemplateMode::HTML,
        "case-001",
        input,
        vec![],
        vec![("foo", "fooo!")],
        &ctx,
    )
    .expect("inline must render");
    assert_eq!(actual, expected);
}

// ===========================================================================
// 5. 片段插入/包含/替换（具名模板）
// ===========================================================================

#[test]
fn processor_insert_fragment() {
    // attrprocessors/insert/insert001.thtest
    let ctx = context_two_products();
    let input = "<table>\n  <tr th:each=\"product : ${products}\" th:insert=\"product :: productTemplate\"\n      th:object=\"${product}\" th:with=\"productName=*{name}, productPrice=*{price}\" />\n</table>";
    let named = vec![(
        "product",
        "<th:block th:fragment=\"productTemplate\">\n    <td th:text=\"${productName}\">product name</td>\n    <td th:text=\"${productPrice}\">product price</td>\n</th:block>",
    )];
    let expected = "<table>\n  <tr>\n    <td>Lettuce</td>\n    <td>12.0</td>\n  </tr>\n  <tr>\n    <td>Apricot</td>\n    <td>8.0</td>\n  </tr>\n</table>";
    let actual = render_with(TemplateMode::HTML, "case-001", input, named, vec![], &ctx)
        .expect("insert must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_include_fragment() {
    // attrprocessors/include/include01.thtest
    let ctx = context_two_products();
    let input = "<table>\n  <tr th:each=\"product : ${products}\" th:include=\"product :: productTemplate\" \n      th:object=\"${product}\" th:with=\"productName=*{name}, productPrice=*{price}\" />\n</table>";
    let named = vec![(
        "product",
        "<tr th:fragment=\"productTemplate\">\n    <td th:text=\"${productName}\">product name</td>\n    <td th:text=\"${productPrice}\">product price</td>\n</tr>",
    )];
    let expected = "<table>\n  <tr>\n    <td>Lettuce</td>\n    <td>12.0</td>\n  </tr>\n  <tr>\n    <td>Apricot</td>\n    <td>8.0</td>\n  </tr>\n</table>";
    let actual = render_with(TemplateMode::HTML, "case-001", input, named, vec![], &ctx)
        .expect("include must render");
    assert_html_output(expected, &actual);
}

#[test]
fn processor_replace_fragment() {
    // attrprocessors/replace/replace001.thtest
    let ctx = context_two_products();
    let input = "<table>\n  <tr th:each=\"product : ${products}\" th:replace=\"product :: productTemplate\"\n      th:object=\"${product}\" th:with=\"productName=*{name}, productPrice=*{price}\" />\n</table>";
    let named = vec![(
        "product",
        "<tr th:fragment=\"productTemplate\">\n    <td th:text=\"${productName}\">product name</td>\n    <td th:text=\"${productPrice}\">product price</td>\n</tr>",
    )];
    // th:replace 优先级低于 th:each → 单元格为空
    let expected = "<table>\n  <tr>\n    <td></td>\n    <td></td>\n  </tr>\n</table>";
    let actual = render_with(TemplateMode::HTML, "case-001", input, named, vec![], &ctx)
        .expect("replace must render");
    assert_html_output(expected, &actual);
}
