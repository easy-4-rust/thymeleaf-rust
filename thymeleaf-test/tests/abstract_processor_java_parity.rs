//! `AbstractProcessor` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::Write;

use thymeleaf::util::ValidateError;
use thymeleaf::{AbstractProcessor, IProcessor, TemplateMode};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/abstract_processor_golden.txt");

#[test]
fn abstract_processor_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    match AbstractProcessor::new(None, 123) {
        Ok(_) => emit(&mut output, "null", "<none>"),
        Err(error) => emit_validate_error(&mut output, "null", &error),
    }

    for (key, template_mode, precedence) in [
        ("html", TemplateMode::HTML, i32::MIN),
        ("xml", TemplateMode::XML, -1),
        ("text", TemplateMode::TEXT, 0),
        ("javascript", TemplateMode::JAVASCRIPT, 1),
        ("css", TemplateMode::CSS, 1_000),
        ("raw", TemplateMode::RAW, i32::MAX),
    ] {
        emit_case(&mut output, key, template_mode, precedence);
    }

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_case(output: &mut String, key: &str, template_mode: TemplateMode, precedence: i32) {
    let implementation =
        AbstractProcessor::new(Some(template_mode), precedence).expect("non-null mode is valid");
    let processor: &dyn IProcessor = &implementation;
    let interface_mode = processor
        .get_template_mode()
        .expect("AbstractProcessor mode is non-null");

    emit(
        output,
        &format!("case.{key}"),
        format!(
            "mode={},precedence={},interfaceMode={},interfacePrecedence={},modeIdentity={},stable={}",
            implementation.get_template_mode(),
            implementation.get_precedence(),
            interface_mode,
            processor.get_precedence(),
            implementation.get_template_mode() == template_mode,
            implementation.get_template_mode() == interface_mode
                && implementation.get_precedence() == processor.get_precedence()
        ),
    );
}

fn emit_validate_error(output: &mut String, key: &str, error: &ValidateError) {
    emit(
        output,
        key,
        format!(
            "ERR:{}:{}",
            error.java_class_name(),
            error.get_message().unwrap_or("null")
        ),
    );
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to string");
}
