//! `org.thymeleaf.util` 惰性序列与处理器基础设施 Java 1:1 差分测试。
//!
//! 覆盖对象（对象表编号）：
//! - `LazyEscapingCharSequence`（453）：HTML/TEXT/XML/RAW/JS/CSS 六模式
//!   转义分发（golden 取自 Java `produceEscapedOutput`：HTML/TEXT 用
//!   `HtmlEscape.escapeHtml4Xml`，XML 用内容版 `XmlEscape.escapeXml10`，
//!   JS/CSS 委托 Standard Serializer，RAW 原样）；
//! - `AbstractLazyCharSequence`（439）：惰性缓存与未解析写出语义（写出
//!   未解析内容不填充缓存）、length/charAt/subSequence/to_utf16_string；
//! - `IWritableCharSequence`（451）：`write_direct` 快路径委托；
//! - `LazyProcessingCharSequence`（454）：写出时按模型处理文本
//!   （`to_utf16_string` 返回处理后结果，对应 Java `resolveText`）；
//! - `ProcessorComparators`（463）：方言 precedence → 处理器 precedence →
//!   Java 类名 → 对象身份的比较链；
//! - `ProcessorConfigurationUtils`（464）：unwrap_* 委托族；
//! - `ResourceLoaderUtils`（444，对应 Java `ClassLoaderUtils`）：
//!   注册类/加载类/查找类/资源存在性。

use std::sync::Arc;

use thymeleaf::context::{Context, IEngineContextFactory, StandardEngineContextFactory};
use thymeleaf::engine::TemplateData;
use thymeleaf::expression::TemplateValue;
use thymeleaf::{TemplateResolutionAttributeValue, TemplateResolutionAttributes};

use thymeleaf::element::AbstractAttributeTagProcessor;
use thymeleaf::processor::IProcessor;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::{
    FastStringWriter, IWritableCharSequence, JavaCharSequence, LazyEscapingCharSequence,
    LazyProcessingCharSequence, ProcessorComparators, ProcessorConfigurationUtils,
    ResourceLoaderUtils, Utf16String,
};
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

fn write_sequence(sequence: &dyn IWritableCharSequence) -> String {
    let mut writer = FastStringWriter::new();
    sequence
        .write(&mut writer)
        .expect("sequence write must not fail");
    writer.to_string().to_string_lossy()
}

// ===========================================================================
// 1. LazyEscapingCharSequence（453）：按模板模式转义
// ===========================================================================

fn escaping_sequence(mode: TemplateMode, input: &str) -> LazyEscapingCharSequence {
    let configuration = engine().get_configuration().expect("configuration");
    LazyEscapingCharSequence::new(
        Some(configuration),
        Some(mode),
        Some(Arc::new(TemplateValue::string(js(input)))),
    )
    .expect("lazy escaping sequence")
}

#[test]
fn lazy_escaping_char_sequence_html_text_xml_raw_match_java() {
    let input = "a<b>&c\"d'e";

    // Java：HTML/TEXT 用 HtmlEscape.escapeHtml4Xml（5 字符转义）
    for mode in [TemplateMode::HTML, TemplateMode::TEXT] {
        let sequence = escaping_sequence(mode, input);
        assert_eq!(
            write_sequence(&sequence),
            "a&lt;b&gt;&amp;c&quot;d&#39;e",
            "mode {mode:?} html4/xml escaping"
        );
        // 惰性缓存：to_utf16_string 后再次写出同一结果
        assert_eq!(
            sequence
                .java_to_string()
                .expect("to string")
                .to_string_lossy(),
            "a&lt;b&gt;&amp;c&quot;d&#39;e"
        );
    }

    // Java：XML 用内容版 XmlEscape.escapeXml10（仅 `&<>` 转义，引号保留）
    let sequence = escaping_sequence(TemplateMode::XML, input);
    assert_eq!(
        write_sequence(&sequence),
        "a&lt;b&gt;&amp;c\"d'e",
        "XML content escaping leaves quotes"
    );

    // Java：RAW 原样写出
    let sequence = escaping_sequence(TemplateMode::RAW, input);
    assert_eq!(write_sequence(&sequence), input);
}

#[test]
fn lazy_escaping_char_sequence_javascript_css_match_java() {
    // Java：JS 用 StandardJavaScriptSerializer.serializeValue（String ->
    // 双引号字面量，与 ScriptInlineTest golden 一致）
    let sequence = escaping_sequence(TemplateMode::JAVASCRIPT, "hello");
    assert_eq!(
        write_sequence(&sequence),
        "\"hello\"",
        "JavaScript string literal"
    );

    // Java：CSS 用 StandardCSSSerializer.serializeValue（String ->
    // `CssEscape.escapeCssIdentifier`，无引号）
    let sequence = escaping_sequence(TemplateMode::CSS, "hello");
    assert_eq!(write_sequence(&sequence), "hello", "CSS identifier literal");
}

#[test]
fn lazy_escaping_char_sequence_null_arguments_match_java() {
    // Java Validate.notNull：配置/模式 null 拒绝并保留精确消息
    let error = LazyEscapingCharSequence::new(None, Some(TemplateMode::HTML), None)
        .err()
        .expect("null configuration must be rejected");
    assert_eq!(
        error.to_string(),
        "Engine Configuraion is null, which is forbidden"
    );
    let error = LazyEscapingCharSequence::new(
        Some(engine().get_configuration().expect("configuration")),
        None,
        None,
    )
    .err()
    .expect("null template mode must be rejected");
    assert_eq!(
        error.to_string(),
        "Template Mode is null, which is forbidden"
    );
}

// ===========================================================================
// 2. AbstractLazyCharSequence（439）：惰性缓存与序列合同
// ===========================================================================

#[test]
fn abstract_lazy_char_sequence_contract_matches_java() {
    // 经 LazyEscapingCharSequence 观察基类合同（Java AbstractLazyCharSequence）
    let sequence = escaping_sequence(TemplateMode::RAW, "hello");
    assert_eq!(sequence.java_length().expect("length"), 5);
    assert_eq!(
        char::from_u32(sequence.java_char_at(0).expect("char at 0") as u32).expect("char"),
        'h'
    );
    assert_eq!(
        char::from_u32(sequence.java_char_at(4).expect("char at 4") as u32).expect("char"),
        'o'
    );
    assert_eq!(
        sequence
            .java_sub_sequence(1, 3)
            .expect("sub sequence")
            .to_string_lossy(),
        "el"
    );
    assert_eq!(
        sequence
            .java_to_string()
            .expect("to string")
            .to_string_lossy(),
        "hello"
    );
    // Java 类名（equals 用精确类判断）
    assert_eq!(
        sequence.java_sequence_class_name(),
        "org.thymeleaf.util.LazyEscapingCharSequence"
    );
}

// ===========================================================================
// 3. IWritableCharSequence（451）：write_direct 快路径
// ===========================================================================

#[test]
fn i_writable_char_sequence_write_direct_matches_java() {
    // Java IWritableCharSequence.write(Writer)：LazyEscapingCharSequence 的
    // write 直接写出（未解析内容路径）
    let sequence = escaping_sequence(TemplateMode::HTML, "<x>");
    let mut writer = FastStringWriter::new();
    let result = sequence.write_direct(&mut writer);
    assert!(result.is_some(), "write_direct must be supported");
    result.expect("direct write").expect("direct write result");
    assert_eq!(
        writer.to_string().to_string_lossy(),
        "&lt;x&gt;",
        "write_direct delegates to write"
    );
}

// ===========================================================================
// 4. LazyProcessingCharSequence（454）：写出时处理模型
// ===========================================================================

#[test]
fn lazy_processing_char_sequence_processes_model_matches_java() {
    let engine = engine();
    let configuration = engine.get_configuration().expect("configuration");
    let factory = StandardEngineContextFactory::new();
    let template_data = Arc::new(TemplateData::new(
        Some(js("test")),
        None,
        None,
        Some(TemplateMode::HTML),
        None,
    ));
    let mut attributes = TemplateResolutionAttributes::new();
    attributes.insert(
        Some("template".to_owned()),
        TemplateResolutionAttributeValue::new("test".to_owned()),
    );

    let user_context = Context::new();
    user_context.set_variable(
        Some(js("name")),
        Some(Arc::new(TemplateValue::string(js("world")))),
    );
    let engine_context = factory.create_engine_context(
        Arc::clone(&configuration),
        template_data.as_ref().clone(),
        Some(&attributes),
        &user_context,
    );

    // 处理模型：`<p th:text="${name}">x</p>` -> `<p>world</p>`
    let model = configuration
        .get_model_factory(TemplateMode::HTML)
        .parse(&template_data, &js("<p th:text=\"${name}\">x</p>"))
        .expect("parsed model");
    let lazy = LazyProcessingCharSequence::new(engine_context, Arc::from(model));
    assert_eq!(
        lazy.java_to_string()
            .expect("processed text")
            .to_string_lossy(),
        "<p>world</p>",
        "LazyProcessingCharSequence resolves processed text"
    );
}

// ===========================================================================
// 5. ProcessorComparators（463）：比较链
// ===========================================================================

type DoProcessFn = dyn Fn(
        &dyn thymeleaf::context::ITemplateContext,
        &mut dyn thymeleaf::model::IModel,
        &thymeleaf::engine::AttributeName,
        Option<Utf16String>,
        &mut dyn thymeleaf::element::IElementModelStructureHandler,
    ) -> Result<(), Box<dyn thymeleaf::exceptions::TemplateEngineException>>
    + Send
    + Sync;

fn noop_process() -> Box<DoProcessFn> {
    Box::new(
        |_ctx: &dyn thymeleaf::context::ITemplateContext,
         _model: &mut dyn thymeleaf::model::IModel,
         _attribute_name: &thymeleaf::engine::AttributeName,
         _attribute_value: Option<Utf16String>,
         _structure_handler: &mut dyn thymeleaf::element::IElementModelStructureHandler|
         -> Result<(), Box<dyn thymeleaf::exceptions::TemplateEngineException>> { Ok(()) },
    )
}

fn attribute_processor(
    name: &'static str,
    precedence: i32,
    class_name: &'static str,
) -> Arc<dyn IProcessor> {
    Arc::new(
        AbstractAttributeTagProcessor::new(
            Some(TemplateMode::HTML),
            Some(js("th")),
            None,
            false,
            Some(js(name)),
            true,
            precedence,
            true,
            class_name,
            noop_process(),
        )
        .expect("valid processor"),
    )
}

#[test]
fn processor_comparators_matches_java() {
    use std::cmp::Ordering;

    // 方言 precedence 相同（同一引擎内比较）时按处理器 precedence 排序
    let low = attribute_processor("low", 1000, "com.example.Low");
    let high = attribute_processor("high", 100, "com.example.High");
    assert_eq!(
        ProcessorComparators::compare_processors(low.as_ref(), high.as_ref()),
        Ordering::Greater,
        "higher precedence (100) comes first"
    );

    // 同一 precedence 时按 Java 类名排序
    let a = attribute_processor("a", 500, "com.example.Alpha");
    let b = attribute_processor("b", 500, "com.example.Beta");
    assert_eq!(
        ProcessorComparators::compare_processors(a.as_ref(), b.as_ref()),
        Ordering::Less,
        "class name ordering"
    );

    // 同一对象是唯一返回 Equal 的场景
    assert_eq!(
        ProcessorComparators::compare_processors(low.as_ref(), low.as_ref()),
        Ordering::Equal,
        "same instance is Equal"
    );
    assert_ne!(
        ProcessorComparators::compare_processors(a.as_ref(), b.as_ref()),
        Ordering::Equal,
        "different instances never Equal"
    );

    // 对称性
    assert_eq!(
        ProcessorComparators::compare_processors(high.as_ref(), low.as_ref()),
        Ordering::Less
    );
}

// ===========================================================================
// 6. ProcessorConfigurationUtils（464）：unwrap 委托族
// ===========================================================================

#[test]
fn processor_configuration_utils_unwrap_matches_java() {
    // Java AbstractProcessorWrapper：wrap 记录方言 precedence，unwrap 返回
    // 包装前的原处理器。标准处理器（实现对象安全 as_element_processor）
    // 与引擎真实配置路径一致。
    let processor: Arc<dyn IProcessor> = Arc::new(
        thymeleaf::processor::StandardDOMEventAttributeTagProcessor::new(
            Some(js("th")),
            js("onclick"),
        )
        .expect("valid dom event processor"),
    );
    let wrapped = ProcessorConfigurationUtils::wrap_element(Arc::clone(&processor), 10)
        .expect("wrap element");
    assert_eq!(
        wrapped.get_dialect_precedence(),
        Some(10),
        "wrapper exposes dialect precedence"
    );
    let unwrapped = ProcessorConfigurationUtils::unwrap_element(wrapped.as_ref());
    // unwrap 恢复原处理器：类名一致、方言 precedence 还原为 None
    assert_eq!(
        unwrapped.java_class_name(),
        "org.thymeleaf.standard.processor.StandardDOMEventAttributeTagProcessor"
    );
    assert_eq!(
        unwrapped.get_dialect_precedence(),
        None,
        "unwrap removes the dialect precedence wrapper"
    );
}

// ===========================================================================
// 7. ResourceLoaderUtils（444，对应 Java ClassLoaderUtils）
// ===========================================================================

#[test]
fn resource_loader_utils_class_loading_matches_java() {
    // Java ClassLoaderUtils.loadClass 对已注册类返回类名；未注册类不可加载
    ResourceLoaderUtils::register_class("com.example.RegisteredClass");
    assert!(ResourceLoaderUtils::is_class_present(
        "com.example.RegisteredClass"
    ));
    assert_eq!(
        ResourceLoaderUtils::load_class("com.example.RegisteredClass").expect("registered class"),
        "com.example.RegisteredClass"
    );
    assert_eq!(
        ResourceLoaderUtils::find_class("com.example.RegisteredClass"),
        Some("com.example.RegisteredClass".to_owned())
    );
    assert!(!ResourceLoaderUtils::is_class_present(
        "com.example.AbsentClass"
    ));
    assert!(ResourceLoaderUtils::find_class("com.example.AbsentClass").is_none());
    assert!(ResourceLoaderUtils::load_class("com.example.AbsentClass").is_err());
}
