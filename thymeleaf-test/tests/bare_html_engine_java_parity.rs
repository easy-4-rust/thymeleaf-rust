//! Bare HTML 引擎差分 —— 1:1 移植 Java `BareHtmlEngineTest`。
//!
//! Java 用例：`HTMLTemplateParser(2, 4096)` + 无方言裸配置 + `OutputTemplateHandler`，
//! 对 26 个畸形/边界 HTML 片段断言 `parseStandalone` 的完整输出（普通用例输出 ==
//! 输入；`//img` 块选择器用例输出 == 选中的片段）。Rust 侧使用相同的公共 API：
//! `HTMLTemplateParser`、`StringTemplateResource`、`OutputTemplateHandler`。

use std::io;
use std::sync::{Arc, Mutex};

use thymeleaf::ITemplateEngine;
use thymeleaf::TemplateMode;
use thymeleaf::engine::ITemplateHandler;
use thymeleaf::markup::HTMLTemplateParser;
use thymeleaf::templateparser::ITemplateParser;
use thymeleaf::templateresource::StringTemplateResource;
use thymeleaf::util::{JavaWriter, Utf16String};

/// 捕获 UTF-16 输出的 Writer（对应 Java `StringWriter`）。
#[derive(Default)]
struct CapturedWriter {
    buffer: Arc<Mutex<Vec<u16>>>,
}

impl JavaWriter for CapturedWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        self.buffer
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .extend_from_slice(characters);
        Ok(())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

/// 构造带可读 Writer 的 `OutputTemplateHandler`（与调用方共享缓冲区）。
fn handler_with_writer() -> (Box<dyn ITemplateHandler>, Arc<Mutex<Vec<u16>>>) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = CapturedWriter {
        buffer: Arc::clone(&buffer),
    };
    (
        Box::new(thymeleaf::engine::OutputTemplateHandler::new(Box::new(
            writer,
        ))),
        buffer,
    )
}

/// `BareHtmlEngineTest#check(String, String, Set<String>)`：
/// `PARSER.parseStandalone(config, "test", "test", selectors, resource, HTML, false, handler)`。
fn check(input: &str, output: &str, block_selectors: Option<&[&str]>) {
    let engine = thymeleaf::TemplateEngine::new();
    let configuration = engine.get_configuration().expect("engine configuration");
    let parser = HTMLTemplateParser::new(2, 4096);
    let (handler, buffer) = handler_with_writer();
    let selectors = block_selectors.map(|selectors| {
        selectors
            .iter()
            .map(|selector| Utf16String::from_rust_str(selector))
            .collect::<Vec<_>>()
    });
    let resource = Arc::new(StringTemplateResource::new(Some(input)).expect("string resource"));
    parser
        .parse_standalone(
            configuration,
            Some(&js("test")),
            &js("test"),
            selectors.as_deref(),
            resource,
            TemplateMode::HTML,
            false,
            handler,
        )
        .expect("parse standalone");
    let actual =
        String::from_utf16_lossy(&buffer.lock().unwrap_or_else(|error| error.into_inner()));
    if actual != output {
        panic!(
            "BareHtmlEngineTest 输入: {input:?}\n  expected: {output:?}\n  actual:   {actual:?}"
        );
    }
}

#[test]
fn bare_html_engine_matches_java_26_cases() {
    // 单输入用例：解析后完整输出与输入逐字节一致（Java check(String)）
    let mut failures = Vec::new();
    for input in [
        "<!doctype html>",
        "<img href='http://something.com'>",
        "<img href='http://something.com'/>",
        "<img href='http://something.com' >",
        "<img href='http://something.com' />",
        "<img href='http://something.com' >",
        "<img \n href='http://something.com' />",
        "<img \n href = \"http://something.com\" />",
        "<img \n href = something >",
        "<img \n href = something disabled>",
        "<img \n href = something disabled= 'disabled'>",
        "<p id='http://something.com'>...</p>",
        "<p id='http://something.com'></p>",
        "<p id='http://something.com'/>",
        "<p id='http://something.com'>...</p>",
        "<p id='http://something.com' >...</p>",
        "<p id='http://something.com' />...</p>",
        "<p id='http://something.com' >...</p>",
        "<p id='http://something.com' >...</p>",
        "<p \n id='http://something.com' />.\n.\n.\n</p>",
        "<p \n id = \"http://something.com\" ></p>",
        "<p \n id = something >\n\n <div>lala</p>",
        "<p \n id = something disabled>...</p>",
        "<p \n id = something disabled= 'disabled'>",
    ] {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check(input, input, None)));
        if let Err(payload) = result {
            failures.push(format!("{input:?} -> {payload:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} 个用例失败:\n{}",
        failures.len(),
        failures.join("\n")
    );

    // 块选择器用例：`//img` 提取片段（Java check(String, String, "//img") 重复两次）
    for _ in 0..2 {
        check(
            "<div><img \n href = something disabled= 'disabled'>",
            "<img \n href = something disabled= 'disabled'>",
            Some(&["//img"]),
        );
    }
}
