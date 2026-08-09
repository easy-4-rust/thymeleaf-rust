//! 鲁棒性 fuzz：随机输入下解析器/渲染不得 panic。
//!
//! - HTML/XML/TEXT 模板：`parse_standalone` 对任意 Unicode 输入（含代理对/emoji/
//!   控制字符）必须返回 `Result`，允许 `Err` 但不允许 panic。
//! - 表达式：通过引擎渲染含表达式模板的 smoke（表达式解析/求值在引擎内执行）。
//! - 资源入口为 `&str`（合法 Rust Unicode）；孤立 UTF-16 代理项由语料运行器与
//!   `Utf16String` 级差分覆盖。
//!
//! proptest 用例数默认 64；本地加深：
//! `PROPTEST_CASES=10000 cargo test -p thymeleaf-test --test robustness_fuzz_smoke`。
//!
//! ## 内存安全设计（OOM 根因修复）
//!
//! 1. **DiscardingWriter**：parse 测试只验证"不 panic"，不需要输出。用丢弃
//!    writer 替代原来的 CapturedWriter（无界 Vec<u16>），从根本上消除
//!    "病态输入 → 巨大 token → 无界输出缓冲"的放大面。
//! 2. **proptest shrink 钳制**：`max_shrink_iters: 256` + `max_shrink_time: 10s`
//!    （默认 u32::MAX / 0=禁用 → 失败 case 被无界重跑导致 OOM）。
//! 3. **proptest timeout**：`timeout: 60_000`——单个 case 超 60s 判定失败并中止
//!    （防回归：`${${||}}` 的无限递归曾让 render case 挂起 >60s）。

use std::sync::Arc;

use proptest::prelude::*;
use serial_test::serial;

use thymeleaf::context::Context;
use thymeleaf::expression::TemplateValue;
use thymeleaf::markup::HTMLTemplateParser;
use thymeleaf::templateparser::ITemplateParser;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::templateresource::StringTemplateResource;
use thymeleaf::util::Utf16String;
use thymeleaf::{
    IEngineConfiguration, ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode,
};

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

/// 惰性引擎配置（proptest 每用例复用）。
fn engine_configuration() -> Arc<dyn IEngineConfiguration> {
    static CONFIG: std::sync::OnceLock<Arc<dyn IEngineConfiguration>> = std::sync::OnceLock::new();
    Arc::clone(CONFIG.get_or_init(|| {
        TemplateEngine::new()
            .get_configuration()
            .expect("engine configuration")
    }))
}

/// 丢弃所有输出的 writer——parse 测试只验证不 panic，不需要输出。
/// 消除原 CapturedWriter（无界 Vec<u16>）在病态输入下的内存放大。
struct DiscardingWriter;

impl thymeleaf::util::TemplateWriter for DiscardingWriter {
    fn write_utf16(&mut self, _characters: &[u16]) -> std::io::Result<()> {
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn close(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn parse_template_no_panic(template: &str, mode: TemplateMode) {
    let configuration = engine_configuration();
    let parser: Box<dyn ITemplateParser> = match mode {
        TemplateMode::HTML | TemplateMode::XML => Box::new(HTMLTemplateParser::new(2, 4096)),
        _ => Box::new(thymeleaf::text::TextTemplateParser::new(2, 4096, true)),
    };
    let handler: Box<dyn thymeleaf::engine::ITemplateHandler> = Box::new(
        thymeleaf::engine::OutputTemplateHandler::new(Box::new(DiscardingWriter)),
    );
    let resource = Arc::new(StringTemplateResource::new(Some(template)).expect("resource"));
    // 只关心不 panic：Err 是合法的解析失败路径。
    let _ = parser.parse_standalone(
        configuration,
        Some(&js("fuzz")),
        &js("fuzz"),
        None,
        resource,
        mode,
        false,
        handler,
    );
}

/// 表达式丰富的模板：任意前缀文本 + `th:text`/`th:if` 属性 + 任意后缀。
fn expression_rich_template(prefix: &str, middle: &str, suffix: &str) -> String {
    format!(
        "<html><body><p th:text=\"${{{middle}}}\" th:if=\"${{{middle}}}\">{prefix}</p>\
         <span th:if=\"${{{middle}}} == 'x'\">{suffix}</span></body></html>"
    )
}

/// 独立 HTML 引擎（每次调用新建，避免 proptest 用例间共享缓存状态）。
fn html_engine() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver");
    engine
}

/// 回归：`th:text="${${||}}"` 必须快速返回 Err（Java parity），绝不挂起/panic。
///
/// Java 3.1.5 实测 ground truth：模板解析期抛 `TemplateInputException`
/// （嵌套 `${||}` 的 OGNL 语法错误：Malformed OGNL expression: ${||}），
/// `process()` 直接失败。Rust 侧曾因 literal substitution 无限递归在此挂起
/// （render smoke fuzz 超时根因，已由 progress 守卫 + 深度上限修复）。
#[test]
#[serial(fuzz)]
fn nested_empty_literal_render_failure_matches_java() {
    let engine = html_engine();
    let context = Context::new();
    let result = engine.process_template("<p th:text=\"${${||}}\">x</p>", &context);
    assert!(
        result.is_err(),
        "Java 在解析期失败，Rust 应同样返回 Err（而不是挂起或输出文本）"
    );
}

/// 回归：自闭合斜杠后跟属性名（`<L/ꟓ>`、`<L=x>`）不得让选择器属性扫描
/// 零前进无限循环。
///
/// render smoke fuzz 实测根因：`tag_content_end` 只剥末尾 `/`，非末尾 `/`
/// 或 `=` 出现在属性名位置时 `markup_selector::parse_attributes` 空名
/// push 后永不前进（无限 `Vec::push` → 14GB 内存膨胀 + 100% CPU 挂起）。
/// 修复后在跳过非法字符保证前进，属性合法性仍由 adapter 侧校验。
#[test]
#[serial(fuzz)]
fn selector_attribute_scan_never_stalls_on_bare_delimiter() {
    for template in [
        "<html><body><span><L/\u{a7d3}></span></body></html>",
        "<html><body><span><L=x></span></body></html>",
        "<html><body><span><L/\u{a7d3}>x</span></body></html>",
    ] {
        // 必须立即完成：Err 是合法解析失败路径，挂起/panic 均为回归。
        parse_template_no_panic(template, TemplateMode::HTML);
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        // shrink 钳制：默认 max_shrink_iters=u32::MAX / max_shrink_time=0（禁用），
        // 失败 case 会被无界重跑导致 OOM/超时。限制到 128 轮 / 5 秒。
        max_shrink_iters: 128,
        max_shrink_time: 5_000,
        // 单 case 超时守卫：`${${||}}` 类回归会让 case 无限挂起，超 60s 即失败。
        timeout: 60_000,
        ..ProptestConfig::default()
    })]

    // html_parser proptest 已恢复：历史 SIGKILL 根因是输出侧无界缓冲
    // （CapturedWriter，已由 DiscardingWriter 消除）；tokenizer 0.8.4 内部缓冲
    // O(n) 有界；解析器侧新增 64MB 模板输入上限 + token 进度守卫（span.end
    // 连续不前进即中止），配合 shrink 钳制 + proptest timeout 兜底。
    #[test]
    #[serial(fuzz)]
    fn html_parser_never_panics(template in "\\PC{0,128}") {
        parse_template_no_panic(&template, TemplateMode::HTML);
    }

    #[test]
    #[serial(fuzz)]
    fn xml_parser_never_panics(template in "\\PC{0,128}") {
        parse_template_no_panic(&template, TemplateMode::XML);
    }

    #[test]
    #[serial(fuzz)]
    fn text_parser_never_panics(template in "\\PC{0,128}") {
        parse_template_no_panic(&template, TemplateMode::TEXT);
    }

    // render smoke：随机三注入（前缀/表达式/后缀）。曾因 `${${||}}` 类输入触发
    // literal substitution 无限递归挂起（>60s）；根因已在引擎侧修复
    // （progress 守卫 + 深度上限），此处以 proptest timeout 60s 兜底防回归。
    #[test]
    #[serial(fuzz)]
    fn template_render_smoke_never_panics(
        prefix in "\\PC{0,32}",
        middle in "\\PC{0,32}",
        suffix in "\\PC{0,32}",
    ) {
        eprintln!(
            "CASE prefix={prefix:?} middle={middle:?} suffix={suffix:?}"
        );
        let template = expression_rich_template(&prefix, &middle, &suffix);
        let engine = html_engine();
        let context = Context::new();
        context.set_variable(
            Some(js("value")),
            Some(Arc::new(TemplateValue::string(js("v")))),
        );
        let _ = engine.process_template(&template, &context);
    }
}
