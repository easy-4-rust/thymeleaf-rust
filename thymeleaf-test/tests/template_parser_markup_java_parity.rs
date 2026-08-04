//! 模板解析器族（markup parser family）Java 1:1 差分测试。
//!
//! 覆盖上游 `thymeleaf-tests-core` 的解析器测试：
//!
//! 1. `org.thymeleaf.parsing.Parsing01Test/02/03Test` —— 引擎 +
//!    `ClassLoaderTemplateResolver`（HTML5 模式，UTF-8 / UTF-16 带 BOM 编码），
//!    输出按 `ResourceUtils.normalize` 规范化后与 `parsingtest0X-result.bulk`
//!    逐字节比较（测试资产字节级一致，SHA-256 在下方逐一固定）；
//! 2. `org.thymeleaf.templateparser.markup.HtmlBlockSelectorMarkupHandlerTest` ——
//!    `HTMLTemplateParser#parseStandalone` 块选择器 + `OutputTemplateHandler`，
//!    输出与 result 文件（首行选择器除外）精确比较；
//! 3. `org.thymeleaf.templateparser.markup.TemplateFragmentMarkupReferenceResolverTest` ——
//!    `forPrefix` 缓存身份 + `resolveSelectorFromReference` 全形态；
//! 4. `org.thymeleaf.templateparser.markup.ParsingDecoupled01Test` ——
//!    `DecoupledTemplateLogicUtils#computeDecoupledTemplateLogic` + toString。

use std::io::{self, Cursor, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use thymeleaf::context::Context;
use thymeleaf::decoupled::{DecoupledTemplateLogic, DecoupledTemplateLogicUtils};
use thymeleaf::engine::OutputTemplateHandler;
use thymeleaf::markup::{
    HTMLTemplateParser, TemplateFragmentMarkupReferenceResolver, XMLTemplateParser,
};
use thymeleaf::templateparser::ITemplateParser;
use thymeleaf::templateresource::{
    ITemplateResource, StringTemplateResource, TemplateResourceError,
};
use thymeleaf::util::{JavaWriter, Utf16String};
use thymeleaf::{
    IEngineConfiguration, ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode,
};

// ===========================================================================
// 通用辅助
// ===========================================================================

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../thymeleaf/tests/fixtures")
}

fn fixture_bytes(relative: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(relative)).expect("fixture 文件必须存在")
}

/// 断言测试资产与上游文件字节级一致（SHA-256 校验）。
fn assert_asset_sha256(relative: &str, expected_sha256: &str) {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(fixture_bytes(relative));
    let digest = hasher.finalize();
    let digest = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        digest, expected_sha256,
        "fixture {relative} 的 SHA-256 与上游不符"
    );
}

/// Java `Character.isWhitespace` 的精确字符集（与
/// `src/templateresource/template_resource_reader.rs` 的内部实现一致）。
fn is_java_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000D}'
            | '\u{001C}'..='\u{0020}'
            | '\u{1680}'
            | '\u{2000}'..='\u{2006}'
            | '\u{2008}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

/// Java `ResourceUtils.normalize`：逐行读取（去 `\r`）、按 `\n` 连接、
/// 最后去除文件尾 Java 空白。
fn normalize(text: &str) -> String {
    let mut segments: Vec<String> = text
        .split('\n')
        .map(|line| line.replace('\r', ""))
        .collect();
    if text.ends_with('\n') && segments.len() > 1 {
        segments.pop();
    }
    let mut result = segments.join("\n");
    while result.chars().last().is_some_and(is_java_whitespace) {
        result.pop();
    }
    result
}

/// 对应 Apache Commons IOUtils.readLines：按 `\n` 切分并剔除文件尾空行。
fn java_read_lines(contents: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = contents.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    lines
}

fn engine_with_resolver(resolver: Arc<dyn ITemplateResolver>) -> TemplateEngine {
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(resolver)
        .expect("set resolver");
    engine
}

fn configuration(engine: &TemplateEngine) -> Arc<dyn IEngineConfiguration> {
    engine.get_configuration().expect("engine configuration")
}

/// 捕获全部 UTF-16 输出的终端 Writer（对应 Java StringWriter）。
#[derive(Clone)]
struct CapturedWriter {
    buffer: Arc<Mutex<Vec<u16>>>,
}

impl CapturedWriter {
    fn new() -> (Self, Arc<Mutex<Vec<u16>>>) {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                buffer: buffer.clone(),
            },
            buffer,
        )
    }
}

fn lock(buffer: &Arc<Mutex<Vec<u16>>>) -> MutexGuard<'_, Vec<u16>> {
    buffer
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl JavaWriter for CapturedWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        lock(&self.buffer).extend_from_slice(characters);
        Ok(())
    }
}

// ===========================================================================
// 1. Parsing01Test / Parsing02Test / Parsing03Test
// ===========================================================================

const PARSING_ASSET_SHA256: &[(&str, &str)] = &[
    (
        "parsing/parsingtest01.bulk",
        "22051a6cb77ddbf76e826c5dba7f128731866dc6de2fd05fdc0d7db47f363023",
    ),
    (
        "parsing/parsingtest01-result.bulk",
        "22051a6cb77ddbf76e826c5dba7f128731866dc6de2fd05fdc0d7db47f363023",
    ),
    (
        "parsing/parsingtest02.bulk",
        "89d99ca763fe6ecd1a76c877a400bb05104e0ee908c45b2ce58c1ca9c2b30a91",
    ),
    (
        "parsing/parsingtest02-result.bulk",
        "22051a6cb77ddbf76e826c5dba7f128731866dc6de2fd05fdc0d7db47f363023",
    ),
    (
        "parsing/parsingtest03.bulk",
        "99febb27e3f01477b8108038c26d87b8fcb03e762ddc17306869041af9e35da3",
    ),
    (
        "parsing/parsingtest03-result.bulk",
        "22051a6cb77ddbf76e826c5dba7f128731866dc6de2fd05fdc0d7db47f363023",
    ),
];

fn parsing_engine(character_encoding: &str) -> TemplateEngine {
    let mut resolver =
        thymeleaf::templateresolver::ClassLoaderTemplateResolver::with_search_roots(vec![
            fixtures_dir(),
        ]);
    resolver
        .set_template_mode_name(Some("HTML5"))
        .expect("HTML5 template mode");
    resolver.set_character_encoding(Some(js(character_encoding)));
    engine_with_resolver(Arc::new(resolver))
}

fn run_parsing_test(template_file: &str, result_file: &str, character_encoding: &str) {
    for (asset, expected_sha256) in PARSING_ASSET_SHA256 {
        assert_asset_sha256(asset, expected_sha256);
    }
    let engine = parsing_engine(character_encoding);
    let context = Context::new();
    let result = engine
        .process_template(template_file, &context)
        .expect("engine process")
        .to_string_lossy();
    let expected_bytes = fixture_bytes(result_file);
    let expected = String::from_utf8(expected_bytes).expect("result fixture 为 UTF-8 可读");
    assert_eq!(
        normalize(&result),
        normalize(&expected),
        "Parsing 测试 {template_file}（编码 {character_encoding}）输出不匹配"
    );
}

/// Parsing01Test：HTML5 + UTF-8。
#[test]
fn parsing01_html5_utf8() {
    run_parsing_test(
        "parsing/parsingtest01.bulk",
        "parsing/parsingtest01-result.bulk",
        "UTF-8",
    );
}

/// Parsing02Test：HTML5 + UTF-16（带 BOM）。
#[test]
fn parsing02_html5_utf16() {
    run_parsing_test(
        "parsing/parsingtest02.bulk",
        "parsing/parsingtest02-result.bulk",
        "UTF-16",
    );
}

/// Parsing03Test：HTML5 + UTF-16（带 BOM）。
#[test]
fn parsing03_html5_utf16() {
    run_parsing_test(
        "parsing/parsingtest03.bulk",
        "parsing/parsingtest03-result.bulk",
        "UTF-16",
    );
}

// ===========================================================================
// 2. HtmlBlockSelectorMarkupHandlerTest
// ===========================================================================

const BLOCK_SELECTOR_FIXTURES: &[(&str, &str)] = &[
    (
        "htmlblockselector/test001.html",
        "htmlblockselector/result001.html",
    ),
    (
        "htmlblockselector/test002.html",
        "htmlblockselector/result002.html",
    ),
    (
        "htmlblockselector/test003.html",
        "htmlblockselector/result003.html",
    ),
];

const BLOCK_SELECTOR_ASSET_SHA256: &[(&str, &str)] = &[
    (
        "htmlblockselector/test001.html",
        "996cd829b18fef41a28ee964d915bb551d811880117de572b4d0c7780de20114",
    ),
    (
        "htmlblockselector/test002.html",
        "3fb81e95e78db331a6a74096a902288cbb23b2733174b39d7f0f355670510275",
    ),
    (
        "htmlblockselector/test003.html",
        "49726d2db91d5e97f76c4a9f30e2c461f9420c4a359a92b042e98d1ee1484419",
    ),
    (
        "htmlblockselector/result001.html",
        "4b8083ea96768a7c552e862087030e23ff40852d0ec398e197443951df536355",
    ),
    (
        "htmlblockselector/result002.html",
        "eb3cc1f0658e77d19d7d5c8fd7e7c625aba729a6b4ae4713b76cdee960065ebc",
    ),
    (
        "htmlblockselector/result003.html",
        "0f7239ee69ab8b976cb8b5824555f46644713212e1e997504a1fd8d2ec840b26",
    ),
];

/// 对应 Java HtmlBlockSelectorMarkupHandlerTest：逐文件按块选择器解析并
/// 与 result 文件（首行选择器除外）精确比较。
#[test]
fn html_block_selector_matches_java() {
    for (asset, expected_sha256) in BLOCK_SELECTOR_ASSET_SHA256 {
        assert_asset_sha256(asset, expected_sha256);
    }
    let parser = HTMLTemplateParser::new(2, 4096);
    let engine = engine_with_resolver(Arc::new(
        thymeleaf::templateresolver::StringTemplateResolver::new(),
    ));
    let config = configuration(&engine);

    for (test_file, result_file) in BLOCK_SELECTOR_FIXTURES {
        let test_bytes = fixture_bytes(test_file);
        let test_contents = String::from_utf8(test_bytes).expect("test fixture UTF-8");
        // Java: IOUtils.readLines（末尾空行剔除）+ StringUtils.join(lines, '\n')
        let test_lines = java_read_lines(&test_contents);
        let test_contents = test_lines.join("\n");

        let result_bytes = fixture_bytes(result_file);
        let result_contents = String::from_utf8(result_bytes).expect("result fixture UTF-8");
        let result_lines = java_read_lines(&result_contents);
        let block_selector = result_lines[0];
        let expected = result_lines[1..].join("\n");

        let selectors: Vec<Utf16String> = block_selector.split(',').map(js).collect();

        let resource: Arc<dyn ITemplateResource> =
            Arc::new(StringTemplateResource::new(Some(&test_contents)).expect("string resource"));
        let (writer, buffer) = CapturedWriter::new();
        let handler = Box::new(OutputTemplateHandler::new(Box::new(writer)));
        parser
            .parse_standalone(
                config.clone(),
                Some(&js(test_file)),
                &js(test_file),
                Some(&selectors),
                resource,
                TemplateMode::HTML,
                false,
                handler,
            )
            .unwrap_or_else(|error| panic!("{test_file} 块选择器解析失败: {error}"));
        let actual = String::from_utf16_lossy(&lock(&buffer));
        assert_eq!(
            expected, actual,
            "Test failed for file: {test_file}（选择器 {block_selector}）"
        );
    }
}

// ===========================================================================
// 3. TemplateFragmentMarkupReferenceResolverTest
// ===========================================================================

fn assert_resolver_html(html: bool, prefix: Option<&str>, expected: &str) {
    let resolver =
        TemplateFragmentMarkupReferenceResolver::for_prefix(html, prefix.map(js).as_ref());
    let shared = TemplateFragmentMarkupReferenceResolver::for_prefix(html, prefix.map(js).as_ref());
    assert!(
        Arc::ptr_eq(&resolver, &shared),
        "forPrefix 必须返回同一共享实例（对应 Java assertSame）"
    );
    let result = resolver.resolve_selector_from_reference(&js("abc"));
    assert_eq!(
        result.to_string_lossy(),
        expected,
        "resolveSelectorFromReference 结果不匹配（html={html}, prefix={prefix:?}）"
    );
    // Java assertSame(result01, result02)：缓存命中返回相同内容
    let again = resolver.resolve_selector_from_reference(&js("abc"));
    assert_eq!(result, again, "重复引用必须返回相同缓存内容");
}

/// 对应 Java TemplateFragmentMarkupReferenceResolverTest#testHtml。
#[test]
fn template_fragment_resolver_html_matches_java() {
    assert_resolver_html(
        true,
        None,
        "/[ref='abc' or data-ref='abc' or fragment='abc' or data-fragment='abc' \
         or fragment^='abc(' or data-fragment^='abc(' or fragment^='abc (' or data-fragment^='abc (']",
    );
    assert_resolver_html(
        true,
        Some("th"),
        "/[th:ref='abc' or data-th-ref='abc' or th:fragment='abc' or data-th-fragment='abc' \
         or th:fragment^='abc(' or data-th-fragment^='abc(' or th:fragment^='abc (' or data-th-fragment^='abc (']",
    );
    assert_resolver_html(
        true,
        Some("q"),
        "/[q:ref='abc' or data-q-ref='abc' or q:fragment='abc' or data-q-fragment='abc' \
         or q:fragment^='abc(' or data-q-fragment^='abc(' or q:fragment^='abc (' or data-q-fragment^='abc (']",
    );
}

/// 对应 Java TemplateFragmentMarkupReferenceResolverTest#testXml。
#[test]
fn template_fragment_resolver_xml_matches_java() {
    assert_resolver_html(
        false,
        None,
        "/[ref='abc' or fragment='abc' or fragment^='abc(' or fragment^='abc (']",
    );
    assert_resolver_html(
        false,
        Some("th"),
        "/[th:ref='abc' or th:fragment='abc' or th:fragment^='abc(' or th:fragment^='abc (']",
    );
    assert_resolver_html(
        false,
        Some("q"),
        "/[q:ref='abc' or q:fragment='abc' or q:fragment^='abc(' or q:fragment^='abc (']",
    );
}

// ===========================================================================
// 4. ParsingDecoupled01Test
// ===========================================================================

const DECOUPLED_FIXTURES: &[(&str, &str)] = &[
    // (模板名, 期望 DecoupledTemplateLogic.toString())
    (
        "parsingdecoupled01",
        "{//form=[th:class=\"greatclass\"], //form//div[0]/label=[thefirstlabel], \
         //form//div[1]//label=[th:text=\"${'MegaCovered'}\"], //form/fieldset=[id=\"fset\"]}",
    ),
    (
        "parsingdecoupled02",
        "{//abbr[1]/a=[id=\"fset\"], //abbr[1]/a/lele=[lala=\"oe\", lala2=\"122\"], \
         //form=[th:class=\"greatclass\", this='that', whatever=those, th:another=\"${lala}\"], \
         //form//.block/div[a='23']//label=[th:text=\"${'MegaCovered'}\"], \
         //form//div[0]/label=[thefirstlabel]}",
    ),
    (
        "parsingdecoupled03",
        "{//abbr[1]/a=[id=\"fset\"], //abbr[1]/a/lele=[lala=\"oe\", lala2=\"122\"], \
         //form=[th:class=\"greatclass\", this='that', whatever=those, th:another=\"${lala}\"], \
         //form//.block/div[a='23']//label=[th:text=\"${'MegaCovered'}\"], \
         //form//div[0]/label=[thefirstlabel]}",
    ),
];

const DECOUPLED_ASSET_SHA256: &[(&str, &str)] = &[
    (
        "parsingdecoupled/parsingdecoupled01.th.xml",
        "6ff6aacf6d25fc1d35b5ceecea083a69d01981aee575328012c5b4281c3824a1",
    ),
    (
        "parsingdecoupled/parsingdecoupled02.th.xml",
        "43594b01974033352520700e65f095eb9c8eed27a874e2c861583331c397b847",
    ),
    (
        "parsingdecoupled/parsingdecoupled03.th.xml",
        "c45fa9a61432729944a16f1d308db40230cf416f30715fb8e4ab103b261995af",
    ),
];

/// 对应 Java ClassLoaderTemplateResource 的解耦测试资源：base name 可派生，
/// 相对位置按 `.th.xml` fixture 内容提供（主模板文件本身不参与读取）。
struct DecoupledFixtureResource {
    base_name: String,
    relative_location: Option<String>,
}

impl ITemplateResource for DecoupledFixtureResource {
    fn get_description(&self) -> String {
        self.relative_location
            .clone()
            .unwrap_or_else(|| self.base_name.clone())
    }

    fn get_base_name(&self) -> Option<String> {
        Some(self.base_name.clone())
    }

    fn exists(&self) -> bool {
        self.relative_location.as_ref().is_some_and(|location| {
            DECOUPLED_ASSET_SHA256
                .iter()
                .any(|(asset, _)| asset.ends_with(location))
        })
    }

    fn reader(&self) -> io::Result<Box<dyn Read>> {
        let location = self.relative_location.as_deref().expect("decoupled 位置");
        let relative = DECOUPLED_ASSET_SHA256
            .iter()
            .map(|(asset, _)| *asset)
            .find(|asset| asset.ends_with(location))
            .expect("decoupled fixture");
        Ok(Box::new(Cursor::new(fixture_bytes(relative))))
    }

    fn relative(
        &self,
        relative_location: Option<&str>,
    ) -> Result<Box<dyn ITemplateResource>, TemplateResourceError> {
        Ok(Box::new(DecoupledFixtureResource {
            base_name: self.base_name.clone(),
            relative_location: relative_location.map(str::to_owned),
        }))
    }
}

/// 对应 Java ParsingDecoupled01Test：三类解耦逻辑文件解析结果与
/// `DecoupledTemplateLogic#toString` 一致。
#[test]
fn parsing_decoupled_01_02_03_matches_java() {
    for (asset, expected_sha256) in DECOUPLED_ASSET_SHA256 {
        assert_asset_sha256(asset, expected_sha256);
    }
    let engine = engine_with_resolver(Arc::new(
        thymeleaf::templateresolver::StringTemplateResolver::new(),
    ));
    let config = configuration(&engine);

    for (template, expected) in DECOUPLED_FIXTURES {
        let resource = DecoupledFixtureResource {
            base_name: template.to_string(),
            relative_location: None,
        };
        let logic: Option<Arc<DecoupledTemplateLogic>> =
            DecoupledTemplateLogicUtils::compute_decoupled_template_logic(
                config.as_ref(),
                None,
                &js(template),
                None,
                &resource,
                TemplateMode::HTML,
            )
            .unwrap_or_else(|error| panic!("{template} 解耦逻辑解析失败: {error}"));
        let logic = logic.expect("解耦逻辑必须存在");
        assert_eq!(
            logic.to_string(),
            expected.to_string(),
            "DecoupledTemplateLogic.toString 不匹配（模板 {template}）"
        );
    }
}

// ===========================================================================
// 5. 文本/RAW/XML 解析器对象级往返差分
// ===========================================================================
//
// 对 `XMLTemplateParser`、`TextTemplateParser`、`JavaScriptTemplateParser`、
// `CSSTemplateParser`、`RawTemplateParser` 五个对象，通过各自
// `ITemplateParser#parseStandalone` 入口 + `OutputTemplateHandler` 验证
// 解析-写回往返与输入逐字节一致（与 corpus 引擎级用例同一条解析管线，
// 这里给出对象级直接证据）。

fn roundtrip(
    parser: Arc<dyn thymeleaf::templateparser::ITemplateParser>,
    config: Arc<dyn IEngineConfiguration>,
    document: &str,
    mode: TemplateMode,
) -> String {
    let resource: Arc<dyn ITemplateResource> =
        Arc::new(StringTemplateResource::new(Some(document)).expect("string resource"));
    let (writer, buffer) = CapturedWriter::new();
    parser
        .parse_standalone(
            config,
            None,
            &js("t"),
            None,
            resource,
            mode,
            false,
            Box::new(OutputTemplateHandler::new(Box::new(writer))),
        )
        .expect("parse standalone");
    String::from_utf16_lossy(&lock(&buffer))
}

/// XML/HTML 模式解析-写回往返。
#[test]
fn markup_parsers_roundtrip_identity() {
    let engine = engine_with_resolver(Arc::new(
        thymeleaf::templateresolver::StringTemplateResolver::new(),
    ));
    let config = configuration(&engine);

    let xml: Arc<dyn thymeleaf::templateparser::ITemplateParser> =
        Arc::new(XMLTemplateParser::new(2, 4096));
    for document in [
        "<div a='1'>x</div>",
        "<?xml version=\"1.0\"?><root><child/></root>",
    ] {
        assert_eq!(
            document,
            roundtrip(xml.clone(), config.clone(), document, TemplateMode::XML),
            "XML 往返必须与输入一致"
        );
    }

    let html: Arc<dyn thymeleaf::templateparser::ITemplateParser> =
        Arc::new(HTMLTemplateParser::new(2, 4096));
    for document in [
        "<div a='1'>x</div>",
        "<!DOCTYPE html><html><body><p>a</p></body></html>",
    ] {
        assert_eq!(
            document,
            roundtrip(html.clone(), config.clone(), document, TemplateMode::HTML),
            "HTML 往返必须与输入一致"
        );
    }
}

/// TEXT/JAVASCRIPT/CSS/RAW 模式解析-写回往返。
#[test]
fn text_mode_parsers_roundtrip_identity() {
    let engine = engine_with_resolver(Arc::new(
        thymeleaf::templateresolver::StringTemplateResolver::new(),
    ));
    let config = configuration(&engine);

    let text: Arc<dyn thymeleaf::templateparser::ITemplateParser> =
        Arc::new(thymeleaf::text::TextTemplateParser::new(2, 4096, true));
    for document in [
        "Hello, World!",
        "[#img src=\"hello\"/]Something",
        "[#hello]...[/hello]",
        "a[#x/]b",
    ] {
        assert_eq!(
            document,
            roundtrip(text.clone(), config.clone(), document, TemplateMode::TEXT),
            "TEXT 往返必须与输入一致"
        );
    }

    let javascript: Arc<dyn thymeleaf::templateparser::ITemplateParser> = Arc::new(
        thymeleaf::text::JavaScriptTemplateParser::new(2, 4096, true),
    );
    for document in ["var x = 1;", "//[#x/]"] {
        assert_eq!(
            document,
            roundtrip(
                javascript.clone(),
                config.clone(),
                document,
                TemplateMode::JAVASCRIPT,
            ),
            "JAVASCRIPT 往返必须与输入一致"
        );
    }

    let css: Arc<dyn thymeleaf::templateparser::ITemplateParser> =
        Arc::new(thymeleaf::text::CSSTemplateParser::new(2, 4096, true));
    let document = "body { color: red; }";
    assert_eq!(
        document,
        roundtrip(css.clone(), config.clone(), document, TemplateMode::CSS),
        "CSS 往返必须与输入一致"
    );

    let raw: Arc<dyn thymeleaf::templateparser::ITemplateParser> =
        Arc::new(thymeleaf::raw::RawTemplateParser::new(2, 4096));
    for document in ["<html>anything</html>", "raw { text }"] {
        assert_eq!(
            document,
            roundtrip(raw.clone(), config.clone(), document, TemplateMode::RAW),
            "RAW 往返必须与输入一致"
        );
    }
}
