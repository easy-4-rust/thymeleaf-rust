//! model 接口族 Java 1:1 差分测试。
//!
//! 逐一对齐上游 `org.thymeleaf.model` 接口在引擎实现上的可观测语义
//! （golden 值全部取自 Java 3.1.5 源码）：
//! - `ITemplateStart`/`ITemplateEnd`：无位置单例契约（Java
//!   `TemplateStart.TEMPLATE_START_INSTANCE`/`TemplateEnd.TEMPLATE_END_INSTANCE`）
//! - `ITemplateEvent`：hasLocation/getTemplateName/getLine/getCol/accept/write
//! - `IModelVisitor`/`AbstractModelVisitor`：11 类事件分发顺序与默认空操作
//! - `IModelFactory`：事件构造的串行化 toString、DocType 类型推导、XML
//!   声明字段、属性 set/replace/remove 与身份保持、parse 往返
//!
//! Java 关键语义来源：
//! - `TemplateStart.toString()` 为空串、`write(Writer)` 为空操作、无位置
//! - `ModelBuilderTemplateHandler` 一律以单例替换模板起止事件
//! - `DocType.computeType`：publicId 非空且 systemId 为空 → 构造错误；
//!   两者皆空 → type null；publicId 非空 → PUBLIC；否则 SYSTEM
//! - `XMLDeclaration`：keyword 恒为 "xml"，version/encoding/standalone 可选
//! - `AbstractProcessableElementTag.removeAttribute`：属性不存在时返回
//!   原对象（身份保持），Rust 侧以 `Arc::ptr_eq` 验证

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use indexmap::IndexMap;

use thymeleaf::engine::{AttributeNames, TemplateData, TemplateEnd, TemplateStart};
use thymeleaf::model::{
    AbstractModelVisitor, AttributeValueQuotes, ICDATASection, ICloseElementTag, IComment,
    IDocType, IModelFactory, IModelVisitor, IOpenElementTag, IProcessableElementTag,
    IProcessingInstruction, IStandaloneElementTag, ITemplateEnd, ITemplateEvent, ITemplateStart,
    IText, IXMLDeclaration,
};
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::{FastStringWriter, JavaCharSequence, Utf16String};
use thymeleaf::{ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode};

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn engine() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

fn with_factory<T>(mode: TemplateMode, f: impl FnOnce(&dyn IModelFactory) -> T) -> T {
    let configuration = engine().get_configuration().expect("configuration");
    f(configuration.get_model_factory(mode))
}

/// 按 Java 语义写出事件（write 到 FastStringWriter 后取回内容）。
fn write_events(events: &[&dyn ITemplateEvent]) -> String {
    let mut writer = FastStringWriter::new();
    for event in events {
        event.write(&mut writer).expect("event write must not fail");
    }
    writer.to_string().to_string_lossy()
}

/// 将任意子接口事件提升为 `&dyn ITemplateEvent`（数组字面量内不支持自动收窄）。
fn as_event(event: &dyn ITemplateEvent) -> &dyn ITemplateEvent {
    event
}

/// 记录 11 类 visit 分发的访问器（对应 Java 匿名 IModelVisitor）。
#[derive(Default)]
struct RecordingVisitor {
    log: Rc<RefCell<Vec<String>>>,
}

impl RecordingVisitor {
    fn log(&self) -> Vec<String> {
        self.log.borrow().clone()
    }
}

impl IModelVisitor for RecordingVisitor {
    fn visit_template_start(&mut self, _template_start: &dyn ITemplateStart) {
        self.log
            .borrow_mut()
            .push("visit_template_start".to_owned());
    }
    fn visit_template_end(&mut self, _template_end: &dyn ITemplateEnd) {
        self.log.borrow_mut().push("visit_template_end".to_owned());
    }
    fn visit_xml_declaration(&mut self, _xml_declaration: &dyn IXMLDeclaration) {
        self.log
            .borrow_mut()
            .push("visit_xml_declaration".to_owned());
    }
    fn visit_doc_type(&mut self, _doc_type: &dyn IDocType) {
        self.log.borrow_mut().push("visit_doc_type".to_owned());
    }
    fn visit_cdata_section(&mut self, _cdata_section: &dyn ICDATASection) {
        self.log.borrow_mut().push("visit_cdata_section".to_owned());
    }
    fn visit_comment(&mut self, _comment: &dyn IComment) {
        self.log.borrow_mut().push("visit_comment".to_owned());
    }
    fn visit_text(&mut self, _text: &dyn IText) {
        self.log.borrow_mut().push("visit_text".to_owned());
    }
    fn visit_standalone_element_tag(
        &mut self,
        _standalone_element_tag: &dyn IStandaloneElementTag,
    ) {
        self.log
            .borrow_mut()
            .push("visit_standalone_element_tag".to_owned());
    }
    fn visit_open_element_tag(&mut self, _open_element_tag: &dyn IOpenElementTag) {
        self.log
            .borrow_mut()
            .push("visit_open_element_tag".to_owned());
    }
    fn visit_close_element_tag(&mut self, _close_element_tag: &dyn ICloseElementTag) {
        self.log
            .borrow_mut()
            .push("visit_close_element_tag".to_owned());
    }
    fn visit_processing_instruction(
        &mut self,
        _processing_instruction: &dyn IProcessingInstruction,
    ) {
        self.log
            .borrow_mut()
            .push("visit_processing_instruction".to_owned());
    }
}

// ===========================================================================
// 1. ITemplateStart/ITemplateEnd 无位置单例契约
// ===========================================================================

#[test]
fn template_start_singleton_contract_matches_java() {
    let first = TemplateStart::instance();
    let second = TemplateStart::instance();
    // Java: TEMPLATE_START_INSTANCE 恒为同一实例
    assert!(Arc::ptr_eq(&first, &second), "singleton identity");
    // Java TemplateStart：无位置（hasLocation false、line/col -1、templateName null）
    assert!(!first.has_location());
    assert_eq!(first.get_line(), -1);
    assert_eq!(first.get_col(), -1);
    assert!(first.get_template_name().is_none());
    // Java toString 为空串
    assert_eq!(first.to_string(), "");
    // Java write(Writer) 为空操作
    assert_eq!(write_events(&[as_event(first.as_ref())]), "");
    // accept 分发到 visitTemplateStart
    let visitor = RecordingVisitor::default();
    {
        let mut visitor = visitor;
        first.accept(&mut visitor);
        assert_eq!(visitor.log(), vec!["visit_template_start"]);
    }
    // ITemplateStart 子接口标记
    assert!(first.is_template_start());
    assert!(!first.is_template_end());
}

#[test]
fn template_end_singleton_contract_matches_java() {
    let first = TemplateEnd::instance();
    let second = TemplateEnd::instance();
    // Java: TEMPLATE_END_INSTANCE 恒为同一实例
    assert!(Arc::ptr_eq(&first, &second), "singleton identity");
    assert!(!first.has_location());
    assert_eq!(first.get_line(), -1);
    assert_eq!(first.get_col(), -1);
    assert!(first.get_template_name().is_none());
    assert_eq!(first.to_string(), "");
    assert_eq!(write_events(&[as_event(first.as_ref())]), "");
    let visitor = RecordingVisitor::default();
    {
        let mut visitor = visitor;
        first.accept(&mut visitor);
        assert_eq!(visitor.log(), vec!["visit_template_end"]);
    }
    assert!(first.is_template_end());
    assert!(!first.is_template_start());
}

// ===========================================================================
// 2. IModelFactory 事件构造：ITemplateEvent 位置契约与串行化
// ===========================================================================

#[test]
fn factory_text_event_contract_matches_java() {
    with_factory(TemplateMode::HTML, |factory| {
        let text = factory.create_text(js("hello"));
        // Java Text：工厂构造无位置
        assert!(!text.has_location());
        assert_eq!(text.get_line(), -1);
        assert_eq!(text.get_col(), -1);
        assert!(text.get_template_name().is_none());
        // Java Text.toString() == getText()
        assert_eq!(
            text.get_text()
                .expect("text access")
                .expect("non-null")
                .to_string_lossy(),
            "hello"
        );
        // Java Text.write 写出内容
        assert_eq!(write_events(&[as_event(text.as_ref())]), "hello");
        // accept 分发到 visitText
        let visitor = RecordingVisitor::default();
        {
            let mut visitor = visitor;
            text.accept(&mut visitor);
            assert_eq!(visitor.log(), vec!["visit_text"]);
        }
        // IText 内容访问
        let content = text.get_text().expect("text access").expect("non-null");
        assert_eq!(content.to_string_lossy(), "hello");
        assert_eq!(content.java_length().expect("length"), 5);
    });
}

#[test]
fn factory_markup_events_serialization_matches_java() {
    with_factory(TemplateMode::HTML, |factory| {
        // Comment：toString == getComment() == "<!--...-->"
        let comment = factory.create_comment(js("a comment")).expect("comment");
        assert_eq!(
            write_events(&[as_event(comment.as_ref())]),
            "<!--a comment-->"
        );
        let comment_content = comment.get_content().expect("content").expect("non-null");
        assert_eq!(comment_content.to_string_lossy(), "a comment");
        assert!(!comment.has_location());

        // CDATASection：toString == "<![CDATA[...]]>"
        let cdata = factory.create_cdata_section(js("<raw>")).expect("cdata");
        assert_eq!(
            write_events(&[as_event(cdata.as_ref())]),
            "<![CDATA[<raw>]]>"
        );
        let cdata_content = cdata.get_content().expect("content").expect("non-null");
        assert_eq!(cdata_content.to_string_lossy(), "<raw>");
        assert!(!cdata.has_location());

        // HTML5 doctype：关键字 DOCTYPE、元素 html、无 type/公共/系统 ID
        let html5 = factory.create_html5_doc_type().expect("html5 doctype");
        assert_eq!(write_events(&[as_event(html5.as_ref())]), "<!DOCTYPE html>");
        assert_eq!(html5.get_keyword().unwrap().to_string_lossy(), "DOCTYPE");
        assert_eq!(html5.get_element_name().unwrap().to_string_lossy(), "html");
        assert!(html5.get_type().is_none(), "both IDs null -> type null");
        assert!(html5.get_public_id().is_none());
        assert!(html5.get_system_id().is_none());
        assert!(html5.get_internal_subset().is_none());

        // DocType(publicId, systemId)：publicId 非空 -> type PUBLIC
        let public_system = factory
            .create_doc_type(Some(js("PUBLIC-ID")), Some(js("SYSTEM-ID")))
            .expect("public doctype");
        assert_eq!(
            write_events(&[as_event(public_system.as_ref())]),
            "<!DOCTYPE html PUBLIC \"PUBLIC-ID\" \"SYSTEM-ID\">"
        );
        assert_eq!(
            public_system.get_type().unwrap().to_string_lossy(),
            "PUBLIC"
        );

        // DocType(null, systemId)：仅系统 ID -> type SYSTEM
        let system_only = factory
            .create_doc_type(None, Some(js("SYSTEM-ID")))
            .expect("system doctype");
        assert_eq!(
            write_events(&[as_event(system_only.as_ref())]),
            "<!DOCTYPE html SYSTEM \"SYSTEM-ID\">"
        );
        assert_eq!(system_only.get_type().unwrap().to_string_lossy(), "SYSTEM");
        assert!(system_only.get_public_id().is_none());

        // 完整构造器：内部子集 -> " [..]"
        let full = factory
            .create_full_doc_type(
                js("DOCTYPE"),
                js("root"),
                Some(js("PUBLIC-ID")),
                Some(js("SYSTEM-ID")),
                Some(js("<!ENTITY x 'y'>")),
            )
            .expect("full doctype");
        assert_eq!(
            write_events(&[as_event(full.as_ref())]),
            "<!DOCTYPE root PUBLIC \"PUBLIC-ID\" \"SYSTEM-ID\" [<!ENTITY x 'y'>]>"
        );

        // XMLDeclaration：keyword 恒为 "xml"
        let declaration = factory
            .create_xml_declaration(Some(js("1.0")), Some(js("UTF-8")), Some(js("yes")))
            .expect("xml declaration");
        assert_eq!(
            write_events(&[as_event(declaration.as_ref())]),
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        );
        assert_eq!(declaration.get_keyword().unwrap().to_string_lossy(), "xml");
        assert_eq!(declaration.get_version().unwrap().to_string_lossy(), "1.0");
        assert_eq!(
            declaration.get_encoding().unwrap().to_string_lossy(),
            "UTF-8"
        );
        assert_eq!(
            declaration.get_standalone().unwrap().to_string_lossy(),
            "yes"
        );
        // 全部字段为空 -> "<?xml?>"
        let bare = factory
            .create_xml_declaration(None, None, None)
            .expect("bare xml declaration");
        assert_eq!(write_events(&[as_event(bare.as_ref())]), "<?xml?>");
        assert!(bare.get_version().is_none());
        assert!(bare.get_encoding().is_none());
        assert!(bare.get_standalone().is_none());

        // ProcessingInstruction：content 非空 -> "<?target content?>"
        let pi = factory
            .create_processing_instruction(js("target"), js("data"))
            .expect("pi");
        assert_eq!(write_events(&[as_event(pi.as_ref())]), "<?target data?>");
        assert_eq!(pi.get_target().unwrap().to_string_lossy(), "target");
        assert_eq!(pi.get_content().unwrap().to_string_lossy(), "data");
    });
}

// ===========================================================================
// 3. IModelFactory 元素标签构造：名称/合成/最小化/位置/串行化
// ===========================================================================

#[test]
fn factory_element_tags_contract_matches_java() {
    with_factory(TemplateMode::HTML, |factory| {
        // 开标签：完整名、属性、双引号、非合成、无位置
        let mut attributes = IndexMap::new();
        attributes.insert(js("class"), Some(js("a")));
        let open = factory
            .create_open_element_tag(
                js("div"),
                Some(&attributes),
                AttributeValueQuotes::DOUBLE,
                false,
            )
            .expect("open tag");
        assert_eq!(open.get_element_complete_name().to_string_lossy(), "div");
        assert_eq!(
            write_events(&[as_event(open.as_ref())]),
            "<div class=\"a\">"
        );
        assert!(!open.is_synthetic());
        assert!(!open.has_location());
        assert_eq!(
            open.get_attribute_value(&js("class"))
                .expect("value access")
                .unwrap()
                .to_string_lossy(),
            "a"
        );

        // 独立标签：minimized -> "<br/>"
        let standalone = factory
            .create_standalone_element_tag(
                js("br"),
                None,
                AttributeValueQuotes::DOUBLE,
                false,
                true,
            )
            .expect("standalone tag");
        assert_eq!(write_events(&[as_event(standalone.as_ref())]), "<br/>");
        assert!(!standalone.is_synthetic());

        // 非 minimized 独立标签 -> "<br>"
        let not_minimized = factory
            .create_standalone_element_tag(
                js("br"),
                None,
                AttributeValueQuotes::DOUBLE,
                false,
                false,
            )
            .expect("standalone tag");
        assert_eq!(write_events(&[as_event(not_minimized.as_ref())]), "<br>");

        // 合成独立标签：Java write 对 synthetic 元素为空操作（“original
        // template 中不存在的合成元素不写出”）
        let synthetic = factory
            .create_standalone_element_tag(
                js("img"),
                None,
                AttributeValueQuotes::DOUBLE,
                true,
                false,
            )
            .expect("synthetic standalone tag");
        assert!(synthetic.is_synthetic());
        assert_eq!(
            synthetic.get_element_complete_name().to_string_lossy(),
            "img"
        );
        assert_eq!(write_events(&[as_event(synthetic.as_ref())]), "");

        // 关闭标签
        let close = factory
            .create_close_element_tag(js("div"), false, false)
            .expect("close tag");
        assert_eq!(write_events(&[as_event(close.as_ref())]), "</div>");
        assert!(!close.has_location());
    });
}

// ===========================================================================
// 4. IModelFactory 属性操作：set/replace/remove 与身份保持
// ===========================================================================

#[test]
fn factory_attribute_operations_matches_java() {
    with_factory(TemplateMode::HTML, |factory| {
        // 无属性开标签（提升为 IProcessableElementTag 以便 set/replace/remove）
        let plain: Arc<dyn IProcessableElementTag> = factory
            .create_open_element_tag(js("div"), None, AttributeValueQuotes::DOUBLE, false)
            .expect("open tag");

        // setAttribute：新增属性 -> 新标签、默认 DOUBLE 引号、值可空
        let with_class = factory
            .set_attribute(plain.clone(), js("class"), Some(js("b")), None)
            .expect("set attribute");
        assert!(
            !Arc::ptr_eq(&plain, &with_class),
            "set on empty tag must create a new tag"
        );
        assert_eq!(
            write_events(&[as_event(with_class.as_ref())]),
            "<div class=\"b\">"
        );

        // setAttribute 覆盖已有属性（顺序保持、数量不变）
        let mut two = IndexMap::new();
        two.insert(js("id"), Some(js("i1")));
        two.insert(js("class"), Some(js("c1")));
        let base: Arc<dyn IProcessableElementTag> = factory
            .create_open_element_tag(js("div"), Some(&two), AttributeValueQuotes::DOUBLE, false)
            .expect("open tag");
        assert_eq!(
            write_events(&[as_event(base.as_ref())]),
            "<div id=\"i1\" class=\"c1\">"
        );
        let replaced = factory
            .set_attribute(base.clone(), js("class"), Some(js("c2")), None)
            .expect("set attribute");
        assert_eq!(
            write_events(&[as_event(replaced.as_ref())]),
            "<div id=\"i1\" class=\"c2\">"
        );

        // 无值属性（value null -> NONE 引号）
        let valueless = factory
            .set_attribute(plain.clone(), js("required"), None, None)
            .expect("set attribute");
        assert_eq!(
            write_events(&[as_event(valueless.as_ref())]),
            "<div required>"
        );

        // replaceAttribute：Java 语义 = 删除旧属性 + setAttribute（新名已存在
        // 时先移除再追加到末尾）——base `<div id="i1" class="c1">` 替换
        // class 为 id=i2：删除 class -> `<div id="i1">`，set id=i2 覆盖原位
        // id 并追加 -> `<div id="i2">`
        let class_name_value =
            AttributeNames::for_name(Some(TemplateMode::HTML), Some(&js("class")))
                .expect("attribute name");
        let class_name = class_name_value.as_attribute_name();
        let replaced_by_name = factory
            .replace_attribute(base.clone(), class_name, js("id"), Some(js("i2")), None)
            .expect("replace attribute");
        assert_eq!(
            write_events(&[as_event(replaced_by_name.as_ref())]),
            "<div id=\"i2\">"
        );

        // removeAttribute：删除存在属性 -> 新标签
        let removed = factory
            .remove_attribute(base.clone(), class_name)
            .expect("remove attribute");
        assert!(!Arc::ptr_eq(&base, &removed));
        assert_eq!(
            write_events(&[as_event(removed.as_ref())]),
            "<div id=\"i1\">"
        );
        assert!(
            removed
                .get_attribute_value(&js("class"))
                .expect("value")
                .is_none()
        );

        // removeAttribute：删除不存在属性 -> 原标签身份保持（Java 返回同一实例）。
        // 注意：Java 侧 AttributeName 全部为 repository 规范化单例，remove
        // 按值匹配；Rust 侧 AttributeNames::for_name 同样返回规范化实例，
        // 二者对可达输入行为一致。
        let absent_name_value =
            AttributeNames::for_name(Some(TemplateMode::HTML), Some(&js("class")))
                .expect("attribute name");
        let absent_name = absent_name_value.as_attribute_name();
        let removed_absent = factory
            .remove_attribute(removed.clone(), absent_name)
            .expect("remove attribute");
        assert!(
            Arc::ptr_eq(&removed, &removed_absent),
            "removing a non-existent attribute must keep identity"
        );
        assert_eq!(
            write_events(&[as_event(removed_absent.as_ref())]),
            "<div id=\"i1\">"
        );
        let removed_absent_twice = factory
            .remove_attribute(removed_absent.clone(), absent_name)
            .expect("remove attribute");
        assert!(
            Arc::ptr_eq(&removed_absent, &removed_absent_twice),
            "repeated removal of a non-existent attribute must keep identity"
        );

        // 独立标签上的属性操作
        let br: Arc<dyn IProcessableElementTag> = factory
            .create_standalone_element_tag(
                js("br"),
                None,
                AttributeValueQuotes::DOUBLE,
                false,
                true,
            )
            .expect("standalone tag");
        let br_with_attr = factory
            .set_attribute(br.clone(), js("title"), Some(js("t")), None)
            .expect("set attribute");
        assert_eq!(
            write_events(&[as_event(br_with_attr.as_ref())]),
            "<br title=\"t\"/>"
        );
    });
}

// ===========================================================================
// 5. IModelVisitor 分发顺序与 AbstractModelVisitor 默认空操作
// ===========================================================================

#[test]
fn model_accept_visitor_dispatch_order_matches_java() {
    with_factory(TemplateMode::HTML, |factory| {
        // Java Model.accept：按队列顺序逐事件分发到对应 visit 方法。
        // 注意：Java Model.add 禁止插入 TemplateStart/TemplateEnd
        // （“These events can only be added to models internally during
        // template parsing”），此处同样验证该拒绝语义。
        let mut model = factory.create_model();
        let boundary_rejected = model.add(Some(TemplateStart::instance()));
        assert!(
            boundary_rejected.is_err(),
            "adding TemplateStart to a Model must be rejected like Java"
        );
        let boundary_rejected = model.add(Some(TemplateEnd::instance()));
        assert!(
            boundary_rejected.is_err(),
            "adding TemplateEnd to a Model must be rejected like Java"
        );

        model
            .add(Some(
                factory
                    .create_xml_declaration(Some(js("1.0")), None, None)
                    .expect("xml declaration"),
            ))
            .expect("add xml declaration");
        model
            .add(Some(factory.create_html5_doc_type().expect("doctype")))
            .expect("add doctype");
        model
            .add(Some(
                factory.create_cdata_section(js("data")).expect("cdata"),
            ))
            .expect("add cdata");
        model
            .add(Some(factory.create_comment(js("c")).expect("comment")))
            .expect("add comment");
        model
            .add(Some(factory.create_text(js("t"))))
            .expect("add text");
        model
            .add(Some(
                factory
                    .create_standalone_element_tag(
                        js("br"),
                        None,
                        AttributeValueQuotes::DOUBLE,
                        false,
                        true,
                    )
                    .expect("standalone tag"),
            ))
            .expect("add standalone tag");
        model
            .add(Some(
                factory
                    .create_open_element_tag(js("div"), None, AttributeValueQuotes::DOUBLE, false)
                    .expect("open tag"),
            ))
            .expect("add open tag");
        model
            .add(Some(
                factory
                    .create_close_element_tag(js("div"), false, false)
                    .expect("close tag"),
            ))
            .expect("add close tag");
        model
            .add(Some(
                factory
                    .create_processing_instruction(js("p"), js("d"))
                    .expect("pi"),
            ))
            .expect("add pi");

        let visitor = RecordingVisitor::default();
        {
            let mut visitor = visitor;
            model.accept(&mut visitor);
            assert_eq!(
                visitor.log(),
                vec![
                    "visit_xml_declaration",
                    "visit_doc_type",
                    "visit_cdata_section",
                    "visit_comment",
                    "visit_text",
                    "visit_standalone_element_tag",
                    "visit_open_element_tag",
                    "visit_close_element_tag",
                    "visit_processing_instruction",
                ]
            );
        }
    });
}

#[test]
fn abstract_model_visitor_noop_defaults_match_java() {
    with_factory(TemplateMode::HTML, |factory| {
        let mut model = factory.create_model();
        model
            .add(Some(factory.create_text(js("a"))))
            .expect("add text");
        model
            .add(Some(factory.create_comment(js("b")).expect("comment")))
            .expect("add comment");
        model
            .add(Some(factory.create_text(js("c"))))
            .expect("add text");

        // 仅覆盖 visitText 的访问器；其余 10 类事件委托 AbstractModelVisitor
        // 的空操作默认（Java AbstractModelVisitor 全部为空实现）。
        struct OnlyTextVisitor {
            log: Rc<RefCell<Vec<String>>>,
        }
        impl IModelVisitor for OnlyTextVisitor {
            fn visit_text(&mut self, text: &dyn IText) {
                self.log.borrow_mut().push(format!(
                    "text:{}",
                    text.get_text()
                        .expect("text access")
                        .expect("non-null")
                        .to_string_lossy()
                ));
            }
            fn visit_template_start(&mut self, event: &dyn ITemplateStart) {
                AbstractModelVisitor::new().visit_template_start(event);
            }
            fn visit_template_end(&mut self, event: &dyn ITemplateEnd) {
                AbstractModelVisitor::new().visit_template_end(event);
            }
            fn visit_xml_declaration(&mut self, event: &dyn IXMLDeclaration) {
                AbstractModelVisitor::new().visit_xml_declaration(event);
            }
            fn visit_doc_type(&mut self, event: &dyn IDocType) {
                AbstractModelVisitor::new().visit_doc_type(event);
            }
            fn visit_cdata_section(&mut self, event: &dyn ICDATASection) {
                AbstractModelVisitor::new().visit_cdata_section(event);
            }
            fn visit_comment(&mut self, event: &dyn IComment) {
                AbstractModelVisitor::new().visit_comment(event);
            }
            fn visit_standalone_element_tag(&mut self, event: &dyn IStandaloneElementTag) {
                AbstractModelVisitor::new().visit_standalone_element_tag(event);
            }
            fn visit_open_element_tag(&mut self, event: &dyn IOpenElementTag) {
                AbstractModelVisitor::new().visit_open_element_tag(event);
            }
            fn visit_close_element_tag(&mut self, event: &dyn ICloseElementTag) {
                AbstractModelVisitor::new().visit_close_element_tag(event);
            }
            fn visit_processing_instruction(&mut self, event: &dyn IProcessingInstruction) {
                AbstractModelVisitor::new().visit_processing_instruction(event);
            }
        }

        let log = Rc::new(RefCell::new(Vec::new()));
        let visitor = OnlyTextVisitor {
            log: Rc::clone(&log),
        };
        {
            let mut visitor = visitor;
            model.accept(&mut visitor);
        }
        assert_eq!(
            log.borrow().clone(),
            vec!["text:a".to_owned(), "text:c".to_owned()],
            "only visitText must be observed; all other visits are no-ops"
        );
    });
}

// ===========================================================================
// 6. IModelFactory.parse 往返：单例起止事件与解析位置
// ===========================================================================

#[test]
fn factory_parse_roundtrip_matches_java() {
    with_factory(TemplateMode::HTML, |factory| {
        let owner = TemplateData::new(
            Some(js("test-template")),
            None,
            None,
            Some(TemplateMode::HTML),
            None,
        );

        // 纯文本模板：TemplateStart 单例 + Text + TemplateEnd 单例
        let model = factory
            .parse(&owner, &js("hello"))
            .expect("parse plain text");
        assert_eq!(model.size(), 3, "start + text + end");
        let start = model.get(0);
        assert!(start.is_template_start());
        assert!(
            !start.has_location(),
            "parsed template start is the location-less singleton (Java asEngineTemplateStart)"
        );
        let text = model.get(1);
        assert_eq!(
            text.as_text()
                .expect("text event")
                .get_text()
                .expect("text access")
                .expect("non-null")
                .to_string_lossy(),
            "hello"
        );
        assert!(text.has_location(), "parsed text carries location");
        assert_eq!(text.get_line(), 1);
        assert_eq!(text.get_col(), 1);
        assert_eq!(
            text.get_template_name().unwrap().to_string_lossy(),
            "test-template"
        );
        let end = model.get(2);
        assert!(end.is_template_end());
        assert!(!end.has_location(), "parsed template end is the singleton");

        // 带标签模板：文档顺序事件队列 + 各事件位置
        let full = factory
            .parse(&owner, &js("<!DOCTYPE html><p>x</p>"))
            .expect("parse markup");
        assert_eq!(
            full.size(),
            6,
            "start + doctype + open + text + close + end"
        );
        let doctype = full.get(1);
        assert!(doctype.has_location());
        assert_eq!(doctype.get_line(), 1);
        assert_eq!(doctype.get_col(), 1);
        assert_eq!(
            write_events(&[as_event(doctype.as_ref())]),
            "<!DOCTYPE html>"
        );
        let open = full.get(2);
        assert!(
            open.as_open_element_tag().is_some(),
            "third event must be the open tag"
        );
        assert_eq!(
            write_events(&[as_event(open.as_open_element_tag().unwrap())]),
            "<p>"
        );
        assert_eq!(open.get_line(), 1);
        assert_eq!(open.get_col(), 16);
        let text = full.get(3);
        assert_eq!(
            write_events(&[as_event(text.as_text().expect("text event"))]),
            "x"
        );
        assert_eq!(text.get_line(), 1);
        assert_eq!(text.get_col(), 19);
        let close = full.get(4);
        assert!(
            close.as_close_element_tag().is_some(),
            "fifth event must be the close tag"
        );
        assert_eq!(
            write_events(&[as_event(close.as_close_element_tag().unwrap())]),
            "</p>"
        );

        // 解析模型含起止单例边界事件：Java 中边界事件仅能由解析内部加入，
        // accept 仍按文档顺序分发 visitTemplateStart/visitTemplateEnd
        let visitor = RecordingVisitor::default();
        {
            let mut visitor = visitor;
            full.accept(&mut visitor);
            assert_eq!(
                visitor.log(),
                vec![
                    "visit_template_start",
                    "visit_doc_type",
                    "visit_open_element_tag",
                    "visit_text",
                    "visit_close_element_tag",
                    "visit_template_end",
                ]
            );
        }
    });
}
