//! VALUE_ADD：`NumberUtils` 覆盖缺口测试（2026-09-02）——风险：数字解析边界与格式分支。
//!
//! 缺失行 36-243 分散在 format/sequence/format_currency/format_percent 及内部辅助函数。
//! Java 侧 `NumberUtilsTest` 不存在独立测试；覆盖来自 `StandardExpression` 集成路径。
//! 以下按 VALUE_ADD 补充边界分支：非法输入、精度、Locale 分支、sequence 反向/零步长。

use thymeleaf::util::{Locale, NumberPointType, NumberUtils, NumberValue, Utf16String};

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
}

fn en_us() -> Locale {
    Locale::new(js("en"), js("US"))
}

fn de_de() -> Locale {
    Locale::new(js("de"), js("DE"))
}

fn fr_fr() -> Locale {
    Locale::new(js("fr"), js("FR"))
}

fn jp_jp() -> Locale {
    Locale::new(js("ja"), js("JP"))
}

fn kr_kr() -> Locale {
    Locale::new(js("ko"), js("KR"))
}

// ===========================================================================
// format: None target returns Ok(None)
// ===========================================================================

#[test]
fn format_none_target_returns_none() {
    let result = NumberUtils::format(
        None,
        Some(1),
        Some(NumberPointType::Default),
        Some(2),
        Some(NumberPointType::Default),
        Some(&en_us()),
    );
    assert_eq!(result.unwrap(), None);
}

// ===========================================================================
// format: negative fraction_digits rejected
// ===========================================================================

#[test]
fn format_rejects_negative_fraction_digits() {
    let result = NumberUtils::format(
        Some(&NumberValue::Integer(42)),
        Some(1),
        Some(NumberPointType::Default),
        Some(-1),
        Some(NumberPointType::Default),
        Some(&en_us()),
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("fraction"));
}

// ===========================================================================
// format: negative min_integer_digits rejected
// ===========================================================================

#[test]
fn format_rejects_negative_min_integer_digits() {
    let result = NumberUtils::format(
        Some(&NumberValue::Integer(42)),
        Some(-1),
        Some(NumberPointType::Default),
        Some(2),
        Some(NumberPointType::Default),
        Some(&en_us()),
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("integer"));
}

// ===========================================================================
// format: missing required parameters
// ===========================================================================

#[test]
fn format_rejects_null_thousands_point_type() {
    let result = NumberUtils::format(
        Some(&NumberValue::Integer(1)),
        None,
        None,
        Some(2),
        Some(NumberPointType::Default),
        Some(&en_us()),
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Thousands"));
}

#[test]
fn format_rejects_null_locale() {
    let result = NumberUtils::format(
        Some(&NumberValue::Integer(1)),
        None,
        Some(NumberPointType::Default),
        Some(2),
        Some(NumberPointType::Default),
        None,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Locale"));
}

// ===========================================================================
// format: Double/Float passthrough path (number_as_f64 fallback)
// ===========================================================================

#[test]
fn format_double_uses_f64_path() {
    let result = NumberUtils::format(
        Some(&NumberValue::Double(3.25)),
        Some(1),
        Some(NumberPointType::None),
        Some(2),
        Some(NumberPointType::Point),
        Some(&en_us()),
    )
    .unwrap()
    .unwrap();
    let text = result.to_string_lossy();
    assert!(
        text.contains("3.25"),
        "Double format must preserve value: {text}"
    );
}

// ===========================================================================
// format: zero fraction_digits omits decimal part
// ===========================================================================

#[test]
fn format_zero_fraction_digits_omits_decimal() {
    let result = NumberUtils::format(
        Some(&NumberValue::Integer(42)),
        Some(1),
        Some(NumberPointType::None),
        Some(0),
        Some(NumberPointType::Point),
        Some(&en_us()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(result.to_string_lossy(), "42");
}

// ===========================================================================
// format: thousands grouping with Point type
// ===========================================================================

#[test]
fn format_thousands_grouping_us_locale() {
    let result = NumberUtils::format(
        Some(&NumberValue::Long(1234567)),
        Some(1),
        Some(NumberPointType::Default),
        Some(0),
        Some(NumberPointType::Default),
        Some(&en_us()),
    )
    .unwrap()
    .unwrap();
    // US: thousands = ',', decimal = '.'
    assert_eq!(result.to_string_lossy(), "1,234,567");
}

#[test]
fn format_thousands_grouping_de_locale() {
    let result = NumberUtils::format(
        Some(&NumberValue::Long(1234567)),
        Some(1),
        Some(NumberPointType::Default),
        Some(2),
        Some(NumberPointType::Default),
        Some(&de_de()),
    )
    .unwrap()
    .unwrap();
    // DE: thousands = '.', decimal = ','
    assert_eq!(result.to_string_lossy(), "1.234.567,00");
}

// ===========================================================================
// format: negative number preserves minus sign
// ===========================================================================

#[test]
fn format_negative_number_preserves_sign() {
    let result = NumberUtils::format(
        Some(&NumberValue::Integer(-42)),
        Some(1),
        Some(NumberPointType::None),
        Some(0),
        Some(NumberPointType::Point),
        Some(&en_us()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(result.to_string_lossy(), "-42");
}

// ===========================================================================
// sequence: basic ascending
// ===========================================================================

#[test]
fn sequence_ascending_basic() {
    let seq = NumberUtils::sequence(Some(1), Some(5)).unwrap();
    assert_eq!(seq, vec![1, 2, 3, 4, 5]);
}

// ===========================================================================
// sequence: basic descending
// ===========================================================================

#[test]
fn sequence_descending_basic() {
    let seq = NumberUtils::sequence(Some(5), Some(1)).unwrap();
    assert_eq!(seq, vec![5, 4, 3, 2, 1]);
}

// ===========================================================================
// sequence: from == to returns single element
// ===========================================================================

#[test]
fn sequence_single_element_when_equal() {
    let seq = NumberUtils::sequence(Some(3), Some(3)).unwrap();
    assert_eq!(seq, vec![3]);
}

// ===========================================================================
// sequence_with_step: step=0 rejected
// ===========================================================================

#[test]
fn sequence_with_step_zero_rejected() {
    let result = NumberUtils::sequence_with_step(Some(1), Some(5), Some(0));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("step"));
}

// ===========================================================================
// sequence_with_step: step direction mismatch returns empty
// ===========================================================================

#[test]
fn sequence_with_step_direction_mismatch_returns_empty() {
    // ascending range with negative step
    let seq = NumberUtils::sequence_with_step(Some(1), Some(5), Some(-1)).unwrap();
    assert!(seq.is_empty());
    // descending range with positive step
    let seq = NumberUtils::sequence_with_step(Some(5), Some(1), Some(1)).unwrap();
    assert!(seq.is_empty());
}

// ===========================================================================
// sequence_with_step: null parameters rejected
// ===========================================================================

#[test]
fn sequence_rejects_null_from() {
    let result = NumberUtils::sequence(None, Some(5));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("start"));
}

#[test]
fn sequence_rejects_null_to() {
    let result = NumberUtils::sequence(Some(1), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("up to"));
}

// ===========================================================================
// format_currency: None target returns None
// ===========================================================================

#[test]
fn format_currency_none_target_returns_none() {
    let result = NumberUtils::format_currency(None, Some(&en_us())).unwrap();
    assert_eq!(result, None);
}

#[test]
fn format_currency_rejects_null_locale() {
    let result = NumberUtils::format_currency(Some(&NumberValue::Integer(100)), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Locale"));
}

// ===========================================================================
// format_currency: JP/KR have 0 fraction digits
// ===========================================================================

#[test]
fn format_currency_jp_has_no_decimals() {
    let result = NumberUtils::format_currency(Some(&NumberValue::Integer(1000)), Some(&jp_jp()))
        .unwrap()
        .unwrap();
    let text = result.to_string_lossy();
    // JP: symbol before, no decimal digits
    assert!(text.contains('￥'), "JP currency must contain ￥: {text}");
    assert!(
        !text.contains('.'),
        "JP currency must have no decimals: {text}"
    );
}

#[test]
fn format_currency_kr_has_no_decimals() {
    let result = NumberUtils::format_currency(Some(&NumberValue::Integer(5000)), Some(&kr_kr()))
        .unwrap()
        .unwrap();
    let text = result.to_string_lossy();
    assert!(text.contains('₩'), "KR currency must contain ₩: {text}");
    assert!(
        !text.contains('.'),
        "KR currency must have no decimals: {text}"
    );
}

// ===========================================================================
// format_currency: DE locale has symbol after amount
// ===========================================================================

#[test]
fn format_currency_de_locale_symbol_after() {
    let result = NumberUtils::format_currency(Some(&NumberValue::Integer(99)), Some(&de_de()))
        .unwrap()
        .unwrap();
    let text = result.to_string_lossy();
    // DE uses decimal comma locale, symbol after
    assert!(text.contains('€'), "DE currency must contain €: {text}");
    assert!(
        text.ends_with('€'),
        "DE currency symbol must be after: {text}"
    );
}

// ===========================================================================
// format_percent: None target returns None
// ===========================================================================

#[test]
fn format_percent_none_target_returns_none() {
    let result = NumberUtils::format_percent(None, Some(1), Some(0), Some(&en_us())).unwrap();
    assert_eq!(result, None);
}

#[test]
fn format_percent_rejects_null_locale() {
    let result =
        NumberUtils::format_percent(Some(&NumberValue::Double(0.5)), Some(1), Some(0), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Locale"));
}

// ===========================================================================
// format_percent: multiplies by 100 and appends %
// ===========================================================================

#[test]
fn format_percent_multiplies_by_100() {
    let result = NumberUtils::format_percent(
        Some(&NumberValue::Double(0.75)),
        Some(1),
        Some(0),
        Some(&en_us()),
    )
    .unwrap()
    .unwrap();
    let text = result.to_string_lossy();
    assert!(text.contains("75"), "0.75 * 100 = 75: {text}");
    assert!(text.contains('%'), "must contain percent sign: {text}");
}

// ===========================================================================
// format_percent: fr locale has space before %
// ===========================================================================

#[test]
fn format_percent_fr_locale_has_space_before_percent() {
    let result = NumberUtils::format_percent(
        Some(&NumberValue::Double(0.5)),
        Some(1),
        Some(0),
        Some(&fr_fr()),
    )
    .unwrap()
    .unwrap();
    let text = result.to_string_lossy();
    // fr locale uses U+00A0 (non-breaking space) before %
    assert!(
        text.contains("\u{00A0}%"),
        "fr locale must have NBSP before %: {text}"
    );
}
