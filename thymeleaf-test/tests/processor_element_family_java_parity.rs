//! `org.thymeleaf.processor.element` 族 Java 1:1 差分测试。
//!
//! 覆盖对象（对象表编号）：
//! - `MatchingAttributeName`（222）：forAttributeName/前缀通配/全属性
//!   匹配规则与 `matches`（Java `AttributeName.equals` 值匹配，HTML
//!   完整名含 `data-th-*` 归一化）；
//! - `MatchingElementName`（223）：forElementName/前缀通配/全元素匹配
//!   与 `matches`（HTML 大小写不敏感）；
//! - `AbstractAttributeTagProcessor`（214）/`AbstractElementTagProcessor`
//!   （216）/`AbstractElementModelProcessor`（215）：构造校验、匹配名、
//!   优先级与类名；
//! - `IElementProcessor`（219）/`IElementTagProcessor`（220）/
//!   `IElementModelProcessor`（217）：对象安全下转与 trait-object 合同。

use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeTagProcessor, AbstractElementModelProcessor, AbstractElementTagProcessor,
    IElementModelProcessor, IElementModelStructureHandler, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::engine::{AttributeNames, ElementNames};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::{IModel, IProcessableElementTag};
use thymeleaf::util::Utf16String;
use thymeleaf::{IProcessor, TemplateMode};

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn attribute_name(name: &str) -> thymeleaf::engine::AttributeNameValue {
    AttributeNames::for_name(Some(TemplateMode::HTML), Some(&js(name))).expect("attribute name")
}

fn element_name(name: &str) -> thymeleaf::engine::ElementNameValue {
    ElementNames::for_name(Some(TemplateMode::HTML), Some(&js(name))).expect("element name")
}

// ===========================================================================
// 1. MatchingAttributeName（222）
// ===========================================================================

#[test]
fn matching_attribute_name_rules_match_java() {
    // Java MatchingAttributeName.forAttributeName(templateMode, name)
    let matching = MatchingAttributeName::for_attribute_name(
        Some(TemplateMode::HTML),
        Some(attribute_name("th:value")),
    )
    .expect("matching attribute name");
    assert_eq!(
        matching
            .to_utf16_string()
            .expect("to java string")
            .to_string_lossy(),
        "{th:value,data-th-value}"
    );

    // matches 按 Java AttributeName.equals 值匹配
    assert!(
        matching
            .matches(Some(&attribute_name("th:value")))
            .expect("match th:value"),
        "exact complete name"
    );
    assert!(
        matching
            .matches(Some(&attribute_name("data-th-value")))
            .expect("match data-th-value"),
        "HTML complete name normalization"
    );
    assert!(
        !matching
            .matches(Some(&attribute_name("th:text")))
            .expect("no match th:text"),
        "different name must not match"
    );

    // Java forAllAttributesWithPrefix("th")：匹配该前缀下全部属性
    let prefixed = MatchingAttributeName::for_all_attributes_with_prefix(
        Some(TemplateMode::HTML),
        Some(js("th")),
    )
    .expect("prefixed rule");
    assert!(
        prefixed
            .matches(Some(&attribute_name("th:text")))
            .expect("prefix th:text"),
        "prefix rule matches th:*"
    );
    // Java：data-th-text 的完整名集合含 th:text，前缀规则同样命中
    assert!(
        prefixed
            .matches(Some(&attribute_name("data-th-text")))
            .expect("match data-th-text"),
        "prefix rule checks all complete names (incl. data-th-*)"
    );
    assert!(
        !prefixed
            .matches(Some(&attribute_name("id")))
            .expect("no match id"),
        "unprefixed attribute must not match"
    );

    // Java forAllAttributes()：匹配任意属性
    let any = MatchingAttributeName::for_all_attributes(Some(TemplateMode::HTML))
        .expect("all attributes rule");
    assert!(
        any.matches(Some(&attribute_name("id"))).expect("any id"),
        "wildcard matches id"
    );
    assert!(
        any.matches(Some(&attribute_name("th:text")))
            .expect("any th:text"),
        "wildcard matches th:*"
    );

    // Java Validate.notNull：null 名称拒绝
    let error = MatchingAttributeName::for_attribute_name(Some(TemplateMode::HTML), None)
        .err()
        .expect("null name rejected");
    assert_eq!(error.to_string(), "Matching attribute name cannot be null");
}

// ===========================================================================
// 2. MatchingElementName（223）
// ===========================================================================

#[test]
fn matching_element_name_rules_match_java() {
    let matching =
        MatchingElementName::for_element_name(Some(TemplateMode::HTML), Some(element_name("div")))
            .expect("matching element name");
    assert_eq!(
        matching
            .to_utf16_string()
            .expect("to java string")
            .to_string_lossy(),
        "{div}"
    );

    // HTML 大小写不敏感：DIV 匹配 div
    assert!(
        matching
            .matches(Some(&element_name("div")))
            .expect("match div"),
        "exact name"
    );
    assert!(
        matching
            .matches(Some(&element_name("DIV")))
            .expect("match DIV"),
        "HTML case-insensitive"
    );
    assert!(
        !matching
            .matches(Some(&element_name("span")))
            .expect("no match span"),
        "different element must not match"
    );

    // Java forAllElementsWithPrefix("th")：th:block 等
    let prefixed =
        MatchingElementName::for_all_elements_with_prefix(Some(TemplateMode::HTML), Some(js("th")))
            .expect("prefixed rule");
    assert!(
        prefixed
            .matches(Some(&element_name("th:block")))
            .expect("prefix th:block"),
        "prefix rule matches th:*"
    );
    assert!(
        !prefixed
            .matches(Some(&element_name("div")))
            .expect("no match div"),
        "unprefixed element must not match"
    );

    // Java forAllElements()
    let any = MatchingElementName::for_all_elements(Some(TemplateMode::HTML)).expect("all rule");
    assert!(
        any.matches(Some(&element_name("div"))).expect("any div"),
        "wildcard matches div"
    );

    // null 名称拒绝（Java Validate.notNull）
    let error = MatchingElementName::for_element_name(Some(TemplateMode::HTML), None)
        .err()
        .expect("null name rejected");
    assert_eq!(error.to_string(), "Matching element name cannot be null");
}

// ===========================================================================
// 3. AbstractAttributeTagProcessor（214）
// ===========================================================================

type TagProcessFn = dyn Fn(
        &dyn ITemplateContext,
        &dyn IProcessableElementTag,
        &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>
    + Send
    + Sync;

fn noop_tag_process() -> Box<TagProcessFn> {
    Box::new(
        |_ctx: &dyn ITemplateContext,
         _tag: &dyn IProcessableElementTag,
         _structure_handler: &mut dyn IElementTagStructureHandler|
         -> Result<(), Box<dyn TemplateEngineException>> { Ok(()) },
    )
}

type AttrTagProcessFn = dyn Fn(
        &dyn ITemplateContext,
        &dyn IProcessableElementTag,
        &thymeleaf::engine::AttributeName,
        Option<Utf16String>,
        &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>
    + Send
    + Sync;

fn noop_attr_tag_process() -> Box<AttrTagProcessFn> {
    Box::new(
        |_ctx: &dyn ITemplateContext,
         _tag: &dyn IProcessableElementTag,
         _attribute_name: &thymeleaf::engine::AttributeName,
         _attribute_value: Option<Utf16String>,
         _structure_handler: &mut dyn IElementTagStructureHandler|
         -> Result<(), Box<dyn TemplateEngineException>> { Ok(()) },
    )
}

#[test]
fn abstract_attribute_tag_processor_contract_matches_java() {
    let processor = AbstractAttributeTagProcessor::new(
        Some(TemplateMode::HTML),
        Some(js("th")),
        None,
        false,
        Some(js("mytagattr")),
        true,
        1000,
        true,
        "com.example.MyTagAttrProcessor",
        noop_attr_tag_process(),
    )
    .expect("valid processor");

    assert_eq!(processor.get_template_mode(), Some(TemplateMode::HTML));
    assert_eq!(processor.get_precedence(), 1000);
    assert_eq!(processor.class_name(), "com.example.MyTagAttrProcessor");
    assert_eq!(
        processor
            .get_matching_attribute_name()
            .expect("matching attribute name")
            .to_utf16_string()
            .expect("matching text")
            .to_string_lossy(),
        "{th:mytagattr,data-th-mytagattr}"
    );

    // 对象安全下转：IElementTagProcessor 合同可见（Java instanceof 等价）
    assert!(
        processor.as_element_tag_processor().is_some(),
        "attribute tag processor is an IElementTagProcessor"
    );
    assert!(
        processor.as_element_model_processor().is_none(),
        "attribute tag processor is not an IElementModelProcessor"
    );

    // Java Validate：null 属性名拒绝
    let error = AbstractAttributeTagProcessor::new(
        Some(TemplateMode::HTML),
        None,
        None,
        false,
        None,
        true,
        1000,
        true,
        "com.example.Bad",
        noop_attr_tag_process(),
    )
    .err()
    .expect("null attribute name rejected");
    assert_eq!(
        error.to_string(),
        "Attribute name cannot be null or empty in Attribute Tag Processor"
    );
}

// ===========================================================================
// 4. AbstractElementTagProcessor（216）
// ===========================================================================

#[test]
fn abstract_element_tag_processor_contract_matches_java() {
    let processor = AbstractElementTagProcessor::new(
        Some(TemplateMode::HTML),
        Some(js("th")),
        Some(js("myelem")),
        true,
        None,
        false,
        100,
        "com.example.MyElementProcessor",
        noop_tag_process(),
    )
    .expect("valid processor");

    assert_eq!(processor.get_template_mode(), Some(TemplateMode::HTML));
    assert_eq!(processor.get_precedence(), 100);
    assert_eq!(processor.class_name(), "com.example.MyElementProcessor");
    assert_eq!(
        processor
            .get_matching_element_name()
            .expect("matching element name")
            .to_utf16_string()
            .expect("matching text")
            .to_string_lossy(),
        "{th:myelem,th-myelem}"
    );
    assert!(
        processor.as_element_tag_processor().is_some(),
        "element tag processor is an IElementTagProcessor"
    );

    // Java Validate：null 模板模式拒绝
    let error = AbstractElementTagProcessor::new(
        None,
        None,
        Some(js("div")),
        false,
        None,
        false,
        100,
        "com.example.Bad",
        noop_tag_process(),
    )
    .err()
    .expect("null template mode rejected");
    assert_eq!(error.to_string(), "Template mode cannot be null");
}

// ===========================================================================
// 5. AbstractElementModelProcessor（215）
// ===========================================================================

type ModelProcessFn = dyn Fn(
        &dyn ITemplateContext,
        &mut dyn IModel,
        &mut dyn IElementModelStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>
    + Send
    + Sync;

fn noop_model_process() -> Box<ModelProcessFn> {
    Box::new(
        |_ctx: &dyn ITemplateContext,
         _model: &mut dyn IModel,
         _structure_handler: &mut dyn IElementModelStructureHandler|
         -> Result<(), Box<dyn TemplateEngineException>> { Ok(()) },
    )
}

#[test]
fn abstract_element_model_processor_contract_matches_java() {
    let processor = AbstractElementModelProcessor::new(
        Some(TemplateMode::HTML),
        Some(js("th")),
        Some(js("mymodelelem")),
        true,
        None,
        false,
        200,
        "com.example.MyModelElementProcessor",
        noop_model_process(),
    )
    .expect("valid processor");

    assert_eq!(processor.get_template_mode(), Some(TemplateMode::HTML));
    assert_eq!(processor.get_precedence(), 200);
    assert_eq!(
        processor.class_name(),
        "com.example.MyModelElementProcessor"
    );
    assert_eq!(
        processor
            .get_matching_element_name()
            .expect("matching element name")
            .to_utf16_string()
            .expect("matching text")
            .to_string_lossy(),
        "{th:mymodelelem,th-mymodelelem}"
    );
    assert!(
        processor.as_element_model_processor().is_some(),
        "element model processor is an IElementModelProcessor"
    );
}

// ===========================================================================
// 6. 接口 trait-object 合同（219/220/217）
// ===========================================================================

#[test]
fn element_processor_interfaces_downcast_match_java() {
    let tag_processor = AbstractElementTagProcessor::new(
        Some(TemplateMode::HTML),
        Some(js("th")),
        Some(js("div")),
        true,
        None,
        false,
        100,
        "com.example.InterfaceProbe",
        noop_tag_process(),
    )
    .expect("valid processor");

    // Java `instanceof IElementProcessor`：静态向上转型获得 trait-object
    // （泛型处理器不覆盖对象安全 as_element_processor，见 RUST_OBLIGATION）
    let element: &dyn thymeleaf::element::IElementProcessor = &tag_processor;
    assert_eq!(element.get_template_mode(), Some(TemplateMode::HTML));

    // Java `instanceof IElementTagProcessor`：tag 处理器下转
    let tag: &dyn IElementTagProcessor = element
        .as_element_tag_processor()
        .expect("tag processor interface");
    assert_eq!(
        tag.get_matching_element_name()
            .expect("matching element name")
            .to_utf16_string()
            .expect("matching text")
            .to_string_lossy(),
        "{th:div,th-div}"
    );

    // 模型处理器是 IElementModelProcessor 而非 IElementTagProcessor
    let model_processor = AbstractElementModelProcessor::new(
        Some(TemplateMode::HTML),
        Some(js("th")),
        Some(js("mymodel")),
        true,
        None,
        false,
        200,
        "com.example.InterfaceProbeModel",
        noop_model_process(),
    )
    .expect("valid processor");
    let element: &dyn thymeleaf::element::IElementProcessor = &model_processor;
    let model: &dyn IElementModelProcessor = element
        .as_element_model_processor()
        .expect("model processor interface");
    assert_eq!(
        model
            .get_matching_element_name()
            .expect("matching element name")
            .to_utf16_string()
            .expect("matching text")
            .to_string_lossy(),
        "{th:mymodel,th-mymodel}"
    );
    assert!(
        element.as_element_tag_processor().is_none(),
        "model processor is not a tag processor"
    );
}
