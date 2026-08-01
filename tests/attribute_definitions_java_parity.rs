//! `AttributeDefinitions` Java 1:1 差分测试。
//!
//! 对应上游 `thymeleaf-tests-core` 的
//! `org.thymeleaf.engine.AttributeDefinitionsTest`：
//! 标准 HTML 属性名缓存同一性、HTML 大小写不敏感/XML 大小写敏感、
//! 前缀组合（t:text、thhhh:text、data- 折叠）、五种模式的
//! data: 形式、null/空/空白前缀与非法参数。

use std::collections::HashMap;
use std::sync::Arc;

use thymeleaf::engine::{AttributeDefinitions, HTMLAttributeDefinition};
use thymeleaf::util::JavaString;

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn definitions() -> AttributeDefinitions {
    // 对应 Java: new AttributeDefinitions(Collections.EMPTY_MAP)
    AttributeDefinitions::new(HashMap::new()).expect("attribute definitions")
}

/// Java `AttributeDefinition#getAttributeName().toString()`。
fn html_name(value: &Arc<HTMLAttributeDefinition>) -> String {
    value
        .as_attribute_definition()
        .get_attribute_name()
        .as_attribute_name()
        .to_java_string()
        .expect("toString")
        .to_string_lossy()
}

fn xml_name(value: &Arc<thymeleaf::engine::XMLAttributeDefinition>) -> String {
    value
        .as_attribute_definition()
        .get_attribute_name()
        .as_attribute_name()
        .to_java_string()
        .expect("toString")
        .to_string_lossy()
}

fn text_name(value: &Arc<thymeleaf::engine::TextAttributeDefinition>) -> String {
    value
        .as_attribute_definition()
        .get_attribute_name()
        .as_attribute_name()
        .to_java_string()
        .expect("toString")
        .to_string_lossy()
}

#[test]
fn attribute_definitions_standard_names_cache() {
    let attribute_definitions = definitions();
    let standard_size = AttributeDefinitions::all_standard_html_attribute_names().len();

    // HTML：大小写不敏感 → 同一实例
    for name in AttributeDefinitions::all_standard_html_attribute_names() {
        let def1 = attribute_definitions
            .for_html_name(Some(&js(name)))
            .expect("html definition");
        let def2 = attribute_definitions
            .for_html_name(Some(&js(name)))
            .expect("html definition");
        let def3 = attribute_definitions
            .for_html_name(Some(&js(&name.to_uppercase())))
            .expect("html definition");
        assert!(Arc::ptr_eq(&def1, &def2), "html {name} cached");
        assert!(Arc::ptr_eq(&def2, &def3), "html {name} case-insensitive");
    }
    // XML：大小写敏感 → 大写是不同实例且名称不同
    for name in AttributeDefinitions::all_standard_html_attribute_names() {
        let def1 = attribute_definitions
            .for_xml_name(Some(&js(name)))
            .expect("xml definition");
        let def2 = attribute_definitions
            .for_xml_name(Some(&js(name)))
            .expect("xml definition");
        let def3 = attribute_definitions
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

    // 非标准名称：NEW → "new"（HTML 规范化），五种模式互不相同
    let new1 = attribute_definitions
        .for_html_name(Some(&js("NEW")))
        .expect("html new");
    assert_eq!(html_name(&new1), "{new}");
    let new2 = attribute_definitions
        .for_html_name(Some(&js("new")))
        .expect("html new");
    assert!(Arc::ptr_eq(&new1, &new2), "html NEW cached");
    let new3 = attribute_definitions
        .for_html_name(Some(&js("NeW")))
        .expect("html new");
    assert!(Arc::ptr_eq(&new1, &new3), "html NeW cached");
    let new4 = attribute_definitions
        .for_xml_name(Some(&js("NeW")))
        .expect("xml new");
    assert_ne!(html_name(&new1), xml_name(&new4), "html vs xml differ");
    let new5 = attribute_definitions
        .for_xml_name(Some(&js("new")))
        .expect("xml new");
    assert!(!Arc::ptr_eq(&new4, &new5), "xml NeW vs new differ");
    let new6 = attribute_definitions
        .for_xml_name(Some(&js("new")))
        .expect("xml new");
    assert!(Arc::ptr_eq(&new5, &new6), "xml new cached");
    let new7 = attribute_definitions
        .for_html_name(Some(&js("new")))
        .expect("html new");
    assert!(Arc::ptr_eq(&new1, &new7), "html new cached again");

    // 标准名单不含 "new"
    assert_eq!(
        standard_size,
        AttributeDefinitions::all_standard_html_attribute_names().len()
    );
    assert!(
        !AttributeDefinitions::all_standard_html_attribute_names().contains(&"new"),
        "new is not standard"
    );

    // 布尔属性标志
    let html_id = attribute_definitions
        .for_html_name(Some(&js("id")))
        .expect("html id");
    let html_disabled = attribute_definitions
        .for_html_name(Some(&js("disabled")))
        .expect("html disabled");
    assert_eq!(html_name(&html_disabled), "{disabled}");
    assert!(!html_id.is_boolean_attribute(), "id is not boolean");
    assert!(html_disabled.is_boolean_attribute(), "disabled is boolean");
}

#[test]
fn attribute_definitions_th_prefixed() {
    let attribute_definitions = definitions();

    // HTML th:text 全形态同一实例
    let thtext = attribute_definitions
        .for_html_name(Some(&js("th:text")))
        .expect("html th:text");
    assert_eq!(html_name(&thtext), "{th:text,data-th-text}");
    let thtext2 = attribute_definitions
        .for_html_name(Some(&js("th:text")))
        .expect("html th:text");
    let thtext3 = attribute_definitions
        .for_html_name(Some(&js("th:TEXT")))
        .expect("html th:TEXT");
    let thtext4 = attribute_definitions
        .for_html_name(Some(&js("data-th-TEXT")))
        .expect("html data-th-TEXT");
    assert!(Arc::ptr_eq(&thtext, &thtext2));
    assert!(Arc::ptr_eq(&thtext, &thtext3));
    assert!(Arc::ptr_eq(&thtext, &thtext4));

    // XML th:text：大小写敏感
    let xmlthtext = attribute_definitions
        .for_xml_name(Some(&js("th:text")))
        .expect("xml th:text");
    assert_eq!(xml_name(&xmlthtext), "{th:text}");
    let xmlthtext2 = attribute_definitions
        .for_xml_name(Some(&js("th:text")))
        .expect("xml th:text");
    let xmlthtext3 = attribute_definitions
        .for_xml_name(Some(&js("th:TEXT")))
        .expect("xml th:TEXT");
    assert_eq!(xml_name(&xmlthtext3), "{th:TEXT}");
    let xmlthtext4 = attribute_definitions
        .for_xml_name(Some(&js("data-th-TEXT")))
        .expect("xml data-th-TEXT");
    assert_eq!(xml_name(&xmlthtext4), "{data-th-TEXT}");
    assert!(Arc::ptr_eq(&xmlthtext, &xmlthtext2));
    assert!(!Arc::ptr_eq(&xmlthtext, &xmlthtext3));
    assert!(!Arc::ptr_eq(&xmlthtext, &xmlthtext4));

    // 显式前缀形式与完整名形式同一实例
    let thtext_2 = attribute_definitions
        .for_html_name_with_prefix(Some(&js("th")), Some(&js("text")))
        .expect("html th+text");
    assert_eq!(html_name(&thtext_2), "{th:text,data-th-text}");
    let thtext2_2 = attribute_definitions
        .for_html_name(Some(&js("th:text")))
        .expect("html th:text");
    let thtext3_2 = attribute_definitions
        .for_html_name(Some(&js("th:TEXT")))
        .expect("html th:TEXT");
    let thtext4_2 = attribute_definitions
        .for_html_name(Some(&js("data-th-TEXT")))
        .expect("html data-th-TEXT");
    assert!(Arc::ptr_eq(&thtext_2, &thtext2_2));
    assert!(Arc::ptr_eq(&thtext_2, &thtext3_2));
    assert!(Arc::ptr_eq(&thtext_2, &thtext4_2));

    let xmlthtext_2 = attribute_definitions
        .for_xml_name_with_prefix(Some(&js("th")), Some(&js("text")))
        .expect("xml th+text");
    assert_eq!(xml_name(&xmlthtext_2), "{th:text}");
    let xmlthtext2_2 = attribute_definitions
        .for_xml_name(Some(&js("th:text")))
        .expect("xml th:text");
    let xmlthtext3_2 = attribute_definitions
        .for_xml_name(Some(&js("th:TEXT")))
        .expect("xml th:TEXT");
    assert_eq!(xml_name(&xmlthtext3_2), "{th:TEXT}");
    let xmlthtext4_2 = attribute_definitions
        .for_xml_name(Some(&js("data-th-TEXT")))
        .expect("xml data-th-TEXT");
    assert_eq!(xml_name(&xmlthtext4_2), "{data-th-TEXT}");
    assert!(Arc::ptr_eq(&xmlthtext_2, &xmlthtext2_2));
    assert!(!Arc::ptr_eq(&xmlthtext_2, &xmlthtext3_2));
    assert!(!Arc::ptr_eq(&xmlthtext_2, &xmlthtext4_2));
}

#[test]
fn attribute_definitions_prefix_combinations() {
    let attribute_definitions = definitions();

    let mut def = attribute_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("text")))
        .expect("t:text");
    assert_eq!(html_name(&def), "{t:text,data-t-text}");
    def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("text")))
        .expect("text");
    assert_eq!(html_name(&def), "{text}");
    let def2 = attribute_definitions
        .for_html_name(Some(&js("text")))
        .expect("text");
    assert!(Arc::ptr_eq(&def, &def2));
    def = attribute_definitions
        .for_html_name_with_prefix(Some(&js("thhhh")), Some(&js("text")))
        .expect("thhhh:text");
    assert_eq!(html_name(&def), "{thhhh:text,data-thhhh-text}");

    def = attribute_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("t")))
        .expect("t:t");
    assert_eq!(html_name(&def), "{t:t,data-t-t}");
    def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("t")))
        .expect("t");
    assert_eq!(html_name(&def), "{t}");
    let def2 = attribute_definitions
        .for_html_name(Some(&js("t")))
        .expect("t");
    assert!(Arc::ptr_eq(&def, &def2));
    def = attribute_definitions
        .for_html_name_with_prefix(Some(&js("thhhh")), Some(&js("teeee")))
        .expect("thhhh:teeee");
    assert_eq!(html_name(&def), "{thhhh:teeee,data-thhhh-teeee}");

    def = attribute_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("te")))
        .expect("t:te");
    assert_eq!(html_name(&def), "{t:te,data-t-te}");
    def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("te")))
        .expect("te");
    assert_eq!(html_name(&def), "{te}");
    let def2 = attribute_definitions
        .for_html_name(Some(&js("te")))
        .expect("te");
    assert!(Arc::ptr_eq(&def, &def2));
    def = attribute_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("teeee")))
        .expect("t:teeee");
    assert_eq!(html_name(&def), "{t:teeee,data-t-teeee}");

    def = attribute_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("ta")))
        .expect("t:ta");
    assert_eq!(html_name(&def), "{t:ta,data-t-ta}");
    def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("ta")))
        .expect("ta");
    assert_eq!(html_name(&def), "{ta}");
    let def2 = attribute_definitions
        .for_html_name(Some(&js("ta")))
        .expect("ta");
    assert!(Arc::ptr_eq(&def, &def2));

    def = attribute_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("ti")))
        .expect("t:ti");
    assert_eq!(html_name(&def), "{t:ti,data-t-ti}");
    def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("ti")))
        .expect("ti");
    assert_eq!(html_name(&def), "{ti}");
    let def2 = attribute_definitions
        .for_html_name(Some(&js("ti")))
        .expect("ti");
    assert!(Arc::ptr_eq(&def, &def2));

    // t:teeee 全形态同一实例
    let t_teeee = attribute_definitions
        .for_html_name_with_prefix(Some(&js("t")), Some(&js("teeee")))
        .expect("t:teeee");
    let full = attribute_definitions
        .for_html_name(Some(&js("t:teeee")))
        .expect("t:teeee full");
    assert_eq!(html_name(&full), "{t:teeee,data-t-teeee}");
    assert!(Arc::ptr_eq(&t_teeee, &full));
    let null_prefix = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("t:teeee")))
        .expect("t:teeee null prefix");
    assert!(Arc::ptr_eq(&t_teeee, &null_prefix));
    let data_form = attribute_definitions
        .for_html_name(Some(&js("data-t-teeee")))
        .expect("data-t-teeee");
    assert!(Arc::ptr_eq(&t_teeee, &data_form));
    let data_null_prefix = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("data-t-teeee")))
        .expect("data-t-teeee null prefix");
    assert!(Arc::ptr_eq(&t_teeee, &data_null_prefix));
}

#[test]
fn attribute_definitions_data_forms_five_modes() {
    let attribute_definitions = definitions();

    // HTML：data: 前缀折叠
    let def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("data:teeee")))
        .expect("html data:teeee");
    assert_eq!(html_name(&def), "{data:teeee,data-data-teeee}");
    let def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("data")))
        .expect("html data");
    assert_eq!(html_name(&def), "{data}");
    let def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("dataa:teeee")))
        .expect("html dataa:teeee");
    assert_eq!(html_name(&def), "{dataa:teeee,data-dataa-teeee}");
    let def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("data:data")))
        .expect("html data:data");
    assert_eq!(html_name(&def), "{data:data,data-data-data}");
    let def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("DATA:TEEEE")))
        .expect("html DATA:TEEEE");
    assert_eq!(html_name(&def), "{data:teeee,data-data-teeee}");
    let def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("DATA")))
        .expect("html DATA");
    assert_eq!(html_name(&def), "{data}");
    let def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("DATAA:TEEEE")))
        .expect("html DATAA:TEEEE");
    assert_eq!(html_name(&def), "{dataa:teeee,data-dataa-teeee}");
    let def = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("DATA:DATA")))
        .expect("html DATA:DATA");
    assert_eq!(html_name(&def), "{data:data,data-data-data}");

    // XML：大小写敏感
    let def = attribute_definitions
        .for_xml_name_with_prefix(None, Some(&js("data:teeee")))
        .expect("xml data:teeee");
    assert_eq!(xml_name(&def), "{data:teeee}");
    let def = attribute_definitions
        .for_xml_name_with_prefix(None, Some(&js("data")))
        .expect("xml data");
    assert_eq!(xml_name(&def), "{data}");
    let def = attribute_definitions
        .for_xml_name_with_prefix(None, Some(&js("dataa:teeee")))
        .expect("xml dataa:teeee");
    assert_eq!(xml_name(&def), "{dataa:teeee}");
    let def = attribute_definitions
        .for_xml_name_with_prefix(None, Some(&js("data:data")))
        .expect("xml data:data");
    assert_eq!(xml_name(&def), "{data:data}");
    let def = attribute_definitions
        .for_xml_name_with_prefix(None, Some(&js("DATA:TEEEE")))
        .expect("xml DATA:TEEEE");
    assert_eq!(xml_name(&def), "{DATA:TEEEE}");
    let def = attribute_definitions
        .for_xml_name_with_prefix(None, Some(&js("DATA")))
        .expect("xml DATA");
    assert_eq!(xml_name(&def), "{DATA}");
    let def = attribute_definitions
        .for_xml_name_with_prefix(None, Some(&js("DATAA:TEEEE")))
        .expect("xml DATAA:TEEEE");
    assert_eq!(xml_name(&def), "{DATAA:TEEEE}");
    let def = attribute_definitions
        .for_xml_name_with_prefix(None, Some(&js("DATA:DATA")))
        .expect("xml DATA:DATA");
    assert_eq!(xml_name(&def), "{DATA:DATA}");

    // TEXT/JAVASCRIPT/CSS：大小写敏感同 XML
    for mode in ["text", "javascript", "css"] {
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
            let def = match mode {
                "text" => attribute_definitions
                    .for_text_name(Some(&js(input)))
                    .expect("text definition"),
                "javascript" => attribute_definitions
                    .for_javascript_name(Some(&js(input)))
                    .expect("javascript definition"),
                _ => attribute_definitions
                    .for_css_name(Some(&js(input)))
                    .expect("css definition"),
            };
            assert_eq!(text_name(&def), expected, "{mode} {input}");
        }
    }
}

#[test]
fn attribute_definitions_invalid_names() {
    let attribute_definitions = definitions();
    // Java: forXxxName(null, null) / (null, "") / (null, " ") → IllegalArgumentException
    for mode in ["html", "xml", "text", "javascript", "css"] {
        let null_name = match mode {
            "html" => attribute_definitions
                .for_html_name_with_prefix(None, None)
                .is_err(),
            "xml" => attribute_definitions
                .for_xml_name_with_prefix(None, None)
                .is_err(),
            "text" => attribute_definitions.for_text_name(None).is_err(),
            "javascript" => attribute_definitions.for_javascript_name(None).is_err(),
            _ => attribute_definitions.for_css_name(None).is_err(),
        };
        assert!(null_name, "{mode} null name rejected");
        let empty_name = match mode {
            "html" => attribute_definitions
                .for_html_name_with_prefix(None, Some(&js("")))
                .is_err(),
            "xml" => attribute_definitions
                .for_xml_name_with_prefix(None, Some(&js("")))
                .is_err(),
            "text" => attribute_definitions.for_text_name(Some(&js(""))).is_err(),
            "javascript" => attribute_definitions
                .for_javascript_name(Some(&js("")))
                .is_err(),
            _ => attribute_definitions.for_css_name(Some(&js(""))).is_err(),
        };
        assert!(empty_name, "{mode} empty name rejected");
        let blank_name = match mode {
            "html" => attribute_definitions
                .for_html_name_with_prefix(None, Some(&js(" ")))
                .is_err(),
            "xml" => attribute_definitions
                .for_xml_name_with_prefix(None, Some(&js(" ")))
                .is_err(),
            "text" => attribute_definitions.for_text_name(Some(&js(" "))).is_err(),
            "javascript" => attribute_definitions
                .for_javascript_name(Some(&js(" ")))
                .is_err(),
            _ => attribute_definitions.for_css_name(Some(&js(" "))).is_err(),
        };
        assert!(blank_name, "{mode} blank name rejected");
    }
}

#[test]
fn attribute_definitions_empty_null_whitespace_prefix() {
    let attribute_definitions = definitions();
    // 对应 Java testEmptyPrefix/testNullPrefix/testWhitespacePrefix
    for prefix in ["", " ", "   "] {
        let ad01 = attribute_definitions
            .for_html_name_with_prefix(Some(&js(prefix)), Some(&js("one")))
            .expect("html");
        let ad02 = attribute_definitions
            .for_xml_name_with_prefix(Some(&js(prefix)), Some(&js("one")))
            .expect("xml");
        let ad03 = attribute_definitions
            .for_text_name(Some(&js("one")))
            .expect("text");
        let ad04 = attribute_definitions
            .for_javascript_name(Some(&js("one")))
            .expect("javascript");
        let ad05 = attribute_definitions
            .for_css_name(Some(&js("one")))
            .expect("css");
        assert_eq!(html_name(&ad01), "{one}", "html prefix {prefix:?}");
        assert_eq!(xml_name(&ad02), "{one}", "xml prefix {prefix:?}");
        assert_eq!(text_name(&ad03), "{one}");
        assert_eq!(text_name(&ad04), "{one}");
        assert_eq!(text_name(&ad05), "{one}");
    }
    let ad01 = attribute_definitions
        .for_html_name_with_prefix(None, Some(&js("one")))
        .expect("html");
    let ad02 = attribute_definitions
        .for_xml_name_with_prefix(None, Some(&js("one")))
        .expect("xml");
    let ad03 = attribute_definitions
        .for_text_name(Some(&js("one")))
        .expect("text");
    let ad04 = attribute_definitions
        .for_javascript_name(Some(&js("one")))
        .expect("javascript");
    let ad05 = attribute_definitions
        .for_css_name(Some(&js("one")))
        .expect("css");
    assert_eq!(html_name(&ad01), "{one}");
    assert_eq!(xml_name(&ad02), "{one}");
    assert_eq!(text_name(&ad03), "{one}");
    assert_eq!(text_name(&ad04), "{one}");
    assert_eq!(text_name(&ad05), "{one}");
}
