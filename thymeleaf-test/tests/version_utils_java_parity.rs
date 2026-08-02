//! `VersionUtils` 与 `VersionSpec` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::{Display, Write};

use thymeleaf::util::{VersionSpec, VersionUtils};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/version_utils_golden.txt");

#[test]
fn version_utils_and_spec_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    emit_spec(&mut output, "null", &VersionUtils::parse_version(None));
    emit_spec(
        &mut output,
        "null_build",
        &VersionUtils::parse_version_with_build_timestamp(None, Some("build-1")),
    );
    emit_spec(&mut output, "empty", &VersionUtils::parse_version(Some("")));
    emit_spec(
        &mut output,
        "ascii_blank",
        &VersionUtils::parse_version(Some("\0\t ")),
    );
    emit_spec(
        &mut output,
        "nbsp",
        &VersionUtils::parse_version(Some("\u{00A0}")),
    );
    emit_spec(
        &mut output,
        "major",
        &VersionUtils::parse_version(Some("7")),
    );
    emit_spec(
        &mut output,
        "minor",
        &VersionUtils::parse_version(Some("7.2")),
    );
    emit_spec(
        &mut output,
        "patch",
        &VersionUtils::parse_version(Some("7.2.4")),
    );
    emit_spec(
        &mut output,
        "trimmed",
        &VersionUtils::parse_version(Some(" \t3.1.5.RELEASE\r\n")),
    );
    emit_spec(
        &mut output,
        "release_joined",
        &VersionUtils::parse_version(Some("3.1.5RELEASE")),
    );
    emit_spec(
        &mut output,
        "release_dash",
        &VersionUtils::parse_version(Some("3.1.5-RELEASE")),
    );
    emit_spec(
        &mut output,
        "release_lower",
        &VersionUtils::parse_version(Some("3.1.5-release")),
    );
    emit_spec(
        &mut output,
        "rc",
        &VersionUtils::parse_version(Some("3.1.5.RC1")),
    );
    emit_spec(
        &mut output,
        "letter",
        &VersionUtils::parse_version(Some("2beta")),
    );
    emit_spec(
        &mut output,
        "underscore",
        &VersionUtils::parse_version(Some("2_beta")),
    );
    emit_spec(
        &mut output,
        "one_dot_rc",
        &VersionUtils::parse_version(Some("1.RC1")),
    );
    emit_spec(
        &mut output,
        "leading_zeroes",
        &VersionUtils::parse_version(Some("001.02.003")),
    );
    emit_spec(
        &mut output,
        "max",
        &VersionUtils::parse_version(Some("2147483647")),
    );
    emit_spec(
        &mut output,
        "overflow",
        &VersionUtils::parse_version(Some("2147483648")),
    );
    emit_spec(
        &mut output,
        "negative",
        &VersionUtils::parse_version(Some("-1")),
    );
    emit_spec(
        &mut output,
        "trailing_dot",
        &VersionUtils::parse_version(Some("1.")),
    );
    emit_spec(
        &mut output,
        "trailing_dash",
        &VersionUtils::parse_version(Some("1-")),
    );
    emit_spec(
        &mut output,
        "double_dot",
        &VersionUtils::parse_version(Some("1..2")),
    );
    emit_spec(
        &mut output,
        "four_parts",
        &VersionUtils::parse_version(Some("1.2.3.4")),
    );
    emit_spec(
        &mut output,
        "separator_blank",
        &VersionUtils::parse_version(Some("1-\t")),
    );
    emit_spec(
        &mut output,
        "separator_nbsp",
        &VersionUtils::parse_version(Some("1-\u{00A0}")),
    );
    emit_spec(
        &mut output,
        "qualifier_space",
        &VersionUtils::parse_version(Some("1- RC ")),
    );
    emit_spec(
        &mut output,
        "arabic_digits",
        &VersionUtils::parse_version(Some("١.٢.٣")),
    );
    emit_spec(
        &mut output,
        "fullwidth_digits",
        &VersionUtils::parse_version(Some("９.８β")),
    );
    emit_spec(
        &mut output,
        "unicode_modifier_letter",
        &VersionUtils::parse_version(Some("1ʰ")),
    );
    emit_spec(
        &mut output,
        "unicode_mark_separator",
        &VersionUtils::parse_version(Some("1\u{0301}mark")),
    );
    emit_spec(
        &mut output,
        "supplementary_letter",
        &VersionUtils::parse_version(Some("1𐐀")),
    );
    emit_spec(
        &mut output,
        "leading_dot_qualifier",
        &VersionUtils::parse_version(Some(".RC")),
    );
    emit_spec(
        &mut output,
        "empty_build",
        &VersionUtils::parse_version_with_build_timestamp(Some("1.2"), Some("")),
    );
    emit_spec(
        &mut output,
        "full_build",
        &VersionUtils::parse_version_with_build_timestamp(
            Some("1.2"),
            Some("2026-07-29T00:00:00Z"),
        ),
    );

    emit(
        &mut output,
        "character.digit_ranges",
        character_ranges(|character| {
            !VersionUtils::parse_version(Some(&character.to_string())).is_unknown()
        }),
    );
    emit(
        &mut output,
        "character.letter_ranges",
        character_ranges(|character| {
            let input = format!("1{character}q");
            VersionUtils::parse_version(Some(&input))
                .get_qualifier()
                .is_some_and(|qualifier| {
                    qualifier.as_utf16().first() == Some(&(character as u32 as u16))
                })
        }),
    );

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_spec(output: &mut String, key: &str, version_spec: &VersionSpec) {
    let prefix = format!("version.{key}.");
    emit(
        output,
        &format!("{prefix}unknown"),
        version_spec.is_unknown(),
    );
    emit(output, &format!("{prefix}major"), version_spec.get_major());
    emit(output, &format!("{prefix}minor"), version_spec.get_minor());
    emit(output, &format!("{prefix}patch"), version_spec.get_patch());
    emit(
        output,
        &format!("{prefix}has_qualifier"),
        version_spec.has_qualifier(),
    );
    emit(
        output,
        &format!("{prefix}qualifier"),
        encode_utf16(
            version_spec
                .get_qualifier()
                .map(thymeleaf::util::VersionQualifier::as_utf16),
        ),
    );
    emit(
        output,
        &format!("{prefix}core"),
        encode(Some(version_spec.get_version_core())),
    );
    emit(
        output,
        &format!("{prefix}version"),
        encode(Some(version_spec.get_version())),
    );
    emit(
        output,
        &format!("{prefix}has_build"),
        version_spec.has_build_timestamp(),
    );
    emit(
        output,
        &format!("{prefix}build"),
        encode(version_spec.get_build_timestamp()),
    );
    emit(
        output,
        &format!("{prefix}full"),
        encode(Some(version_spec.get_full_version())),
    );
    emit(
        output,
        &format!("{prefix}stable"),
        version_spec.is_stable_release(),
    );
    emit(
        output,
        &format!("{prefix}at_least_neg1"),
        version_spec.is_at_least(-1),
    );
    emit(
        output,
        &format!("{prefix}at_least_0"),
        version_spec.is_at_least(0),
    );
    emit(
        output,
        &format!("{prefix}at_least_3"),
        version_spec.is_at_least(3),
    );
    emit(
        output,
        &format!("{prefix}at_least_3_1"),
        version_spec.is_at_least_with_minor(3, 1),
    );
    emit(
        output,
        &format!("{prefix}at_least_3_1_5"),
        version_spec.is_at_least_with_patch(3, 1, 5),
    );
    emit(
        output,
        &format!("{prefix}at_least_3_1_6"),
        version_spec.is_at_least_with_patch(3, 1, 6),
    );
    emit(
        output,
        &format!("{prefix}at_least_4"),
        version_spec.is_at_least(4),
    );
}

fn encode(value: Option<&str>) -> String {
    encode_utf16(
        value
            .map(|value| value.encode_utf16().collect::<Vec<_>>())
            .as_deref(),
    )
}

fn encode_utf16(value: Option<&[u16]>) -> String {
    value.map_or_else(
        || "<null>".to_owned(),
        |value| {
            value
                .iter()
                .map(|unit| format!("{unit:04X}"))
                .collect::<Vec<_>>()
                .join(",")
        },
    )
}

fn character_ranges(mut predicate: impl FnMut(char) -> bool) -> String {
    let mut ranges = Vec::new();
    let mut start = None;
    let mut end = 0_u16;
    for code_unit in 0_u16..=u16::MAX {
        let matches = char::from_u32(u32::from(code_unit)).is_some_and(&mut predicate);
        if matches {
            if start.is_none() {
                start = Some(code_unit);
            }
            end = code_unit;
        } else if let Some(range_start) = start.take() {
            ranges.push(format_range(range_start, end));
        }
    }
    if let Some(range_start) = start {
        ranges.push(format_range(range_start, end));
    }
    ranges.join(";")
}

fn format_range(start: u16, end: u16) -> String {
    if start == end {
        format!("{start:04X}")
    } else {
        format!("{start:04X}-{end:04X}")
    }
}

fn emit(output: &mut String, key: &str, value: impl Display) {
    writeln!(output, "{key}={value}").expect("string output");
}
