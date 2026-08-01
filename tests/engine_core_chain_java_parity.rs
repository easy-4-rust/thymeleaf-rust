//! engine 核心链 Java 1:1 差分测试。
//!
//! 覆盖对象（对象表编号）：
//! - `DataDrivenTemplateIterator`（69）：FIFO 缓冲、hasNext/next、
//!   NoSuchElementException、remove 不支持、feedBuffer/feedingComplete/
//!   continueBufferExecution、hasBeenQueried，以及 SSE 前缀/首 ID/回退
//!   语义（记录型 SSE 控制器直测）；
//! - `AbstractTemplateHandler`（56）：next 链委托（全部 11 类事件）、
//!   setNext(null) 空操作、getNext/getContext；
//! - `ModelBuilderTemplateHandler`（99）：事件收集为 TemplateModel，
//!   模板起止事件替换为无位置单例（Java `asEngineTemplateStart`）。

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use thymeleaf::context::ITemplateContext;
use thymeleaf::engine::{
    DataDrivenTemplateIterator, DataDrivenTemplateIteratorError, IThrottledTemplateWriterControl,
    ISSEThrottledTemplateWriterControl, ITemplateHandler, ModelBuilderTemplateHandler, TemplateData,
    TemplateEnd, TemplateStart, TemplateHandlerHandle,
};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IModel, IModelVisitor, IOpenElementTag,
    IProcessingInstruction, IStandaloneElementTag, ITemplateEnd, ITemplateStart, IText,
    IXMLDeclaration,
};
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::JavaString;
use thymeleaf::{ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode};

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn engine() -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

// ===========================================================================
// 1. DataDrivenTemplateIterator（数据驱动迭代器）
// ===========================================================================

#[test]
fn data_driven_template_iterator_semantics_match_java() {
    // Java：初始空缓冲，未查询
    let mut iterator = DataDrivenTemplateIterator::<i32>::new();
    assert!(!iterator.has_been_queried(), "initial not queried");
    assert!(!iterator.has_next(), "initial hasNext false");
    assert!(!iterator.continue_buffer_execution(), "initial no buffer");

    // Java：hasNext/next 会置 queried=true
    let _ = iterator.has_next();
    assert!(iterator.has_been_queried(), "hasNext sets queried");

    // Java：空缓冲 next() 抛 NoSuchElementException
    let mut empty = DataDrivenTemplateIterator::<i32>::new();
    let error = empty.next_java().expect_err("empty next must fail");
    assert!(matches!(
        error,
        DataDrivenTemplateIteratorError::NoSuchElement
    ));
    assert_eq!(error.to_string(), "java.util.NoSuchElementException");

    // Java：feedBuffer 后按 FIFO 取出（values.remove(0)）
    iterator.feed_buffer([1, 2, 3]);
    assert!(iterator.has_next());
    assert!(iterator.continue_buffer_execution());
    assert_eq!(iterator.next_java().expect("next 1"), 1);
    assert_eq!(iterator.next_java().expect("next 2"), 2);
    assert_eq!(iterator.next_java().expect("next 3"), 3);
    assert!(!iterator.has_next());
    assert!(!iterator.continue_buffer_execution());

    // Java：remove() 抛 UnsupportedOperationException
    let error = iterator.remove().expect_err("remove must fail");
    assert!(matches!(error, DataDrivenTemplateIteratorError::RemoveUnsupported));
    assert_eq!(
        error.to_string(),
        "remove() is not supported in Throttled Iterator"
    );

    // Java：feedingComplete 后空缓冲保持非可继续
    iterator.feeding_complete();
    assert!(!iterator.continue_buffer_execution());
    assert!(!iterator.has_next());

    // 再次喂入仍可继续（Java values 与 feedingComplete 独立）
    iterator.feed_buffer([7]);
    assert!(iterator.has_next());
    assert_eq!(iterator.next_java().expect("next 7"), 7);
}

/// 记录 SSE 事件的控制器替身（对应 Java 匿名 ISSEThrottledTemplateWriterControl）。
struct RecordingSseControl {
    events: Arc<Mutex<Vec<String>>>,
}

impl RecordingSseControl {
    fn new(events: Arc<Mutex<Vec<String>>>) -> Self {
        Self { events }
    }
}

impl IThrottledTemplateWriterControl for RecordingSseControl {
    fn as_sse_control(&mut self) -> Option<&mut dyn ISSEThrottledTemplateWriterControl> {
        Some(self)
    }
    fn is_overflown(&mut self) -> std::io::Result<bool> {
        Ok(false)
    }
    fn is_stopped(&mut self) -> std::io::Result<bool> {
        Ok(false)
    }
    fn get_written_count(&self) -> i32 {
        0
    }
    fn get_max_overflow_size(&self) -> i32 {
        0
    }
    fn get_overflow_grow_count(&self) -> i32 {
        0
    }
}

impl ISSEThrottledTemplateWriterControl for RecordingSseControl {
    fn start_event(&mut self, id: Option<&[u16]>, event: Option<&[u16]>) {
        let id = id
            .map(String::from_utf16_lossy)
            .unwrap_or_default();
        let event = event
            .map(String::from_utf16_lossy)
            .unwrap_or_default();
        self.events
            .lock()
            .expect("events lock")
            .push(format!("start({id},{event})"));
    }
    fn end_event(&mut self) -> std::io::Result<()> {
        self.events.lock().expect("events lock").push("end".to_owned());
        Ok(())
    }
}

#[test]
fn data_driven_template_iterator_sse_events_match_java() {
    // Java：默认无前缀时 message 事件名为 "message"（composeToken 拼接前缀）
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut iterator = DataDrivenTemplateIterator::<i32>::new();
    iterator.set_writer_control(Box::new(RecordingSseControl::new(Arc::clone(&events))));

    // startIteration：id = 当前 SSE id，event = 组合 message 名；id 自增
    iterator.start_iteration();
    iterator.finish_iteration().expect("finish iteration");
    iterator.start_iteration();
    iterator.finish_iteration().expect("finish iteration");

    // Java：takeBackLastEventID 回退最后发出的 id
    iterator.take_back_last_event_id();
    iterator.start_iteration();
    iterator.finish_iteration().expect("finish iteration");

    // Java：startHead/startTail 使用 "head"/"tail" 事件名
    iterator.start_head();
    iterator.finish_step().expect("finish head step");
    iterator.start_tail();
    iterator.finish_step().expect("finish tail step");

    assert_eq!(
        events.lock().expect("events lock").clone(),
        vec![
            "start(0,message)".to_owned(),
            "end".to_owned(),
            "start(1,message)".to_owned(),
            "end".to_owned(),
            "start(1,message)".to_owned(),
            "end".to_owned(),
            "start(2,head)".to_owned(),
            "end".to_owned(),
            "start(3,tail)".to_owned(),
            "end".to_owned(),
        ]
    );
}

#[test]
fn data_driven_template_iterator_sse_prefix_and_first_id_match_java() {
    // Java：setSseEventsPrefix 后 composeToken 对 id 与事件名都拼接
    // `prefix + "-" + token`（Java `composedToken[prefix.length] = '-'`）；
    // setSseEventsFirstID 设定起始 id
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut iterator = DataDrivenTemplateIterator::<i32>::new();
    iterator.set_writer_control(Box::new(RecordingSseControl::new(Arc::clone(&events))));
    iterator.set_sse_events_prefix(Some(&js("ev-")));
    iterator.set_sse_events_first_id(5);
    iterator.start_iteration();
    iterator.finish_iteration().expect("finish iteration");
    // 前缀与首 id 对 head/tail 同样生效
    iterator.start_head();
    iterator.finish_step().expect("finish head step");

    assert_eq!(
        events.lock().expect("events lock").clone(),
        vec![
            "start(ev--5,ev--message)".to_owned(),
            "end".to_owned(),
            "start(ev--6,ev--head)".to_owned(),
            "end".to_owned(),
        ]
    );
}

// ===========================================================================
// 2. AbstractTemplateHandler（next 链委托）
// ===========================================================================

/// 记录全部 11 类事件的处理链末端。
#[derive(Default)]
struct RecordingHandler {
    log: Rc<RefCell<Vec<String>>>,
}

impl ITemplateHandler for RecordingHandler {
    fn set_next(&mut self, _next: Option<TemplateHandlerHandle>) {}
    fn set_context(&mut self, _context: Arc<dyn ITemplateContext>) {}
    fn handle_template_start(
        &mut self,
        _template_start: Arc<dyn ITemplateStart>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.log.borrow_mut().push("template_start".to_owned());
        Ok(())
    }
    fn handle_template_end(
        &mut self,
        _template_end: Arc<dyn ITemplateEnd>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.log.borrow_mut().push("template_end".to_owned());
        Ok(())
    }
    fn handle_xml_declaration(
        &mut self,
        _xml_declaration: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.log.borrow_mut().push("xml_declaration".to_owned());
        Ok(())
    }
    fn handle_doc_type(
        &mut self,
        _doc_type: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.log.borrow_mut().push("doc_type".to_owned());
        Ok(())
    }
    fn handle_cdata_section(
        &mut self,
        _cdata_section: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.log.borrow_mut().push("cdata_section".to_owned());
        Ok(())
    }
    fn handle_comment(
        &mut self,
        _comment: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.log.borrow_mut().push("comment".to_owned());
        Ok(())
    }
    fn handle_text(
        &mut self,
        _text: Arc<dyn IText>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.log.borrow_mut().push("text".to_owned());
        Ok(())
    }
    fn handle_standalone_element(
        &mut self,
        _standalone_element_tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.log.borrow_mut().push("standalone_element".to_owned());
        Ok(())
    }
    fn handle_open_element(
        &mut self,
        _open_element_tag: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.log.borrow_mut().push("open_element".to_owned());
        Ok(())
    }
    fn handle_close_element(
        &mut self,
        _close_element_tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.log.borrow_mut().push("close_element".to_owned());
        Ok(())
    }
    fn handle_processing_instruction(
        &mut self,
        _processing_instruction: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.log.borrow_mut().push("processing_instruction".to_owned());
        Ok(())
    }
}

#[test]
fn abstract_template_handler_chain_delegation_matches_java() {
    // Java：AbstractTemplateHandler 未覆盖的事件原样转发给 next；
    // next 为 null 时静默忽略。
    let log = Rc::new(RefCell::new(Vec::new()));
    let recording = RecordingHandler {
        log: Rc::clone(&log),
    };
    let mut base = thymeleaf::engine::AbstractTemplateHandler::with_next(Box::new(recording));

    // 11 类事件逐类转发
    base.handle_template_start(TemplateStart::instance())
        .expect("forward template start");
    base.handle_xml_declaration(
        engine()
            .get_configuration()
            .expect("config")
            .get_model_factory(TemplateMode::HTML)
            .create_xml_declaration(Some(js("1.0")), None, None)
            .expect("xml declaration"),
    )
    .expect("forward xml declaration");
    base.handle_doc_type(
        engine()
            .get_configuration()
            .expect("config")
            .get_model_factory(TemplateMode::HTML)
            .create_html5_doc_type()
            .expect("doctype"),
    )
    .expect("forward doctype");
    base.handle_cdata_section(
        engine()
            .get_configuration()
            .expect("config")
            .get_model_factory(TemplateMode::HTML)
            .create_cdata_section(js("d"))
            .expect("cdata"),
    )
    .expect("forward cdata");
    base.handle_comment(
        engine()
            .get_configuration()
            .expect("config")
            .get_model_factory(TemplateMode::HTML)
            .create_comment(js("c"))
            .expect("comment"),
    )
    .expect("forward comment");
    base.handle_text(
        engine()
            .get_configuration()
            .expect("config")
            .get_model_factory(TemplateMode::HTML)
            .create_text(js("t")),
    )
    .expect("forward text");
    base.handle_standalone_element(
        engine()
            .get_configuration()
            .expect("config")
            .get_model_factory(TemplateMode::HTML)
            .create_standalone_element_tag(
                js("br"),
                None,
                thymeleaf::model::AttributeValueQuotes::DOUBLE,
                false,
                true,
            )
            .expect("standalone"),
    )
    .expect("forward standalone");
    base.handle_open_element(
        engine()
            .get_configuration()
            .expect("config")
            .get_model_factory(TemplateMode::HTML)
            .create_open_element_tag(js("div"), None, thymeleaf::model::AttributeValueQuotes::DOUBLE, false)
            .expect("open"),
    )
    .expect("forward open");
    base.handle_close_element(
        engine()
            .get_configuration()
            .expect("config")
            .get_model_factory(TemplateMode::HTML)
            .create_close_element_tag(js("div"), false, false)
            .expect("close"),
    )
    .expect("forward close");
    base.handle_processing_instruction(
        engine()
            .get_configuration()
            .expect("config")
            .get_model_factory(TemplateMode::HTML)
            .create_processing_instruction(js("p"), js("d"))
            .expect("pi"),
    )
    .expect("forward pi");
    base.handle_template_end(TemplateEnd::instance())
        .expect("forward template end");

    assert_eq!(
        log.borrow().clone(),
        vec![
            "template_start",
            "xml_declaration",
            "doc_type",
            "cdata_section",
            "comment",
            "text",
            "standalone_element",
            "open_element",
            "close_element",
            "processing_instruction",
            "template_end",
        ]
    );

    // Java：setNext(null) 后事件静默忽略
    base.set_next(None);
    base.handle_text(
        engine()
            .get_configuration()
            .expect("config")
            .get_model_factory(TemplateMode::HTML)
            .create_text(js("ignored")),
    )
    .expect("no next no-op");
    assert_eq!(log.borrow().len(), 11, "no forwarding without next");
    assert!(
        base.get_next().is_none(),
        "getNext must reflect the cleared chain"
    );
}

// ===========================================================================
// 3. ModelBuilderTemplateHandler（事件收集为 TemplateModel）
// ===========================================================================

#[test]
fn model_builder_template_handler_collects_events_matches_java() {
    let configuration = engine().get_configuration().expect("configuration");
    let template_data = Arc::new(TemplateData::new(
        Some(js("test")),
        None,
        None,
        Some(TemplateMode::HTML),
        None,
    ));
    let mut builder = ModelBuilderTemplateHandler::new(configuration.clone(), Arc::clone(&template_data));

    builder
        .handle_template_start(TemplateStart::instance())
        .expect("start");
    builder
        .handle_text(
            configuration
                .get_model_factory(TemplateMode::HTML)
                .create_text(js("hello")),
        )
        .expect("text");
    builder
        .handle_template_end(TemplateEnd::instance())
        .expect("end");

    let model = builder.get_model().expect("template model");
    assert_eq!(model.size(), 3, "start + text + end");

    // Java TemplateModel：queue[0] 恒为 TEMPLATE_START_INSTANCE（无位置单例）
    let start = model.get(0);
    assert!(start.is_template_start());
    assert!(!start.has_location());

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

    let end = model.get(2);
    assert!(end.is_template_end());
    assert!(!end.has_location());

    // Java TemplateModel.getTemplateData() 返回构建时的模板数据
    assert_eq!(
        model
            .get_template_data()
            .get_template()
            .unwrap()
            .to_string_lossy(),
        "test"
    );

    // accept 分发仍按文档顺序工作（起止单例 -> visitTemplateStart/End）
    struct BoundaryVisitor {
        log: Rc<RefCell<Vec<String>>>,
    }
    impl IModelVisitor for BoundaryVisitor {
        fn visit_template_start(&mut self, _template_start: &dyn ITemplateStart) {
            self.log.borrow_mut().push("start".to_owned());
        }
        fn visit_template_end(&mut self, _template_end: &dyn ITemplateEnd) {
            self.log.borrow_mut().push("end".to_owned());
        }
        fn visit_xml_declaration(&mut self, _xml_declaration: &dyn IXMLDeclaration) {}
        fn visit_doc_type(&mut self, _doc_type: &dyn IDocType) {}
        fn visit_cdata_section(&mut self, _cdata_section: &dyn ICDATASection) {}
        fn visit_comment(&mut self, _comment: &dyn IComment) {}
        fn visit_text(&mut self, _text: &dyn IText) {}
        fn visit_standalone_element_tag(&mut self, _standalone: &dyn IStandaloneElementTag) {}
        fn visit_open_element_tag(&mut self, _open: &dyn IOpenElementTag) {}
        fn visit_close_element_tag(&mut self, _close: &dyn ICloseElementTag) {}
        fn visit_processing_instruction(&mut self, _pi: &dyn IProcessingInstruction) {}
    }
    let log = Rc::new(RefCell::new(Vec::new()));
    let visitor = BoundaryVisitor { log: Rc::clone(&log) };
    {
        let mut visitor = visitor;
        model.accept(&mut visitor);
    }
    assert_eq!(log.borrow().clone(), vec!["start".to_owned(), "end".to_owned()]);
}
