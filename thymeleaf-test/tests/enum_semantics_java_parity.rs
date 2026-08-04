//! 三个独立 enum 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::Write;

use thymeleaf::engine::HTMLElementType;
use thymeleaf::inline::{StandardInlineMode, StandardInlineModeParseError};
use thymeleaf::model::AttributeValueQuotes;
use thymeleaf::util::Utf16String;

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/enum_semantics_golden.txt");
const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[test]
fn independent_enums_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_attribute_value_quotes(&mut output);
    emit_html_element_types(&mut output);
    emit_standard_inline_modes(&mut output);
    emit_parse_cases(&mut output);
    emit_exhaustive_inline_parsing(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_attribute_value_quotes(output: &mut String) {
    emit(output, "quotes.count", AttributeValueQuotes::VALUES.len());
    for value in AttributeValueQuotes::VALUES {
        emit(
            output,
            &format!("quotes.{value}"),
            format!("{},{value},{value}", value.ordinal()),
        );
    }
}

fn emit_html_element_types(output: &mut String) {
    emit(output, "html.count", HTMLElementType::VALUES.len());
    for value in HTMLElementType::VALUES {
        emit(
            output,
            &format!("html.{value}"),
            format!("{},{value},{value},{}", value.ordinal(), value.is_void()),
        );
    }
}

fn emit_standard_inline_modes(output: &mut String) {
    emit(output, "inline.count", StandardInlineMode::VALUES.len());
    for value in StandardInlineMode::VALUES {
        emit(
            output,
            &format!("inline.{value}"),
            format!("{},{value},{value}", value.ordinal()),
        );
    }
}

fn emit_parse_cases(output: &mut String) {
    for (key, input) in [
        ("null", None),
        ("empty", Some(Utf16String::from_rust_str(""))),
        ("space", Some(Utf16String::from_rust_str(" "))),
        (
            "controls",
            Some(Utf16String::from_utf16([0x0000, 0x0009, 0x0020])),
        ),
        ("nbsp", Some(Utf16String::from_utf16([0x00A0]))),
        ("raw", Some(Utf16String::from_rust_str("RAW"))),
        ("noneLower", Some(Utf16String::from_rust_str("none"))),
        ("htmlMixed", Some(Utf16String::from_rust_str("hTmL"))),
        ("xmlLower", Some(Utf16String::from_rust_str("xml"))),
        ("textMixed", Some(Utf16String::from_rust_str("TeXt"))),
        (
            "javascriptLower",
            Some(Utf16String::from_rust_str("javascript")),
        ),
        ("cssLower", Some(Utf16String::from_rust_str("css"))),
        (
            "cssLongS",
            Some(Utf16String::from_utf16([b'C' as u16, 0x017F, 0x017F])),
        ),
        (
            "javascriptDotlessI",
            Some(Utf16String::from_rust_str("JAVASCRıPT")),
        ),
        (
            "javascriptDottedI",
            Some(Utf16String::from_rust_str("JAVASCRİPT")),
        ),
    ] {
        emit_parse(output, key, input.as_ref());
    }
    emit_parse_utf16(
        output,
        "paddedHtml",
        Some(&Utf16String::from_rust_str(" HTML ")),
    );
    emit_parse_utf16(
        output,
        "isolatedHighSurrogate",
        Some(&Utf16String::from_utf16([0xD800])),
    );
}

fn emit_exhaustive_inline_parsing(output: &mut String) {
    let mut single_code_unit_hash = FNV_OFFSET;
    for code_unit in u16::MIN..=u16::MAX {
        single_code_unit_hash = mix(
            single_code_unit_hash,
            parse_code(Some(&Utf16String::from_utf16([code_unit]))),
        );
    }
    emit(
        output,
        "exhaustive.singleCodeUnitHash",
        format!("{single_code_unit_hash:016x}"),
    );

    for mode in StandardInlineMode::VALUES {
        let mut units = Utf16String::from_rust_str(&mode.to_string())
            .as_utf16()
            .to_vec();
        for position in 0..units.len() {
            let original = units[position];
            let mut hash = FNV_OFFSET;
            for code_unit in u16::MIN..=u16::MAX {
                units[position] = code_unit;
                hash = mix(
                    hash,
                    parse_code(Some(&Utf16String::from_utf16(units.clone()))),
                );
            }
            units[position] = original;
            emit(
                output,
                &format!("exhaustive.{mode}.{position}"),
                format!("{hash:016x}"),
            );
        }
    }
}

fn parse_code(input: Option<&Utf16String>) -> u8 {
    match StandardInlineMode::parse(input) {
        Ok(mode) => mode.ordinal() as u8,
        Err(StandardInlineModeParseError::NullOrEmpty) => 6,
        Err(StandardInlineModeParseError::Unrecognized(_)) => 7,
    }
}

fn emit_parse(output: &mut String, key: &str, input: Option<&Utf16String>) {
    match StandardInlineMode::parse(input) {
        Ok(mode) => emit(output, &format!("parse.{key}"), format!("OK:{mode}")),
        Err(error) => emit(
            output,
            &format!("parse.{key}"),
            format!("ERR:java.lang.IllegalArgumentException:{error}"),
        ),
    }
}

fn emit_parse_utf16(output: &mut String, key: &str, input: Option<&Utf16String>) {
    match StandardInlineMode::parse(input) {
        Ok(mode) => emit(output, &format!("parse.{key}"), format!("OK:{mode}")),
        Err(error) => emit(
            output,
            &format!("parse.{key}"),
            format!(
                "ERR:{}:{}",
                error.java_class_name(),
                to_utf16_hex(error.message().as_utf16())
            ),
        ),
    }
}

fn to_utf16_hex(units: &[u16]) -> String {
    units
        .iter()
        .map(|unit| format!("{unit:04x}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn mix(hash: u64, value: u8) -> u64 {
    (hash ^ u64::from(value)).wrapping_mul(FNV_PRIME)
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to string");
}
