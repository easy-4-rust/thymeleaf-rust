//! `IProcessor` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::Write;
use std::sync::Mutex;

use thymeleaf::{IProcessor, TemplateMode};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/processor_contract_golden.txt");

struct MutableProcessor {
    template_mode: Mutex<Option<TemplateMode>>,
    precedence: Mutex<i32>,
}

impl IProcessor for MutableProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        *self.template_mode.lock().expect("template mode lock")
    }

    fn get_precedence(&self) -> i32 {
        *self.precedence.lock().expect("precedence lock")
    }
}

#[test]
fn processor_contract_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    let implementation = MutableProcessor {
        template_mode: Mutex::new(None),
        precedence: Mutex::new(i32::MIN),
    };
    let processor: &dyn IProcessor = &implementation;

    emit_mode(&mut output, "initial.mode", processor.get_template_mode());
    emit(
        &mut output,
        "initial.precedence",
        processor.get_precedence(),
    );

    for template_mode in [
        TemplateMode::HTML,
        TemplateMode::XML,
        TemplateMode::TEXT,
        TemplateMode::JAVASCRIPT,
        TemplateMode::CSS,
        TemplateMode::RAW,
    ] {
        *implementation
            .template_mode
            .lock()
            .expect("template mode lock") = Some(template_mode);
        emit_mode(
            &mut output,
            &format!("mode.{template_mode}"),
            processor.get_template_mode(),
        );
    }

    *implementation.precedence.lock().expect("precedence lock") = 0;
    emit(&mut output, "precedence.zero", processor.get_precedence());
    *implementation.precedence.lock().expect("precedence lock") = i32::MAX;
    emit(&mut output, "precedence.max", processor.get_precedence());

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_mode(output: &mut String, key: &str, template_mode: Option<TemplateMode>) {
    match template_mode {
        Some(template_mode) => emit(output, key, template_mode),
        None => emit(output, key, "null"),
    }
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to string");
}
