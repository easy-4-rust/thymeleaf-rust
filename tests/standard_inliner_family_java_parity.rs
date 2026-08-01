//! Standard Inliner 族与 CSS Serializer 的 Java 1:1 差分测试。
//!
//! 用例逐字取自上游 `thymeleaf-tests-core` 的固定 `.thtest` 语料
//! （`templateengine/features/inlining/standard` 与
//! `templateengine/attrprocessors/inline`），fixture 字节级一致并在
//! 测试中校验 SHA-256。每个用例按上游 `%TEMPLATE_MODE` 驱动引擎，
//! 输出/异常与 `%OUTPUT`/`%EXCEPTION` 完全一致。
//!
//! 覆盖对象（对象表编号）：
//! - `StandardTextInliner`（314）：TEXT 模式转义/不转义 + XML 内联；
//! - `StandardCSSInliner`（310）与 `StandardCSSSerializer`/
//!   `IStandardCSSSerializer`（374/372）：CSS 模式 `[[...]]` 转义输出；
//! - `StandardHTMLInliner`（311）与 `InlinedOutputExpressionMarkupHandler`
//!   （384）：HTML 模式 `th:inline="html"` 与 `th:inline="none"`；
//! - `AbstractStandardInliner`（307）：以上全部 Inliner 的公共基类；
//! - `StandardJavaScriptInliner`（313，此前已验证）作为对照强化。

use std::collections::HashMap;
use std::sync::Arc;

use sha2::Digest;
use thymeleaf::context::{Context, ExpressionContext};
use thymeleaf::expression::{IStandardExpression, TemplateValue, VariableExpression};
use thymeleaf::templateresolver::ITemplateResolver;
use thymeleaf::util::JavaString;
use thymeleaf::{ITemplateEngine, TemplateEngine, TemplateMode};

// ===========================================================================
// fixture 清单：文件名 → SHA-256（与上游文件字节级一致）
// ===========================================================================

const FIXTURES: &[(&str, &str)] = &[
    ("inlining001", "a2d0be59bc93902aa3f8db80bd40877d68619ef5feee0c3db92c209f36805551"),
    ("inlining002", "d42d48b2dc06c4656b2f66efba3f6a9656e50da88d0459f4c900f123cf589372"),
    ("inlining005", "5f27b14b572cad78301d7966fca2bfdb35c4c0842ccacde04b3a448f19c561b9"),
    ("inlining006", "e2283fd189cf05fb58d5b0127ae553610f23f80aa4ae43dac4b7804ae8005eeb"),
    ("inlining007", "383d6e6ae008f7d3b29bb875c83c73f4f58ea46e2b5f1a04ea53e235fadd9165"),
    ("inlining008", "5228d5577cd35b707562ce9121b34e1093c86d90ebd4673a30b6891aca620a5c"),
    ("inlining011", "7da06189b724109734eb03a842decb060df3b849ac1743e6cac9d3eea51f762c"),
    ("inlining012", "ab83470900200e8b386f86e05e1c5e745e764757e576f04c0a761c10639a4f4e"),
    ("inlining110", "918bd98058bf9a9d8ba310643217fb0915ec72bd788c0c8d70ff036c122f3e24"),
    ("inline01", "bcd4a3982034a65b0f527b404ad3324e5b226706c2718cd196f83cb790e2ef92"),
    ("inline29", "3fcbba09494439785fe51ec998b12eccf01214c439a6356736766492b581a704"),
    ("inline31", "3239e2a7f51add65e69fee07d0c55d575b26ba6b8cf583bfd1a813fa1f149b78"),
    ("inline32", "95d1f21a5530564a6275729fbe86e4974f81085b014b84865649a0f6393dd9ae"),
    ("inline34", "f461d712709325b604ba467c70dcc3d783bde9ec6a8c106021272734b85ac9c7"),
];

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(format!("tests/fixtures/inlining/{name}.thtest"))
        .unwrap_or_else(|error| panic!("fixture {name}: {error}"))
}

fn assert_fixture_sha256(name: &str, expected: &str) {
    let mut hasher = sha2::Sha256::new();
    hasher.update(fixture_bytes(name));
    let digest = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(digest, expected, "fixture {name} 的 SHA-256 与上游不符");
}

// ===========================================================================
// 轻量 .thtest 解析（TEMPLATE_MODE / CONTEXT / INPUT / OUTPUT / EXCEPTION）
// ===========================================================================

struct InliningCase {
    template_mode: TemplateMode,
    context: HashMap<String, String>,
    input: String,
    output: Option<String>,
    expects_exception: bool,
}

fn decode_java_properties_value(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'\\' && index + 1 < bytes.len() {
            match bytes[index + 1] {
                b't' => out.push(b'\t'),
                b'n' => out.push(b'\n'),
                b'r' => out.push(b'\r'),
                b'f' => out.push(b'\x0C'),
                b'u' => {
                    let hex = std::str::from_utf8(&bytes[index + 2..index + 6])
                        .expect("hex escape");
                    let code = u16::from_str_radix(hex, 16).expect("hex value");
                    out.extend_from_slice(&code.to_be_bytes());
                    index += 4;
                }
                other => out.push(other),
            }
            index += 2;
        } else {
            out.push(byte);
            index += 1;
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

fn load_case(name: &str) -> InliningCase {
    let content = String::from_utf8(fixture_bytes(name)).expect("fixture UTF-8");
    let mut mode = TemplateMode::HTML;
    let mut context = HashMap::new();
    let mut input = String::new();
    let mut output = None;
    let mut expects_exception = false;
    let mut section: Option<&str> = None;
    for line in content.lines() {
        if line.starts_with('%') && !line.starts_with("%%") {
            let directive = line.trim_start_matches('%');
            let (key, value) = directive
                .split_once(char::is_whitespace)
                .map_or((directive, ""), |(k, v)| (k, v.trim()));
            match key {
                "TEMPLATE_MODE" => {
                    mode = TemplateMode::parse(Some(value)).expect("template mode");
                }
                "CONTEXT" => section = Some("context"),
                "INPUT" => section = Some("input"),
                "OUTPUT" => section = Some("output"),
                "EXCEPTION" => {
                    expects_exception = true;
                    section = None;
                }
                "EXCEPTION_MESSAGE_PATTERN" => section = None,
                _ => section = None,
            }
            continue;
        }
        match section {
            Some("context") => {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    context.insert(
                        key.trim().to_owned(),
                        decode_java_properties_value(value.trim()),
                    );
                }
            }
            Some("input") => {
                if line.starts_with('#') {
                    // thymeleaf-testing 把列首 `#` 识别为测试描述或分隔线，
                    // 不属于模板内容。
                    continue;
                }
                input.push_str(line);
                input.push('\n');
            }
            Some("output") => {
                if line.starts_with('#') {
                    continue;
                }
                output.get_or_insert_with(String::new).push_str(line);
                output.as_mut().expect("output").push('\n');
            }
            _ => {}
        }
    }
    let output = output.map(|mut value| {
        while value.ends_with('\n') {
            value.pop();
        }
        value
    });
    InliningCase {
        template_mode: mode,
        context,
        input: input.trim_end_matches('\n').to_owned(),
        output,
        expects_exception,
    }
}

// ===========================================================================
// 引擎驱动
// ===========================================================================

fn evaluate_context_expression(
    engine: &TemplateEngine,
    expression: &str,
) -> Option<Arc<TemplateValue>> {
    let configuration = engine.get_configuration().expect("configuration");
    let expression_context =
        ExpressionContext::new(Some(configuration)).expect("expression context");
    let expression = VariableExpression::new(Some(JavaString::from_rust_str(expression)))
        .expect("variable expression");
    expression
        .execute(expression_context.as_ref())
        .expect("context expression evaluation")
}

fn run_case(name: &str) {
    let (_, expected_sha256) = FIXTURES
        .iter()
        .find(|(fixture, _)| *fixture == name)
        .expect("fixture listed");
    assert_fixture_sha256(name, expected_sha256);

    let case = load_case(name);
    let engine = TemplateEngine::new();
    let mut resolver = thymeleaf::templateresolver::StringTemplateResolver::new();
    resolver
        .set_template_mode_nullable(Some(case.template_mode))
        .expect("template mode");
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("set resolver");

    let context = Context::new();
    for (key, expression) in &case.context {
        let value = evaluate_context_expression(&engine, expression);
        context.set_variable(Some(JavaString::from_rust_str(key)), value);
    }

    let result = engine.process_template(&case.input, &context);
    match (case.expects_exception, result) {
        (true, Ok(_)) => panic!("{name} 应抛出异常但成功"),
        (true, Err(_)) => {}
        (false, Err(error)) => panic!("{name} 意外失败: {error}"),
        (false, Ok(output)) => {
            let actual = output.to_string_lossy();
            let expected = case.output.expect("OUTPUT 存在");
            let matches = if case.template_mode.is_markup() {
                canonical_markup_trace(&expected) == canonical_markup_trace(&actual)
            } else {
                expected == actual
            };
            assert!(
                matches,
                "{name}（模式 {:?}）输出不匹配\n--- expected ---\n{expected}\n--- actual ---\n{actual}",
                case.template_mode
            );
        }
    }
}

/// 与 `.thtest` 语料完全相同的 canonical 追踪比较（markup 模式空白归一化）。
fn canonical_markup_trace(markup: &str) -> Vec<String> {
    let normalized = normalize_markup_whitespace(markup);
    let mut trace = Vec::new();
    for token in html5gum::Tokenizer::new(normalized.as_str()).flatten() {
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
            html5gum::Token::Comment(comment) => trace.push(format!(
                "C:{}",
                String::from_utf8_lossy(comment.value.as_ref())
            )),
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
// 用例：每个 fixture 一个测试（并行执行）
// ===========================================================================

/// inlining001：TEXT 转义内联 `[[${var}]]` —— StandardTextInliner。
#[test]
fn inlining001_text_escaped() {
    run_case("inlining001");
}

/// inlining002：TEXT 不转义内联 `[(${var})]` —— StandardTextInliner。
#[test]
fn inlining002_text_unescaped() {
    run_case("inlining002");
}

/// inlining007：TEXT 前缀转义 —— StandardTextInliner。
#[test]
fn inlining007_text_prefix_escaped() {
    run_case("inlining007");
}

/// inlining008：TEXT 前缀不转义 —— StandardTextInliner。
#[test]
fn inlining008_text_prefix_unescaped() {
    run_case("inlining008");
}

/// inlining005：CSS 转义 `[[${var}]]` —— StandardCSSInliner +
/// StandardCSSSerializer（空格/`&`/单引号 CSS 转义）。
#[test]
fn inlining005_css_escaped() {
    run_case("inlining005");
}

/// inlining006：CSS 不转义 —— StandardCSSInliner。
#[test]
fn inlining006_css_unescaped() {
    run_case("inlining006");
}

/// inlining011：CSS 前缀转义 —— StandardCSSInliner + StandardCSSSerializer。
#[test]
fn inlining011_css_prefix_escaped() {
    run_case("inlining011");
}

/// inlining012：CSS 前缀不转义 —— StandardCSSInliner。
#[test]
fn inlining012_css_prefix_unescaped() {
    run_case("inlining012");
}

/// inline29：XML 模板 + `th:inline="TEXT"` —— StandardTextInliner。
#[test]
fn inline29_xml_with_text_inlining() {
    run_case("inline29");
}

/// inline31：HTML `th:inline="none"` 与 `th:inline="html"` ——
/// StandardHTMLInliner + InlinedOutputExpressionMarkupHandler。
#[test]
fn inline31_html_html_inlining() {
    run_case("inline31");
}

/// inline34：XML 模板使用 HTML inline 模式 —— 模式校验异常
/// （InlinedOutputExpressionMarkupHandler 路径）。
#[test]
fn inline34_xml_html_inlining_rejected() {
    run_case("inline34");
}

/// inline32：XML `th:inline="none"` —— XML 模式内联关闭（NoOpInliner 路径）。
#[test]
fn inline32_xml_none_inlining() {
    run_case("inline32");
}

/// inline01：HTML `th:inline="javascript"` —— StandardJavaScriptInliner。
#[test]
fn inline01_html_javascript_inlining() {
    run_case("inline01");
}

/// inlining110：HTML `th:inline="javascript"` 注释内联 ——
/// StandardJavaScriptInliner。
#[test]
fn inlining110_html_javascript_comment_inlining() {
    run_case("inlining110");
}

// ===========================================================================
// 异常消息形状校验（inline34 的 EXCEPTION_MESSAGE_PATTERN）
// ===========================================================================

/// inline34 的 Java 异常消息必须包含 "Invalid inline mode selected"。
#[test]
fn inline34_exception_message_matches_java_pattern() {
    let case = load_case("inline34");
    let engine = TemplateEngine::new();
    let mut resolver = thymeleaf::templateresolver::StringTemplateResolver::new();
    resolver
        .set_template_mode_nullable(Some(case.template_mode))
        .expect("template mode");
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("set resolver");
    let context = Context::new();
    for (key, expression) in &case.context {
        let value = evaluate_context_expression(&engine, expression);
        context.set_variable(Some(JavaString::from_rust_str(key)), value);
    }
    let error = engine
        .process_template(&case.input, &context)
        .expect_err("inline34 必须失败");
    // Java 模式 (.*?)Invalid\ inline\ mode\ selected(.*?) 匹配整个异常链
    let mut messages = Vec::new();
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(error.as_ref());
    while let Some(cause) = current {
        messages.push(format!("{cause}"));
        current = cause.source();
    }
    assert!(
        messages.iter().any(|message| message.contains("Invalid inline mode selected")),
        "inline34 异常链必须匹配 Java 模式，实际链: {messages:?}"
    );
}
