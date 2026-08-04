//! `StandardModelFactory`/`AbstractAttributeModelProcessor` Java Golden 差分测试。
//!
//! 直接实例化公共 API：
//! - StandardModelFactory：create_model/事件创建/TEXT 模式限制
//! - AbstractAttributeModelProcessor：构造校验、匹配规则、公共 getter

use std::sync::Arc;

use thymeleaf::element::IElementModelStructureHandler;
use thymeleaf::element::{AbstractAttributeModelProcessor, IElementProcessor};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::{IModel, IModelFactory};
use thymeleaf::processor::IProcessor;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Utf16String;
use thymeleaf::{AttributeName, ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode};

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
}

fn engine() -> TemplateEngine {
    let mut r = StringTemplateResolver::new();
    r.set_template_mode(TemplateMode::HTML);
    let e = TemplateEngine::new();
    e.set_template_resolver(Arc::new(r) as Arc<dyn ITemplateResolver>)
        .unwrap();
    e
}

fn with_factory<T>(mode: TemplateMode, f: impl FnOnce(&dyn IModelFactory) -> T) -> T {
    let config = engine().get_configuration().expect("config");
    f(config.get_model_factory(mode))
}

/// AbstractAttributeModelProcessor 的 doProcess 回调签名（clippy type_complexity 别名）。
type DoProcessFn = dyn Fn(
        &dyn thymeleaf::context::ITemplateContext,
        &mut dyn IModel,
        &AttributeName,
        Option<Utf16String>,
        &mut dyn IElementModelStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>
    + Send
    + Sync;

fn noop_process() -> Box<DoProcessFn> {
    Box::new(
        |_ctx: &dyn thymeleaf::context::ITemplateContext,
         _model: &mut dyn IModel,
         _attribute_name: &AttributeName,
         _attribute_value: Option<Utf16String>,
         _structure_handler: &mut dyn IElementModelStructureHandler|
         -> Result<(), Box<dyn TemplateEngineException>> { Ok(()) },
    )
}

// ===========================================================================
// 1. StandardModelFactory 基础模型创建
// ===========================================================================

#[test]
fn create_empty_model() {
    with_factory(TemplateMode::HTML, |factory| {
        let model = factory.create_model();
        assert_eq!(model.size(), 0, "empty model");
    });
}

#[test]
fn create_text_event() {
    with_factory(TemplateMode::HTML, |factory| {
        let text = factory.create_text(js("hello"));
        let c = text.get_text().expect("text access").expect("non-null");
        assert_eq!(c.to_string_lossy(), "hello");
    });
}

#[test]
fn create_comment_event() {
    with_factory(TemplateMode::HTML, |factory| {
        let comment = factory.create_comment(js("a comment")).expect("comment");
        let c = comment.get_content().expect("content").expect("non-null");
        assert_eq!(c.to_string_lossy(), "a comment");
    });
}

#[test]
fn create_html5_doc_type() {
    with_factory(TemplateMode::HTML, |factory| {
        let doc_type = factory.create_html5_doc_type().expect("html5 doctype");
        assert_eq!(doc_type.get_keyword().unwrap().to_string_lossy(), "DOCTYPE");
        assert_eq!(
            doc_type.get_element_name().unwrap().to_string_lossy(),
            "html"
        );
    });
}

#[test]
fn create_full_doc_type() {
    with_factory(TemplateMode::HTML, |factory| {
        let doc_type = factory
            .create_full_doc_type(
                js("DOCTYPE"),
                js("html"),
                Some(js("-//W3C//DTD XHTML 1.0//EN")),
                Some(js("http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd")),
                None,
            )
            .expect("full doctype");
        assert_eq!(
            doc_type.get_element_name().unwrap().to_string_lossy(),
            "html"
        );
    });
}

#[test]
fn create_processing_instruction() {
    with_factory(TemplateMode::HTML, |factory| {
        let pi = factory
            .create_processing_instruction(js("target"), js("data"))
            .expect("pi");
        assert_eq!(pi.get_target().unwrap().to_string_lossy(), "target");
        assert_eq!(pi.get_content().expect("content").to_string_lossy(), "data");
    });
}

#[test]
fn create_cdata_section() {
    with_factory(TemplateMode::HTML, |factory| {
        let cdata = factory.create_cdata_section(js("<raw>")).expect("cdata");
        let c = cdata.get_content().expect("content").expect("non-null");
        assert_eq!(c.to_string_lossy(), "<raw>");
    });
}

#[test]
fn create_xml_declaration() {
    with_factory(TemplateMode::HTML, |factory| {
        let decl = factory
            .create_xml_declaration(Some(js("1.0")), Some(js("UTF-8")), Some(js("yes")))
            .expect("xml declaration");
        assert_eq!(decl.get_version().unwrap().to_string_lossy(), "1.0");
    });
}

// ===========================================================================
// 2. TEXT 模式限制（Java 语义：markup 事件禁止在 text 模式创建）
// ===========================================================================

#[test]
fn text_mode_rejects_comment() {
    with_factory(TemplateMode::TEXT, |factory| {
        assert!(
            factory.create_comment(js("x")).is_err(),
            "comment in TEXT mode must fail like Java"
        );
    });
}

#[test]
fn text_mode_rejects_doc_type() {
    with_factory(TemplateMode::TEXT, |factory| {
        assert!(
            factory.create_html5_doc_type().is_err(),
            "doctype in TEXT mode must fail like Java"
        );
    });
}

#[test]
fn text_mode_allows_text() {
    with_factory(TemplateMode::TEXT, |factory| {
        let text = factory.create_text(js("ok"));
        let c = text.get_text().expect("text access").expect("non-null");
        assert_eq!(c.to_string_lossy(), "ok");
    });
}

// ===========================================================================
// 3. AbstractAttributeModelProcessor 直接实例化（公共扩展点）
// ===========================================================================

#[test]
fn attribute_model_processor_construction() {
    let processor = AbstractAttributeModelProcessor::new(
        Some(TemplateMode::HTML),
        Some(js("th")),
        None,
        false,
        Some(js("mymodelattr")),
        true,
        1000,
        true,
        "com.example.MyModelProcessor",
        noop_process(),
    )
    .expect("valid processor");
    assert_eq!(
        processor.get_dialect_prefix().unwrap().to_string_lossy(),
        "th"
    );
    assert_eq!(processor.get_template_mode(), Some(TemplateMode::HTML));
    assert_eq!(processor.get_precedence(), 1000);
    assert_eq!(processor.class_name(), "com.example.MyModelProcessor");
}

#[test]
fn attribute_model_processor_null_attribute_errors() {
    // 属性名 null → 构造失败（Java Validate.notNull 等价）
    let result = AbstractAttributeModelProcessor::new(
        Some(TemplateMode::HTML),
        None,
        None,
        false,
        None,
        true,
        1000,
        true,
        "com.example.MyModelProcessor",
        noop_process(),
    );
    assert!(result.is_err(), "null attribute name must be rejected");
}

#[test]
fn attribute_model_processor_matching_attribute_name() {
    let processor = AbstractAttributeModelProcessor::new(
        Some(TemplateMode::HTML),
        Some(js("th")),
        None,
        false,
        Some(js("mymodelattr")),
        true,
        1000,
        true,
        "com.example.MyModelProcessor",
        noop_process(),
    )
    .expect("valid processor");
    // 匹配属性名应解析为 th:mymodelattr
    let matching = processor.get_matching_attribute_name().expect("matching");
    let name = matching.to_utf16_string().expect("matching name text");
    assert!(
        name.to_string_lossy().contains("mymodelattr"),
        "matching attribute name must contain mymodelattr"
    );
}
