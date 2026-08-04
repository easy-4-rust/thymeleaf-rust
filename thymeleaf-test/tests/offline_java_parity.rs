//! Offline 渲染差分 —— 1:1 移植 Java `OfflineTest`。
//!
//! Java 用例：`ClassLoaderTemplateResolver` + 普通 `Context`（`one` 变量），
//! `process("offline/offline01.html")` 后与 `offline01-result.html` 归一化比较
//! （`ResourceUtils.normalize`：去 `\r`、去末尾空白）。Rust 侧用文件解析器
//! 指向本 crate 的 `tests/fixtures/offline/`（对应 Java 类路径资源），归一化
//! 语义与 Java `ResourceUtils#normalize` 一致。

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::Utf16String;

/// Java `ResourceUtils#normalize`：逐行去 `\r`，再去末尾空白。
fn normalize(text: &str) -> String {
    let no_cr = text
        .chars()
        .filter(|character| *character != '\r')
        .collect::<String>();
    no_cr.trim_end().to_owned()
}

#[test]
fn offline01_matches_java() {
    let mut template_resolver = thymeleaf::templateresolver::FileTemplateResolver::new();
    template_resolver.set_prefix(Some(Utf16String::from_rust_str(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/offline/"
    ))));
    template_resolver.set_suffix(Some(Utf16String::from_rust_str(".html")));
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(template_resolver))
        .expect("template resolver");

    // Java：`ctx.setVariable("one", "This is one")`
    let context = thymeleaf::context::Context::new();
    context.set_variable(
        Some(Utf16String::from_rust_str("one")),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "This is one",
        )))),
    );
    let result = engine
        .process_template("offline01", &context)
        .expect("offline render");
    let actual = normalize(&result.to_string_lossy());

    // Java：`ResourceUtils.read(result.html, "UTF-8", true)` —— 同样归一化
    let expected_path = format!(
        "{}/tests/fixtures/offline/offline01-result.html",
        env!("CARGO_MANIFEST_DIR")
    );
    let expected = std::fs::read_to_string(expected_path).expect("offline result fixture");
    let expected = normalize(&expected);

    assert_eq!(
        actual, expected,
        "offline01.html 渲染输出与 Java offline01-result.html 不一致"
    );
}
