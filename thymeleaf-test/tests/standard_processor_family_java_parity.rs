//! Standard 处理器族 Java 1:1 差分测试（fixture 驱动 + 直测）。
//!
//! fixture 逐字取自上游 `thymeleaf-tests-core` 固定 `.thtest` 语料，字节级
//! 一致并校验 SHA-256；每个用例按上游 `%TEMPLATE_MODE` 驱动引擎，输出/异常
//! 与 `%OUTPUT`/`%EXCEPTION` 完全一致（markup 模式 canonical 空白归一化）。
//!
//! 覆盖对象（对象表编号）：
//! - `StandardBlockTagProcessor`（331）：`elementprocessors/block/block01-08`；
//! - `StandardConditionalCommentProcessor`（334）+ `StandardConditionalCommentUtils`
//!   （377）+ `StandardConditionalFixedValueTagProcessor`（335）：
//!   `conditionalcomments/conditionalcomments01-08`；
//! - `StandardDOMEventAttributeTagProcessor`（336）：`attrprocessors/domevent`
//!   onclick/onchange 各 2 例；
//! - `StandardUtextTagProcessor`（365）：`attrprocessors/insert/insert090/100`；
//! - `StandardRefAttributeTagProcessor`（355）：`attrprocessors/insert|replace`
//!   058/059/060 共 6 例；
//! - `StandardXmlNsTagProcessor`（370）+ `StandardTranslationDocTypeProcessor`
//!   （363）：`xmlns/xmlns01-09`（02/05/06/07/08/09 含 thymeleaf DTD 翻译）；
//! - `StandardInlineTextualTagProcessor`（345）+ `StandardInliningTextProcessor`
//!   （349）：`attrprocessors/inline/inline08/09`（HTML `th:inline="text"`）；
//! - `StandardXMLInliner`（315）+ `StandardInlineXMLTagProcessor`（346）：
//!   `inline29`（XML 模式 `th:inline="TEXT"`）与 `inline33`（HTML 模式
//!   `th:inline="xml"` + `th:inline="none"` 无效模式异常）；
//! - `StandardStyleappendTagProcessor`（360）：`dataprefix/.../styleappend01`；
//! - `StandardInliningCommentProcessor`（348）+ `StandardInliningCDATASectionProcessor`
//!   （347）：直测（Java `AbstractStandardInliner#inline(IComment/ICDATASection)`
//!   语义：跳过 `<!--`/`<![CDATA[` 前缀与 `-->`/`]]>` 后缀对内文执行内联，
//!   结果经 structureHandler.setContent 替换）。

use std::sync::Arc;

use sha2::Digest;
use thymeleaf::context::Context;
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::ITemplateResolver;
use thymeleaf::util::JavaString;
use thymeleaf::{TemplateEngine, TemplateMode};

#[allow(dead_code, unused_imports)]
mod support;

use support::thtest_harness::{
    CorpusStringTemplateResolver, build_context, directive_section_for_marker,
    named_input_sections, named_template_modes,
};

// ===========================================================================
// fixture 清单：文件名 → SHA-256（与上游文件字节级一致）
// ===========================================================================

const FIXTURES: &[(&str, &str)] = &[
    // elementprocessors/block（StandardBlockTagProcessor）
    (
        "block01",
        "e067e11199c27ebccb7aa64b2a25fb749e6c11be3cf0d6e645077f3269be7fcb",
    ),
    (
        "block02",
        "f082f2a8f74f1367b281d6ceaca26e44fb513bbce518c46f148fdd7f4d444cb0",
    ),
    (
        "block03",
        "030e977d96f201589d5a2bdd262899074a845a2ae8d18d300599bf65150f3337",
    ),
    (
        "block04",
        "b0a2ffa952f051eb1d4dd8444301872418e66a200828c13bba302a306dec7ecd",
    ),
    (
        "block05",
        "ac74e85fbfcd379b093174d4a9513475fb77bd1ce45caf0e8bdf95e7672edeca",
    ),
    (
        "block06",
        "c7298bf9499bfcd2738f2073630f1c69d8e30a348fef66af90b6a89655583f14",
    ),
    (
        "block07",
        "8b773e98e9fc3d610a29703e54dfeedf28de0ed19991fba3f0c67945948dac6a",
    ),
    (
        "block08",
        "3b89478d5c40fb3c47c87fbf920cfb6a7b0e639e9f794ad874489d9041cea4bf",
    ),
    // conditionalcomments（StandardConditionalCommentProcessor/Utils/FixedValue）
    (
        "conditionalcomments01",
        "649a0b0c3b5c20127c2ec929f7a6d11f9dc7ece29783f15b1f29dac878859c83",
    ),
    (
        "conditionalcomments02",
        "7c138601fb27f6cb1a2af58ce8482565e8aebd6d2327316e1bc2795a7cea08ff",
    ),
    (
        "conditionalcomments03",
        "98655a679f2df67b3cd997e9eb89c49043a3e9127e4d8ffeac5073be106c809a",
    ),
    (
        "conditionalcomments04",
        "cc78a2c44817b2eb935ca04aae590fd168e800d5f2038cc56995be85dc4780bf",
    ),
    (
        "conditionalcomments05",
        "649152b26b5c45ee547a3f8d7b693a3065a70aa57c138441061e60e5fcaf85bd",
    ),
    (
        "conditionalcomments06",
        "e8ec53adc28d371f28574ec9dbc0a56aacd13c4466aa38c722611f3d29050337",
    ),
    (
        "conditionalcomments07",
        "d5dc400215c240f719899572a2009a13e76cacc2bef67623d59baec908a073a1",
    ),
    (
        "conditionalcomments08",
        "8772d2e0d7a82f384ba01b8b6c51bd219fd9999184acbea5848953c177fa2a57",
    ),
    // attrprocessors/domevent（StandardDOMEventAttributeTagProcessor）
    (
        "onclick01",
        "952d3e53c68a8476c279cafb756fad880e6a3dcdff6acf2bcf6d6350734fd5a4",
    ),
    (
        "onclick02",
        "ae6bfb9cfc5145ea8e8634ee6374371b4440bb70fb320136ac3d2f31c7eb2da5",
    ),
    (
        "onchange01",
        "e7f734155a91363a9494969cc8f197fcebc38198f28333e77c84a081eb130bfb",
    ),
    (
        "onchange02",
        "2cdf3d66b0390f18bd5111042dfe2ac0edabc36f2ee7d870f58781d44240bd78",
    ),
    // attrprocessors/insert（StandardUtextTagProcessor）
    (
        "insert090",
        "19dacdaf06688399bc30304bf2f5fd71a766c2357c5b4f7cfde3ea1362f7930c",
    ),
    (
        "insert100",
        "fc60e14e55f5568514922172d72b7f32635910cdc7090cc36c7dce58370d8adc",
    ),
    // attrprocessors/insert|replace（StandardRefAttributeTagProcessor）
    (
        "insert058",
        "e757ad78c5553714c77bfd77c7835ab371cc1fb73c276c8271e0e644bfb8f007",
    ),
    (
        "insert059",
        "cf8d422b8a2a5dd90e6ac65dd71d8a670a9f6d2f265fe364640acb3f0a556ef5",
    ),
    (
        "insert060",
        "eaadfde5b39e66d0b49f1191310b88607e24636c6a377e2927f62447f4c17cf1",
    ),
    (
        "replace058",
        "facc9577e65e22ab6bf013028e1b912761397ea9c505e9b661f7d88cb5215ad0",
    ),
    (
        "replace059",
        "73cbb75084b7f20708b7285f8f85be6b598774fa670fdb951a620bb6a86d4822",
    ),
    (
        "replace060",
        "e058b4eefa7f8594ec33498002f795ac8992625d1a2eef961cd5aa5a340484dd",
    ),
    // xmlns（StandardXmlNsTagProcessor + StandardTranslationDocTypeProcessor）
    (
        "xmlns01",
        "7a81940c898e36c4dd541317982ccafff2f268b63e776f165cfacd9af38789ac",
    ),
    (
        "xmlns02",
        "28a741bd4a01941f7bcd4b3c1cfb2c394b8942cdfa6ec2b23bfb68d92cb46a07",
    ),
    (
        "xmlns03",
        "bc92fd0b54fd8f55b44d3a08f558e8d6427db23c762c2d15bde21a498dc90cd4",
    ),
    (
        "xmlns04",
        "b29dd50430694a3d9d40452dc6f4b58ef4313c4910577caa5c6c95f9df096efe",
    ),
    (
        "xmlns05",
        "91450dffd66e13fffd16a1293faa64fd4c553a8b5f90c306e3f021997e8004e0",
    ),
    (
        "xmlns06",
        "3940d4252fc58ecfe8473f7b8330a3e49230003b6ae7b2b350fa6ffb79d748e2",
    ),
    (
        "xmlns07",
        "e996b8b0b02b7cb7be38b539194c5109a8f6608f66628582141596c19c25ff2c",
    ),
    (
        "xmlns08",
        "959259de46e29c87f42df760d24d1ed6fd1abe73f0b104f69ecdcb92288aee09",
    ),
    (
        "xmlns09",
        "f6b17e178b79433dd7aeb5996c65362423358b5f73cbbd88eae448ca81550888",
    ),
    // attrprocessors/inline（StandardInlineTextualTagProcessor/InliningText/XMLInliner）
    (
        "inline08",
        "bd113ad961730e3acb8a667f57efeb1460d54538232a8d89453151e4044f811e",
    ),
    (
        "inline09",
        "03149fe9d57fa22eb3c4ad4a76174543a86901ede0119c94e17413d078195395",
    ),
    (
        "inline29",
        "3fcbba09494439785fe51ec998b12eccf01214c439a6356736766492b581a704",
    ),
    (
        "inline33",
        "061789b7359cef7ab314bb7f10cbae2978c1bfd82a56daf7d5e92aa1a075c361",
    ),
    // dataprefix（StandardStyleappendTagProcessor）
    (
        "styleappend01",
        "a57cef5a29f431c4df52f1ef8b6b1eae045c0b0d57d76f437575fb7b3f347fb1",
    ),
];

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/../thymeleaf/tests/fixtures/processorfamily/{name}.thtest",
        env!("CARGO_MANIFEST_DIR")
    ))
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

struct ProcessorCase {
    template_mode: TemplateMode,
    context_source: Option<String>,
    input: String,
    output: Option<String>,
    expects_exception: bool,
    named_inputs: indexmap::IndexMap<JavaString, JavaString>,
    named_modes: indexmap::IndexMap<JavaString, TemplateMode>,
}

fn load_case(name: &str) -> ProcessorCase {
    let content = String::from_utf8(fixture_bytes(name)).expect("fixture UTF-8");
    let mut mode = TemplateMode::HTML;
    let mut expects_exception = false;
    let mut output = None;
    for line in content.lines() {
        if line.starts_with('%') && !line.starts_with("%%") {
            let directive = line.trim_start_matches('%');
            let (key, value) = directive
                .split_once(char::is_whitespace)
                .map_or((directive, ""), |(k, v)| (k, v.trim()));
            match key {
                "TEMPLATE_MODE" => {
                    // XHTML/VALIDXHTML 与语料运行器一致回退（TemplateMode::parse 语义）
                    mode = TemplateMode::parse(Some(value)).expect("template mode");
                }
                "EXCEPTION" => {
                    expects_exception = true;
                }
                _ => {}
            }
        }
    }
    let context_source = directive_section_for_marker(&content, "%CONTEXT");
    let input = directive_section_for_marker(&content, "%INPUT").expect("INPUT section exists");
    if let Some(section) = directive_section_for_marker(&content, "%OUTPUT") {
        output = Some(section);
    }
    ProcessorCase {
        template_mode: mode,
        context_source,
        input,
        output,
        expects_exception,
        named_inputs: named_input_sections(&content).expect("named inputs"),
        named_modes: named_template_modes(&content).expect("named modes"),
    }
}

// ===========================================================================
// 引擎驱动（复用 `.thtest` 语料共享机制：CONTEXT/命名片段/命名模板解析器）
// ===========================================================================

fn run_case(name: &str) {
    let (_, expected_sha256) = FIXTURES
        .iter()
        .find(|(fixture, _)| *fixture == name)
        .expect("fixture listed");
    assert_fixture_sha256(name, expected_sha256);

    let case = load_case(name);
    let engine = TemplateEngine::new();
    let resolver = CorpusStringTemplateResolver::new(
        case.template_mode,
        name,
        &case.input,
        case.named_inputs,
        case.named_modes,
    );
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("set resolver");
    // 语料运行器对全部用例设置固定 LinkBuilder（上游 TestLinkBuilder 等价）；
    // 缺失时 `@{/something}` 等链接表达式无法求值。
    engine
        .set_link_builder(Arc::new(support::TestLinkBuilder))
        .expect("set link builder");

    let context = build_context(&engine, case.context_source.as_deref())
        .unwrap_or_else(|error| panic!("{name} CONTEXT: {error}"));

    let result = engine.process_template(name, &context);
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
            html5gum::Token::String(string) => {
                let text = String::from_utf8_lossy(string.as_ref()).to_string();
                for part in text.split(|character: char| character.is_whitespace()) {
                    if !part.is_empty() {
                        trace.push(format!("T:{part}"));
                    }
                }
            }
            _ => {}
        }
    }
    trace
}

/// 与 `.thtest` 语料完全相同的 markup 空白归一化（全部空白折叠为单空格）。
fn normalize_markup_whitespace(markup: &str) -> String {
    let mut result = String::with_capacity(markup.len());
    let mut pending_space = false;
    for character in markup.chars() {
        if character.is_whitespace() {
            pending_space = true;
        } else {
            if pending_space {
                result.push(' ');
                pending_space = false;
            }
            result.push(character);
        }
    }
    result
}

// ===========================================================================
// 1. StandardBlockTagProcessor（th:block）
// ===========================================================================

#[test]
fn block_processor_fixtures_match_java() {
    for name in [
        "block01", "block02", "block03", "block04", "block05", "block06", "block07", "block08",
    ] {
        run_case(name);
    }
}

// ===========================================================================
// 2. StandardConditionalCommentProcessor + Utils + FixedValue
// ===========================================================================

#[test]
fn conditional_comment_fixtures_match_java() {
    for name in [
        "conditionalcomments01",
        "conditionalcomments02",
        "conditionalcomments03",
        "conditionalcomments04",
        "conditionalcomments05",
        "conditionalcomments06",
        "conditionalcomments07",
        "conditionalcomments08",
    ] {
        run_case(name);
    }
}

// ===========================================================================
// 3. StandardDOMEventAttributeTagProcessor（th:onclick/th:onchange）
// ===========================================================================

#[test]
fn dom_event_processor_fixtures_match_java() {
    for name in ["onclick01", "onclick02", "onchange01", "onchange02"] {
        run_case(name);
    }
}

// ===========================================================================
// 4. StandardUtextTagProcessor（th:utext）
// ===========================================================================

#[test]
fn utext_processor_fixtures_match_java() {
    for name in ["insert090", "insert100"] {
        run_case(name);
    }
}

// ===========================================================================
// 5. StandardRefAttributeTagProcessor（th:ref）
// ===========================================================================

#[test]
fn ref_processor_fixtures_match_java() {
    for name in [
        "insert058",
        "insert059",
        "insert060",
        "replace058",
        "replace059",
        "replace060",
    ] {
        run_case(name);
    }
}

// ===========================================================================
// 6. StandardXmlNsTagProcessor + StandardTranslationDocTypeProcessor
// ===========================================================================

#[test]
fn xmlns_and_doctype_translation_fixtures_match_java() {
    for name in [
        "xmlns01", "xmlns02", "xmlns03", "xmlns04", "xmlns05", "xmlns06", "xmlns07", "xmlns08",
        "xmlns09",
    ] {
        run_case(name);
    }
}

// ===========================================================================
// 7. StandardInlineTextualTagProcessor + StandardInliningTextProcessor +
//    StandardXMLInliner
// ===========================================================================

#[test]
fn inline_textual_processor_fixtures_match_java() {
    for name in ["inline08", "inline09", "inline29", "inline33"] {
        run_case(name);
    }
}

// ===========================================================================
// 8. StandardStyleappendTagProcessor（th:styleappend）
// ===========================================================================

#[test]
fn styleappend_processor_fixture_matches_java() {
    run_case("styleappend01");
}

// ===========================================================================
// 9. StandardInliningCommentProcessor / StandardInliningCDATASectionProcessor
//    直测：th:inline 设置 inliner 后，Comment/CDATASection 事件内文被内联。
//    Java 语义（AbstractStandardInliner）：comment 内容跳过 `<!--`/`-->`
//    包裹后对内文执行 `[[...]]` 替换，结果经 setContent 写回。
// ===========================================================================

fn run_template(input: &str, mode: TemplateMode) -> String {
    let engine = TemplateEngine::new();
    let mut resolver = thymeleaf::templateresolver::StringTemplateResolver::new();
    resolver
        .set_template_mode_nullable(Some(mode))
        .expect("template mode");
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("set resolver");
    let context = Context::new();
    context.set_variable(
        Some(JavaString::from_rust_str("var")),
        Some(Arc::new(TemplateValue::String(Arc::new(
            JavaString::from_rust_str("10"),
        )))),
    );
    engine
        .process_template(input, &context)
        .expect("process must succeed")
        .to_string_lossy()
}

#[test]
fn inlining_comment_processor_matches_java() {
    // HTML 模式 `th:inline="text"`：注释内文 `[[${var}]]` 经
    // StandardInliningCommentProcessor -> StandardTextInliner 内联为值。
    let output = run_template(
        "<div th:inline=\"text\"><!--[[${var}]]--></div>",
        TemplateMode::HTML,
    );
    assert_eq!(output, "<div><!--10--></div>");
    // 非 inlineable 内容（无 `[[...]]` 表达式）保持原样
    let output = run_template(
        "<div th:inline=\"text\"><!--plain comment--></div>",
        TemplateMode::HTML,
    );
    assert_eq!(output, "<div><!--plain comment--></div>");
}

#[test]
fn inlining_cdata_section_processor_matches_java() {
    // XML 模式 `th:inline="text"`（StandardInlineTextualTagProcessor 在 XML
    // 模式同样注册）：CDATA 内文 `[[${var}]]` 经 StandardInliningCDATASectionProcessor
    // 内联为值，包裹保留。
    let output = run_template(
        "<root th:inline=\"text\"><![CDATA[[[${var}]]]]></root>",
        TemplateMode::XML,
    );
    assert_eq!(output, "<root><![CDATA[10]]></root>");
}
