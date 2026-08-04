//! `ElementDefinitions` Java 1:1 差分测试。
//!
//! 对应上游 `thymeleaf-tests-core` 的
//! `org.thymeleaf.engine.ElementDefinitionsTest`：标准 HTML 元素名
//! 缓存同一性、大小写规则、前缀组合（t:text、thhhh:text、t-teeee
//! 破折号折叠）、五种模式 data: 形式、空名（文本模式允许）、
//! null/空/空白前缀。

use std::collections::HashMap;
use std::sync::Arc;

use thymeleaf::engine::{ElementDefinitions, HTMLElementDefinition};
use thymeleaf::util::Utf16String;

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn definitions() -> ElementDefinitions {
    // 对应 Java: new ElementDefinitions(Collections.EMPTY_MAP)
    ElementDefinitions::new(HashMap::new()).expect("element definitions")
}

/// Java `ElementDefinition#getElementName().toString()`。
fn html_name(value: &Arc<HTMLElementDefinition>) -> String {
    value
        .as_element_definition()
        .get_element_name()
        .as_element_name()
        .to_utf16_string()
        .expect("toString")
        .to_string_lossy()
}

fn xml_name(value: &Arc<thymeleaf::engine::XMLElementDefinition>) -> String {
    value
        .as_element_definition()
        .get_element_name()
        .as_element_name()
        .to_utf16_string()
        .expect("toString")
        .to_string_lossy()
}

fn text_name(value: &Arc<thymeleaf::engine::TextElementDefinition>) -> String {
    value
        .as_element_definition()
        .get_element_name()
        .as_element_name()
        .to_utf16_string()
        .expect("toString")
        .to_string_lossy()
}

#[test]
fn element_definitions_standard_names_cache() {
    let element_definitions = definitions();
    let standard_size = ElementDefinitions::all_standard_html_element_names().len();

    // HTML：大小写不敏感 → 同一实例
    for name in ElementDefinitions::all_standard_html_element_names() {
        let def1 = element_definitions
            .for_html_name(Some(&js(name)))
            .expect("html definition");
        let def2 = element_definitions
            .for_html_name(Some(&js(name)))
            .expect("html definition");
        let def3 = element_definitions
            .for_html_name(Some(&js(&name.to_uppercase())))
            .expect("html definition");
        assert!(Arc::ptr_eq(&def1, &def2), "html {name} cached");
        assert!(Arc::ptr_eq(&def2, &def3), "html {name} case-insensitive");
    }
    // XML：大小写敏感
    for name in ElementDefinitions::all_standard_html_element_names() {
        let def1 = element_definitions
            .for_xml_name(Some(&js(name)))
            .expect("xml definition");
        let def2 = element_definitions
            .for_xml_name(Some(&js(name)))
            .expect("xml definition");
        let def3 = element_definitions
            .for_xml_name(Some(&js(&name.to_uppercase())))
            .expect("xml definition");
        assert!(Arc::ptr_eq(&def1, &def2), "xml {name} cached");
        assert!(
            !Arc::ptr_eq(&def2, &def3),
            "xml {name} case-sensitive instance"
        );
        assert_ne!(
            xml_name(&def2),
            xml_name(&def3),
            "xml {name} case-sensitive name"
        );
    }

    // 非标准名称：NEW → "new"（HTML 规范化）
    let new1 = element_definitions
        .for_html_name(Some(&js("NEW")))
        .expect("html new");
    assert_eq!(html_name(&new1), "{new}");
    let new2 = element_definitions
        .for_html_name(Some(&js("new")))
        .expect("html new");
    assert!(Arc::ptr_eq(&new1, &new2));
    let new3 = element_definitions
        .for_html_name(Some(&js("NeW")))
        .expect("html new");
    assert!(Arc::ptr_eq(&new1, &new3));
    let new4 = element_definitions
        .for_xml_name(Some(&js("NeW")))
        .expect("xml new");
    assert_ne!(html_name(&new1), xml_name(&new4));
    let new5 = element_definitions
        .for_xml_name(Some(&js("new")))
        .expect("xml new");
    // Java assertNotSame(new1, new5)：XML 与 HTML 定义是不同类型实例
    let new6 = element_definitions
        .for_xml_name(Some(&js("new")))
        .expect("xml new");
    assert!(Arc::ptr_eq(&new5, &new6));
    let new7 = element_definitions
        .for_html_name(Some(&js("new")))
        .expect("html new");
    assert!(Arc::ptr_eq(&new1, &new7));

    assert_eq!(
        standard_size,
        ElementDefinitions::all_standard_html_element_names().len()
    );
    assert!(
        !ElementDefinitions::all_standard_html_element_names().contains(&"new"),
        "new is not standard"
    );
}

#[test]
fn element_definitions_prefix_combinations() {
    let element_definitions = definitions();

    let mut def = element_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("text")))
        .expect("t:text");
    assert_eq!(html_name(&def), "{t:text,t-text}");
    def = element_definitions
        .for_html_name_with_prefix(None, Some(&js("text")))
        .expect("text");
    assert_eq!(html_name(&def), "{text}");
    let mut def2 = element_definitions
        .for_html_name(Some(&js("text")))
        .expect("text");
    assert!(Arc::ptr_eq(&def, &def2));
    def = element_definitions
        .for_html_name_with_prefix(Some(&js("thhhh")), Some(&js("text")))
        .expect("thhhh:text");
    assert_eq!(html_name(&def), "{thhhh:text,thhhh-text}");

    def = element_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("t")))
        .expect("t:t");
    assert_eq!(html_name(&def), "{t:t,t-t}");
    def = element_definitions
        .for_html_name_with_prefix(None, Some(&js("t")))
        .expect("t");
    assert_eq!(html_name(&def), "{t}");
    def2 = element_definitions
        .for_html_name(Some(&js("t")))
        .expect("t");
    assert!(Arc::ptr_eq(&def, &def2));
    def = element_definitions
        .for_html_name_with_prefix(Some(&js("thhhh")), Some(&js("teeee")))
        .expect("thhhh:teeee");
    assert_eq!(html_name(&def), "{thhhh:teeee,thhhh-teeee}");

    def = element_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("te")))
        .expect("t:te");
    assert_eq!(html_name(&def), "{t:te,t-te}");
    def = element_definitions
        .for_html_name_with_prefix(None, Some(&js("te")))
        .expect("te");
    assert_eq!(html_name(&def), "{te}");
    def2 = element_definitions
        .for_html_name(Some(&js("te")))
        .expect("te");
    assert!(Arc::ptr_eq(&def, &def2));
    def = element_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("teeee")))
        .expect("t:teeee");
    assert_eq!(html_name(&def), "{t:teeee,t-teeee}");

    def = element_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("ta")))
        .expect("t:ta");
    assert_eq!(html_name(&def), "{t:ta,t-ta}");
    def = element_definitions
        .for_html_name_with_prefix(None, Some(&js("ta")))
        .expect("ta");
    assert_eq!(html_name(&def), "{ta}");
    def2 = element_definitions
        .for_html_name(Some(&js("ta")))
        .expect("ta");
    assert!(Arc::ptr_eq(&def, &def2));

    def = element_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("ti")))
        .expect("t:ti");
    assert_eq!(html_name(&def), "{t:ti,t-ti}");
    def = element_definitions
        .for_html_name_with_prefix(None, Some(&js("ti")))
        .expect("ti");
    assert_eq!(html_name(&def), "{ti}");
    def2 = element_definitions
        .for_html_name(Some(&js("ti")))
        .expect("ti");
    assert!(Arc::ptr_eq(&def, &def2));

    // t:teeee 全形态同一实例（含破折号形式 t-teeee）
    let t_teeee = element_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("teeee")))
        .expect("t:teeee");
    let full = element_definitions
        .for_html_name(Some(&js("t:teeee")))
        .expect("t:teeee full");
    assert_eq!(html_name(&full), "{t:teeee,t-teeee}");
    assert!(Arc::ptr_eq(&t_teeee, &full));
    let null_prefix = element_definitions
        .for_html_name_with_prefix(None, Some(&js("t:teeee")))
        .expect("t:teeee null prefix");
    assert!(Arc::ptr_eq(&t_teeee, &null_prefix));
    let dash_form = element_definitions
        .for_html_name(Some(&js("t-teeee")))
        .expect("t-teeee");
    assert_eq!(html_name(&dash_form), "{t:teeee,t-teeee}");
    assert!(Arc::ptr_eq(&t_teeee, &dash_form));
    let dash_null_prefix = element_definitions
        .for_html_name_with_prefix(None, Some(&js("t-teeee")))
        .expect("t-teeee null prefix");
    assert!(Arc::ptr_eq(&t_teeee, &dash_null_prefix));
}

#[test]
fn element_definitions_data_forms_five_modes() {
    let element_definitions = definitions();

    // HTML：data: 前缀折叠（元素无 data- HTML5 形式，只有破折号形式）
    let cases_html: Vec<(&str, &str)> = vec![
        ("data:teeee", "{data:teeee,data-teeee}"),
        ("data", "{data}"),
        ("dataa:teeee", "{dataa:teeee,dataa-teeee}"),
        ("data:data", "{data:data,data-data}"),
        ("DATA:TEEEE", "{data:teeee,data-teeee}"),
        ("DATA", "{data}"),
        ("DATAA:TEEEE", "{dataa:teeee,dataa-teeee}"),
        ("DATA:DATA", "{data:data,data-data}"),
    ];
    for (input, expected) in cases_html {
        let def = element_definitions
            .for_html_name_with_prefix(None, Some(&js(input)))
            .expect("html element");
        assert_eq!(html_name(&def), expected, "html {input}");
    }

    // XML/TEXT/JAVASCRIPT/CSS：大小写敏感
    for mode in ["xml", "text", "javascript", "css"] {
        let cases: Vec<(&str, &str)> = vec![
            ("data:teeee", "{data:teeee}"),
            ("data", "{data}"),
            ("dataa:teeee", "{dataa:teeee}"),
            ("data:data", "{data:data}"),
            ("DATA:TEEEE", "{DATA:TEEEE}"),
            ("DATA", "{DATA}"),
            ("DATAA:TEEEE", "{DATAA:TEEEE}"),
            ("DATA:DATA", "{DATA:DATA}"),
        ];
        for (input, expected) in cases {
            let name = match mode {
                "xml" => {
                    let def = element_definitions
                        .for_xml_name_with_prefix(None, Some(&js(input)))
                        .expect("xml element");
                    xml_name(&def)
                }
                "text" => {
                    let def = element_definitions
                        .for_text_name(Some(&js(input)))
                        .expect("text element");
                    text_name(&def)
                }
                "javascript" => {
                    let def = element_definitions
                        .for_javascript_name(Some(&js(input)))
                        .expect("javascript element");
                    text_name(&def)
                }
                _ => {
                    let def = element_definitions
                        .for_css_name(Some(&js(input)))
                        .expect("css element");
                    text_name(&def)
                }
            };
            assert_eq!(name, expected, "{mode} {input}");
        }
    }
}

#[test]
fn element_definitions_invalid_and_empty_names() {
    let element_definitions = definitions();

    // HTML/XML：null/空/空白 → 非法
    assert!(
        element_definitions
            .for_html_name_with_prefix(None, None)
            .is_err()
    );
    assert!(
        element_definitions
            .for_html_name_with_prefix(None, Some(&js("")))
            .is_err()
    );
    assert!(
        element_definitions
            .for_html_name_with_prefix(None, Some(&js(" ")))
            .is_err()
    );
    assert!(
        element_definitions
            .for_xml_name_with_prefix(None, None)
            .is_err()
    );
    assert!(
        element_definitions
            .for_xml_name_with_prefix(None, Some(&js("")))
            .is_err()
    );
    assert!(
        element_definitions
            .for_xml_name_with_prefix(None, Some(&js(" ")))
            .is_err()
    );

    // 文本模式：空名允许 → "{}"；空白名非法
    let def = element_definitions
        .for_text_name(Some(&js("")))
        .expect("text empty name");
    assert_eq!(text_name(&def), "{}");
    assert!(
        element_definitions.for_text_name(Some(&js(" "))).is_err(),
        "text blank name rejected"
    );
    let def = element_definitions
        .for_javascript_name(Some(&js("")))
        .expect("javascript empty name");
    assert_eq!(text_name(&def), "{}");
    assert!(
        element_definitions
            .for_javascript_name(Some(&js(" ")))
            .is_err(),
        "javascript blank name rejected"
    );
    let def = element_definitions
        .for_css_name(Some(&js("")))
        .expect("css empty name");
    assert_eq!(text_name(&def), "{}");
    assert!(
        element_definitions.for_css_name(Some(&js(" "))).is_err(),
        "css blank name rejected"
    );
}

#[test]
fn element_definitions_empty_null_whitespace_prefix() {
    let element_definitions = definitions();
    // 对应 Java testEmptyPrefix/testNullPrefix/testWhitespacePrefix
    for prefix in ["", " ", "   "] {
        let ed01 = element_definitions
            .for_html_name_with_prefix(Some(&js(prefix)), Some(&js("one")))
            .expect("html");
        let ed02 = element_definitions
            .for_xml_name_with_prefix(Some(&js(prefix)), Some(&js("one")))
            .expect("xml");
        let ed03 = element_definitions
            .for_text_name(Some(&js("one")))
            .expect("text");
        let ed04 = element_definitions
            .for_javascript_name(Some(&js("one")))
            .expect("javascript");
        let ed05 = element_definitions
            .for_css_name(Some(&js("one")))
            .expect("css");
        assert_eq!(html_name(&ed01), "{one}", "html prefix {prefix:?}");
        assert_eq!(xml_name(&ed02), "{one}", "xml prefix {prefix:?}");
        assert_eq!(text_name(&ed03), "{one}");
        assert_eq!(text_name(&ed04), "{one}");
        assert_eq!(text_name(&ed05), "{one}");
    }
    let ed01 = element_definitions
        .for_html_name_with_prefix(None, Some(&js("one")))
        .expect("html");
    let ed02 = element_definitions
        .for_xml_name_with_prefix(None, Some(&js("one")))
        .expect("xml");
    let ed03 = element_definitions
        .for_text_name(Some(&js("one")))
        .expect("text");
    let ed04 = element_definitions
        .for_javascript_name(Some(&js("one")))
        .expect("javascript");
    let ed05 = element_definitions
        .for_css_name(Some(&js("one")))
        .expect("css");
    assert_eq!(html_name(&ed01), "{one}");
    assert_eq!(xml_name(&ed02), "{one}");
    assert_eq!(text_name(&ed03), "{one}");
    assert_eq!(text_name(&ed04), "{one}");
    assert_eq!(text_name(&ed05), "{one}");
}
