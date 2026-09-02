//! VALUE_ADD：`StringUtils` 覆盖缺口测试（2026-09-02）——风险：Escape/Unescape/whitespace/locale 分支。
//!
//! 缺失行 198-214/467-490 等分散在 prepend/append/escape_java_script/unescape_java_script/
//! capitalize_words/turkic locale 分支。Java 侧有完整 golden 覆盖，以下补齐 Java 测试未
//! 触达的 Rust 分支。

use thymeleaf::util::{Locale, StringUtils, Utf16String};

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
}

fn tr_locale() -> Locale {
    Locale::new(js("tr"), js("TR"))
}

// ===========================================================================
// prepend/append: null target returns None, non-null prepends/appends
// ===========================================================================

#[test]
fn prepend_null_target_returns_none() {
    let result = StringUtils::prepend(None, Some(&js("prefix"))).unwrap();
    assert_eq!(result, None);
}

#[test]
fn prepend_non_null_target_prepends() {
    let result = StringUtils::prepend(Some(&js("world")), Some(&js("hello ")))
        .unwrap()
        .unwrap();
    assert_eq!(result.to_string_lossy(), "hello world");
}

#[test]
fn append_null_target_returns_none() {
    let result = StringUtils::append(None, Some(&js("suffix"))).unwrap();
    assert_eq!(result, None);
}

#[test]
fn append_non_null_target_appends() {
    let result = StringUtils::append(Some(&js("hello")), Some(&js(" world")))
        .unwrap()
        .unwrap();
    assert_eq!(result.to_string_lossy(), "hello world");
}

// ===========================================================================
// escape_java_script: control characters and special chars
// ===========================================================================

#[test]
fn escape_java_script_handles_backslash_and_quotes() {
    let input = js(r#"say "hello" and 'bye'\"#);
    let escaped = StringUtils::escape_java_script(Some(&input)).unwrap();
    let text = escaped.to_string_lossy();
    assert!(
        text.contains("\\\""),
        "double quote must be escaped: {text}"
    );
    assert!(text.contains("\\'"), "single quote must be escaped: {text}");
    assert!(text.contains("\\\\"), "backslash must be escaped: {text}");
}

#[test]
fn escape_java_script_handles_control_chars() {
    let input = js("line1\nline2\ttab");
    let escaped = StringUtils::escape_java_script(Some(&input)).unwrap();
    let text = escaped.to_string_lossy();
    assert!(text.contains("\\n"), "newline must be escaped: {text}");
    assert!(text.contains("\\t"), "tab must be escaped: {text}");
}

#[test]
fn escape_java_script_escapes_forward_slash() {
    let input = js("</script>");
    let escaped = StringUtils::escape_java_script(Some(&input)).unwrap();
    let text = escaped.to_string_lossy();
    assert!(
        text.contains("\\/"),
        "forward slash must be escaped in JS: {text}"
    );
}

// ===========================================================================
// escape_java: does NOT escape forward slash (differs from JS)
// ===========================================================================

#[test]
fn escape_java_does_not_escape_forward_slash() {
    let input = js("path/to/file");
    let escaped = StringUtils::escape_java(Some(&input)).unwrap();
    assert_eq!(escaped.to_string_lossy(), "path/to/file");
}

// ===========================================================================
// escape_xml: handles all XML special characters
// ===========================================================================

#[test]
fn escape_xml_handles_all_special_chars() {
    let input = js("<b>\"hello\" & 'world'</b>");
    let escaped = StringUtils::escape_xml(Some(&input)).unwrap();
    let text = escaped.to_string_lossy();
    assert!(text.contains("&lt;"), "must escape <: {text}");
    assert!(text.contains("&gt;"), "must escape >: {text}");
    assert!(text.contains("&amp;"), "must escape &: {text}");
    assert!(text.contains("&quot;"), "must escape \": {text}");
    assert!(text.contains("&#39;"), "must escape ': {text}");
}

// ===========================================================================
// unescape_java_script: round-trip with escape
// ===========================================================================

#[test]
fn unescape_java_script_round_trips() {
    let input = js("say \"hello\"\nand \\backslash");
    let escaped = StringUtils::escape_java_script(Some(&input)).unwrap();
    let unescaped = StringUtils::unescape_java_script(Some(&escaped)).unwrap();
    assert_eq!(unescaped.to_string_lossy(), input.to_string_lossy());
}

// ===========================================================================
// unescape_java_script: \\uXXXX unicode escapes
// ===========================================================================

#[test]
fn unescape_java_script_handles_unicode_escapes() {
    let input = js("\\u0048\\u0065\\u006C\\u006C\\u006F");
    let unescaped = StringUtils::unescape_java_script(Some(&input)).unwrap();
    assert_eq!(unescaped.to_string_lossy(), "Hello");
}

// ===========================================================================
// unescape_java_script: invalid \\uXXXX passes through
// ===========================================================================

#[test]
fn unescape_java_script_invalid_unicode_passes_through() {
    let input = js("\\uZZZZ");
    let unescaped = StringUtils::unescape_java_script(Some(&input)).unwrap();
    // invalid hex digits: \\u stays, ZZZZ stays
    assert!(
        unescaped.to_string_lossy().contains('Z'),
        "invalid unicode must pass through"
    );
}

// ===========================================================================
// capitalize_words: custom delimiters
// ===========================================================================

#[test]
fn capitalize_words_with_custom_delimiters() {
    let input = js("hello-world");
    let result = StringUtils::capitalize_words(Some(&input), Some(&js("-"))).unwrap();
    assert_eq!(result.to_string_lossy(), "Hello-World");
}

// ===========================================================================
// to_upper_case / to_lower_case: Turkic locale (i -> I with dot)
// ===========================================================================

#[test]
fn to_upper_case_turkic_i() {
    let input = js("istanbul");
    let result = StringUtils::to_upper_case(Some(&input), Some(&tr_locale()))
        .unwrap()
        .unwrap();
    // Turkish: 'i' -> 'I' (U+0130, Latin capital I with dot above)
    assert!(
        result.to_string_lossy().contains('\u{0130}'),
        "Turkish i must uppercase to I-with-dot: {}",
        result.to_string_lossy()
    );
}

#[test]
fn to_lower_case_turkic_capital_i() {
    let input = js("I");
    let result = StringUtils::to_lower_case(Some(&input), Some(&tr_locale()))
        .unwrap()
        .unwrap();
    // Turkish: 'I' -> dotless 'i' (U+0131)
    assert_eq!(result.to_string_lossy(), "\u{0131}");
}

// ===========================================================================
// capitalize / un_capitalize
// ===========================================================================

#[test]
fn capitalize_first_char() {
    let input = js("hello");
    let result = StringUtils::capitalize(Some(&input)).unwrap();
    assert_eq!(result.to_string_lossy(), "Hello");
}

#[test]
fn un_capitalize_first_char() {
    let input = js("Hello");
    let result = StringUtils::un_capitalize(Some(&input)).unwrap();
    assert_eq!(result.to_string_lossy(), "hello");
}

#[test]
fn capitalize_empty_string() {
    let input = js("");
    let result = StringUtils::capitalize(Some(&input)).unwrap();
    assert_eq!(result.to_string_lossy(), "");
}

// ===========================================================================
// pack: removes whitespace and lowercases
// ===========================================================================

#[test]
fn pack_removes_whitespace_and_lowercases() {
    let input = js("  Hello  World  ");
    let result = StringUtils::pack(Some(&input)).unwrap();
    assert_eq!(result.to_string_lossy(), "helloworld");
}

// ===========================================================================
// repeat: zero and negative times produce empty
// ===========================================================================

#[test]
fn repeat_zero_times_produces_empty() {
    let input = js("abc");
    let result = StringUtils::repeat(Some(&input), 0).unwrap();
    assert!(result.is_empty());
}

#[test]
fn repeat_negative_times_produces_empty() {
    let input = js("abc");
    let result = StringUtils::repeat(Some(&input), -5).unwrap();
    assert!(result.is_empty());
}

// ===========================================================================
// random_alphanumeric: produces correct length and charset
// ===========================================================================

#[test]
fn random_alphanumeric_correct_length() {
    for count in [0, 1, 10, 100] {
        let result = StringUtils::random_alphanumeric(count);
        assert_eq!(result.len(), count as usize, "length must be {count}");
    }
}

#[test]
fn random_alphanumeric_only_uppercase_and_digits() {
    let result = StringUtils::random_alphanumeric(1000);
    for ch in result.to_string_lossy().chars() {
        assert!(
            ch.is_ascii_uppercase() || ch.is_ascii_digit(),
            "unexpected char: {ch}"
        );
    }
}
