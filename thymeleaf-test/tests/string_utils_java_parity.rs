//! `StringUtils` Java Golden 差分测试。

use thymeleaf::util::{StringUtils, Utf16String};

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
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
    let values: Vec<Option<Utf16String>> = vec![];
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

// ===========================================================================
// Java StringUtilsTest 逐方法 1:1 复刻（capitalize/unCapitalize/
// capitalizeWords/substring/pack 边界与 null/空串/空白变体）
// ===========================================================================

/// 把 Option<Utf16String> 压成可比较文本（None -> "<null>"）。
fn text(value: Option<Utf16String>) -> String {
    value
        .map(|value| value.to_string_lossy())
        .unwrap_or_else(|| "<null>".to_owned())
}

#[test]
fn capitalize_all_java_variants_match() {
    // testCapitalize1-6：首字符 title-case；首字符为空白时不变；空串不变；null -> null
    assert_eq!(text(StringUtils::capitalize(Some(&js("abc")))), "Abc");
    assert_eq!(
        text(StringUtils::capitalize(Some(&js("          abc")))),
        "          abc"
    );
    assert_eq!(text(StringUtils::capitalize(Some(&js("")))), "");
    assert_eq!(text(StringUtils::capitalize(None)), "<null>");
    assert_eq!(
        text(StringUtils::capitalize(Some(&js("          Abc")))),
        "          Abc"
    );
    assert_eq!(
        text(StringUtils::capitalize(Some(&js("abc def")))),
        "Abc def"
    );
}

#[test]
fn un_capitalize_all_java_variants_match() {
    // testUnCapitalize1-6
    assert_eq!(text(StringUtils::un_capitalize(Some(&js("ABC")))), "aBC");
    assert_eq!(
        text(StringUtils::un_capitalize(Some(&js("          ABC")))),
        "          ABC"
    );
    assert_eq!(text(StringUtils::un_capitalize(Some(&js("")))), "");
    assert_eq!(text(StringUtils::un_capitalize(None)), "<null>");
    assert_eq!(
        text(StringUtils::un_capitalize(Some(&js("          Abc")))),
        "          Abc"
    );
    assert_eq!(
        text(StringUtils::un_capitalize(Some(&js("Abc Def")))),
        "abc Def"
    );
}

#[test]
fn capitalize_words_all_java_variants_match() {
    // testCapitalizeWords1-13：默认 whitespace 分隔 + 自定义分隔符 + 空串/null
    assert_eq!(text(StringUtils::capitalize_words(Some(&js("")), None)), "");
    assert_eq!(
        text(StringUtils::capitalize_words(Some(&js("   ")), None)),
        "   "
    );
    assert_eq!(
        text(StringUtils::capitalize_words(Some(&js("a")), None)),
        "A"
    );
    assert_eq!(
        text(StringUtils::capitalize_words(Some(&js("A")), None)),
        "A"
    );
    assert_eq!(
        text(StringUtils::capitalize_words(
            Some(&js("aaa bbb ccc")),
            None
        )),
        "Aaa Bbb Ccc"
    );
    assert_eq!(
        text(StringUtils::capitalize_words(
            Some(&js("aaa   bbb   ccc")),
            None
        )),
        "Aaa   Bbb   Ccc"
    );
    // testCapitalizeWords7：自定义分隔符 " "
    assert_eq!(
        text(StringUtils::capitalize_words(
            Some(&js("a.ze tyu iop")),
            Some(&js(" "))
        )),
        "A.ze Tyu Iop"
    );
    // testCapitalizeWords8/9/10：自定义分隔符 " ."
    assert_eq!(
        text(StringUtils::capitalize_words(
            Some(&js("a....ze       tyu     iop")),
            Some(&js(" ."))
        )),
        "A....Ze       Tyu     Iop"
    );
    assert_eq!(
        text(StringUtils::capitalize_words(
            Some(&js("     aaaaa....zzzzz       ttttt     nnnnn")),
            Some(&js(" ."))
        )),
        "     Aaaaa....Zzzzz       Ttttt     Nnnnn"
    );
    assert_eq!(
        text(StringUtils::capitalize_words(
            Some(&js("     aaaaa....zzzzz       ttttt     nnnnn   ")),
            Some(&js(" ."))
        )),
        "     Aaaaa....Zzzzz       Ttttt     Nnnnn   "
    );
    // testCapitalizeWords11-13：空串与 null
    assert_eq!(
        text(StringUtils::capitalize_words(
            Some(&js("")),
            Some(&js(" ."))
        )),
        ""
    );
    assert_eq!(text(StringUtils::capitalize_words(None, None)), "<null>");
    assert_eq!(
        text(StringUtils::capitalize_words(None, Some(&js(" .")))),
        "<null>"
    );
}

#[test]
fn substring_java_variants_match() {
    // testSubstring1-5：2 参 substring（Java StringUtils.substring(target, begin)）
    assert_eq!(
        text(StringUtils::substring_from(Some(&js("abcdef")), 0).unwrap()),
        "abcdef"
    );
    assert_eq!(
        text(StringUtils::substring_from(Some(&js("abcdef")), 2).unwrap()),
        "cdef"
    );
    assert!(StringUtils::substring_from(None, 2).unwrap().is_none());
    // 负索引与越界 -> Java IllegalArgumentException（Rust 错误路径）
    assert!(StringUtils::substring_from(Some(&js("abcdef")), -2).is_err());
    assert!(StringUtils::substring_from(Some(&js("abcdef")), 7).is_err());
}

#[test]
fn pack_all_java_variants_match() {
    // Java testPack 全部 13 条断言（不含 Java 独有的 assertSame 实例复用语义）
    assert_eq!(text(StringUtils::pack(None)), "<null>");
    assert_eq!(text(StringUtils::pack(Some(&js("")))), "");
    assert_eq!(text(StringUtils::pack(Some(&js(" ")))), "");
    assert_eq!(text(StringUtils::pack(Some(&js("  ")))), "");
    assert_eq!(text(StringUtils::pack(Some(&js("    \n ")))), "");
    assert_eq!(text(StringUtils::pack(Some(&js("   abc  ")))), "abc");
    assert_eq!(text(StringUtils::pack(Some(&js("   AbC  ")))), "abc");
    assert_eq!(text(StringUtils::pack(Some(&js("   a   b   c  ")))), "abc");
    assert_eq!(
        text(StringUtils::pack(Some(&js("   a   b   \nc\n  ")))),
        "abc"
    );
    assert_eq!(
        text(StringUtils::pack(Some(&js(
            "   a23   b   (\n%\t& __\nc\n  "
        )))),
        "a23b(%&__c"
    );
    assert_eq!(
        text(StringUtils::pack(Some(&js(
            "   A23   B   (\n%\t& __\nC\n  "
        )))),
        "a23b(%&__c"
    );
}
