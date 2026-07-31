//! `StringUtils` Java Golden 差分测试。

use thymeleaf::util::{JavaString, StringUtils};

fn js(s: &str) -> JavaString {
    JavaString::from_rust_str(s)
}

#[test]
fn equals_both_none() {
    assert!(StringUtils::equals(None, None));
}

#[test]
fn equals_first_none() {
    assert!(!StringUtils::equals(None, Some(&js("a"))));
}

#[test]
fn equals_same() {
    assert!(StringUtils::equals(Some(&js("hello")), Some(&js("hello"))));
}

#[test]
fn equals_different() {
    assert!(!StringUtils::equals(Some(&js("hello")), Some(&js("world"))));
}

#[test]
fn equals_ignore_case_same() {
    assert!(StringUtils::equals_ignore_case(
        Some(&js("Hello")),
        Some(&js("hello"))
    ));
}

#[test]
fn equals_ignore_case_different() {
    assert!(!StringUtils::equals_ignore_case(
        Some(&js("Hello")),
        Some(&js("World"))
    ));
}

#[test]
fn contains_found() {
    assert!(StringUtils::contains(Some(&js("hello world")), Some(&js("world"))).unwrap());
}

#[test]
fn contains_not_found() {
    assert!(!StringUtils::contains(Some(&js("hello")), Some(&js("xyz"))).unwrap());
}

#[test]
fn contains_none_target_errors() {
    assert!(StringUtils::contains(None, Some(&js("a"))).is_err());
}

#[test]
fn starts_with_match() {
    assert!(StringUtils::starts_with(Some(&js("hello world")), Some(&js("hello"))).unwrap());
}

#[test]
fn starts_with_no_match() {
    assert!(!StringUtils::starts_with(Some(&js("hello")), Some(&js("world"))).unwrap());
}

#[test]
fn ends_with_match() {
    assert!(StringUtils::ends_with(Some(&js("hello world")), Some(&js("world"))).unwrap());
}

#[test]
fn ends_with_no_match() {
    assert!(!StringUtils::ends_with(Some(&js("hello")), Some(&js("world"))).unwrap());
}

#[test]
fn is_empty_none() {
    assert!(StringUtils::is_empty(None));
}

#[test]
fn is_empty_empty_string() {
    assert!(StringUtils::is_empty(Some(&js(""))));
}

#[test]
fn is_empty_non_empty() {
    assert!(!StringUtils::is_empty(Some(&js("hello"))));
}

#[test]
fn is_empty_or_whitespace_none() {
    assert!(StringUtils::is_empty_or_whitespace(None));
}

#[test]
fn is_empty_or_whitespace_spaces() {
    assert!(StringUtils::is_empty_or_whitespace(Some(&js("   "))));
}

#[test]
fn is_empty_or_whitespace_non_empty() {
    assert!(!StringUtils::is_empty_or_whitespace(Some(&js("hello"))));
}

#[test]
fn to_string_none() {
    assert!(StringUtils::to_string(None).is_none());
}

#[test]
fn to_string_some() {
    let result = StringUtils::to_string(Some(&js("hello"))).unwrap();
    assert_eq!(result.to_string_lossy(), "hello");
}

#[test]
fn trim_basic() {
    let result = StringUtils::trim(Some(&js("  hello  "))).unwrap();
    assert_eq!(result.to_string_lossy(), "hello");
}

#[test]
fn trim_none() {
    assert!(StringUtils::trim(None).is_none());
}

#[test]
fn repeat_basic() {
    let result = StringUtils::repeat(Some(&js("ab")), 3).unwrap();
    assert_eq!(result.to_string_lossy(), "ababab");
}

#[test]
fn repeat_zero() {
    let result = StringUtils::repeat(Some(&js("ab")), 0).unwrap();
    assert_eq!(result.to_string_lossy(), "");
}

#[test]
fn repeat_none() {
    assert!(StringUtils::repeat(None, 3).is_none());
}

#[test]
fn concat_basic() {
    let values = vec![Some(js("a")), Some(js("b")), Some(js("c"))];
    let result = StringUtils::concat(Some(&values));
    assert_eq!(result.to_string_lossy(), "abc");
}

#[test]
fn concat_with_nulls() {
    let values = vec![Some(js("a")), None, Some(js("c"))];
    let result = StringUtils::concat(Some(&values));
    assert_eq!(result.to_string_lossy(), "ac");
}

#[test]
fn concat_empty() {
    let values: Vec<Option<JavaString>> = vec![];
    let result = StringUtils::concat(Some(&values));
    assert_eq!(result.to_string_lossy(), "");
}

#[test]
fn length_basic() {
    assert_eq!(StringUtils::length(Some(&js("hello"))).unwrap(), 5);
}

#[test]
fn length_empty() {
    assert_eq!(StringUtils::length(Some(&js(""))).unwrap(), 0);
}

#[test]
fn length_none() {
    assert!(StringUtils::length(None).is_err());
}

#[test]
fn capitalize_basic() {
    let result = StringUtils::capitalize(Some(&js("hello"))).unwrap();
    assert_eq!(result.to_string_lossy(), "Hello");
}

#[test]
fn un_capitalize_basic() {
    let result = StringUtils::un_capitalize(Some(&js("Hello"))).unwrap();
    assert_eq!(result.to_string_lossy(), "hello");
}

#[test]
fn escape_xml_basic() {
    let result = StringUtils::escape_xml(Some(&js("<b>bold</b>"))).unwrap();
    assert_eq!(result.to_string_lossy(), "&lt;b&gt;bold&lt;/b&gt;");
}

#[test]
fn escape_xml_ampersand() {
    let result = StringUtils::escape_xml(Some(&js("a&b"))).unwrap();
    assert_eq!(result.to_string_lossy(), "a&amp;b");
}

#[test]
fn random_alphanumeric_length() {
    let result = StringUtils::random_alphanumeric(10);
    assert_eq!(result.to_string_lossy().len(), 10);
}

#[test]
fn random_alphanumeric_zero() {
    let result = StringUtils::random_alphanumeric(0);
    assert_eq!(result.to_string_lossy().len(), 0);
}

#[test]
fn pack_removes_whitespace_and_lowercases() {
    let result = StringUtils::pack(Some(&js("  Hello   World  "))).unwrap();
    assert_eq!(result.to_string_lossy(), "helloworld");
}

#[test]
fn pack_none() {
    assert!(StringUtils::pack(None).is_none());
}
