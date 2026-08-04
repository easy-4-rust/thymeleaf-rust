//! `LoggingUtils` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::{Display, Write};

use thymeleaf::util::{LoggingUtils, Utf16String};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/logging_utils_golden.txt");

#[test]
fn logging_utils_matches_java_utf16_and_identity_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit(
        &mut output,
        "null",
        LoggingUtils::loggify_template_name(None).is_none(),
    );

    emit_case(&mut output, "empty", Utf16String::from_rust_str(""));
    emit_case(&mut output, "short", Utf16String::from_rust_str("home"));
    emit_case(
        &mut output,
        "short_lf",
        Utf16String::from_rust_str("home\npage"),
    );
    emit_case(
        &mut output,
        "short_cr",
        Utf16String::from_rust_str("home\rpage"),
    );
    emit_case(
        &mut output,
        "length_120",
        Utf16String::from_rust_str(&"x".repeat(120)),
    );
    emit_case(
        &mut output,
        "length_121",
        Utf16String::from_rust_str(&"x".repeat(121)),
    );
    emit_case(
        &mut output,
        "long_lf",
        Utf16String::from_rust_str(&format!("{}\n{}", "a".repeat(34), "b".repeat(90))),
    );
    emit_case(
        &mut output,
        "prefix_surrogate_split",
        Utf16String::from_rust_str(&format!("{}😀{}", "a".repeat(34), "b".repeat(90))),
    );
    emit_case(
        &mut output,
        "suffix_surrogate_split",
        Utf16String::from_rust_str(&format!("{}😀{}", "a".repeat(41), "b".repeat(79))),
    );

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_case(output: &mut String, key: &str, source: Utf16String) {
    let result = LoggingUtils::loggify_template_name(Some(&source)).expect("result");
    emit(output, &format!("{key}.source_length"), source.len());
    emit(
        output,
        &format!("{key}.result_length"),
        result.as_utf16_string().len(),
    );
    emit(
        output,
        &format!("{key}.same"),
        result.is_borrowed_from(&source),
    );
    emit(
        output,
        &format!("{key}.utf16"),
        utf16_hex(result.as_utf16_string()),
    );
}

fn utf16_hex(value: &Utf16String) -> String {
    value
        .as_utf16()
        .iter()
        .map(|unit| format!("{unit:04X}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn emit(output: &mut String, key: &str, value: impl Display) {
    writeln!(output, "{key}={value}").expect("string output");
}
