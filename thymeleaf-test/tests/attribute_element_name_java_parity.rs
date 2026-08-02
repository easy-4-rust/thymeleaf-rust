//! `AttributeNames`/`ElementNames` Java 1:1 差分测试。
//!
//! 逐一对应上游 `thymeleaf-tests-core` 的：
//! - AttributeNamesTest（HTML/XML buffer 与 string 四种入口、
//!   大小写规则、缓存同一性、非法参数）
//! - ElementNamesTest（同上）
//!
//! Java `assertSame` 对应 Rust repository 缓存同一性
//! （`Arc::ptr_eq`）。

use std::sync::Arc;

use thymeleaf::engine::{
    AttributeNameValue, AttributeNames, ElementNameValue, ElementNames, TextAttributeName,
    TextElementName,
};
use thymeleaf::util::JavaString;

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn as_utf16(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

/// Java `AttributeName#toString()`（`{n1,n2}` 格式）。
fn attribute_to_string(value: &AttributeNameValue) -> String {
    value
        .as_attribute_name()
        .to_java_string()
        .expect("attribute name to string")
        .to_string_lossy()
}

/// `TextAttributeName#toString()`（TEXT 模式 string 入口返回 Arc<TextAttributeName>）。
fn text_attribute_to_string(value: &Arc<TextAttributeName>) -> String {
    value
        .as_attribute_name()
        .to_java_string()
        .expect("text attribute name to string")
        .to_string_lossy()
}

/// `TextElementName#toString()`。
fn text_element_to_string(value: &Arc<TextElementName>) -> String {
    value
        .as_element_name()
        .to_java_string()
        .expect("text element name to string")
        .to_string_lossy()
}

/// Java `ElementName#toString()`（`{n1,n2}` 格式）。
fn element_to_string(value: &ElementNameValue) -> String {
    value
        .as_element_name()
        .to_java_string()
        .expect("element name to string")
        .to_string_lossy()
}

// ===========================================================================
// 1. AttributeNamesTest#testHTMLBuffer
// ===========================================================================

#[test]
fn attribute_names_html_buffer() {
    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("data-something")),
        0,
        "data-something".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(attribute_to_string(&name), "{data-something}");
    assert!(name.as_attribute_name().get_prefix().is_none());

    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("th:something")),
        0,
        "th:something".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(
        attribute_to_string(&name),
        "{th:something,data-th-something}"
    );

    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("something")),
        0,
        "something".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(attribute_to_string(&name), "{something}");

    // buffer 切片：offset/len 子范围
    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("absomethingba")),
        2,
        "something".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(attribute_to_string(&name), "{something}");

    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("abcdefghijkliklmnsomethingba")),
        17,
        "something".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(attribute_to_string(&name), "{something}");

    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("abcdefghijkliklmnth:somethingba")),
        17,
        "th:something".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(
        attribute_to_string(&name),
        "{th:something,data-th-something}"
    );

    // HTML 大小写不敏感
    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("SOMETHING")),
        0,
        "SOMETHING".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(attribute_to_string(&name), "{something}");

    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("TH:SOMETHING")),
        0,
        "TH:SOMETHING".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(
        attribute_to_string(&name),
        "{th:something,data-th-something}"
    );

    // 空前缀 ":something"
    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16(":something")),
        0,
        ":something".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(attribute_to_string(&name), "{:something}");

    // data-th-something → th:something（HTML5 形式折叠）
    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("data-th-something")),
        0,
        "data-th-something".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(
        attribute_to_string(&name),
        "{th:something,data-th-something}"
    );

    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("data-something")),
        0,
        "data-something".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(attribute_to_string(&name), "{data-something}");

    // xml: 前缀保留原样
    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("xml:ns")),
        0,
        "xml:ns".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(attribute_to_string(&name), "{xml:ns}");

    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("xml:space")),
        0,
        "xml:space".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(attribute_to_string(&name), "{xml:space}");

    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("XML:SPACE")),
        0,
        "XML:SPACE".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(attribute_to_string(&name), "{xml:space}");

    // xmlns:th 不是 prefixed
    let name = AttributeNames::for_name_buffer(
        Some(thymeleaf::TemplateMode::HTML),
        Some(&as_utf16("xmlns:th")),
        0,
        "xmlns:th".len() as i32,
    )
    .expect("html attribute name");
    assert_eq!(attribute_to_string(&name), "{xmlns:th}");
    assert!(!name.as_attribute_name().is_prefixed());
}

#[test]
fn attribute_names_html_buffer_cache_identity() {
    let html = thymeleaf::TemplateMode::HTML;
    let first = AttributeNames::for_html_name(Some(&js("data-something"))).expect("name");
    let second = AttributeNames::for_html_name(Some(&js("data-something"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "data-something cached");

    let first = AttributeNames::for_html_name(Some(&js("xmlns:th"))).expect("name");
    let second = AttributeNames::for_html_name(Some(&js("xmlns:th"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "xmlns:th cached");

    let first = AttributeNames::for_html_name(Some(&js("data-th-something"))).expect("name");
    let second = AttributeNames::for_html_name(Some(&js("data-th-something"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "data-th-something cached");

    let first = AttributeNames::for_html_name(Some(&js("data-th-something"))).expect("name");
    let second = AttributeNames::for_html_name(Some(&js("DATA-TH-SOMETHING"))).expect("name");
    assert!(
        Arc::ptr_eq(&first, &second),
        "case-insensitive cache hit for data-th-something"
    );
    let _ = html;
}

#[test]
fn attribute_names_html_buffer_errors() {
    let html = Some(thymeleaf::TemplateMode::HTML);
    // Java: forHTMLName(null, 0, 0) → IllegalArgumentException
    assert!(
        AttributeNames::for_name_buffer(html, None, 0, 0).is_err(),
        "null buffer rejected"
    );
    // Java: forHTMLName("".toCharArray(), 0, 0) → IllegalArgumentException
    assert!(
        AttributeNames::for_name_buffer(html, Some(&[]), 0, 0).is_err(),
        "empty name rejected"
    );
    // Java: forHTMLName(" ".toCharArray(), 0, 1) → IllegalArgumentException
    assert!(
        AttributeNames::for_name_buffer(html, Some(&as_utf16(" ")), 0, 1).is_err(),
        "blank name rejected"
    );
}

// ===========================================================================
// 2. AttributeNamesTest#testHTMLString / 前缀形式
// ===========================================================================

#[test]
fn attribute_names_html_string() {
    let name = AttributeNames::for_html_name(Some(&js("data-something"))).expect("name");
    assert_eq!(
        name.as_attribute_name()
            .to_java_string()
            .expect("toString")
            .to_string_lossy(),
        "{data-something}"
    );
    assert!(name.as_attribute_name().get_prefix().is_none());

    let name = AttributeNames::for_html_name(Some(&js("th:something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name.clone())),
        "{th:something,data-th-something}"
    );

    let name = AttributeNames::for_html_name(Some(&js("something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name)),
        "{something}"
    );

    let name = AttributeNames::for_html_name(Some(&js("SOMETHING"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name)),
        "{something}"
    );

    let name = AttributeNames::for_html_name(Some(&js("TH:SOMETHING"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name)),
        "{th:something,data-th-something}"
    );

    let name = AttributeNames::for_html_name(Some(&js(":something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name.clone())),
        "{:something}"
    );
    assert!(!name.as_attribute_name().is_prefixed());

    let name = AttributeNames::for_html_name(Some(&js("data-th-something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name)),
        "{th:something,data-th-something}"
    );

    let name = AttributeNames::for_html_name(Some(&js("data-something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name)),
        "{data-something}"
    );

    let name = AttributeNames::for_html_name(Some(&js("xml:ns"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name)),
        "{xml:ns}"
    );

    let name = AttributeNames::for_html_name(Some(&js("xml:space"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name)),
        "{xml:space}"
    );

    let name = AttributeNames::for_html_name(Some(&js("XML:SPACE"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name)),
        "{xml:space}"
    );

    let name = AttributeNames::for_html_name(Some(&js("xmlns:th"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name.clone())),
        "{xmlns:th}"
    );
    assert!(!name.as_attribute_name().is_prefixed());

    // 显式 prefix 形式
    let name = AttributeNames::for_html_name_with_prefix(Some(&js("th")), Some(&js("something")))
        .expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name)),
        "{th:something,data-th-something}"
    );
}

#[test]
fn attribute_names_html_cache_aliases() {
    // Java assertSame 系列：data-th-something 与 th:something 同一实例
    let th = AttributeNames::for_html_name(Some(&js("th:something"))).expect("name");
    let data = AttributeNames::for_html_name(Some(&js("data-th-something"))).expect("name");
    assert!(
        Arc::ptr_eq(&th, &data),
        "data-th-something aliases th:something"
    );

    let th = AttributeNames::for_html_name(Some(&js("th:something"))).expect("name");
    let lower = AttributeNames::for_html_name(Some(&js("TH:SOMETHING"))).expect("name");
    assert!(
        Arc::ptr_eq(&th, &lower),
        "TH:SOMETHING aliases th:something"
    );

    // 空/空白/null 前缀折叠到无前缀
    let plain = AttributeNames::for_html_name(Some(&js("something"))).expect("name");
    let empty = AttributeNames::for_html_name_with_prefix(Some(&js("")), Some(&js("something")))
        .expect("name");
    assert!(
        Arc::ptr_eq(&plain, &empty),
        "empty prefix aliases no prefix"
    );
    let null_prefix =
        AttributeNames::for_html_name_with_prefix(None, Some(&js("something"))).expect("name");
    assert!(
        Arc::ptr_eq(&plain, &null_prefix),
        "null prefix aliases no prefix"
    );
    let blank = AttributeNames::for_html_name_with_prefix(Some(&js("  ")), Some(&js("SOMETHING")))
        .expect("name");
    assert!(
        Arc::ptr_eq(&plain, &blank),
        "blank prefix aliases no prefix"
    );

    let th = AttributeNames::for_html_name_with_prefix(Some(&js("xmlns")), Some(&js("th")))
        .expect("name");
    let ns = AttributeNames::for_html_name(Some(&js("XMLNS:TH"))).expect("name");
    assert!(Arc::ptr_eq(&th, &ns), "XMLNS:TH aliases xmlns:th");
}

#[test]
fn attribute_names_html_string_errors() {
    // Java: forHTMLName(null)/("")/("t","")/(" ")/("t"," ") → IllegalArgumentException
    assert!(
        AttributeNames::for_html_name(None).is_err(),
        "null name rejected"
    );
    assert!(
        AttributeNames::for_html_name(Some(&js(""))).is_err(),
        "empty name rejected"
    );
    assert!(
        AttributeNames::for_html_name_with_prefix(Some(&js("t")), Some(&js(""))).is_err(),
        "empty local name rejected"
    );
    assert!(
        AttributeNames::for_html_name(Some(&js(" "))).is_err(),
        "blank name rejected"
    );
    assert!(
        AttributeNames::for_html_name_with_prefix(Some(&js("t")), Some(&js(" "))).is_err(),
        "blank local name rejected"
    );
}

// ===========================================================================
// 3. AttributeNamesTest#testXMLBuffer / testXMLString
// ===========================================================================

#[test]
fn attribute_names_xml_buffer_and_string() {
    let xml = Some(thymeleaf::TemplateMode::XML);
    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16("th:something")),
        0,
        "th:something".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{th:something}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "th"
    );

    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16("something")),
        0,
        "something".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{something}");

    // buffer 切片
    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16("abcdefghijkliklmnsomethingba")),
        17,
        "something".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{something}");

    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16("abcdefghijkliklmnth:somethingba")),
        17,
        "th:something".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{th:something}");

    // XML 大小写敏感
    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16("SOMETHING")),
        0,
        "SOMETHING".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{SOMETHING}");

    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16("TH:SOMETHING")),
        0,
        "TH:SOMETHING".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{TH:SOMETHING}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "TH"
    );

    // 空前缀
    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16(":something")),
        0,
        ":something".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{:something}");
    assert!(!name.as_attribute_name().is_prefixed());

    // data- 形式不折叠
    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16("data-th-something")),
        0,
        "data-th-something".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{data-th-something}");
    assert!(!name.as_attribute_name().is_prefixed());

    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16("data-something")),
        0,
        "data-something".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{data-something}");

    let name =
        AttributeNames::for_name_buffer(xml, Some(&as_utf16("xml:ns")), 0, "xml:ns".len() as i32)
            .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{xml:ns}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "xml"
    );

    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16("xml:space")),
        0,
        "xml:space".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{xml:space}");

    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16("XML:SPACE")),
        0,
        "XML:SPACE".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{XML:SPACE}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "XML"
    );

    let name = AttributeNames::for_name_buffer(
        xml,
        Some(&as_utf16("xmlns:th")),
        0,
        "xmlns:th".len() as i32,
    )
    .expect("xml attribute name");
    assert_eq!(attribute_to_string(&name), "{xmlns:th}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "xmlns"
    );

    // string 形式 + 缓存同一性
    let first = AttributeNames::for_xml_name(Some(&js("data-th-something"))).expect("name");
    let second = AttributeNames::for_xml_name(Some(&js("data-th-something"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "xml name cached");

    // XML 大小写敏感：不同实例
    let first = AttributeNames::for_xml_name(Some(&js("data-th-something"))).expect("name");
    let second = AttributeNames::for_xml_name(Some(&js("DATA-TH-SOMETHING"))).expect("name");
    assert!(
        !Arc::ptr_eq(&first, &second),
        "xml names are case-sensitive"
    );

    // null 前缀 string 形式
    let name = AttributeNames::for_xml_name(Some(&js("th:something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name)),
        "{th:something}"
    );
}

#[test]
fn attribute_names_xml_errors() {
    let xml = Some(thymeleaf::TemplateMode::XML);
    assert!(
        AttributeNames::for_name_buffer(xml, None, 0, 0).is_err(),
        "null buffer rejected"
    );
    assert!(
        AttributeNames::for_name_buffer(xml, Some(&[]), 0, 0).is_err(),
        "empty name rejected"
    );
    assert!(
        AttributeNames::for_name_buffer(xml, Some(&as_utf16(" ")), 0, 1).is_err(),
        "blank name rejected"
    );
}

// ===========================================================================
// 4. ElementNamesTest#testHTMLBuffer / testHTMLString
// ===========================================================================

#[test]
fn element_names_html_buffer_and_string() {
    let html = Some(thymeleaf::TemplateMode::HTML);
    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16("th:something")),
        0,
        "th:something".len() as i32,
    )
    .expect("html element name");
    assert_eq!(element_to_string(&name), "{th:something,th-something}");

    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16("something")),
        0,
        "something".len() as i32,
    )
    .expect("html element name");
    assert_eq!(element_to_string(&name), "{something}");

    // buffer 切片
    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16("abcdefghijkliklmnsomething")),
        17,
        "something".len() as i32,
    )
    .expect("html element name");
    assert_eq!(element_to_string(&name), "{something}");

    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16("abcdefghijkliklmnth:something")),
        17,
        "th:something".len() as i32,
    )
    .expect("html element name");
    assert_eq!(element_to_string(&name), "{th:something,th-something}");

    // 大小写不敏感
    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16("SOMETHING")),
        0,
        "SOMETHING".len() as i32,
    )
    .expect("html element name");
    assert_eq!(element_to_string(&name), "{something}");

    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16("TH:SOMETHING")),
        0,
        "TH:SOMETHING".len() as i32,
    )
    .expect("html element name");
    assert_eq!(element_to_string(&name), "{th:something,th-something}");

    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16(":something")),
        0,
        ":something".len() as i32,
    )
    .expect("html element name");
    assert_eq!(element_to_string(&name), "{:something}");

    // 元素 data-th-something 不折叠为 th
    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16("data-th-something")),
        0,
        "data-th-something".len() as i32,
    )
    .expect("html element name");
    assert_eq!(
        element_to_string(&name),
        "{data:th-something,data-th-something}"
    );

    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16("data-something")),
        0,
        "data-something".len() as i32,
    )
    .expect("html element name");
    assert_eq!(element_to_string(&name), "{data:something,data-something}");

    let name =
        ElementNames::for_name_buffer(html, Some(&as_utf16("xml:ns")), 0, "xml:ns".len() as i32)
            .expect("html element name");
    assert_eq!(element_to_string(&name), "{xml:ns}");

    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16("xml:space")),
        0,
        "xml:space".len() as i32,
    )
    .expect("html element name");
    assert_eq!(element_to_string(&name), "{xml:space}");

    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16("XML:SPACE")),
        0,
        "XML:SPACE".len() as i32,
    )
    .expect("html element name");
    assert_eq!(element_to_string(&name), "{xml:space}");

    let name = ElementNames::for_name_buffer(
        html,
        Some(&as_utf16("xmlns:th")),
        0,
        "xmlns:th".len() as i32,
    )
    .expect("html element name");
    assert_eq!(element_to_string(&name), "{xmlns:th}");

    // string 形式
    let name = ElementNames::for_html_name(Some(&js("th:something"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Html(name)),
        "{th:something,th-something}"
    );

    let name = ElementNames::for_html_name(Some(&js("something"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Html(name)),
        "{something}"
    );

    // 显式 prefix
    let name = ElementNames::for_html_name_with_prefix(Some(&js("th")), Some(&js("something")))
        .expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Html(name)),
        "{th:something,th-something}"
    );
}

#[test]
fn element_names_html_cache_aliases() {
    let first = ElementNames::for_html_name(Some(&js("data-something"))).expect("name");
    let second = ElementNames::for_html_name(Some(&js("data-something"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "data-something cached");

    let first = ElementNames::for_html_name(Some(&js("xmlns:th"))).expect("name");
    let second = ElementNames::for_html_name(Some(&js("xmlns:th"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "xmlns:th cached");

    let first = ElementNames::for_html_name(Some(&js("data-th-something"))).expect("name");
    let second = ElementNames::for_html_name(Some(&js("data-th-something"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "data-th-something cached");

    let first = ElementNames::for_html_name(Some(&js("data-th-something"))).expect("name");
    let second = ElementNames::for_html_name(Some(&js("DATA-TH-SOMETHING"))).expect("name");
    assert!(
        Arc::ptr_eq(&first, &second),
        "html element names case-insensitive"
    );

    // Java: assertNotSame(data-th-something, th:something)
    let first = ElementNames::for_html_name(Some(&js("data-th-something"))).expect("name");
    let second = ElementNames::for_html_name(Some(&js("th:something"))).expect("name");
    assert!(
        !Arc::ptr_eq(&first, &second),
        "data-th-something is NOT th:something for elements"
    );

    // Java: assertSame(th-something, th:something)
    let first = ElementNames::for_html_name(Some(&js("th-something"))).expect("name");
    let second = ElementNames::for_html_name(Some(&js("th:something"))).expect("name");
    assert!(
        Arc::ptr_eq(&first, &second),
        "th-something aliases th:something"
    );

    // 空/空白/null 前缀折叠
    let plain = ElementNames::for_html_name(Some(&js("something"))).expect("name");
    let empty = ElementNames::for_html_name_with_prefix(Some(&js("")), Some(&js("something")))
        .expect("name");
    assert!(
        Arc::ptr_eq(&plain, &empty),
        "empty prefix aliases no prefix"
    );
    let null_prefix =
        ElementNames::for_html_name_with_prefix(None, Some(&js("something"))).expect("name");
    assert!(
        Arc::ptr_eq(&plain, &null_prefix),
        "null prefix aliases no prefix"
    );
    let blank = ElementNames::for_html_name_with_prefix(Some(&js("  ")), Some(&js("something")))
        .expect("name");
    assert!(
        Arc::ptr_eq(&plain, &blank),
        "blank prefix aliases no prefix"
    );
}

#[test]
fn element_names_html_errors() {
    let html = Some(thymeleaf::TemplateMode::HTML);
    assert!(
        ElementNames::for_name_buffer(html, None, 0, 0).is_err(),
        "null buffer rejected"
    );
    assert!(
        ElementNames::for_name_buffer(html, Some(&[]), 0, 0).is_err(),
        "empty name rejected"
    );
    assert!(
        ElementNames::for_name_buffer(html, Some(&as_utf16(" ")), 0, 1).is_err(),
        "blank name rejected"
    );
}

// ===========================================================================
// 5. ElementNamesTest#testXMLBuffer / testXMLString
// ===========================================================================

#[test]
fn element_names_xml_buffer_and_string() {
    let xml = Some(thymeleaf::TemplateMode::XML);
    let name = ElementNames::for_name_buffer(
        xml,
        Some(&as_utf16("th:something")),
        0,
        "th:something".len() as i32,
    )
    .expect("xml element name");
    assert_eq!(element_to_string(&name), "{th:something}");
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "th"
    );

    let name = ElementNames::for_name_buffer(
        xml,
        Some(&as_utf16("something")),
        0,
        "something".len() as i32,
    )
    .expect("xml element name");
    assert_eq!(element_to_string(&name), "{something}");
    assert!(name.as_element_name().get_prefix().is_none());

    // 大小写敏感
    let name = ElementNames::for_name_buffer(
        xml,
        Some(&as_utf16("SOMETHING")),
        0,
        "SOMETHING".len() as i32,
    )
    .expect("xml element name");
    assert_eq!(element_to_string(&name), "{SOMETHING}");

    let name = ElementNames::for_name_buffer(
        xml,
        Some(&as_utf16("TH:SOMETHING")),
        0,
        "TH:SOMETHING".len() as i32,
    )
    .expect("xml element name");
    assert_eq!(element_to_string(&name), "{TH:SOMETHING}");
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "TH"
    );

    // string 形式
    let name = ElementNames::for_xml_name(Some(&js("th:something"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name)),
        "{th:something}"
    );

    let name = ElementNames::for_xml_name(Some(&js("SOMETHING"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name)),
        "{SOMETHING}"
    );

    // 缓存同一性（大小写敏感）
    let first = ElementNames::for_xml_name(Some(&js("data-something"))).expect("name");
    let second = ElementNames::for_xml_name(Some(&js("data-something"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "xml element name cached");

    let first = ElementNames::for_xml_name(Some(&js("data-something"))).expect("name");
    let second = ElementNames::for_xml_name(Some(&js("DATA-SOMETHING"))).expect("name");
    assert!(
        !Arc::ptr_eq(&first, &second),
        "xml element names case-sensitive"
    );
}

#[test]
fn element_names_xml_errors() {
    let xml = Some(thymeleaf::TemplateMode::XML);
    assert!(
        ElementNames::for_name_buffer(xml, None, 0, 0).is_err(),
        "null buffer rejected"
    );
    assert!(
        ElementNames::for_name_buffer(xml, Some(&[]), 0, 0).is_err(),
        "empty name rejected"
    );
    assert!(
        ElementNames::for_name_buffer(xml, Some(&as_utf16(" ")), 0, 1).is_err(),
        "blank name rejected"
    );
}

// ===========================================================================
// AttributeNamesTest#testTextBuffer / testTextString（Java 逐字）
// ===========================================================================

#[test]
fn attribute_names_text_buffer() {
    let text = Some(thymeleaf::TemplateMode::TEXT);
    let name = AttributeNames::for_name_buffer(text, Some(&as_utf16("th:something")), 0, 12)
        .expect("text attr");
    assert_eq!(attribute_to_string(&name), "{th:something}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .map(|p| p.to_string_lossy()),
        Some("th".to_owned())
    );

    let name = AttributeNames::for_name_buffer(text, Some(&as_utf16("something")), 0, 9)
        .expect("text attr");
    assert_eq!(attribute_to_string(&name), "{something}");

    // buffer 偏移切片
    let name = AttributeNames::for_name_buffer(
        text,
        Some(&as_utf16("abcdefghijkliklmnsomethingba")),
        17,
        9,
    )
    .expect("text attr offset");
    assert_eq!(attribute_to_string(&name), "{something}");
    let name = AttributeNames::for_name_buffer(
        text,
        Some(&as_utf16("abcdefghijkliklmnth:somethingba")),
        17,
        12,
    )
    .expect("text attr offset prefixed");
    assert_eq!(attribute_to_string(&name), "{th:something}");

    // 大小写保留
    let name = AttributeNames::for_name_buffer(text, Some(&as_utf16("SOMETHING")), 0, 9)
        .expect("text attr upper");
    assert_eq!(attribute_to_string(&name), "{SOMETHING}");
    let name = AttributeNames::for_name_buffer(text, Some(&as_utf16("TH:SOMETHING")), 0, 12)
        .expect("text attr upper prefixed");
    assert_eq!(attribute_to_string(&name), "{TH:SOMETHING}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .map(|p| p.to_string_lossy()),
        Some("TH".to_owned())
    );

    // 无前缀冒号 / data- 形式
    let name = AttributeNames::for_name_buffer(text, Some(&as_utf16(":something")), 0, 10)
        .expect("text attr colon");
    assert_eq!(attribute_to_string(&name), "{:something}");
    assert!(!name.as_attribute_name().is_prefixed());
    let name = AttributeNames::for_name_buffer(text, Some(&as_utf16("data-th-something")), 0, 17)
        .expect("text attr data-th");
    assert_eq!(attribute_to_string(&name), "{data-th-something}");
    assert!(!name.as_attribute_name().is_prefixed());
    let name = AttributeNames::for_name_buffer(text, Some(&as_utf16("data-something")), 0, 14)
        .expect("text attr data");
    assert_eq!(attribute_to_string(&name), "{data-something}");

    // xml:/xmlns: 前缀
    let name = AttributeNames::for_name_buffer(text, Some(&as_utf16("xml:ns")), 0, 6)
        .expect("text attr xml:ns");
    assert_eq!(attribute_to_string(&name), "{xml:ns}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .map(|p| p.to_string_lossy()),
        Some("xml".to_owned())
    );
    let name = AttributeNames::for_name_buffer(text, Some(&as_utf16("XML:SPACE")), 0, 9)
        .expect("text attr XML:SPACE");
    assert_eq!(attribute_to_string(&name), "{XML:SPACE}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .map(|p| p.to_string_lossy()),
        Some("XML".to_owned())
    );
    let name = AttributeNames::for_name_buffer(text, Some(&as_utf16("xmlns:th")), 0, 8)
        .expect("text attr xmlns:th");
    assert_eq!(attribute_to_string(&name), "{xmlns:th}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .map(|p| p.to_string_lossy()),
        Some("xmlns".to_owned())
    );

    // 缓存同一性（assertSame 语义，string 入口返回 Arc）
    let first = AttributeNames::for_text_name(Some(&js("data-something"))).expect("first");
    let second = AttributeNames::for_text_name(Some(&js("data-something"))).expect("second");
    assert!(Arc::ptr_eq(&first, &second), "TEXT 名称仓库缓存同一性");

    // 非法输入：null/空/空白
    assert!(AttributeNames::for_name_buffer(text, None, 0, 0).is_err());
    assert!(AttributeNames::for_name_buffer(text, Some(&[]), 0, 0).is_err());
    assert!(AttributeNames::for_name_buffer(text, Some(&as_utf16(" ")), 0, 1).is_err());
}

#[test]
fn attribute_names_text_string() {
    let name = AttributeNames::for_text_name(Some(&js("th:something"))).expect("text attr");
    assert_eq!(text_attribute_to_string(&name), "{th:something}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .map(|p| p.to_string_lossy()),
        Some("th".to_owned())
    );
    let name = AttributeNames::for_text_name(Some(&js("something"))).expect("text attr");
    assert_eq!(text_attribute_to_string(&name), "{something}");
    let name = AttributeNames::for_text_name(Some(&js("TH:SOMETHING"))).expect("text attr");
    assert_eq!(text_attribute_to_string(&name), "{TH:SOMETHING}");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .map(|p| p.to_string_lossy()),
        Some("TH".to_owned())
    );
    let name = AttributeNames::for_text_name(Some(&js(":something"))).expect("text attr");
    assert_eq!(text_attribute_to_string(&name), "{:something}");
    assert!(!name.as_attribute_name().is_prefixed());
    let name = AttributeNames::for_text_name(Some(&js("data-th-something"))).expect("text attr");
    assert_eq!(text_attribute_to_string(&name), "{data-th-something}");
    assert!(!name.as_attribute_name().is_prefixed());
}

// ===========================================================================
// ElementNamesTest#testTextBuffer / testTextString（Java 逐字）
// ===========================================================================

#[test]
fn element_names_text_buffer() {
    let text = Some(thymeleaf::TemplateMode::TEXT);
    let name = ElementNames::for_name_buffer(text, Some(&as_utf16("th:something")), 0, 12)
        .expect("text element");
    assert_eq!(element_to_string(&name), "{th:something}");
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .map(|p| p.to_string_lossy()),
        Some("th".to_owned())
    );

    let name = ElementNames::for_name_buffer(text, Some(&as_utf16("something")), 0, 9)
        .expect("text element");
    assert_eq!(element_to_string(&name), "{something}");

    let name =
        ElementNames::for_name_buffer(text, Some(&as_utf16("abcdefghijkliklmnsomething")), 17, 9)
            .expect("text element offset");
    assert_eq!(element_to_string(&name), "{something}");
    let name = ElementNames::for_name_buffer(
        text,
        Some(&as_utf16("abcdefghijkliklmnth:something")),
        17,
        12,
    )
    .expect("text element offset prefixed");
    assert_eq!(element_to_string(&name), "{th:something}");

    let name = ElementNames::for_name_buffer(text, Some(&as_utf16("SOMETHING")), 0, 9)
        .expect("text element upper");
    assert_eq!(element_to_string(&name), "{SOMETHING}");
    let name = ElementNames::for_name_buffer(text, Some(&as_utf16("TH:SOMETHING")), 0, 12)
        .expect("text element upper prefixed");
    assert_eq!(element_to_string(&name), "{TH:SOMETHING}");
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .map(|p| p.to_string_lossy()),
        Some("TH".to_owned())
    );

    // 空名在 TEXT 模式允许（Java forTextName("",0,0) 不抛）
    let name = ElementNames::for_name_buffer(text, Some(&[]), 0, 0).expect("empty text element ok");
    assert_eq!(element_to_string(&name), "{}");
}

#[test]
fn element_names_text_string() {
    let name = ElementNames::for_text_name(Some(&js("th:something"))).expect("text element");
    assert_eq!(text_element_to_string(&name), "{th:something}");
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .map(|p| p.to_string_lossy()),
        Some("th".to_owned())
    );
    let name = ElementNames::for_text_name(Some(&js("TH:SOMETHING"))).expect("text element");
    assert_eq!(text_element_to_string(&name), "{TH:SOMETHING}");
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .map(|p| p.to_string_lossy()),
        Some("TH".to_owned())
    );
    let name = ElementNames::for_text_name(Some(&js(":something"))).expect("text element");
    assert_eq!(text_element_to_string(&name), "{:something}");
    assert!(!name.as_element_name().is_prefixed());
    let name = ElementNames::for_text_name(Some(&js("data-th-something"))).expect("text element");
    assert_eq!(text_element_to_string(&name), "{data-th-something}");
    assert!(!name.as_element_name().is_prefixed());
}

// ===========================================================================
// 6. AttributeNamesTest#testXMLString 全序列（Java 21 逐字）
//    toString/前缀/isPrefixed + assertSame 别名 + IllegalArgumentException
// ===========================================================================

#[test]
fn attribute_names_xml_string_full_java_parity() {
    // Java: forHTMLName(null, "data-something") -> {data-something}, prefix null
    let name = AttributeNames::for_html_name(Some(&js("data-something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name)),
        "{data-something}"
    );
    let name = AttributeNames::for_html_name(Some(&js("data-something"))).expect("name");
    assert!(name.as_attribute_name().get_prefix().is_none());

    // Java: forXMLName(null, "th:something") -> {th:something}
    let name = AttributeNames::for_xml_name(Some(&js("th:something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name.clone())),
        "{th:something}"
    );
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "th"
    );

    let name = AttributeNames::for_xml_name(Some(&js("something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name)),
        "{something}"
    );

    let name = AttributeNames::for_xml_name(Some(&js("SOMETHING"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name)),
        "{SOMETHING}"
    );

    let name = AttributeNames::for_xml_name(Some(&js("TH:SOMETHING"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name.clone())),
        "{TH:SOMETHING}"
    );
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "TH"
    );

    let name = AttributeNames::for_xml_name(Some(&js(":something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name.clone())),
        "{:something}"
    );
    assert!(!name.as_attribute_name().is_prefixed());

    let name = AttributeNames::for_xml_name(Some(&js("data-th-something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name.clone())),
        "{data-th-something}"
    );
    assert!(!name.as_attribute_name().is_prefixed());

    let name = AttributeNames::for_xml_name(Some(&js("data-something"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name)),
        "{data-something}"
    );

    let name = AttributeNames::for_xml_name(Some(&js("xml:ns"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name.clone())),
        "{xml:ns}"
    );
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "xml"
    );

    let name = AttributeNames::for_xml_name(Some(&js("xml:space"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name)),
        "{xml:space}"
    );

    let name = AttributeNames::for_xml_name(Some(&js("XML:SPACE"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name.clone())),
        "{XML:SPACE}"
    );
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "XML"
    );

    // Java: forHTMLName("xmlns:th") -> {xmlns:th}（HTML 不识别 xmlns 前缀）
    let name = AttributeNames::for_html_name(Some(&js("xmlns:th"))).expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Html(name)),
        "{xmlns:th}"
    );
    let name = AttributeNames::for_xml_name(Some(&js("xmlns:th"))).expect("name");
    assert_eq!(
        name.as_attribute_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "xmlns"
    );

    // Java: forXMLName("th","something") -> {th:something}
    let name = AttributeNames::for_xml_name_with_prefix(Some(&js("th")), Some(&js("something")))
        .expect("name");
    assert_eq!(
        attribute_to_string(&AttributeNameValue::Xml(name)),
        "{th:something}"
    );

    // ---- assertSame 别名系列（XML 大小写敏感、无 HTML5 折叠） ----
    let first = AttributeNames::for_xml_name(Some(&js("data-something"))).expect("name");
    let second = AttributeNames::for_xml_name(Some(&js("data-something"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "data-something cached");

    let first = AttributeNames::for_xml_name(Some(&js("xmlns:th"))).expect("name");
    let second = AttributeNames::for_xml_name(Some(&js("xmlns:th"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "xmlns:th cached");

    let first = AttributeNames::for_xml_name(Some(&js("data-th-something"))).expect("name");
    let second = AttributeNames::for_xml_name(Some(&js("data-th-something"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "data-th-something cached");

    // Java assertNotSame: data-th-something 与 th:something 不同（XML 无折叠）
    let first = AttributeNames::for_xml_name(Some(&js("data-th-something"))).expect("name");
    let second = AttributeNames::for_xml_name(Some(&js("th:something"))).expect("name");
    assert!(
        !Arc::ptr_eq(&first, &second),
        "xml data-th-something is NOT th:something"
    );

    let first = AttributeNames::for_xml_name_with_prefix(Some(&js("xmlns")), Some(&js("th")))
        .expect("name");
    let second = AttributeNames::for_xml_name_with_prefix(Some(&js("xmlns")), Some(&js("th")))
        .expect("name");
    assert!(Arc::ptr_eq(&first, &second), "xmlns/th cached");

    let first = AttributeNames::for_xml_name_with_prefix(Some(&js("th")), Some(&js("something")))
        .expect("name");
    let second = AttributeNames::for_xml_name_with_prefix(Some(&js("th")), Some(&js("something")))
        .expect("name");
    assert!(Arc::ptr_eq(&first, &second), "th/something cached");

    // 空/空白/null 前缀折叠到无前缀
    let plain = AttributeNames::for_xml_name(Some(&js("something"))).expect("name");
    let empty = AttributeNames::for_xml_name_with_prefix(Some(&js("")), Some(&js("something")))
        .expect("name");
    assert!(Arc::ptr_eq(&plain, &empty), "empty prefix aliases none");
    let null_prefix =
        AttributeNames::for_xml_name_with_prefix(None, Some(&js("something"))).expect("name");
    assert!(
        Arc::ptr_eq(&plain, &null_prefix),
        "null prefix aliases none"
    );
    let blank = AttributeNames::for_xml_name_with_prefix(Some(&js("  ")), Some(&js("something")))
        .expect("name");
    assert!(Arc::ptr_eq(&plain, &blank), "blank prefix aliases none");

    // ---- IllegalArgumentException 系列 ----
    assert!(
        AttributeNames::for_xml_name(None).is_err(),
        "forXMLName(null) rejected"
    );
    assert!(
        AttributeNames::for_xml_name(Some(&js(""))).is_err(),
        "forXMLName(\"\") rejected"
    );
    assert!(
        AttributeNames::for_xml_name_with_prefix(Some(&js("t")), Some(&js(""))).is_err(),
        "forXMLName(\"t\",\"\") rejected"
    );
    assert!(
        AttributeNames::for_xml_name(Some(&js(" "))).is_err(),
        "forXMLName(\" \") rejected"
    );
    assert!(
        AttributeNames::for_xml_name_with_prefix(Some(&js("t")), Some(&js(" "))).is_err(),
        "forXMLName(\"t\",\" \") rejected"
    );
}

// ===========================================================================
// 7. ElementNamesTest#testXMLString 全序列（Java 21 逐字）
//    含 data:something 前缀（data:/xml:/xmlns: 前缀族）
// ===========================================================================

#[test]
fn element_names_xml_string_full_java_parity() {
    // Java: forXMLName(null, "th:something") -> {th:something}
    let name = ElementNames::for_xml_name(Some(&js("th:something"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name.clone())),
        "{th:something}"
    );
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "th"
    );

    let name = ElementNames::for_xml_name(Some(&js("something"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name)),
        "{something}"
    );

    let name = ElementNames::for_xml_name(Some(&js("SOMETHING"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name)),
        "{SOMETHING}"
    );

    let name = ElementNames::for_xml_name(Some(&js("TH:SOMETHING"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name.clone())),
        "{TH:SOMETHING}"
    );
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "TH"
    );

    let name = ElementNames::for_xml_name(Some(&js(":something"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name.clone())),
        "{:something}"
    );
    assert!(!name.as_element_name().is_prefixed());

    let name = ElementNames::for_xml_name(Some(&js("data-th-something"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name.clone())),
        "{data-th-something}"
    );
    assert!(!name.as_element_name().is_prefixed());

    let name = ElementNames::for_xml_name(Some(&js("data-something"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name)),
        "{data-something}"
    );

    // data:something -> 前缀 data（XML 冒号分隔）
    let name = ElementNames::for_xml_name(Some(&js("data:something"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name.clone())),
        "{data:something}"
    );
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "data"
    );

    let name = ElementNames::for_xml_name(Some(&js("xml:ns"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name.clone())),
        "{xml:ns}"
    );
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "xml"
    );

    let name = ElementNames::for_xml_name(Some(&js("xml:space"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name)),
        "{xml:space}"
    );

    let name = ElementNames::for_xml_name(Some(&js("XML:SPACE"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name.clone())),
        "{XML:SPACE}"
    );
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "XML"
    );

    let name = ElementNames::for_xml_name(Some(&js("xmlns:th"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name.clone())),
        "{xmlns:th}"
    );
    assert_eq!(
        name.as_element_name()
            .get_prefix()
            .unwrap()
            .to_string_lossy(),
        "xmlns"
    );

    // Java: forXMLName("th","something") -> {th:something}
    let name = ElementNames::for_xml_name_with_prefix(Some(&js("th")), Some(&js("something")))
        .expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Xml(name)),
        "{th:something}"
    );

    // ---- assertSame 别名系列 ----
    let first = ElementNames::for_xml_name(Some(&js("data-something"))).expect("name");
    let second = ElementNames::for_xml_name(Some(&js("data-something"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "data-something cached");

    let first = ElementNames::for_xml_name(Some(&js("xmlns:th"))).expect("name");
    let second = ElementNames::for_xml_name(Some(&js("xmlns:th"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "xmlns:th cached");

    let first = ElementNames::for_xml_name(Some(&js("data-th-something"))).expect("name");
    let second = ElementNames::for_xml_name(Some(&js("data-th-something"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "data-th-something cached");

    // Java assertNotSame: XML 下 data-th-something 与 th:something 不同
    let first = ElementNames::for_xml_name(Some(&js("data-th-something"))).expect("name");
    let second = ElementNames::for_xml_name(Some(&js("th:something"))).expect("name");
    assert!(
        !Arc::ptr_eq(&first, &second),
        "xml data-th-something is NOT th:something"
    );

    let first =
        ElementNames::for_xml_name_with_prefix(Some(&js("xmlns")), Some(&js("th"))).expect("name");
    let second =
        ElementNames::for_xml_name_with_prefix(Some(&js("xmlns")), Some(&js("th"))).expect("name");
    assert!(Arc::ptr_eq(&first, &second), "xmlns/th cached");

    let first = ElementNames::for_xml_name_with_prefix(Some(&js("th")), Some(&js("something")))
        .expect("name");
    let second = ElementNames::for_xml_name_with_prefix(Some(&js("th")), Some(&js("something")))
        .expect("name");
    assert!(Arc::ptr_eq(&first, &second), "th/something cached");

    let plain = ElementNames::for_xml_name(Some(&js("something"))).expect("name");
    let empty = ElementNames::for_xml_name_with_prefix(Some(&js("")), Some(&js("something")))
        .expect("name");
    assert!(Arc::ptr_eq(&plain, &empty), "empty prefix aliases none");
    let null_prefix =
        ElementNames::for_xml_name_with_prefix(None, Some(&js("something"))).expect("name");
    assert!(
        Arc::ptr_eq(&plain, &null_prefix),
        "null prefix aliases none"
    );
    let blank = ElementNames::for_xml_name_with_prefix(Some(&js("  ")), Some(&js("something")))
        .expect("name");
    assert!(Arc::ptr_eq(&plain, &blank), "blank prefix aliases none");

    // ---- IllegalArgumentException 系列 ----
    assert!(
        ElementNames::for_xml_name(None).is_err(),
        "forXMLName(null) rejected"
    );
    assert!(
        ElementNames::for_xml_name(Some(&js(""))).is_err(),
        "forXMLName(\"\") rejected"
    );
    assert!(
        ElementNames::for_xml_name_with_prefix(Some(&js("t")), Some(&js(""))).is_err(),
        "forXMLName(\"t\",\"\") rejected"
    );
    assert!(
        ElementNames::for_xml_name(Some(&js(" "))).is_err(),
        "forXMLName(\" \") rejected"
    );
    assert!(
        ElementNames::for_xml_name_with_prefix(Some(&js("t")), Some(&js(" "))).is_err(),
        "forXMLName(\"t\",\" \") rejected"
    );
}

// ===========================================================================
// 8. ElementNamesTest#testHTMLString 补充别名与非法参数（Java 21 逐字）
// ===========================================================================

#[test]
fn element_names_html_string_extra_aliases() {
    // Java: forHTMLName("th","something") -> {th:something,th-something}
    let name = ElementNames::for_html_name_with_prefix(Some(&js("th")), Some(&js("something")))
        .expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Html(name)),
        "{th:something,th-something}"
    );

    // Java: forHTMLName(null, "th:something") -> {th:something,th-something}
    let name = ElementNames::for_html_name(Some(&js("th:something"))).expect("name");
    assert_eq!(
        element_to_string(&ElementNameValue::Html(name)),
        "{th:something,th-something}"
    );

    // assertSame 补充系列
    let plain = ElementNames::for_html_name(Some(&js("something"))).expect("name");
    let blank_upper =
        ElementNames::for_html_name_with_prefix(Some(&js("  ")), Some(&js("SOMETHING")))
            .expect("name");
    assert!(
        Arc::ptr_eq(&plain, &blank_upper),
        "blank prefix + uppercase aliases no prefix lowercase"
    );

    let th = ElementNames::for_html_name(Some(&js("th:something"))).expect("name");
    let hyphen_upper = ElementNames::for_html_name(Some(&js("TH-SOMETHING"))).expect("name");
    assert!(
        Arc::ptr_eq(&th, &hyphen_upper),
        "TH-SOMETHING aliases th:something"
    );

    let xmlns =
        ElementNames::for_html_name_with_prefix(Some(&js("xmlns")), Some(&js("th"))).expect("name");
    let upper =
        ElementNames::for_html_name_with_prefix(Some(&js("XMLNS")), Some(&js("TH"))).expect("name");
    assert!(Arc::ptr_eq(&xmlns, &upper), "XMLNS/TH aliases xmlns/th");

    let th = ElementNames::for_html_name(Some(&js("th:something"))).expect("name");
    let hyphen = ElementNames::for_html_name(Some(&js("th-something"))).expect("name");
    let upper_colon = ElementNames::for_html_name(Some(&js("TH:SOMETHING"))).expect("name");
    assert!(
        Arc::ptr_eq(&th, &hyphen) && Arc::ptr_eq(&th, &upper_colon),
        "th-something / TH:SOMETHING alias th:something"
    );

    // IllegalArgumentException：string 入口（Java forHTMLName 系列）
    assert!(
        ElementNames::for_html_name(None).is_err(),
        "forHTMLName(null) rejected"
    );
    assert!(
        ElementNames::for_html_name(Some(&js(""))).is_err(),
        "forHTMLName(\"\") rejected"
    );
    assert!(
        ElementNames::for_html_name_with_prefix(Some(&js("t")), Some(&js(""))).is_err(),
        "forHTMLName(\"t\",\"\") rejected"
    );
    assert!(
        ElementNames::for_html_name(Some(&js(" "))).is_err(),
        "forHTMLName(\" \") rejected"
    );
    assert!(
        ElementNames::for_html_name_with_prefix(Some(&js("t")), Some(&js(" "))).is_err(),
        "forHTMLName(\"t\",\" \") rejected"
    );
}
