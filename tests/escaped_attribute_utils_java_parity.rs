//! `EscapedAttributeUtils` Java Golden 差分测试。
//!
//! 覆盖：HTML/XML 属性转义、TEXT/HTML/XML/JS/CSS/RAW 反转义、
//! null 输入与错误路径。

use thymeleaf::TemplateMode;
use thymeleaf::util::{EscapedAttributeUtils, JavaString};

fn js(s: &str) -> JavaString {
    JavaString::from_rust_str(s)
}

// ===========================================================================
// 1. escape_attribute：HTML
// ===========================================================================

#[test]
fn escape_html_basic() {
    let result =
        EscapedAttributeUtils::escape_attribute(Some(TemplateMode::HTML), Some(&js("a<b>c")))
            .unwrap()
            .unwrap();
    assert_eq!(result.to_string_lossy(), "a&lt;b&gt;c");
}

#[test]
fn escape_html_ampersand() {
    let result =
        EscapedAttributeUtils::escape_attribute(Some(TemplateMode::HTML), Some(&js("a&b")))
            .unwrap()
            .unwrap();
    assert_eq!(result.to_string_lossy(), "a&amp;b");
}

#[test]
fn escape_html_quotes() {
    let result = EscapedAttributeUtils::escape_attribute(
        Some(TemplateMode::HTML),
        Some(&js("say \"hi\" and 'bye'")),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        result.to_string_lossy(),
        "say &quot;hi&quot; and &#39;bye&#39;"
    );
}

#[test]
fn escape_html_plain_passthrough() {
    let result =
        EscapedAttributeUtils::escape_attribute(Some(TemplateMode::HTML), Some(&js("plain text")))
            .unwrap()
            .unwrap();
    assert_eq!(result.to_string_lossy(), "plain text");
}

#[test]
fn escape_html_empty() {
    let result = EscapedAttributeUtils::escape_attribute(Some(TemplateMode::HTML), Some(&js("")))
        .unwrap()
        .unwrap();
    assert_eq!(result.to_string_lossy(), "");
}

#[test]
fn escape_html_null_input() {
    assert!(
        EscapedAttributeUtils::escape_attribute(Some(TemplateMode::HTML), None)
            .unwrap()
            .is_none()
    );
}

// ===========================================================================
// 2. escape_attribute：XML
// ===========================================================================

#[test]
fn escape_xml_basic() {
    let result =
        EscapedAttributeUtils::escape_attribute(Some(TemplateMode::XML), Some(&js("a<b>c")))
            .unwrap()
            .unwrap();
    assert_eq!(result.to_string_lossy(), "a&lt;b&gt;c");
}

#[test]
fn escape_xml_apos() {
    let result =
        EscapedAttributeUtils::escape_attribute(Some(TemplateMode::XML), Some(&js("it's")))
            .unwrap()
            .unwrap();
    assert_eq!(result.to_string_lossy(), "it&apos;s");
}

#[test]
fn escape_xml_control_chars_removed() {
    // XML 1.0 非法控制字符被丢弃
    let result =
        EscapedAttributeUtils::escape_attribute(Some(TemplateMode::XML), Some(&js("a\u{0001}b")))
            .unwrap()
            .unwrap();
    assert_eq!(result.to_string_lossy(), "ab");
}

// ===========================================================================
// 3. escape_attribute：错误路径
// ===========================================================================

#[test]
fn escape_html_null_mode_errors() {
    assert!(EscapedAttributeUtils::escape_attribute(None, Some(&js("abc"))).is_err());
}

#[test]
fn escape_text_mode_errors() {
    assert!(
        EscapedAttributeUtils::escape_attribute(Some(TemplateMode::TEXT), Some(&js("abc")))
            .is_err()
    );
}

#[test]
fn escape_javascript_mode_errors() {
    assert!(
        EscapedAttributeUtils::escape_attribute(Some(TemplateMode::JAVASCRIPT), Some(&js("abc")))
            .is_err()
    );
}

// ===========================================================================
// 4. unescape_attribute：TEXT/HTML
// ===========================================================================

#[test]
fn unescape_html_basic() {
    let result =
        EscapedAttributeUtils::unescape_attribute(Some(TemplateMode::HTML), Some(&js("a&amp;b")))
            .unwrap()
            .unwrap();
    assert_eq!(result.to_string_lossy(), "a&b");
}

#[test]
fn unescape_html_lt() {
    let result = EscapedAttributeUtils::unescape_attribute(
        Some(TemplateMode::HTML),
        Some(&js("&lt;tag&gt;")),
    )
    .unwrap()
    .unwrap();
    assert_eq!(result.to_string_lossy(), "<tag>");
}

#[test]
fn unescape_html_numeric() {
    let result =
        EscapedAttributeUtils::unescape_attribute(Some(TemplateMode::TEXT), Some(&js("&#65;")))
            .unwrap()
            .unwrap();
    assert_eq!(result.to_string_lossy(), "A");
}

#[test]
fn unescape_plain_passthrough() {
    let result =
        EscapedAttributeUtils::unescape_attribute(Some(TemplateMode::HTML), Some(&js("plain")))
            .unwrap()
            .unwrap();
    assert_eq!(result.to_string_lossy(), "plain");
}

// ===========================================================================
// 5. unescape_attribute：XML
// ===========================================================================

#[test]
fn unescape_xml_basic() {
    let result =
        EscapedAttributeUtils::unescape_attribute(Some(TemplateMode::XML), Some(&js("a&amp;b")))
            .unwrap()
            .unwrap();
    assert_eq!(result.to_string_lossy(), "a&b");
}

// ===========================================================================
// 6. unescape_attribute：RAW 原样返回
// ===========================================================================

#[test]
fn unescape_raw_identity() {
    let input = js("a&amp;b&lt;c");
    let result = EscapedAttributeUtils::unescape_attribute(Some(TemplateMode::RAW), Some(&input))
        .unwrap()
        .unwrap();
    assert_eq!(result.to_string_lossy(), "a&amp;b&lt;c");
}

// ===========================================================================
// 7. unescape_attribute：null 与错误
// ===========================================================================

#[test]
fn unescape_null_input() {
    assert!(
        EscapedAttributeUtils::unescape_attribute(Some(TemplateMode::HTML), None)
            .unwrap()
            .is_none()
    );
}

#[test]
fn unescape_null_mode_errors() {
    assert!(EscapedAttributeUtils::unescape_attribute(None, Some(&js("abc"))).is_err());
}

// ===========================================================================
// 8. Unicode 保留
// ===========================================================================

#[test]
fn escape_html_unicode_preserved() {
    let result =
        EscapedAttributeUtils::escape_attribute(Some(TemplateMode::HTML), Some(&js("日本語")))
            .unwrap()
            .unwrap();
    assert_eq!(result.to_string_lossy(), "日本語");
}

#[test]
fn escape_xml_unicode_to_hex_references() {
    // Java unbescape 将非 ASCII 码点转义为 &#x...; 十六进制引用
    let result =
        EscapedAttributeUtils::escape_attribute(Some(TemplateMode::XML), Some(&js("こんにちは")))
            .unwrap()
            .unwrap();
    assert_eq!(
        result.to_string_lossy(),
        "&#x3053;&#x3093;&#x306b;&#x3061;&#x306f;"
    );
}
