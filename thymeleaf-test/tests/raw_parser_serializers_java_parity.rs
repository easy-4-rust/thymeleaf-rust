//! raw 解析族与序列化/处理器工具 Java 1:1 差分测试。
//!
//! 覆盖对象（对象表编号）：
//! - `RawParser`（396）+ `IRawHandler`（394）+ `RawParseException`（395）：
//!   文档起止 + 单文本事件（Java RawParser 语义），异常构造与 Java 类名；
//! - `StandardSerializers`（376）：Standard Dialect 注册的 JS/CSS 序列化器
//!   获取（执行参数属性合同）；
//! - `StandardProcessorUtils`（379）：`replaceAttribute`/`setAttribute`
//!   辅助（标签属性替换语义，与 IModelFactory 差分一致）；
//! - 表滞后结算：`OutputExpressionInlinePreProcessorHandler`（309）与
//!   `InlinedOutputExpressionMarkupHandler`（384）← 处理器族批 inline
//!   fixture（inline08/09/29/33 全程经过两个处理器）；
//!   `DecoupledTemplateLogicMarkupHandler`（390）←
//!   `decoupled_logic_java_parity.rs` 既有差分。

use std::sync::Arc;

use thymeleaf::raw::{IRawHandler, RawParseCause, RawParseException, RawParser};
use thymeleaf::serializer::StandardSerializers;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::{JavaString, StandardProcessorUtils};
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
// 1. RawParser（396）+ IRawHandler（394）：文档起止 + 单文本事件
// ===========================================================================

/// 记录 raw 解析事件的 Handler（对应 Java 匿名 IRawHandler）。
#[derive(Default)]
struct RecordingRawHandler {
    log: Vec<String>,
}

impl IRawHandler for RecordingRawHandler {
    fn handle_document_start(
        &mut self,
        start_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), RawParseException> {
        self.log.push(format!(
            "start(nanos={start_time_nanos},line={line},col={col})"
        ));
        Ok(())
    }
    fn handle_document_end(
        &mut self,
        end_time_nanos: i64,
        total_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), RawParseException> {
        self.log.push(format!(
            "end(end={end_time_nanos},total={total_time_nanos},line={line},col={col})"
        ));
        Ok(())
    }
    fn handle_text(
        &mut self,
        buffer: Option<&[u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), RawParseException> {
        let text = buffer
            .map(|units| {
                units
                    .get(offset as usize..(offset + len) as usize)
                    .map(String::from_utf16_lossy)
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        self.log
            .push(format!("text({text:?},line={line},col={col})"));
        Ok(())
    }
}

#[test]
fn raw_parser_document_events_match_java() {
    // Java RawParser：文档开始（位置 1,1）-> 单个 Text（完整内容）->
    // 文档结束（位置随内容长度）；内容原样保留（RAW 不解析）
    let parser = RawParser::new(2, 4096);
    let mut handler = RecordingRawHandler::default();

    parser
        .parse_string(Some(js("<html>anything</html>")), Some(&mut handler))
        .expect("raw parse");
    assert_eq!(handler.log.len(), 3, "start + text + end");
    // 起始时间由 Java 计时产生（纳秒），位置恒为 1,1
    assert!(
        handler.log[0].starts_with("start(nanos="),
        "start event shape"
    );
    assert!(handler.log[0].ends_with(",line=1,col=1)"), "start at 1,1");
    assert_eq!(
        handler.log[1],
        "text(\"<html>anything</html>\",line=1,col=1)"
    );
    // 文档结束：col = 末字符列（内容长度），时间字段由 Java 计时产生
    assert!(handler.log[2].starts_with("end(end="), "end event shape");
    assert!(
        handler.log[2].contains(",line=1,col=21)"),
        "end col = 内容长度"
    );
    assert!(handler.log[2].contains(",total="), "total time present");

    // 多行内容：行/列随内容推进
    let mut handler = RecordingRawHandler::default();
    parser
        .parse_string(Some(js("line1\nline2")), Some(&mut handler))
        .expect("raw parse");
    assert_eq!(handler.log[1], "text(\"line1\\nline2\",line=1,col=1)");

    // Java Validate.notNull：null 文档/Handler 拒绝
    match parser.parse_string(None, Some(&mut handler)) {
        Err(error) => assert!(
            error.to_string().contains("Document cannot be null"),
            "error text: {error}"
        ),
        Ok(()) => panic!("null document must be rejected"),
    }
    match parser.parse_string(Some(js("x")), None) {
        Err(error) => assert!(
            error.to_string().contains("Handler cannot be null"),
            "error text: {error}"
        ),
        Ok(()) => panic!("null handler must be rejected"),
    }
}

// ===========================================================================
// 2. RawParseException（395）：构造与 Java 类名
// ===========================================================================

#[test]
fn raw_parse_exception_contract_matches_java() {
    // Java RawParseException(message)：消息与行/列 null
    let exception = RawParseException::with_message(Some(js("boom")));
    assert_eq!(exception.to_string(), "boom");

    // Java RawParseException(message, cause)
    let exception = RawParseException::with_message_and_cause(
        Some(js("at 3:5")),
        Some(RawParseCause::from_raw_parse(
            RawParseException::with_message(Some(js("inner"))),
        )),
    );
    assert_eq!(exception.to_string(), "at 3:5");

    // Java 消息为 null 时 Display 回退到原因消息（with_cause 语义）
    let exception = RawParseException::with_cause(Some(RawParseCause::from_raw_parse(
        RawParseException::with_message(Some(js("inner"))),
    )));
    assert_eq!(exception.to_string(), "inner");

    // 带行/列的构造（Java RawParseException(message, line, col)）：
    // Display 前缀带 "(Line = L, Column = C)"
    let exception = RawParseException::with_message_at(Some(&js("at")), 3, 5);
    assert_eq!(exception.to_string(), "(Line = 3, Column = 5) at");
}

// ===========================================================================
// 3. StandardSerializers（376）：执行参数属性合同
// ===========================================================================

#[test]
fn standard_serializers_execution_attributes_match_java() {
    let configuration = engine().get_configuration().expect("configuration");

    // Standard Dialect 注册了两个序列化器执行参数（Java 属性名合同）
    let js_serializer = StandardSerializers::get_java_script_serializer(configuration.as_ref())
        .expect("javascript serializer");
    let css_serializer =
        StandardSerializers::get_css_serializer(configuration.as_ref()).expect("css serializer");

    // 序列化器可实际产出 JS/CSS 值（与 inliner 批 golden 一致）
    let mut writer = thymeleaf::util::FastStringWriter::new();
    js_serializer
        .serialize_value(
            Some(&thymeleaf::expression::TemplateValue::string(js("hello"))),
            &mut writer,
        )
        .expect("js serialize");
    assert_eq!(writer.to_string().to_string_lossy(), "\"hello\"");
    let mut writer = thymeleaf::util::FastStringWriter::new();
    css_serializer
        .serialize_value(
            Some(&thymeleaf::expression::TemplateValue::string(js("hello"))),
            &mut writer,
        )
        .expect("css serialize");
    assert_eq!(writer.to_string().to_string_lossy(), "hello");
}

// ===========================================================================
// 4. StandardProcessorUtils（379）：标签属性辅助
// ===========================================================================

#[test]
fn standard_processor_utils_attribute_helpers_match_java() {
    let configuration = engine().get_configuration().expect("configuration");
    // Java StandardProcessorUtils 是结构处理器辅助：把属性操作委托给
    // structureHandler（等价 IModelFactory 语义），此处以记录型 handler
    // 验证委托参数。
    #[derive(Default)]
    struct RecordingStructureHandler {
        log: Vec<String>,
    }
    impl thymeleaf::element::IElementTagStructureHandler for RecordingStructureHandler {
        fn reset(&mut self) {}
        fn set_local_variable(
            &mut self,
            _name: JavaString,
            _value: Option<Arc<thymeleaf::expression::TemplateValue>>,
        ) {
        }
        fn remove_local_variable(&mut self, _name: JavaString) {}
        fn set_attribute(
            &mut self,
            attribute_name: JavaString,
            attribute_value: Option<JavaString>,
            _quotes: Option<thymeleaf::model::AttributeValueQuotes>,
        ) {
            self.log.push(format!(
                "set({},{})",
                attribute_name.to_string_lossy(),
                attribute_value
                    .map(|v| v.to_string_lossy())
                    .unwrap_or_default()
            ));
        }
        fn replace_attribute(
            &mut self,
            old_attribute_name: thymeleaf::engine::AttributeNameValue,
            attribute_name: JavaString,
            attribute_value: Option<JavaString>,
            _quotes: Option<thymeleaf::model::AttributeValueQuotes>,
        ) {
            self.log.push(format!(
                "replace({},{},{})",
                old_attribute_name
                    .as_attribute_name()
                    .to_java_string()
                    .expect("name text")
                    .to_string_lossy(),
                attribute_name.to_string_lossy(),
                attribute_value
                    .map(|v| v.to_string_lossy())
                    .unwrap_or_default()
            ));
        }
        fn remove_attribute(&mut self, _attribute_name: JavaString) {}
        fn remove_attribute_with_prefix(&mut self, _prefix: Option<JavaString>, _name: JavaString) {
        }
        fn remove_attribute_name(
            &mut self,
            _attribute_name: thymeleaf::engine::AttributeNameValue,
        ) {
        }
        fn set_selection_target(
            &mut self,
            _selection_target: Option<Arc<thymeleaf::expression::TemplateValue>>,
        ) {
        }
        fn set_inliner(&mut self, _inliner: Option<Arc<dyn thymeleaf::inline::IInliner>>) {}
        fn set_template_data(&mut self, _template_data: Arc<thymeleaf::engine::TemplateData>) {}
        fn set_body_text(&mut self, _text: JavaString, _processable: bool) {}
        fn set_body_sequence(
            &mut self,
            _text: Arc<dyn thymeleaf::util::JavaCharSequence>,
            _processable: bool,
        ) {
        }
        fn set_body_model(
            &mut self,
            _model: Arc<dyn thymeleaf::model::IModel>,
            _processable: bool,
        ) {
        }
        fn insert_before(&mut self, _model: Arc<dyn thymeleaf::model::IModel>) {}
        fn insert_immediately_after(
            &mut self,
            _model: Arc<dyn thymeleaf::model::IModel>,
            _processable: bool,
        ) {
        }
        fn replace_with_text(&mut self, _text: JavaString, _processable: bool) {}
        fn replace_with_model(
            &mut self,
            _model: Arc<dyn thymeleaf::model::IModel>,
            _processable: bool,
        ) {
        }
        fn remove_element(&mut self) {}
        fn remove_tags(&mut self) {}
        fn remove_body(&mut self) {}
        fn remove_all_but_first_child(&mut self) {}
        fn iterate_element(
            &mut self,
            _iter_variable_name: JavaString,
            _iter_status_variable_name: Option<JavaString>,
            _iterated_object: Option<Arc<thymeleaf::expression::TemplateValue>>,
        ) -> Result<(), thymeleaf::util::ValidateError> {
            Ok(())
        }
    }

    let mut handler = RecordingStructureHandler::default();
    // setAttribute(structureHandler, definition, name, value)
    let attribute_definition_value = configuration
        .get_attribute_definitions()
        .for_name(Some(TemplateMode::HTML), Some(&js("id")))
        .expect("attribute definition");
    let attribute_definition = attribute_definition_value.as_attribute_definition();
    StandardProcessorUtils::set_attribute(
        &mut handler,
        attribute_definition,
        js("id"),
        Some(js("x")),
    );
    assert_eq!(handler.log, vec!["set(id,x)"]);

    // replaceAttribute(structureHandler, oldName, definition, name, value)
    let old_name =
        thymeleaf::engine::AttributeNames::for_name(Some(TemplateMode::HTML), Some(&js("class")))
            .expect("attribute name");
    handler.log.clear();
    StandardProcessorUtils::replace_attribute(
        &mut handler,
        old_name,
        attribute_definition,
        js("id"),
        Some(js("y")),
    );
    assert_eq!(handler.log, vec!["replace({class},id,y)"]);
}
