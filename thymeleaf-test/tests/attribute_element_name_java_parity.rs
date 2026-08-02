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

use thymeleaf::engine::{AttributeNameValue, AttributeNames, ElementNameValue, ElementNames};
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
