//! Decoupled Template Logic Java Golden 差分测试。
//!
//! 覆盖：`DecoupledTemplateLogic` 注入属性容器、
//! `DecoupledInjectedAttribute` 属性解析（buffer+offset API）、
//! `StandardDecoupledTemplateLogicResolver` 配置，
//! 以及 .th.xml 解耦逻辑文件的端到端注入渲染。

use std::sync::Arc;

use thymeleaf::decoupled::{
    DecoupledInjectedAttribute, DecoupledTemplateLogic, StandardDecoupledTemplateLogicResolver,
};
use thymeleaf::util::JavaString;

fn js(s: &str) -> JavaString {
    JavaString::from_rust_str(s)
}

// ===========================================================================
// 1. DecoupledTemplateLogic 容器
// ===========================================================================

#[test]
fn empty_logic_has_no_injected_attributes() {
    let logic = DecoupledTemplateLogic::new();
    assert!(!logic.has_injected_attributes());
    assert!(logic.get_all_injected_attribute_selectors().is_empty());
    assert!(
        logic
            .get_injected_attributes_for_selector(&js("form"))
            .is_none()
    );
}

#[test]
fn add_injected_attribute_then_query() {
    let units: Vec<u16> = "th:class=greatclass".encode_utf16().collect();
    let attribute = Arc::new(
        DecoupledInjectedAttribute::create_attribute(
            Some(&units),
            0,
            8, // name "th:class"
            8,
            0, // 无 operator
            9,
            10, // value content "greatclass"
            9,
            10, // value outer "greatclass"
        )
        .expect("valid attribute"),
    );
    let mut logic = DecoupledTemplateLogic::new();
    logic.add_injected_attribute(js("form"), attribute);

    assert!(logic.has_injected_attributes());
    let selectors = logic.get_all_injected_attribute_selectors();
    assert_eq!(selectors.len(), 1);
    assert_eq!(selectors[0].to_string_lossy(), "form");
    let attrs = logic
        .get_injected_attributes_for_selector(&js("form"))
        .expect("selector present");
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].get_name().unwrap().to_string_lossy(), "th:class");
}

#[test]
fn multiple_attributes_preserve_order() {
    let mk = |text: &str, name_len: i32, value_offset: i32, value_len: i32| {
        let units: Vec<u16> = text.encode_utf16().collect();
        Arc::new(
            DecoupledInjectedAttribute::create_attribute(
                Some(&units),
                0,
                name_len,
                name_len,
                0,
                value_offset,
                value_len,
                value_offset,
                value_len,
            )
            .expect("valid attribute"),
        )
    };
    let a1 = mk("th:text='one'", 7, 8, 5);
    let a2 = mk("th:class='two'", 8, 9, 5);
    let mut logic = DecoupledTemplateLogic::new();
    logic.add_injected_attribute(js("div"), a1);
    logic.add_injected_attribute(js("div"), a2);
    let attrs = logic
        .get_injected_attributes_for_selector(&js("div"))
        .expect("selector present");
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].get_name().unwrap().to_string_lossy(), "th:text");
    assert_eq!(attrs[1].get_name().unwrap().to_string_lossy(), "th:class");
}

#[test]
fn multiple_selectors_isolated() {
    let units: Vec<u16> = "id=x".encode_utf16().collect();
    let attr = Arc::new(
        DecoupledInjectedAttribute::create_attribute(Some(&units), 0, 2, 2, 0, 3, 1, 3, 1)
            .expect("attr"),
    );
    let mut logic = DecoupledTemplateLogic::new();
    logic.add_injected_attribute(js("a"), Arc::clone(&attr));
    logic.add_injected_attribute(js("b"), attr);
    assert!(
        logic
            .get_injected_attributes_for_selector(&js("a"))
            .is_some()
    );
    assert!(
        logic
            .get_injected_attributes_for_selector(&js("b"))
            .is_some()
    );
    assert!(
        logic
            .get_injected_attributes_for_selector(&js("c"))
            .is_none()
    );
}

// ===========================================================================
// 2. DecoupledInjectedAttribute 错误路径
// ===========================================================================

#[test]
fn create_attribute_null_buffer_errors() {
    assert!(DecoupledInjectedAttribute::create_attribute(None, 0, 0, 0, 0, 0, 0, 0, 0).is_err());
}

#[test]
fn create_attribute_negative_length_errors() {
    let units: Vec<u16> = "ab".encode_utf16().collect();
    assert!(
        DecoupledInjectedAttribute::create_attribute(Some(&units), 0, 1, 0, 1, 0, 1, 0, i32::MIN,)
            .is_err()
    );
}

#[test]
fn create_attribute_out_of_range_errors() {
    let units: Vec<u16> = "ab".encode_utf16().collect();
    assert!(
        DecoupledInjectedAttribute::create_attribute(Some(&units), 0, 10, 0, 0, 0, 0, 0, 0)
            .is_err()
    );
}

#[test]
fn create_attribute_boolean_style() {
    // 布尔属性：整个文本都是名称，value 为空
    let units: Vec<u16> = "thefirstlabel".encode_utf16().collect();
    let attr = DecoupledInjectedAttribute::create_attribute(Some(&units), 0, 13, 0, 0, 0, 0, 0, 0)
        .expect("boolean attribute");
    assert_eq!(attr.get_name().unwrap().to_string_lossy(), "thefirstlabel");
}

#[test]
fn attribute_to_java_string_contains_name() {
    let units: Vec<u16> = "th:class=greatclass".encode_utf16().collect();
    let attr = DecoupledInjectedAttribute::create_attribute(Some(&units), 0, 8, 8, 0, 9, 10, 9, 10)
        .expect("valid");
    let repr = attr.to_java_string().to_string_lossy();
    assert!(repr.contains("th:class"), "name must be present: {repr}");
}

// ===========================================================================
// 3. StandardDecoupledTemplateLogicResolver
// ===========================================================================

#[test]
fn resolver_default_suffix() {
    let resolver = StandardDecoupledTemplateLogicResolver::new();
    assert_eq!(resolver.get_suffix().unwrap().to_string_lossy(), ".th.xml");
    assert_eq!(
        StandardDecoupledTemplateLogicResolver::DECOUPLED_TEMPLATE_LOGIC_FILE_SUFFIX,
        ".th.xml"
    );
}

#[test]
fn resolver_set_suffix() {
    let resolver = StandardDecoupledTemplateLogicResolver::new();
    resolver.set_suffix(Some(js(".logic.xml")));
    assert_eq!(
        resolver.get_suffix().unwrap().to_string_lossy(),
        ".logic.xml"
    );
    resolver.set_suffix(None);
    assert!(resolver.get_suffix().is_none());
}

#[test]
fn resolver_prefix_configuration() {
    let resolver = StandardDecoupledTemplateLogicResolver::new();
    assert!(resolver.get_prefix().is_none());
    resolver.set_prefix(Some(js("logic_")));
    assert_eq!(resolver.get_prefix().unwrap().to_string_lossy(), "logic_");
}

// ===========================================================================
// 4. 端到端：.th.xml 解耦逻辑注入
// ===========================================================================

#[test]
fn decoupled_logic_file_end_to_end() {
    use thymeleaf::context::Context;
    use thymeleaf::templateresolver::FileTemplateResolver;
    use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

    let temp_dir = std::env::temp_dir().join(format!("thymeleaf-decoupled-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");
    let template_path = temp_dir.join("main.html");
    let logic_path = temp_dir.join("main.th.xml");
    std::fs::write(
        &template_path,
        "<!DOCTYPE html><html><body><p id=\"message\">original</p></body></html>",
    )
    .expect("write template");
    std::fs::write(
        &logic_path,
        "<?xml version=\"1.0\"?>\n<thlogic>\n  <attr sel=\"#message\" th:text=\"'injected value'\" />\n</thlogic>\n",
    )
    .expect("write logic");

    let mut resolver = FileTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    resolver.set_prefix(Some(JavaString::from_rust_str(&format!(
        "{}/",
        temp_dir.display()
    ))));
    resolver.set_use_decoupled_logic(true);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let ctx = Context::new();

    let result = engine.process_template("main.html", &ctx);
    let output = result.expect("decoupled template must render");
    let s = output.to_string_lossy();
    assert!(
        s.contains("injected value"),
        "decoupled-injected th:text must replace original text: {s}"
    );
    assert!(
        !s.contains(">original<"),
        "original text must be replaced: {s}"
    );

    let _ = std::fs::remove_file(&template_path);
    let _ = std::fs::remove_file(&logic_path);
    let _ = std::fs::remove_dir(&temp_dir);
}

#[test]
fn decoupled_logic_disabled_keeps_original() {
    use thymeleaf::context::Context;
    use thymeleaf::templateresolver::FileTemplateResolver;
    use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

    let temp_dir =
        std::env::temp_dir().join(format!("thymeleaf-decoupled-off-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");
    let template_path = temp_dir.join("main.html");
    let logic_path = temp_dir.join("main.th.xml");
    std::fs::write(&template_path, "<p id=\"message\">original</p>").expect("write template");
    std::fs::write(
        &logic_path,
        "<?xml version=\"1.0\"?>\n<thlogic>\n  <attr sel=\"#message\" th:text=\"'injected value'\" />\n</thlogic>\n",
    )
    .expect("write logic");

    let mut resolver = FileTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    resolver.set_prefix(Some(JavaString::from_rust_str(&format!(
        "{}/",
        temp_dir.display()
    ))));
    // 默认 use_decoupled_logic=false：不查找 .th.xml
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    let ctx = Context::new();

    let output = engine
        .process_template("main.html", &ctx)
        .expect("must render without decoupled logic");
    let s = output.to_string_lossy();
    assert!(
        s.contains("original"),
        "without decoupled logic the original text stays: {s}"
    );

    let _ = std::fs::remove_file(&template_path);
    let _ = std::fs::remove_file(&logic_path);
    let _ = std::fs::remove_dir(&temp_dir);
}
