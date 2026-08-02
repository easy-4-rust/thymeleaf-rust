//! `Attributes`/`Attribute` 运行时语义 Java 1:1 差分测试。
//!
//! 对应上游 `thymeleaf-tests-core` 的
//! `org.thymeleaf.engine.ElementAttributesTest`：解析真实模板片段
//! 获取 Attributes，验证 setAttribute/removeAttribute 全形态
//! （完整名/prefix/AttributeName）、引号形态、空白保留、属性顺序、
//! getAttributeMap 键集与 Attribute 位置信息。

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use thymeleaf::context::ITemplateContext;
use thymeleaf::engine::{
    AttributeDefinitions, AttributeNames, Attributes, ITemplateHandler, TemplateHandlerHandle,
};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::markup::{HTMLTemplateParser, XMLTemplateParser};
use thymeleaf::model::{
    AttributeValueQuotes, IAttribute, ICDATASection, ICloseElementTag, IComment, IDocType,
    IOpenElementTag, IProcessableElementTag, IProcessingInstruction, IStandaloneElementTag,
    ITemplateEnd, ITemplateStart, IText, IXMLDeclaration,
};
use thymeleaf::templateresource::{ITemplateResource, StringTemplateResource};
use thymeleaf::util::JavaString;
use thymeleaf::{ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode};

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn engine() -> TemplateEngine {
    let mut r = thymeleaf::templateresolver::StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

/// 捕获打开/独立标签的解析 Handler（对应 Java
/// ElementAttributeObtentionTemplateHandler）。
struct AttributeObtentionHandler {
    attributes: Rc<RefCell<Option<Arc<Attributes>>>>,
}

impl ITemplateHandler for AttributeObtentionHandler {
    fn set_next(&mut self, _next: Option<TemplateHandlerHandle>) {}
    fn set_context(&mut self, _context: Arc<dyn ITemplateContext>) {}
    fn handle_template_start(
        &mut self,
        _t: Arc<dyn ITemplateStart>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_template_end(
        &mut self,
        _t: Arc<dyn ITemplateEnd>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_xml_declaration(
        &mut self,
        _t: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_doc_type(
        &mut self,
        _t: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_cdata_section(
        &mut self,
        _t: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_comment(
        &mut self,
        _t: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_text(&mut self, _t: Arc<dyn IText>) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_standalone_element(
        &mut self,
        tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(tag) = tag.into_engine_standalone_element_tag()
            && let Some(attributes) = tag
                .as_engine_processable_element_tag()
                .and_then(|tag| tag.attributes())
        {
            *self.attributes.borrow_mut() = Some(attributes.clone());
        }
        Ok(())
    }
    fn handle_open_element(
        &mut self,
        tag: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(tag) = tag.into_engine_open_element_tag()
            && let Some(attributes) = tag
                .as_engine_processable_element_tag()
                .and_then(|tag| tag.attributes())
        {
            *self.attributes.borrow_mut() = Some(attributes.clone());
        }
        Ok(())
    }
    fn handle_close_element(
        &mut self,
        _t: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_processing_instruction(
        &mut self,
        _t: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
}

fn compute_attributes(input: &str, mode: TemplateMode) -> Arc<Attributes> {
    let parser: Arc<dyn thymeleaf::templateparser::ITemplateParser> = match mode {
        TemplateMode::HTML => Arc::new(HTMLTemplateParser::new(2, 4096)),
        TemplateMode::XML => Arc::new(XMLTemplateParser::new(2, 4096)),
        _ => panic!("element attributes tests only cover HTML/XML"),
    };
    let config = engine().get_configuration().expect("config");
    let resource: Arc<dyn ITemplateResource> =
        Arc::new(StringTemplateResource::new(Some(input)).expect("string resource"));
    let attributes: Rc<RefCell<Option<Arc<Attributes>>>> = Rc::new(RefCell::new(None));
    let handler = Box::new(AttributeObtentionHandler {
        attributes: attributes.clone(),
    });
    parser
        .parse_standalone(
            config,
            None,
            &js("test"),
            None,
            resource,
            mode,
            false,
            handler,
        )
        .expect("parse");
    attributes
        .borrow_mut()
        .take()
        .unwrap_or_else(Attributes::empty)
}

fn html_attrs(input: &str) -> Arc<Attributes> {
    compute_attributes(input, TemplateMode::HTML)
}

fn xml_attrs(input: &str) -> Arc<Attributes> {
    compute_attributes(input, TemplateMode::XML)
}

/// Java `Attributes#toString()`。
fn attrs_text(attrs: &Arc<Attributes>) -> String {
    attrs.to_java_string().to_string_lossy()
}

/// Java `getAttributeMap().keySet().toString()`。
fn map_keys(attrs: &Arc<Attributes>) -> String {
    let keys = attrs
        .get_attribute_map()
        .keys()
        .map(|key| key.to_string_lossy())
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{keys}]")
}

fn empty_attribute_definitions() -> AttributeDefinitions {
    AttributeDefinitions::new(std::collections::HashMap::new()).expect("attribute definitions")
}

fn html_name(name: &str) -> thymeleaf::engine::AttributeNameValue {
    AttributeNames::for_name(Some(TemplateMode::HTML), Some(&js(name)))
        .expect("html attribute name")
}

fn xml_name(name: &str) -> thymeleaf::engine::AttributeNameValue {
    AttributeNames::for_name(Some(TemplateMode::XML), Some(&js(name))).expect("xml attribute name")
}

// ===========================================================================
// 1. ElementAttributesTest#testHtmlElementAttributesAttrManagement
// ===========================================================================

#[test]
fn html_element_attributes_attr_management() {
    let attribute_definitions = empty_attribute_definitions();

    let mut attrs = html_attrs("<input>");
    assert_eq!(attrs_text(&attrs), "");

    attrs = html_attrs("<input type=\"text\">");
    assert_eq!(map_keys(&attrs), "[type]");
    assert_eq!(attrs_text(&attrs), " type=\"text\"");

    attrs = html_attrs("<input type=\"text\"   value='hello!!!'>");
    assert_eq!(map_keys(&attrs), "[type, value]");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   value='hello!!!'");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("type"))
        .expect("remove type");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'");
    assert_eq!(map_keys(&attrs), "[value]");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("value"))
        .expect("remove value");
    assert_eq!(attrs_text(&attrs), "");
    assert_eq!(map_keys(&attrs), "[]");

    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    >");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   value='hello!!!'    ");
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::HTML, None, &js("type"))
        .expect("remove type");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'    ");
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::HTML, None, &js("value"))
        .expect("remove value");
    assert_eq!(attrs_text(&attrs), "    ");

    attrs = html_attrs("<input th:type=\"text\"   th:value='hello!!!'    >");
    assert_eq!(
        attrs_text(&attrs),
        " th:type=\"text\"   th:value='hello!!!'    "
    );
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::HTML, Some(&js("th")), &js("type"))
        .expect("remove th:type");
    assert_eq!(attrs_text(&attrs), " th:value='hello!!!'    ");
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::HTML, Some(&js("th")), &js("value"))
        .expect("remove th:value");
    assert_eq!(attrs_text(&attrs), "    ");

    // HTML 大小写不敏感前缀
    attrs = html_attrs("<input th:type=\"text\"   th:value='hello!!!'    >");
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::HTML, Some(&js("TH")), &js("TYPE"))
        .expect("remove TH:TYPE");
    assert_eq!(attrs_text(&attrs), " th:value='hello!!!'    ");
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::HTML, Some(&js("tH")), &js("Value"))
        .expect("remove tH:Value");
    assert_eq!(attrs_text(&attrs), "    ");

    // data-th- 折叠删除
    attrs = html_attrs("<input th:type=\"text\"   th:value='hello!!!'    >");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("data-th-type"))
        .expect("remove data-th-type");
    assert_eq!(attrs_text(&attrs), " th:value='hello!!!'    ");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("data-th-value"))
        .expect("remove data-th-value");
    assert_eq!(attrs_text(&attrs), "    ");

    // AttributeName 删除
    attrs = html_attrs("<input th:type=\"text\"   th:value='hello!!!'    >");
    attrs = attrs.remove_attribute_name(&html_name("th:type"));
    assert_eq!(attrs_text(&attrs), " th:value='hello!!!'    ");
    attrs = attrs.remove_attribute_name(&html_name("th:value"));
    assert_eq!(attrs_text(&attrs), "    ");

    attrs = html_attrs("<input th:type=\"text\"   th:value='hello!!!'    >");
    attrs = attrs.remove_attribute_name(&html_name("th:type"));
    assert_eq!(attrs_text(&attrs), " th:value='hello!!!'    ");
    attrs = attrs.remove_attribute_name(&html_name("TH:VALUE"));
    assert_eq!(attrs_text(&attrs), "    ");

    // 无值属性与空白
    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba>");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba"
    );
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("value"))
        .expect("remove value");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   ba");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("ba"))
        .expect("remove ba");
    assert_eq!(attrs_text(&attrs), " type=\"text\"");

    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba>");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("value"))
        .expect("remove value");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   ba");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("type"))
        .expect("remove type");
    assert_eq!(attrs_text(&attrs), " ba");

    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba >");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba "
    );
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("value"))
        .expect("remove value");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   ba ");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("ba"))
        .expect("remove ba");
    assert_eq!(attrs_text(&attrs), " type=\"text\" ");

    // setAttribute 引号形态
    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba >");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("value"),
            Some(js("bye! :(")),
            None,
        )
        .expect("set value");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='bye! :('    ba "
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("type"),
            Some(js("one")),
            None,
        )
        .expect("set type");
    assert_eq!(attrs_text(&attrs), " type=\"one\"   value='bye! :('    ba ");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            Some(js("two")),
            None,
        )
        .expect("set ba");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"one\"   value='bye! :('    ba=\"two\" "
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            Some(js("three")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set ba single");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"one\"   value='bye! :('    ba='three' "
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            Some(js("four")),
            Some(AttributeValueQuotes::NONE),
        )
        .expect("set ba none");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"one\"   value='bye! :('    ba=four "
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            Some(js("five")),
            None,
        )
        .expect("set ba five");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"one\"   value='bye! :('    ba=five "
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            None,
            None,
        )
        .expect("set ba null");
    assert_eq!(attrs_text(&attrs), " type=\"one\"   value='bye! :('    ba ");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            Some(js("six")),
            None,
        )
        .expect("set ba six");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"one\"   value='bye! :('    ba=\"six\" "
    );

    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba=twenty >");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            Some(js("thirty")),
            None,
        )
        .expect("set ba thirty");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba=thirty "
    );

    // 无间隔属性
    attrs = html_attrs("<input type=\"text\"value='hello!!!' >");
    assert_eq!(attrs_text(&attrs), " type=\"text\"value='hello!!!' ");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("type"))
        .expect("remove type");
    assert_eq!(attrs_text(&attrs), " value='hello!!!' ");

    attrs = html_attrs("<input type=\"text\"value='hello!!!' name='one' >");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"value='hello!!!' name='one' "
    );
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("type"))
        .expect("remove type");
    assert_eq!(attrs_text(&attrs), " value='hello!!!' name='one' ");

    attrs = html_attrs("<input type=\"text\"value='hello!!!' name='one'>");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"value='hello!!!' name='one'"
    );
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("name"))
        .expect("remove name");
    assert_eq!(attrs_text(&attrs), " type=\"text\"value='hello!!!'");

    // 空 Attributes 构建
    attrs = Attributes::new(None, None);
    assert_eq!(attrs_text(&attrs), "");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("name"),
            Some(js("onename")),
            None,
        )
        .expect("set name");
    assert_eq!(attrs_text(&attrs), " name=\"onename\"");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("value"),
            Some(js("val")),
            None,
        )
        .expect("set value");
    assert_eq!(attrs_text(&attrs), " name=\"onename\" value=\"val\"");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("placeholder"),
            None,
            None,
        )
        .expect("set placeholder");
    assert_eq!(
        attrs_text(&attrs),
        " name=\"onename\" value=\"val\" placeholder"
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("placeholder"),
            Some(js("a")),
            None,
        )
        .expect("set placeholder a");
    assert_eq!(
        attrs_text(&attrs),
        " name=\"onename\" value=\"val\" placeholder=\"a\""
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("value"),
            None,
            None,
        )
        .expect("set value null");
    assert_eq!(
        attrs_text(&attrs),
        " name=\"onename\" value placeholder=\"a\""
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("name"),
            Some(js("")),
            None,
        )
        .expect("set name empty");
    assert_eq!(attrs_text(&attrs), " name=\"\" value placeholder=\"a\"");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("name"),
            Some(js("")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set name empty single");
    assert_eq!(attrs_text(&attrs), " name='' value placeholder=\"a\"");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("name"),
            None,
            None,
        )
        .expect("set name null");
    assert_eq!(attrs_text(&attrs), " name value placeholder=\"a\"");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("name"),
            Some(js("")),
            None,
        )
        .expect("set name empty again");
    assert_eq!(attrs_text(&attrs), " name=\"\" value placeholder=\"a\"");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("name"))
        .expect("remove name");
    assert_eq!(attrs_text(&attrs), " value placeholder=\"a\"");
    assert_eq!(attrs.get_all_attributes().len(), 2);
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("name"),
            Some(js("")),
            None,
        )
        .expect("set name append");
    assert_eq!(map_keys(&attrs), "[value, placeholder, name]");
    assert_eq!(attrs_text(&attrs), " value placeholder=\"a\" name=\"\"");

    // 追加顺序
    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba>");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            Some(js("one")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set ba one");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba='one'"
    );
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("ba"))
        .expect("remove ba");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   value='hello!!!'");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            Some(js("two")),
            None,
        )
        .expect("set ba two");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"two\""
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("be"),
            Some(js("three")),
            None,
        )
        .expect("set be");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"two\" be=\"three\""
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("bi"),
            Some(js("four")),
            None,
        )
        .expect("set bi");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"two\" be=\"three\" bi=\"four\""
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("bo"),
            Some(js("five")),
            None,
        )
        .expect("set bo");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"two\" be=\"three\" bi=\"four\" bo=\"five\""
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("bu"),
            Some(js("six")),
            None,
        )
        .expect("set bu");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"two\" be=\"three\" bi=\"four\" bo=\"five\" bu=\"six\""
    );
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("be"))
        .expect("remove be");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("bu"))
        .expect("remove bu");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"two\" bi=\"four\" bo=\"five\""
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("bi"),
            None,
            None,
        )
        .expect("set bi null");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"two\" bi bo=\"five\""
    );
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("bi"))
        .expect("remove bi");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"two\" bo=\"five\""
    );
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("type"))
        .expect("remove type");
    assert_eq!(
        attrs_text(&attrs),
        " value='hello!!!' ba=\"two\" bo=\"five\""
    );
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("type"))
        .expect("remove type again");
    assert_eq!(
        attrs_text(&attrs),
        " value='hello!!!' ba=\"two\" bo=\"five\""
    );

    // 新属性追加引号形态
    attrs = html_attrs("<input>");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("a"),
            Some(js("one")),
            None,
        )
        .expect("set a");
    assert_eq!(attrs_text(&attrs), " a=\"one\"");

    attrs = html_attrs("<input>");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("a"),
            Some(js("one")),
            Some(AttributeValueQuotes::NONE),
        )
        .expect("set a none");
    assert_eq!(attrs_text(&attrs), " a=one");

    attrs = html_attrs("<input   >");
    assert_eq!(attrs_text(&attrs), "   ");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("a"),
            Some(js("one")),
            None,
        )
        .expect("set a");
    assert_eq!(attrs_text(&attrs), " a=\"one\"   ");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("b"),
            Some(js("two")),
            None,
        )
        .expect("set b");
    assert_eq!(attrs_text(&attrs), " a=\"one\" b=\"two\"   ");

    attrs = html_attrs("<input\none  />");
    assert_eq!(attrs_text(&attrs), "\none  ");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("a"),
            Some(js("two")),
            None,
        )
        .expect("set a");
    assert_eq!(attrs_text(&attrs), "\none a=\"two\"  ");

    attrs = html_attrs("<input\none two/>");
    assert_eq!(attrs_text(&attrs), "\none two");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("one"))
        .expect("remove one");
    assert_eq!(attrs_text(&attrs), "\ntwo");
}

// ===========================================================================
// 2. ElementAttributesTest#testXmlElementAttributesAttrManagement
// ===========================================================================

#[test]
fn xml_element_attributes_attr_management() {
    let attribute_definitions = empty_attribute_definitions();

    let mut attrs = xml_attrs("<input/>");
    assert_eq!(attrs_text(&attrs), "");

    attrs = xml_attrs("<input type=\"text\"/>");
    assert_eq!(attrs_text(&attrs), " type=\"text\"");

    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'/>");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   value='hello!!!'");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("type"))
        .expect("remove type");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("value"))
        .expect("remove value");
    assert_eq!(attrs_text(&attrs), "");

    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    />");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   value='hello!!!'    ");
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::XML, None, &js("type"))
        .expect("remove type");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'    ");
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::XML, None, &js("value"))
        .expect("remove value");
    assert_eq!(attrs_text(&attrs), "    ");

    attrs = xml_attrs("<input th:type=\"text\"   th:value='hello!!!'    />");
    assert_eq!(
        attrs_text(&attrs),
        " th:type=\"text\"   th:value='hello!!!'    "
    );
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::XML, Some(&js("th")), &js("type"))
        .expect("remove th:type");
    assert_eq!(attrs_text(&attrs), " th:value='hello!!!'    ");
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::XML, Some(&js("th")), &js("value"))
        .expect("remove th:value");
    assert_eq!(attrs_text(&attrs), "    ");

    // XML 大小写敏感：TH:TYPE 不匹配
    attrs = xml_attrs("<input th:type=\"text\"   th:value='hello!!!'    />");
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::XML, Some(&js("TH")), &js("TYPE"))
        .expect("remove TH:TYPE");
    assert_eq!(
        attrs_text(&attrs),
        " th:type=\"text\"   th:value='hello!!!'    "
    );
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::XML, Some(&js("tH")), &js("Value"))
        .expect("remove tH:Value");
    assert_eq!(
        attrs_text(&attrs),
        " th:type=\"text\"   th:value='hello!!!'    "
    );

    // XML data-th- 不折叠
    attrs = xml_attrs("<input th:type=\"text\"   th:value='hello!!!'    />");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("data-th-type"))
        .expect("remove data-th-type");
    assert_eq!(
        attrs_text(&attrs),
        " th:type=\"text\"   th:value='hello!!!'    "
    );
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("data-th-value"))
        .expect("remove data-th-value");
    assert_eq!(
        attrs_text(&attrs),
        " th:type=\"text\"   th:value='hello!!!'    "
    );

    // AttributeName 删除
    attrs = xml_attrs("<input th:type=\"text\"   th:value='hello!!!'    />");
    attrs = attrs.remove_attribute_name(&xml_name("th:type"));
    assert_eq!(attrs_text(&attrs), " th:value='hello!!!'    ");
    attrs = attrs.remove_attribute_name(&xml_name("th:value"));
    assert_eq!(attrs_text(&attrs), "    ");

    attrs = xml_attrs("<input th:type=\"text\"   th:value='hello!!!'    />");
    assert_eq!(map_keys(&attrs), "[th:type, th:value]");
    attrs = attrs.remove_attribute_name(&xml_name("th:type"));
    assert_eq!(attrs_text(&attrs), " th:value='hello!!!'    ");
    attrs = attrs.remove_attribute_name(&xml_name("TH:VALUE"));
    assert_eq!(attrs_text(&attrs), " th:value='hello!!!'    ");

    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba=''/>");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba=''"
    );
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("value"))
        .expect("remove value");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   ba=''");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("ba"))
        .expect("remove ba");
    assert_eq!(attrs_text(&attrs), " type=\"text\"");

    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba='' />");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba='' "
    );
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("value"))
        .expect("remove value");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   ba='' ");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("ba"))
        .expect("remove ba");
    assert_eq!(attrs_text(&attrs), " type=\"text\" ");

    // XML setAttribute
    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba='' />");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("value"),
            Some(js("bye! :(")),
            None,
        )
        .expect("set value");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='bye! :('    ba='' "
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("type"),
            Some(js("one")),
            None,
        )
        .expect("set type");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"one\"   value='bye! :('    ba='' "
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("ba"),
            Some(js("two")),
            None,
        )
        .expect("set ba");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"one\"   value='bye! :('    ba='two' "
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("ba"),
            Some(js("three")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set ba single");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"one\"   value='bye! :('    ba='three' "
    );

    // XML NONE 引号/null 值非法
    assert!(
        attrs
            .set_attribute(
                &attribute_definitions,
                TemplateMode::XML,
                None,
                js("ba"),
                Some(js("four")),
                Some(AttributeValueQuotes::NONE),
            )
            .is_err(),
        "XML NONE quotes rejected"
    );
    assert!(
        attrs
            .set_attribute(
                &attribute_definitions,
                TemplateMode::XML,
                None,
                js("ba"),
                None,
                Some(AttributeValueQuotes::NONE),
            )
            .is_err(),
        "XML NONE quotes null value rejected"
    );
    assert!(
        attrs
            .set_attribute(
                &attribute_definitions,
                TemplateMode::XML,
                None,
                js("ba"),
                None,
                None
            )
            .is_err(),
        "XML null value rejected"
    );

    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("ba"),
            Some(js("five")),
            None,
        )
        .expect("set ba five");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"one\"   value='bye! :('    ba='five' "
    );

    // 空 Attributes 构建（XML）
    attrs = Attributes::new(None, None);
    assert_eq!(attrs_text(&attrs), "");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("name"),
            Some(js("onename")),
            None,
        )
        .expect("set name");
    assert_eq!(attrs_text(&attrs), " name=\"onename\"");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("value"),
            Some(js("val")),
            None,
        )
        .expect("set value");
    assert_eq!(attrs_text(&attrs), " name=\"onename\" value=\"val\"");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("placeholder"),
            Some(js("a")),
            None,
        )
        .expect("set placeholder");
    assert_eq!(
        attrs_text(&attrs),
        " name=\"onename\" value=\"val\" placeholder=\"a\""
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("name"),
            Some(js("")),
            None,
        )
        .expect("set name empty");
    assert_eq!(
        attrs_text(&attrs),
        " name=\"\" value=\"val\" placeholder=\"a\""
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("name"),
            Some(js("")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set name empty single");
    assert_eq!(
        attrs_text(&attrs),
        " name='' value=\"val\" placeholder=\"a\""
    );
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("name"))
        .expect("remove name");
    assert_eq!(attrs_text(&attrs), " value=\"val\" placeholder=\"a\"");
    assert_eq!(attrs.get_all_attributes().len(), 2);
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("name"),
            Some(js("")),
            None,
        )
        .expect("set name append");
    assert_eq!(
        attrs_text(&attrs),
        " value=\"val\" placeholder=\"a\" name=\"\""
    );

    // XML 追加与引号形态（Java 每个子案例重新解析）
    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba= 's'/>");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("type"))
        .expect("remove type");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'    ba= 's'");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("type"),
            Some(js("")),
            None,
        )
        .expect("set type empty");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'    ba= 's' type=\"\"");

    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba= 's'/>");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("type"))
        .expect("remove type");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("title"),
            Some(js(" ")),
            None,
        )
        .expect("set title space");
    assert_eq!(
        attrs_text(&attrs),
        " value='hello!!!'    ba= 's' title=\" \""
    );

    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba= 's'/>");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("type"))
        .expect("remove type");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("title"),
            Some(js("")),
            None,
        )
        .expect("set title empty");
    assert_eq!(
        attrs_text(&attrs),
        " value='hello!!!'    ba= 's' title=\"\""
    );

    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba= 's'/>");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("type"))
        .expect("remove type");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("title"),
            Some(js("")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set title empty single");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'    ba= 's' title=''");
    assert!(
        attrs
            .set_attribute(
                &attribute_definitions,
                TemplateMode::XML,
                None,
                js("title"),
                Some(js("")),
                Some(AttributeValueQuotes::NONE),
            )
            .is_err(),
        "XML empty string with NONE quotes rejected"
    );
}

// ===========================================================================
// 3. Attribute 位置信息（getAttribute(...).line/.col/.hasLocation）
// ===========================================================================

#[test]
fn attribute_location_information() {
    let attribute_definitions = empty_attribute_definitions();

    // HTML 位置：<input type='text' \nth:type="${thetype}">
    let attrs = html_attrs("<input type='text' \nth:type=\"${thetype}\">");
    let type_attr = attrs
        .get_attribute(TemplateMode::HTML, &js("type"))
        .expect("get type")
        .expect("type exists");
    assert_eq!(type_attr.get_line(), 1);
    assert_eq!(type_attr.get_col(), 8);
    let th_type = attrs
        .get_attribute_with_prefix(TemplateMode::HTML, Some(&js("th")), &js("type"))
        .expect("get th:type")
        .expect("th:type exists");
    assert_eq!(th_type.get_line(), 2);
    assert_eq!(th_type.get_col(), 1);

    // HTML：删除 a 后 th:type 列位置变为 7
    let attrs = html_attrs("<input type='text' \na=\"b\" th:type=\"${thetype}\">");
    let attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("a"))
        .expect("remove a");
    let th_type = attrs
        .get_attribute_with_prefix(TemplateMode::HTML, Some(&js("th")), &js("type"))
        .expect("get th:type")
        .expect("th:type exists");
    assert_eq!(th_type.get_line(), 2);
    assert_eq!(th_type.get_col(), 7);

    // XML 位置：<input type='text' \na="b" th:type="${thetype}"/>
    let attrs = xml_attrs("<input type='text' \na=\"b\" th:type=\"${thetype}\"/>");
    let type_attr = attrs
        .get_attribute(TemplateMode::XML, &js("type"))
        .expect("get type")
        .expect("type exists");
    assert_eq!(type_attr.get_line(), 1);
    assert_eq!(type_attr.get_col(), 8);
    let th_type = attrs
        .get_attribute_with_prefix(TemplateMode::XML, Some(&js("th")), &js("type"))
        .expect("get th:type")
        .expect("th:type exists");
    assert_eq!(th_type.get_line(), 2);
    assert_eq!(th_type.get_col(), 7);
    assert!(
        attrs
            .get_attribute_with_prefix(TemplateMode::XML, Some(&js("TH")), &js("Type"))
            .expect("get TH:Type")
            .is_none(),
        "XML case-sensitive lookup misses"
    );

    // 删除后保留位置；新设置属性无位置
    let attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("a"))
        .expect("remove a");
    let th_type = attrs
        .get_attribute_with_prefix(TemplateMode::XML, Some(&js("th")), &js("type"))
        .expect("get th:type")
        .expect("th:type exists");
    assert_eq!(th_type.get_line(), 2);
    assert_eq!(th_type.get_col(), 7);
    let attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("one"),
            Some(js("two")),
            None,
        )
        .expect("set one");
    let one = attrs
        .get_attribute(TemplateMode::XML, &js("one"))
        .expect("get one")
        .expect("one exists");
    assert_eq!(one.get_line(), -1, "new attribute has no location");
    assert_eq!(one.get_col(), -1, "new attribute has no location");
}

// ===========================================================================
// ElementAttributesTest 缺失族补移植（Java 21 逐字复刻）
//   - HTML：`ba= s` 家族（null/空值/引号形态/重加折叠）
//   - XML：`ba='twenty'→'thirty'`、无间隔属性、空/空白输入追加序列
// ===========================================================================

#[test]
fn html_element_attributes_ba_family_matches_java() {
    let attribute_definitions = empty_attribute_definitions();
    let mut attrs;

    // Java L217-221：remove type 后 set type=null -> 追加 null 属性
    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba= s>");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("type"))
        .expect("remove type");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'    ba= s");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("type"),
            None,
            None,
        )
        .expect("set type null");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'    ba= s type");

    // Java L223-227：set title=null -> 追加
    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba= s>");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("type"))
        .expect("remove type");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("title"),
            None,
            None,
        )
        .expect("set title null");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'    ba= s title");

    // Java L229-233：set title="" -> 双引号空值
    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba= s>");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("type"))
        .expect("remove type");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("title"),
            Some(js("")),
            None,
        )
        .expect("set title empty");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'    ba= s title=\"\"");

    // Java L235-241：SINGLE 保留单引号；NONE 被 HTML 强制回 DOUBLE
    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba= s>");
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("type"))
        .expect("remove type");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("title"),
            Some(js("")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set title empty single");
    assert_eq!(attrs_text(&attrs), " value='hello!!!'    ba= s title=''");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("title"),
            Some(js("")),
            Some(AttributeValueQuotes::NONE),
        )
        .expect("set title empty none");
    assert_eq!(
        attrs_text(&attrs),
        " value='hello!!!'    ba= s title=\"\"",
        "HTML 下 NONE 引号强制回 DOUBLE"
    );

    // Java L243-248：value="one" NONE -> 无引号；value="" -> 双引号
    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba= s>");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba= s"
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("value"),
            Some(js("one")),
            Some(AttributeValueQuotes::NONE),
        )
        .expect("set value one none");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   value=one    ba= s");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("value"),
            Some(js("")),
            None,
        )
        .expect("set value empty");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   value=\"\"    ba= s");

    // Java L250-253：ba="" -> HTML 保留空值带引号写法
    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba= s>");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            Some(js("")),
            None,
        )
        .expect("set ba empty");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba= \"\""
    );

    // Java L255-262：ba="one" -> 无引号；remove ba -> 恢复原状；重加 -> 单空格双引号
    attrs = html_attrs("<input type=\"text\"   value='hello!!!'    ba= s>");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            Some(js("one")),
            None,
        )
        .expect("set ba one");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba= one"
    );
    attrs = attrs
        .remove_attribute(TemplateMode::HTML, &js("ba"))
        .expect("remove ba");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   value='hello!!!'");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::HTML,
            None,
            js("ba"),
            Some(js("one")),
            None,
        )
        .expect("re-add ba");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"one\"",
        "重加属性空白折叠为单空格"
    );
}

#[test]
fn xml_element_attributes_extra_families_match_java() {
    let attribute_definitions = empty_attribute_definitions();
    let mut attrs;

    // Java L444-446：ba='twenty' -> set 'thirty'
    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba='twenty' />");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("ba"),
            Some(js("thirty")),
            None,
        )
        .expect("set ba thirty");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba='thirty' "
    );

    // Java L448-462：无间隔属性族
    attrs = xml_attrs("<input type=\"text\"value='hello!!!' />");
    assert_eq!(attrs_text(&attrs), " type=\"text\"value='hello!!!' ");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("type"))
        .expect("remove type");
    assert_eq!(attrs_text(&attrs), " value='hello!!!' ");

    attrs = xml_attrs("<input type=\"text\"value='hello!!!' name='one' />");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("type"))
        .expect("remove type");
    assert_eq!(attrs_text(&attrs), " value='hello!!!' name='one' ");

    attrs = xml_attrs("<input type=\"text\"value='hello!!!' name='one'/>");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"value='hello!!!' name='one'"
    );
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("name"))
        .expect("remove name");
    assert_eq!(attrs_text(&attrs), " type=\"text\"value='hello!!!'");

    // Java L526-536：value 引号形态 + ba 空值
    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba= 's'/>");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("value"),
            Some(js("one")),
            Some(AttributeValueQuotes::DOUBLE),
        )
        .expect("set value double");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value=\"one\"    ba= 's'"
    );
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("value"),
            Some(js("")),
            None,
        )
        .expect("set value empty");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   value=\"\"    ba= 's'");

    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba= 's'/>");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("ba"),
            Some(js("")),
            None,
        )
        .expect("set ba empty");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba= ''"
    );

    // Java L538-545：ba='one' -> remove -> 重加单空格双引号
    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba= 's'/>");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("ba"),
            Some(js("one")),
            None,
        )
        .expect("set ba one");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba= 'one'"
    );
    attrs = attrs
        .remove_attribute_with_prefix(TemplateMode::XML, None, &js("ba"))
        .expect("remove ba");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   value='hello!!!'");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("ba"),
            Some(js("one")),
            None,
        )
        .expect("re-add ba");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"one\""
    );

    // Java L547-570：ba=\"\" 追加序列（ba/be/bi/bo/bu 顺序追加 + 逐步删除）
    attrs = xml_attrs("<input type=\"text\"   value='hello!!!'    ba=\"\"/>");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("ba"),
            Some(js("one")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set ba single");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!'    ba='one'"
    );
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("ba"))
        .expect("remove ba");
    assert_eq!(attrs_text(&attrs), " type=\"text\"   value='hello!!!'");
    for (name, value) in [
        ("ba", "two"),
        ("be", "three"),
        ("bi", "four"),
        ("bo", "five"),
        ("bu", "six"),
    ] {
        attrs = attrs
            .set_attribute(
                &attribute_definitions,
                TemplateMode::XML,
                None,
                js(name),
                Some(js(value)),
                None,
            )
            .expect("append attribute");
    }
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"two\" be=\"three\" bi=\"four\" bo=\"five\" bu=\"six\""
    );
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("be"))
        .expect("remove be");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("bu"))
        .expect("remove bu");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"two\" bi=\"four\" bo=\"five\""
    );
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("bi"))
        .expect("remove bi");
    assert_eq!(
        attrs_text(&attrs),
        " type=\"text\"   value='hello!!!' ba=\"two\" bo=\"five\""
    );
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("type"))
        .expect("remove type");
    assert_eq!(
        attrs_text(&attrs),
        " value='hello!!!' ba=\"two\" bo=\"five\""
    );
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("type"))
        .expect("remove type again");
    assert_eq!(
        attrs_text(&attrs),
        " value='hello!!!' ba=\"two\" bo=\"five\"",
        "重复 remove 无副作用"
    );

    // Java L572-597：空/空白/换行输入追加序列
    attrs = xml_attrs("<input/>");
    assert_eq!(attrs_text(&attrs), "");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("a"),
            Some(js("one")),
            None,
        )
        .expect("set a");
    assert_eq!(attrs_text(&attrs), " a=\"one\"");

    attrs = xml_attrs("<input/>");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("a"),
            Some(js("one")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set a single");
    assert_eq!(attrs_text(&attrs), " a='one'");

    attrs = xml_attrs("<input   />");
    assert_eq!(attrs_text(&attrs), "   ");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("a"),
            Some(js("one")),
            None,
        )
        .expect("set a");
    assert_eq!(attrs_text(&attrs), " a=\"one\"   ");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("b"),
            Some(js("two")),
            None,
        )
        .expect("set b");
    assert_eq!(attrs_text(&attrs), " a=\"one\" b=\"two\"   ");

    attrs = xml_attrs("<input\none=\"\"  />");
    assert_eq!(attrs_text(&attrs), "\none=\"\"  ");
    attrs = attrs
        .set_attribute(
            &attribute_definitions,
            TemplateMode::XML,
            None,
            js("a"),
            Some(js("two")),
            None,
        )
        .expect("set a");
    assert_eq!(attrs_text(&attrs), "\none=\"\" a=\"two\"  ");

    attrs = xml_attrs("<input\none=\"\" two=\"\"/>");
    assert_eq!(attrs_text(&attrs), "\none=\"\" two=\"\"");
    attrs = attrs
        .remove_attribute(TemplateMode::XML, &js("one"))
        .expect("remove one");
    assert_eq!(attrs_text(&attrs), "\ntwo=\"\"");
}
