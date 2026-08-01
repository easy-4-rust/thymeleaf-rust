//! `AggregateCharSequence`/`LinkExpression` Java Golden 差分测试。
//!
//! 覆盖：`AggregateCharSequence` 多段字符序列聚合（char_at/subsequence/
//! hash/equals/unicode）、`LinkExpression` 构造校验。
//! `@{}` 链接端到端见 `link_expression_web_java_parity.rs`（Web 上下文）。

use std::sync::Arc;

use thymeleaf::expression::LinkExpression;
use thymeleaf::util::{AggregateCharSequence, JavaString};

fn js(s: &str) -> JavaString {
    JavaString::from_rust_str(s)
}

fn two_parts(a: &str, b: &str) -> AggregateCharSequence {
    AggregateCharSequence::from_components(Some(vec![Some(Arc::new(js(a))), Some(Arc::new(js(b)))]))
        .expect("valid")
}

// ===========================================================================
// 1. AggregateCharSequence 聚合
// ===========================================================================

#[test]
fn aggregate_from_components_concatenates() {
    let seq = AggregateCharSequence::from_components(Some(vec![
        Some(Arc::new(js("Hello"))),
        Some(Arc::new(js(" "))),
        Some(Arc::new(js("World"))),
    ]))
    .expect("valid");
    let text = seq.to_java_string().unwrap();
    assert_eq!(text.to_string_lossy(), "Hello World");
}

#[test]
fn aggregate_char_at() {
    let seq = two_parts("ab", "cd");
    assert_eq!(seq.char_at(0).unwrap(), b'a' as u16);
    assert_eq!(seq.char_at(1).unwrap(), b'b' as u16);
    assert_eq!(seq.char_at(2).unwrap(), b'c' as u16);
    assert_eq!(seq.char_at(3).unwrap(), b'd' as u16);
}

#[test]
fn aggregate_char_at_out_of_bounds() {
    let seq = AggregateCharSequence::from_components(Some(vec![Some(Arc::new(js("ab")))]))
        .expect("valid");
    assert!(seq.char_at(5).is_err(), "out of bounds must fail");
    assert!(seq.char_at(-1).is_err(), "negative index must fail");
}

#[test]
fn aggregate_sub_sequence() {
    let seq = two_parts("Hello", "World");
    let sub = seq.sub_sequence(1, 6).expect("valid range");
    assert_eq!(sub.to_string_lossy(), "elloW");
}

#[test]
fn aggregate_sub_sequence_full() {
    let seq = AggregateCharSequence::from_components(Some(vec![Some(Arc::new(js("abc")))]))
        .expect("valid");
    let sub = seq.sub_sequence(0, 3).expect("valid range");
    assert_eq!(sub.to_string_lossy(), "abc");
}

#[test]
fn aggregate_content_equals() {
    let seq = two_parts("Hello", "World");
    let other = js("HelloWorld");
    assert!(
        seq.content_equals(&other).expect("compare"),
        "aggregate must equal concatenated string"
    );
}

#[test]
fn aggregate_content_not_equals() {
    let seq = two_parts("Hello", "World");
    let other = js("HelloRust");
    assert!(
        !seq.content_equals(&other).expect("compare"),
        "different strings must not be equal"
    );
}

#[test]
fn aggregate_hash_code() {
    let seq = two_parts("ab", "cd");
    let hash = seq.hash_code().expect("hash");
    // Java String.hashCode: s[0]*31^(n-1) + ... + s[n-1]
    let expected = "abcd"
        .encode_utf16()
        .fold(0i64, |acc, unit| (acc * 31 + i64::from(unit)) & 0xFFFF_FFFF);
    assert_eq!(i64::from(hash) & 0xFFFF_FFFF, expected, "Java hashCode");
}

#[test]
fn aggregate_from_five() {
    let seq = AggregateCharSequence::from_five(
        Some(Arc::new(js("a"))),
        Some(Arc::new(js("b"))),
        Some(Arc::new(js("c"))),
        Some(Arc::new(js("d"))),
        Some(Arc::new(js("e"))),
    )
    .expect("valid");
    assert_eq!(seq.to_java_string().unwrap().to_string_lossy(), "abcde");
}

#[test]
fn aggregate_from_one() {
    let seq = AggregateCharSequence::from_one(Some(Arc::new(js("solo")))).expect("valid");
    assert_eq!(seq.to_java_string().unwrap().to_string_lossy(), "solo");
}

#[test]
fn aggregate_unicode_preserved() {
    let seq = two_parts("日本語", "テスト");
    assert_eq!(
        seq.to_java_string().unwrap().to_string_lossy(),
        "日本語テスト"
    );
}

#[test]
fn aggregate_equals_java() {
    let a = two_parts("ab", "cd");
    let b =
        AggregateCharSequence::from_components(Some(vec![Some(Arc::new(js("abcd")))])).expect("b");
    assert!(a.equals_java(&b).expect("equals"), "equal content");
}

#[test]
fn aggregate_null_component_errors() {
    assert!(
        AggregateCharSequence::from_components(Some(vec![None])).is_err(),
        "null contained component must fail"
    );
}

#[test]
fn aggregate_length() {
    let seq = two_parts("ab", "cde");
    assert_eq!(seq.length(), 5, "total UTF-16 length");
}

// ===========================================================================
// 2. LinkExpression 构造校验
// ===========================================================================

#[test]
fn link_expression_null_base_errors() {
    let result = LinkExpression::new(None, None);
    assert!(result.is_err(), "null base must be rejected");
}
