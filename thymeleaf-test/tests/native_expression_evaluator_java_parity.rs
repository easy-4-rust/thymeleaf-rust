//! `NativeVariableExpressionEvaluator` Java Golden 差分测试。
//!
//! 通过模板引擎 `${}`/`*{}` 端到端覆盖 OGNL 兼容求值器的：
//! 属性导航、方法调用、集合索引/键访问、算术/逻辑/比较、
//! 三元/Elvis、空值传播、宿主对象属性与方法。

use std::any::Any;
use std::sync::Arc;

use thymeleaf::context::{Context, IContext};
use thymeleaf::expression::{TemplateObject, TemplateValue};
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
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

/// 测试用宿主对象：提供 JavaBean 风格属性与受控方法。
struct Person {
    name: Utf16String,
    age: i64,
}

impl TemplateObject for Person {
    fn java_class_name(&self) -> &str {
        "com.example.Person"
    }
    fn to_utf16_string(&self) -> Utf16String {
        js(&format!("Person({})", self.name.to_string_lossy()))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn java_get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<
        Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectPropertyError>,
    > {
        let name = property_name.to_string_lossy();
        match name.as_str() {
            "name" => Some(Ok(Some(Arc::new(TemplateValue::string(self.name.clone()))))),
            "age" => Some(Ok(Some(Arc::new(TemplateValue::Number(
                thymeleaf::util::JavaNumber::Long(self.age),
            ))))),
            _ => None,
        }
    }
    fn java_invoke_method(
        &self,
        method_name: &Utf16String,
        _arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectMethodError>>
    {
        let name = method_name.to_string_lossy();
        match name.as_str() {
            "greet" => Some(Ok(Some(Arc::new(TemplateValue::string(js(&format!(
                "Hello, {}!",
                self.name.to_string_lossy()
            ))))))),
            _ => None,
        }
    }
}

fn ctx_with_person() -> Context {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("person")),
        Some(Arc::new(TemplateValue::Object(Arc::new(Person {
            name: js("Alice"),
            age: 30,
        })))),
    );
    ctx
}

// ===========================================================================
// 1. 属性导航
// ===========================================================================

#[test]
fn property_navigation() {
    let ctx = ctx_with_person();
    let s = render("<p th:text=\"${person.name}\">x</p>", &ctx);
    assert!(s.contains("Alice"));
}

#[test]
fn numeric_property_navigation() {
    let ctx = ctx_with_person();
    let s = render("<p th:text=\"${person.age}\">x</p>", &ctx);
    assert!(s.contains("30"));
}

#[test]
fn method_invocation() {
    let ctx = ctx_with_person();
    let s = render("<p th:text=\"${person.greet()}\">x</p>", &ctx);
    assert!(s.contains("Hello, Alice!"));
}

#[test]
fn method_and_property_chain() {
    let ctx = ctx_with_person();
    let s = render("<p th:text=\"${person.name}\">x</p>", &ctx);
    assert!(s.contains("Alice"));
}

// ===========================================================================
// 2. 集合访问
// ===========================================================================

#[test]
fn list_index_access() {
    let ctx = Context::new();
    let list = vec![
        Arc::new(TemplateValue::string(js("zero"))),
        Arc::new(TemplateValue::string(js("one"))),
    ];
    ctx.set_variable(
        Some(js("items")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    let s = render("<p th:text=\"${items[1]}\">x</p>", &ctx);
    assert!(s.contains("one"));
}

#[test]
fn list_first_index() {
    let ctx = Context::new();
    let list = vec![
        Arc::new(TemplateValue::string(js("zero"))),
        Arc::new(TemplateValue::string(js("one"))),
    ];
    ctx.set_variable(
        Some(js("items")),
        Some(Arc::new(TemplateValue::List(Arc::new(list)))),
    );
    let s = render("<p th:text=\"${items[0]}\">x</p>", &ctx);
    assert!(s.contains("zero"));
}

#[test]
fn map_key_access() {
    let ctx = Context::new();
    let map = vec![(
        Arc::new(TemplateValue::string(js("key1"))),
        Arc::new(TemplateValue::string(js("value1"))),
    )];
    ctx.set_variable(
        Some(js("map")),
        Some(Arc::new(TemplateValue::Map(Arc::new(map)))),
    );
    let s = render("<p th:text=\"${map['key1']}\">x</p>", &ctx);
    assert!(s.contains("value1"));
}

// ===========================================================================
// 3. 算术运算
// ===========================================================================

#[test]
fn arithmetic_add() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${1 + 2}\">x</p>", &ctx).contains("3"));
}

#[test]
fn arithmetic_sub() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${10 - 4}\">x</p>", &ctx).contains("6"));
}

#[test]
fn arithmetic_mul() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${3 * 4}\">x</p>", &ctx).contains("12"));
}

#[test]
fn arithmetic_div() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${20 / 5}\">x</p>", &ctx).contains("4"));
}

#[test]
fn arithmetic_mod() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${17 % 5}\">x</p>", &ctx).contains("2"));
}

#[test]
fn arithmetic_with_variables() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("a")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(7),
        ))),
    );
    ctx.set_variable(
        Some(js("b")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::JavaNumber::Integer(3),
        ))),
    );
    let s = render("<p th:text=\"${a + b}\">x</p>", &ctx);
    assert!(s.contains("10"));
}

// ===========================================================================
// 4. 比较运算
// ===========================================================================

#[test]
fn comparison_eq() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${1 == 1}\">x</p>", &ctx).contains("true"));
}

#[test]
fn comparison_neq() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${1 != 2}\">x</p>", &ctx).contains("true"));
}

#[test]
fn comparison_lt_gt() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${1 < 2}\">x</p>", &ctx).contains("true"));
    assert!(render("<p th:text=\"${3 > 2}\">x</p>", &ctx).contains("true"));
}

#[test]
fn comparison_le_ge() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${2 <= 2}\">x</p>", &ctx).contains("true"));
    assert!(render("<p th:text=\"${2 >= 3}\">x</p>", &ctx).contains("false"));
}

// ===========================================================================
// 5. 逻辑运算
// ===========================================================================

#[test]
fn logical_and_or_not() {
    let ctx = Context::new();
    ctx.set_variable(Some(js("t")), Some(Arc::new(TemplateValue::Boolean(true))));
    ctx.set_variable(Some(js("f")), Some(Arc::new(TemplateValue::Boolean(false))));
    assert!(render("<p th:text=\"${t and t}\">x</p>", &ctx).contains("true"));
    assert!(render("<p th:text=\"${t or f}\">x</p>", &ctx).contains("true"));
    assert!(render("<p th:text=\"${!f}\">x</p>", &ctx).contains("true"));
    assert!(render("<p th:text=\"${t and f}\">x</p>", &ctx).contains("false"));
}

// ===========================================================================
// 6. 三元 / Elvis
// ===========================================================================

#[test]
fn ternary_true() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${1 < 2 ? 'yes' : 'no'}\">x</p>", &ctx).contains("yes"));
}

#[test]
fn ternary_false() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${1 > 2 ? 'yes' : 'no'}\">x</p>", &ctx).contains("no"));
}

#[test]
fn elvis_null_default() {
    let ctx = Context::new();
    let s = render("<p th:text=\"${missing ?: 'fallback'}\">x</p>", &ctx);
    assert!(s.contains("fallback"));
}

#[test]
fn elvis_present_value() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("v")),
        Some(Arc::new(TemplateValue::string(js("value")))),
    );
    let s = render("<p th:text=\"${v ?: 'fallback'}\">x</p>", &ctx);
    assert!(s.contains("value"));
}

// ===========================================================================
// 7. 字符串方法调用（内建 String）
// ===========================================================================

#[test]
fn string_method_uppercase() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("name")),
        Some(Arc::new(TemplateValue::string(js("alice")))),
    );
    let s = render("<p th:text=\"${name.toUpperCase()}\">x</p>", &ctx);
    assert!(s.contains("ALICE"));
}

#[test]
fn string_method_length() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("name")),
        Some(Arc::new(TemplateValue::string(js("alice")))),
    );
    let s = render("<p th:text=\"${name.length()}\">x</p>", &ctx);
    assert!(s.contains("5"));
}

#[test]
fn string_method_substring() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("name")),
        Some(Arc::new(TemplateValue::string(js("alice")))),
    );
    let s = render("<p th:text=\"${name.substring(0, 3)}\">x</p>", &ctx);
    assert!(s.contains("ali"));
}

#[test]
fn string_concat_with_plus() {
    let ctx = Context::new();
    ctx.set_variable(
        Some(js("first")),
        Some(Arc::new(TemplateValue::string(js("Hello")))),
    );
    ctx.set_variable(
        Some(js("second")),
        Some(Arc::new(TemplateValue::string(js("World")))),
    );
    let s = render("<p th:text=\"${first + ' ' + second}\">x</p>", &ctx);
    assert!(s.contains("Hello World"));
}

// ===========================================================================
// 8. 空值传播
// ===========================================================================

#[test]
fn null_variable_renders_empty() {
    let ctx = Context::new();
    let s = render("<p th:text=\"${missing}\">x</p>", &ctx);
    assert!(!s.contains("x"));
}

#[test]
fn null_property_raises_error() {
    // Java OGNL 对 null 引用的属性访问抛异常 → 渲染失败，与上游一致
    let ctx = Context::new();
    let result = engine().process_template("<p th:text=\"${person.name}\">x</p>", &ctx);
    assert!(result.is_err(), "null property access must fail like OGNL");
}

#[test]
fn null_condition_is_false() {
    let ctx = Context::new();
    let s = render("<p th:if=\"${missing}\">gone</p><span>stay</span>", &ctx);
    assert!(!s.contains("gone"));
    assert!(s.contains("stay"));
}

// ===========================================================================
// 9. 字面量
// ===========================================================================

#[test]
fn string_literal() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"'hello'\">x</p>", &ctx).contains("hello"));
}

#[test]
fn number_literal() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"42\">x</p>", &ctx).contains("42"));
}

#[test]
fn boolean_literals() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"true\">x</p>", &ctx).contains("true"));
    assert!(render("<p th:text=\"false\">x</p>", &ctx).contains("false"));
}

#[test]
fn null_literal() {
    let ctx = Context::new();
    let s = render("<p th:text=\"null\">x</p>", &ctx);
    assert!(!s.contains("x"));
}

// ===========================================================================
// 10. 嵌套表达式与复合
// ===========================================================================

#[test]
fn nested_arithmetic() {
    let ctx = Context::new();
    assert!(render("<p th:text=\"${(1 + 2) * 3}\">x</p>", &ctx).contains("9"));
}

#[test]
fn property_in_condition() {
    let ctx = ctx_with_person();
    let s = render(
        "<p th:if=\"${person.age >= 18}\" th:text=\"'adult'\">x</p>",
        &ctx,
    );
    assert!(s.contains("adult"));
}

#[test]
fn property_in_arithmetic() {
    let ctx = ctx_with_person();
    let s = render("<p th:text=\"${person.age + 10}\">x</p>", &ctx);
    assert!(s.contains("40"));
}
