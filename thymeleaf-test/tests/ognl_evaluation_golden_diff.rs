//! OGNL 兼容变量表达式求值器的 Java Golden 逐案差分（V3_GOLDEN_DIFF）。
//!
//! Golden 由 `tests/java/OgnlEvaluationGolden.java` 在 pinned 上游
//! （Thymeleaf 3.1.5.RELEASE @ 10f9dd2eb8cbd98515ce14b149d115e0287d0add）
//! 上生成：同一表达式矩阵 + 同一变量集，经 TemplateEngine `th:text`/`th:if`
//! 端到端渲染，记录完整可观察结果或异常类名。本测试对每个 case 断言
//! Rust 输出与 golden 逐字节一致（含 Java 侧渲染失败的 `EXCEPTION:*` 行）。
//!
//! 关键语义锚点（Java 3.1.5 实测）：
//! - `${a ?: b}`（内部 Elvis）由 OGNL 3.3.4 拒绝 → `EXCEPTION:TemplateInputException`；
//! - `${a} ?: b`（外部 default expression）由 Thymeleaf 层求值，正常渲染；
//! - `${missing}`（th:text null）→ 属性移除、body 清空（`<p></p>`）；
//! - 无 `person` 时 `${person.name}`（null 属性访问）→ 渲染失败。

use std::any::Any;
use std::sync::Arc;

use thymeleaf::context::Context;
use thymeleaf::expression::{TemplateObject, TemplateValue};
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/ognl_evaluation_golden.txt");

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
}

fn engine() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver");
    engine
}

/// 与 Java 导出器 `Person` bean 语义一致的宿主对象。
struct Person;

impl TemplateObject for Person {
    fn class_name(&self) -> &str {
        "com.example.Person"
    }

    fn to_utf16_string(&self) -> Utf16String {
        js("Person(Alice)")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<
        Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectPropertyError>,
    > {
        match property_name.to_string_lossy().as_str() {
            "name" => Some(Ok(Some(Arc::new(TemplateValue::string(js("Alice")))))),
            "age" => Some(Ok(Some(Arc::new(TemplateValue::Number(
                thymeleaf::util::NumberValue::Integer(30),
            ))))),
            _ => None,
        }
    }

    fn invoke_method(
        &self,
        method_name: &Utf16String,
        _arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectMethodError>>
    {
        match method_name.to_string_lossy().as_str() {
            "greet" => Some(Ok(Some(Arc::new(TemplateValue::string(js(
                "Hello, Alice!",
            )))))),
            _ => None,
        }
    }
}

/// 与 Java 导出器 `context()` 完全一致的变量集。
fn context(with_person: bool) -> Context {
    let ctx = Context::new();
    if with_person {
        ctx.set_variable(
            Some(js("person")),
            Some(Arc::new(TemplateValue::Object(Arc::new(Person)))),
        );
    }
    let items = vec![
        Arc::new(TemplateValue::string(js("zero"))),
        Arc::new(TemplateValue::string(js("one"))),
    ];
    ctx.set_variable(
        Some(js("items")),
        Some(Arc::new(TemplateValue::List(Arc::new(items)))),
    );
    let map = vec![(
        Arc::new(TemplateValue::string(js("key1"))),
        Arc::new(TemplateValue::string(js("value1"))),
    )];
    ctx.set_variable(
        Some(js("map")),
        Some(Arc::new(TemplateValue::Map(Arc::new(map)))),
    );
    ctx.set_variable(
        Some(js("a")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::NumberValue::Integer(7),
        ))),
    );
    ctx.set_variable(
        Some(js("b")),
        Some(Arc::new(TemplateValue::Number(
            thymeleaf::util::NumberValue::Integer(3),
        ))),
    );
    ctx.set_variable(Some(js("t")), Some(Arc::new(TemplateValue::Boolean(true))));
    ctx.set_variable(Some(js("f")), Some(Arc::new(TemplateValue::Boolean(false))));
    ctx.set_variable(
        Some(js("name")),
        Some(Arc::new(TemplateValue::string(js("alice")))),
    );
    ctx.set_variable(
        Some(js("first")),
        Some(Arc::new(TemplateValue::string(js("Hello")))),
    );
    ctx.set_variable(
        Some(js("second")),
        Some(Arc::new(TemplateValue::string(js("World")))),
    );
    ctx.set_variable(
        Some(js("v")),
        Some(Arc::new(TemplateValue::string(js("value")))),
    );
    ctx
}

/// 单个 golden case：模板 + 上下文是否含 `person`。
struct GoldenCase {
    id: &'static str,
    template: &'static str,
    with_person: bool,
}

fn cases() -> Vec<GoldenCase> {
    // 与 OgnlEvaluationGolden.java 的 case 表同源镜像。
    let expr = |id, expression| GoldenCase {
        id,
        template: Box::leak(format!("<p th:text=\"{expression}\">KEEP</p>").into_boxed_str()),
        with_person: true,
    };
    let expr_no_person = |id, expression| GoldenCase {
        id,
        template: Box::leak(format!("<p th:text=\"{expression}\">KEEP</p>").into_boxed_str()),
        with_person: false,
    };
    let raw = |id, template| GoldenCase {
        id,
        template,
        with_person: true,
    };
    vec![
        GoldenCase {
            id: "baseline_case",
            template: "<p>10f9dd2eb8cbd98515ce14b149d115e0287d0add</p>",
            with_person: true,
        },
        expr("property_navigation", "${person.name}"),
        expr("numeric_property_navigation", "${person.age}"),
        expr("method_invocation", "${person.greet()}"),
        expr("method_and_property_chain", "${person.name}"),
        expr("list_index_access", "${items[1]}"),
        expr("list_first_index", "${items[0]}"),
        expr("map_key_access", "${map['key1']}"),
        expr("arithmetic_add", "${1 + 2}"),
        expr("arithmetic_sub", "${10 - 4}"),
        expr("arithmetic_mul", "${3 * 4}"),
        expr("arithmetic_div", "${20 / 5}"),
        expr("arithmetic_mod", "${17 % 5}"),
        expr("arithmetic_with_variables", "${a + b}"),
        expr("comparison_eq", "${1 == 1}"),
        expr("comparison_neq", "${1 != 2}"),
        expr("comparison_lt", "${1 < 2}"),
        expr("comparison_gt", "${3 > 2}"),
        expr("comparison_le", "${2 <= 2}"),
        expr("comparison_ge", "${2 >= 3}"),
        expr("logical_and_true", "${t and t}"),
        expr("logical_or", "${t or f}"),
        expr("logical_not", "${!f}"),
        expr("logical_and_false", "${t and f}"),
        expr("ternary_true", "${1 < 2 ? 'yes' : 'no'}"),
        expr("ternary_false", "${1 > 2 ? 'yes' : 'no'}"),
        expr("elvis_null_default", "${missing ?: 'fallback'}"),
        expr("elvis_present_value", "${v ?: 'fallback'}"),
        raw(
            "external_elvis_present",
            "<p th:text=\"${v} ?: 'fallback'\">KEEP</p>",
        ),
        raw(
            "external_elvis_null",
            "<p th:text=\"${missing} ?: 'outside'\">KEEP</p>",
        ),
        raw(
            "external_elvis_chain",
            "<p th:text=\"${missing} ?: (${v} ?: 'deep')\">KEEP</p>",
        ),
        expr("string_method_uppercase", "${name.toUpperCase()}"),
        expr("string_method_length", "${name.length()}"),
        expr("string_method_substring", "${name.substring(0, 3)}"),
        expr("string_concat_plus", "${first + ' ' + second}"),
        expr("null_variable", "${missing}"),
        expr_no_person("null_property_access", "${person.name}"),
        raw(
            "null_condition",
            "<p th:if=\"${missing}\">gone</p><span>stay</span>",
        ),
        expr("string_literal", "'hello'"),
        expr("number_literal", "42"),
        expr("boolean_literal_true", "true"),
        expr("boolean_literal_false", "false"),
        expr("null_literal", "null"),
        expr("nested_arithmetic", "${(1 + 2) * 3}"),
        raw(
            "property_in_condition",
            "<p th:if=\"${person.age >= 18}\" th:text=\"'adult'\">x</p>",
        ),
        expr("property_in_arithmetic", "${person.age + 10}"),
    ]
}

/// 收集错误链（含 source）的全部消息文本。
fn error_chain_text(error: &dyn std::error::Error) -> String {
    let mut text = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(err) = source {
        text.push(' ');
        text.push_str(&err.to_string());
        source = err.source();
    }
    text
}

/// Rust 渲染结果 → 与 golden 相同的单行转义。
fn escape_line(value: &str) -> String {
    value
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[test]
fn ognl_evaluation_matches_java_golden_case_by_case() {
    let golden: Vec<(String, String)> = JAVA_GOLDEN
        .lines()
        .filter_map(|line| {
            if line.is_empty() {
                return None;
            }
            let (id, outcome) = line.split_once('\t')?;
            Some((id.to_owned(), outcome.to_owned()))
        })
        .collect();
    assert_eq!(
        golden.first().map(|(id, _)| id.as_str()),
        Some("baseline_case"),
        "golden 首行必须是 baseline"
    );

    let engine = engine();
    let mut mismatches = Vec::new();
    let mut matched = 0_usize;

    for case in cases() {
        let expected = golden
            .iter()
            .find(|(id, _)| id == case.id)
            .map(|(_, outcome)| outcome.clone())
            .unwrap_or_else(|| panic!("golden 缺少 case: {}", case.id));

        let ctx = context(case.with_person);
        let actual = match engine.process_template(case.template, &ctx) {
            Ok(rendered) => escape_line(&rendered.to_string_lossy()),
            Err(error) => {
                // Java golden 记录顶层异常 SimpleName。两侧错误链消息逐字
                // 对齐（消息基线锁定），但顶层包装类不同属合法架构差异：
                // Java 在 parse 期求值（OGNL 失败 → TemplateInputException），
                // Rust parse/model/render 分层，求值失败在渲染期包装。以链上
                // 稳定消息锚映射回 Java 顶层类名：
                // - "evaluating OGNL expression"（Java CAUSE[2] 同文）→ Java
                //   顶层 TemplateInputException；
                // - "template parsing" → TemplateInputException；
                // - 其余处理类失败 → TemplateProcessingException。
                let chain = error_chain_text(error.as_ref());
                let class_name = if chain.contains("evaluating OGNL expression")
                    || chain.contains("template parsing")
                {
                    "TemplateInputException"
                } else if chain.contains("processing") || chain.contains("evaluating") {
                    "TemplateProcessingException"
                } else {
                    "TemplateEngineException"
                };
                format!("EXCEPTION:{class_name}")
            }
        };

        if actual == expected {
            matched += 1;
        } else {
            mismatches.push(format!(
                "case {}:\n  golden: {expected}\n  rust:   {actual}",
                case.id
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "OGNL 求值器与 Java golden 差分失败（matched={matched}）：\n{}",
        mismatches.join("\n")
    );
    assert_eq!(
        matched,
        golden.len(),
        "case 总数必须与 golden 行数一致（golden={} rust={matched}）",
        golden.len()
    );
}
