//! `OpenElementTag`/`CloseElementTag`/`StandaloneElementTag` Java 1:1 差分测试。
//!
//! 对应上游 `thymeleaf-tests-core` 的 `org.thymeleaf.engine` 包：
//! - OpenElementTagTest（HTML/XML 解析获取标签 + setAttribute/
//!   removeAttribute 完整序列，含引号形态与空白保留）
//! - StandaloneElementTagTest（同上 + minimized 属性重建）
//! - CloseElementTagTest（上游仅有辅助脚手架，无 @Test；关闭标签的
//!   可观测行为并入本文件的解析输出校验）
//!
//! 与 Java `TagObtentionTemplateHandler` 相同：用
//! `HTMLTemplateParser`/`XMLTemplateParser` 解析真实模板片段，
//! 从事件流捕获引擎标签对象。

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use thymeleaf::context::ITemplateContext;
use thymeleaf::engine::{
    AttributeDefinitions, ElementDefinitionValue, ITemplateHandler, OpenElementTag,
    StandaloneElementTag,
};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::markup::{HTMLTemplateParser, XMLTemplateParser};
use thymeleaf::model::{
    AttributeValueQuotes, ICDATASection, ICloseElementTag, IComment, IDocType, IElementTag,
    IOpenElementTag, IProcessableElementTag, IProcessingInstruction, IStandaloneElementTag,
    ITemplateEnd, ITemplateEvent, ITemplateStart, IText, IXMLDeclaration,
};
use thymeleaf::templateresource::{ITemplateResource, StringTemplateResource};
use thymeleaf::util::{FastStringWriter, JavaString};
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

/// 捕获三种标签事件的解析 Handler（对应 Java TagObtentionTemplateHandler）。
struct TagObtentionHandler {
    open: Rc<RefCell<Option<Arc<OpenElementTag>>>>,
    close: Rc<RefCell<Option<Arc<dyn ICloseElementTag>>>>,
    standalone: Rc<RefCell<Option<Arc<StandaloneElementTag>>>>,
}

impl ITemplateHandler for TagObtentionHandler {
    fn set_next(&mut self, _next: Option<thymeleaf::engine::TemplateHandlerHandle>) {}
    fn set_context(&mut self, _context: Arc<dyn ITemplateContext>) {}
    fn handle_template_start(
        &mut self,
        _template_start: Arc<dyn ITemplateStart>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_template_end(
        &mut self,
        _template_end: Arc<dyn ITemplateEnd>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_xml_declaration(
        &mut self,
        _xml_declaration: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_doc_type(
        &mut self,
        _doc_type: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_cdata_section(
        &mut self,
        _cdata_section: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_comment(
        &mut self,
        _comment: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_text(
        &mut self,
        _text: Arc<dyn IText>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn handle_standalone_element(
        &mut self,
        tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        *self.standalone.borrow_mut() = tag.into_engine_standalone_element_tag();
        Ok(())
    }
    fn handle_open_element(
        &mut self,
        tag: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        *self.open.borrow_mut() = tag.into_engine_open_element_tag();
        Ok(())
    }
    fn handle_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        *self.close.borrow_mut() = Some(tag);
        Ok(())
    }
    fn handle_processing_instruction(
        &mut self,
        _instruction: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
}

/// 解析捕获结果（open/close/standalone 三个标签槽）。
type CapturedTags = (
    Rc<RefCell<Option<Arc<OpenElementTag>>>>,
    Rc<RefCell<Option<Arc<dyn ICloseElementTag>>>>,
    Rc<RefCell<Option<Arc<StandaloneElementTag>>>>,
);

fn parse(
    config: &Arc<dyn thymeleaf::IEngineConfiguration>,
    input: &str,
    mode: TemplateMode,
) -> CapturedTags {
    let parser: Arc<dyn thymeleaf::templateparser::ITemplateParser> = match mode {
        TemplateMode::HTML => Arc::new(HTMLTemplateParser::new(2, 4096)),
        TemplateMode::XML => Arc::new(XMLTemplateParser::new(2, 4096)),
        _ => panic!("element tag tests only cover HTML/XML"),
    };
    let resource: Arc<dyn ITemplateResource> =
        Arc::new(StringTemplateResource::new(Some(input)).expect("string resource"));
    let open = Rc::new(RefCell::new(None));
    let close: Rc<RefCell<Option<Arc<dyn ICloseElementTag>>>> = Rc::new(RefCell::new(None));
    let standalone = Rc::new(RefCell::new(None));
    let handler = Box::new(TagObtentionHandler {
        open: open.clone(),
        close: close.clone(),
        standalone: standalone.clone(),
    });
    parser
        .parse_standalone(
            config.clone(),
            None,
            &js("test"),
            None,
            resource,
            mode,
            false,
            handler,
        )
        .expect("parse");
    (open, close, standalone)
}

fn html_open_tag(
    config: &Arc<dyn thymeleaf::IEngineConfiguration>,
    input: &str,
) -> Arc<OpenElementTag> {
    let (open, _, _) = parse(config, input, TemplateMode::HTML);
    open.borrow_mut().take().expect("HTML open tag captured")
}

fn xml_open_tag(
    config: &Arc<dyn thymeleaf::IEngineConfiguration>,
    input: &str,
) -> Arc<OpenElementTag> {
    let (open, _, _) = parse(config, input, TemplateMode::XML);
    open.borrow_mut().take().expect("XML open tag captured")
}

fn html_standalone_tag(
    config: &Arc<dyn thymeleaf::IEngineConfiguration>,
    input: &str,
) -> Arc<StandaloneElementTag> {
    let (_, _, standalone) = parse(config, input, TemplateMode::HTML);
    standalone
        .borrow_mut()
        .take()
        .expect("HTML standalone tag captured")
}

fn xml_standalone_tag(
    config: &Arc<dyn thymeleaf::IEngineConfiguration>,
    input: &str,
) -> Arc<StandaloneElementTag> {
    let (_, _, standalone) = parse(config, input, TemplateMode::XML);
    standalone
        .borrow_mut()
        .take()
        .expect("XML standalone tag captured")
}

fn tag_text(tag: &Arc<OpenElementTag>) -> String {
    tag.to_java_string().to_string_lossy()
}

fn standalone_text(tag: &Arc<StandaloneElementTag>) -> String {
    tag.to_java_string().to_string_lossy()
}

fn close_text(tag: &dyn ICloseElementTag) -> String {
    let mut writer = FastStringWriter::new();
    tag.write(&mut writer).expect("close tag write");
    writer.to_string().to_string_lossy()
}

fn empty_attribute_definitions() -> AttributeDefinitions {
    // 对应 Java: new AttributeDefinitions(Collections.EMPTY_MAP)
    AttributeDefinitions::new(std::collections::HashMap::new()).expect("attribute definitions")
}

// ===========================================================================
// 1. OpenElementTagTest#testHtmlOpenElementAttrManagement
// ===========================================================================

#[test]
fn open_element_tag_html_attribute_management() {
    let config = engine().get_configuration().expect("config");
    let attribute_definitions = empty_attribute_definitions();

    let mut tag = html_open_tag(&config, "<div>");
    assert_eq!(tag_text(&tag), "<div>");

    tag = html_open_tag(&config, "<div type=\"text\">");
    assert_eq!(tag_text(&tag), "<div type=\"text\">");

    tag = html_open_tag(&config, "<div type=\"text\"   value='hello!!!'>");
    assert_eq!(tag_text(&tag), "<div type=\"text\"   value='hello!!!'>");
    tag = tag.remove_attribute(&js("type")).expect("remove type");
    assert_eq!(tag_text(&tag), "<div value='hello!!!'>");
    tag = tag.remove_attribute(&js("value")).expect("remove value");
    assert_eq!(tag_text(&tag), "<div>");

    tag = html_open_tag(&config, "<div type=\"text\"   value='hello!!!'    >");
    assert_eq!(tag_text(&tag), "<div type=\"text\"   value='hello!!!'    >");
    tag = tag
        .remove_attribute_with_prefix(None, &js("type"))
        .expect("remove type");
    assert_eq!(tag_text(&tag), "<div value='hello!!!'    >");
    tag = tag
        .remove_attribute_with_prefix(None, &js("value"))
        .expect("remove value");
    assert_eq!(tag_text(&tag), "<div    >");

    tag = html_open_tag(&config, "<div type=\"text\"   value='hello!!!'    ba >");
    assert_eq!(
        tag_text(&tag),
        "<div type=\"text\"   value='hello!!!'    ba >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("value"),
            Some(js("bye! :(")),
            None,
        )
        .expect("set value");
    assert_eq!(
        tag_text(&tag),
        "<div type=\"text\"   value='bye! :('    ba >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("type"),
            Some(js("one")),
            None,
        )
        .expect("set type");
    assert_eq!(
        tag_text(&tag),
        "<div type=\"one\"   value='bye! :('    ba >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("two")),
            None,
        )
        .expect("set ba");
    assert_eq!(
        tag_text(&tag),
        "<div type=\"one\"   value='bye! :('    ba=\"two\" >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("three")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set ba single");
    assert_eq!(
        tag_text(&tag),
        "<div type=\"one\"   value='bye! :('    ba='three' >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("four")),
            Some(AttributeValueQuotes::NONE),
        )
        .expect("set ba none");
    assert_eq!(
        tag_text(&tag),
        "<div type=\"one\"   value='bye! :('    ba=four >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("five")),
            None,
        )
        .expect("set ba five");
    assert_eq!(
        tag_text(&tag),
        "<div type=\"one\"   value='bye! :('    ba=five >"
    );
    tag = tag
        .set_attribute(&attribute_definitions, None, js("ba"), None, None)
        .expect("set ba null value");
    assert_eq!(
        tag_text(&tag),
        "<div type=\"one\"   value='bye! :('    ba >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("six")),
            None,
        )
        .expect("set ba six");
    assert_eq!(
        tag_text(&tag),
        "<div type=\"one\"   value='bye! :('    ba=\"six\" >"
    );

    // 无引号属性原样更新
    tag = html_open_tag(
        &config,
        "<div type=\"text\"   value='hello!!!'    ba=twenty >",
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("thirty")),
            None,
        )
        .expect("set ba thirty");
    assert_eq!(
        tag_text(&tag),
        "<div type=\"text\"   value='hello!!!'    ba=thirty >"
    );

    // 多标签输入只取第一个 open 事件
    tag = html_open_tag(
        &config,
        "<div type=\"text\"   value='hello!!!'    ba=twenty ><p id='one'>",
    );
    assert_eq!(tag_text(&tag), "<p id='one'>");
}

// ===========================================================================
// 2. OpenElementTagTest#testXmlOpenElementAttrManagement
// ===========================================================================

#[test]
fn open_element_tag_xml_attribute_management() {
    let config = engine().get_configuration().expect("config");
    let attribute_definitions = empty_attribute_definitions();

    let mut tag = xml_open_tag(&config, "<input></input>");
    assert_eq!(tag_text(&tag), "<input>");

    tag = xml_open_tag(&config, "<input type=\"text\"></input>");
    assert_eq!(tag_text(&tag), "<input type=\"text\">");

    tag = xml_open_tag(&config, "<input type=\"text\"   value='hello!!!'></input>");
    assert_eq!(tag_text(&tag), "<input type=\"text\"   value='hello!!!'>");
    tag = tag.remove_attribute(&js("type")).expect("remove type");
    assert_eq!(tag_text(&tag), "<input value='hello!!!'>");
    tag = tag.remove_attribute(&js("value")).expect("remove value");
    assert_eq!(tag_text(&tag), "<input>");

    tag = xml_open_tag(
        &config,
        "<input type=\"text\"   value='hello!!!'    ></input>",
    );
    assert_eq!(
        tag_text(&tag),
        "<input type=\"text\"   value='hello!!!'    >"
    );
    tag = tag
        .remove_attribute_with_prefix(None, &js("type"))
        .expect("remove type");
    assert_eq!(tag_text(&tag), "<input value='hello!!!'    >");
    tag = tag
        .remove_attribute_with_prefix(None, &js("value"))
        .expect("remove value");
    assert_eq!(tag_text(&tag), "<input    >");

    // XML 前缀删除
    tag = xml_open_tag(
        &config,
        "<input th:type=\"text\"   th:value='hello!!!'    ></input>",
    );
    assert_eq!(
        tag_text(&tag),
        "<input th:type=\"text\"   th:value='hello!!!'    >"
    );
    tag = tag
        .remove_attribute_with_prefix(Some(&js("th")), &js("type"))
        .expect("remove th:type");
    assert_eq!(tag_text(&tag), "<input th:value='hello!!!'    >");
    tag = tag
        .remove_attribute_with_prefix(Some(&js("th")), &js("value"))
        .expect("remove th:value");
    assert_eq!(tag_text(&tag), "<input    >");

    // XML setAttribute：值替换与引号形态
    tag = xml_open_tag(
        &config,
        "<input type=\"text\"   value='hello!!!'    ba='' ></input>",
    );
    assert_eq!(
        tag_text(&tag),
        "<input type=\"text\"   value='hello!!!'    ba='' >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("value"),
            Some(js("bye! :(")),
            None,
        )
        .expect("set value");
    assert_eq!(
        tag_text(&tag),
        "<input type=\"text\"   value='bye! :('    ba='' >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("type"),
            Some(js("one")),
            None,
        )
        .expect("set type");
    assert_eq!(
        tag_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba='' >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("two")),
            None,
        )
        .expect("set ba");
    assert_eq!(
        tag_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba='two' >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("three")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set ba single");
    assert_eq!(
        tag_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba='three' >"
    );

    // XML 不允许 NONE 引号 / null 值 → Java IllegalArgumentException
    assert!(
        tag.set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("four")),
            Some(AttributeValueQuotes::NONE),
        )
        .is_err(),
        "XML NONE quotes rejected"
    );
    assert!(
        tag.set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            None,
            Some(AttributeValueQuotes::NONE),
        )
        .is_err(),
        "XML NONE quotes with null value rejected"
    );
    assert!(
        tag.set_attribute(&attribute_definitions, None, js("ba"), None, None)
            .is_err(),
        "XML null value rejected"
    );

    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("five")),
            None,
        )
        .expect("set ba five");
    assert_eq!(
        tag_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba='five' >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("six")),
            None,
        )
        .expect("set ba six");
    assert_eq!(
        tag_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba='six' >"
    );

    // 多标签输入只取第一个 open 事件
    tag = xml_open_tag(
        &config,
        "<div type=\"text\"   value='hello!!!'    ba='twenty' ><p id='one'></p></div>",
    );
    assert_eq!(tag_text(&tag), "<p id='one'>");
}

// ===========================================================================
// 3. 关闭标签（CloseElementTagTest 无独立 @Test，可观测行为并入解析输出）
// ===========================================================================

#[test]
fn close_element_tag_output() {
    let config = engine().get_configuration().expect("config");
    // HTML 关闭标签 round-trip
    let (_, close, _) = parse(&config, "<div type=\"text\">text</div>", TemplateMode::HTML);
    let close = close.borrow_mut().take().expect("close tag captured");
    assert_eq!(close_text(close.as_ref()), "</div>");
    assert!(!close.is_unmatched(), "matched close tag");

    // HTML 隐式闭合（<p> 不闭合）→ 解析器补齐的 synthetic 关闭标签。
    // Java: synthetic 标签原模板中不存在，write 不输出任何内容。
    let (_, close, _) = parse(&config, "<p>text", TemplateMode::HTML);
    let close = close.borrow_mut().take().expect("close tag captured");
    assert!(close.is_synthetic(), "auto-balanced close tag is synthetic");
    assert_eq!(close_text(close.as_ref()), "");

    // XML 关闭标签
    let (_, close, _) = parse(&config, "<input type=\"text\"></input>", TemplateMode::XML);
    let close = close.borrow_mut().take().expect("close tag captured");
    assert_eq!(close_text(close.as_ref()), "</input>");
}

// ===========================================================================
// 4. StandaloneElementTagTest#testHtmlStandaloneElementAttrManagement
// ===========================================================================

#[test]
fn standalone_element_tag_html_attribute_management() {
    let config = engine().get_configuration().expect("config");
    let attribute_definitions = empty_attribute_definitions();

    let mut tag = html_standalone_tag(&config, "<input>");
    assert_eq!(standalone_text(&tag), "<input>");

    tag = html_standalone_tag(&config, "<input type=\"text\">");
    assert_eq!(standalone_text(&tag), "<input type=\"text\">");

    tag = html_standalone_tag(&config, "<input type=\"text\"   value='hello!!!'>");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"text\"   value='hello!!!'>"
    );
    tag = tag.remove_attribute(&js("type")).expect("remove type");
    assert_eq!(standalone_text(&tag), "<input value='hello!!!'>");
    tag = tag.remove_attribute(&js("value")).expect("remove value");
    assert_eq!(standalone_text(&tag), "<input>");

    tag = html_standalone_tag(&config, "<input type=\"text\"   value='hello!!!'    >");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"text\"   value='hello!!!'    >"
    );
    tag = tag
        .remove_attribute_with_prefix(None, &js("type"))
        .expect("remove type");
    assert_eq!(standalone_text(&tag), "<input value='hello!!!'    >");
    tag = tag
        .remove_attribute_with_prefix(None, &js("value"))
        .expect("remove value");
    assert_eq!(standalone_text(&tag), "<input    >");

    tag = html_standalone_tag(&config, "<input type=\"text\"   value='hello!!!'    ba >");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"text\"   value='hello!!!'    ba >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("value"),
            Some(js("bye! :(")),
            None,
        )
        .expect("set value");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"text\"   value='bye! :('    ba >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("type"),
            Some(js("one")),
            None,
        )
        .expect("set type");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("two")),
            None,
        )
        .expect("set ba");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba=\"two\" >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("three")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set ba single");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba='three' >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("four")),
            Some(AttributeValueQuotes::NONE),
        )
        .expect("set ba none");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba=four >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("five")),
            None,
        )
        .expect("set ba five");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba=five >"
    );
    tag = tag
        .set_attribute(&attribute_definitions, None, js("ba"), None, None)
        .expect("set ba null value");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba >"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("six")),
            None,
        )
        .expect("set ba six");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba=\"six\" >"
    );

    tag = html_standalone_tag(
        &config,
        "<input type=\"text\"   value='hello!!!'    ba=twenty >",
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("thirty")),
            None,
        )
        .expect("set ba thirty");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"text\"   value='hello!!!'    ba=thirty >"
    );

    tag = html_standalone_tag(
        &config,
        "<input type=\"text\"   value='hello!!!'    ba=twenty ><p id='one'/>",
    );
    assert_eq!(standalone_text(&tag), "<p id='one'/>");
}

// ===========================================================================
// 5. StandaloneElementTagTest#testXmlStandaloneElementAttrManagement
// ===========================================================================

#[test]
fn standalone_element_tag_xml_attribute_management() {
    let config = engine().get_configuration().expect("config");
    let attribute_definitions = empty_attribute_definitions();

    let mut tag = xml_standalone_tag(&config, "<input/>");
    assert_eq!(standalone_text(&tag), "<input/>");

    tag = xml_standalone_tag(&config, "<input type=\"text\"/>");
    assert_eq!(standalone_text(&tag), "<input type=\"text\"/>");

    tag = xml_standalone_tag(&config, "<input type=\"text\"   value='hello!!!'/>");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"text\"   value='hello!!!'/>"
    );
    tag = tag.remove_attribute(&js("type")).expect("remove type");
    assert_eq!(standalone_text(&tag), "<input value='hello!!!'/>");
    tag = tag.remove_attribute(&js("value")).expect("remove value");
    assert_eq!(standalone_text(&tag), "<input/>");

    tag = xml_standalone_tag(&config, "<input type=\"text\"   value='hello!!!'    />");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"text\"   value='hello!!!'    />"
    );
    tag = tag
        .remove_attribute_with_prefix(None, &js("type"))
        .expect("remove type");
    assert_eq!(standalone_text(&tag), "<input value='hello!!!'    />");
    tag = tag
        .remove_attribute_with_prefix(None, &js("value"))
        .expect("remove value");
    assert_eq!(standalone_text(&tag), "<input    />");

    tag = xml_standalone_tag(
        &config,
        "<input th:type=\"text\"   th:value='hello!!!'    />",
    );
    assert_eq!(
        standalone_text(&tag),
        "<input th:type=\"text\"   th:value='hello!!!'    />"
    );
    tag = tag
        .remove_attribute_with_prefix(Some(&js("th")), &js("type"))
        .expect("remove th:type");
    assert_eq!(standalone_text(&tag), "<input th:value='hello!!!'    />");
    tag = tag
        .remove_attribute_with_prefix(Some(&js("th")), &js("value"))
        .expect("remove th:value");
    assert_eq!(standalone_text(&tag), "<input    />");

    tag = xml_standalone_tag(
        &config,
        "<input type=\"text\"   value='hello!!!'    ba='' />",
    );
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"text\"   value='hello!!!'    ba='' />"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("value"),
            Some(js("bye! :(")),
            None,
        )
        .expect("set value");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"text\"   value='bye! :('    ba='' />"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("type"),
            Some(js("one")),
            None,
        )
        .expect("set type");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba='' />"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("two")),
            None,
        )
        .expect("set ba");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba='two' />"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("three")),
            Some(AttributeValueQuotes::SINGLE),
        )
        .expect("set ba single");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba='three' />"
    );

    // XML 不允许 NONE 引号 / null 值 → Java IllegalArgumentException
    assert!(
        tag.set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("four")),
            Some(AttributeValueQuotes::NONE),
        )
        .is_err(),
        "XML NONE quotes rejected"
    );
    assert!(
        tag.set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            None,
            Some(AttributeValueQuotes::NONE),
        )
        .is_err(),
        "XML NONE quotes with null value rejected"
    );
    assert!(
        tag.set_attribute(&attribute_definitions, None, js("ba"), None, None)
            .is_err(),
        "XML null value rejected"
    );

    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("five")),
            None,
        )
        .expect("set ba five");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba='five' />"
    );
    tag = tag
        .set_attribute(
            &attribute_definitions,
            None,
            js("ba"),
            Some(js("six")),
            None,
        )
        .expect("set ba six");
    assert_eq!(
        standalone_text(&tag),
        "<input type=\"one\"   value='bye! :('    ba='six' />"
    );

    tag = xml_standalone_tag(
        &config,
        "<input type=\"text\"   value='hello!!!'    ba='twenty' /><meta id='one' />",
    );
    assert_eq!(standalone_text(&tag), "<meta id='one' />");
}

// ===========================================================================
// 6. StandaloneElementTagTest#testHtmlStandaloneElementPropertyManagement
// ===========================================================================

#[test]
fn standalone_element_tag_html_property_management() {
    let config = engine().get_configuration().expect("config");
    let element_definitions = config.get_element_definitions();

    // HTML "<input>"（未最小化）→ 重建为 minimized 输出 "<input/>"
    let tag = html_standalone_tag(&config, "<input>");
    let input_definition = element_definitions
        .for_html_name(Some(&js("input")))
        .expect("input definition");
    let processable = tag
        .as_engine_processable_element_tag()
        .expect("processable tag");
    let ElementDefinitionValue::Html(definition) = processable
        .as_element_tag()
        .element_definition_value()
        .clone()
    else {
        panic!("HTML definition expected");
    };
    assert!(
        Arc::ptr_eq(&definition, &input_definition),
        "getElementDefinition() same as ElementDefinitions.forHTMLName(input)"
    );
    let rebuilt = StandaloneElementTag::with_location(
        tag.get_template_mode(),
        processable
            .as_element_tag()
            .element_definition_value()
            .clone(),
        tag.get_element_complete_name().clone(),
        processable.attributes().cloned(),
        tag.is_synthetic(),
        true, // minimized
        tag.get_template_name().cloned(),
        tag.get_line(),
        tag.get_col(),
    )
    .expect("minimized standalone");
    assert_eq!(standalone_text(&Arc::new(rebuilt)), "<input/>");

    // HTML "<input />" → 重建为非 minimized 输出 "<input >"
    let tag = html_standalone_tag(&config, "<input />");
    let input_definition = element_definitions
        .for_html_name(Some(&js("input")))
        .expect("input definition");
    let processable = tag
        .as_engine_processable_element_tag()
        .expect("processable tag");
    let ElementDefinitionValue::Html(definition) = processable
        .as_element_tag()
        .element_definition_value()
        .clone()
    else {
        panic!("HTML definition expected");
    };
    assert!(
        Arc::ptr_eq(&definition, &input_definition),
        "getElementDefinition() same as ElementDefinitions.forHTMLName(input)"
    );
    let rebuilt = StandaloneElementTag::with_location(
        tag.get_template_mode(),
        processable
            .as_element_tag()
            .element_definition_value()
            .clone(),
        tag.get_element_complete_name().clone(),
        processable.attributes().cloned(),
        tag.is_synthetic(),
        false, // 非 minimized
        tag.get_template_name().cloned(),
        tag.get_line(),
        tag.get_col(),
    )
    .expect("non-minimized standalone");
    assert_eq!(standalone_text(&Arc::new(rebuilt)), "<input >");
}

// ===========================================================================
// 7. StandaloneElementTagTest#testXmlStandaloneElementPropertyManagement
// ===========================================================================

#[test]
fn standalone_element_tag_xml_property_management() {
    let config = engine().get_configuration().expect("config");
    let element_definitions = config.get_element_definitions();

    // XML "<input/>" → 元素定义同一性
    let tag = xml_standalone_tag(&config, "<input/>");
    let input_definition = element_definitions
        .for_xml_name(Some(&js("input")))
        .expect("input definition");
    let processable = tag
        .as_engine_processable_element_tag()
        .expect("processable tag");
    let ElementDefinitionValue::Xml(definition) = processable
        .as_element_tag()
        .element_definition_value()
        .clone()
    else {
        panic!("XML definition expected");
    };
    assert!(
        Arc::ptr_eq(&definition, &input_definition),
        "getElementDefinition() same as ElementDefinitions.forXMLName(input)"
    );

    // XML 不允许非 minimized standalone → IllegalArgumentException
    assert!(
        StandaloneElementTag::with_location(
            tag.get_template_mode(),
            processable
                .as_element_tag()
                .element_definition_value()
                .clone(),
            tag.get_element_complete_name().clone(),
            processable.attributes().cloned(),
            tag.is_synthetic(),
            false, // 非 minimized 对 XML 非法
            tag.get_template_name().cloned(),
            tag.get_line(),
            tag.get_col(),
        )
        .is_err(),
        "XML non-minimized standalone rejected"
    );
}
