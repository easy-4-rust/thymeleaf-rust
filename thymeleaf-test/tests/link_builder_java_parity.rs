//! LinkBuilder 对象族的固定 Java Golden 差分与 Rust 共享义务测试。

use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};

use indexmap::IndexMap;
use num_bigint::BigInt;
use thymeleaf::context::{ExpressionContext, IExpressionContext, WebExpressionContext};
use thymeleaf::expression::TemplateValue;
use thymeleaf::linkbuilder::{AbstractLinkBuilder, ILinkBuilder, StandardLinkBuilder};
use thymeleaf::util::{JavaBigDecimal, JavaNumber, Locale, Utf16String, ValidateError};
use thymeleaf::web::{IWebApplication, IWebExchange, IWebRequest, IWebSession};
use thymeleaf::{
    IEngineConfiguration, ITemplateEngine, TemplateEngine, TemplateProcessingException,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/link_builder_golden.txt");

type LinkParameters = IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>;

#[test]
fn standard_link_builder_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "java_baseline", JAVA_BASELINE);
    let configuration = configuration();
    let non_web = ExpressionContext::new(Some(Arc::clone(&configuration)))
        .expect("non-web expression context");

    export_abstract_state(&mut output);
    export_validation_and_classification(&mut output, non_web.as_ref());
    export_query_parameters(&mut output, non_web.as_ref());
    export_template_parameters(&mut output, non_web.as_ref());
    export_escaping(&mut output, non_web.as_ref());
    export_web_and_extension_points(&mut output, configuration, non_web.as_ref());

    assert_eq!(output, JAVA_GOLDEN);
}

#[test]
fn abstract_link_builder_preserves_subclass_state_and_nullable_contract() {
    let configuration = configuration();
    let context = ExpressionContext::new(Some(configuration)).expect("non-web expression context");
    let calls = Arc::new(AtomicUsize::new(0));
    let observed_calls = Arc::clone(&calls);
    let mut builder = AbstractLinkBuilder::new(
        "com.example.TestLinkBuilder",
        move |_context: &dyn IExpressionContext,
              base: Option<&Utf16String>,
              _parameters: Option<&LinkParameters>|
              -> Result<Option<Utf16String>, TemplateProcessingException> {
            observed_calls.fetch_add(1, Ordering::SeqCst);
            Ok(base.cloned())
        },
    );

    assert_eq!(
        builder.get_name().map(Utf16String::to_string_lossy),
        Some("com.example.TestLinkBuilder".to_owned())
    );
    assert_eq!(builder.get_order(), None);
    builder.set_name(None);
    builder.set_order(Some(-17));
    assert_eq!(builder.get_name(), None);
    assert_eq!(builder.get_order(), Some(-17));

    let error = builder
        .build_link_nullable(None, Some(&java("path")), None)
        .expect_err("null context must preserve Java validation");
    let error = error
        .downcast_ref::<ValidateError>()
        .expect("validation error retains its concrete type");
    assert_eq!(
        error.java_class_name(),
        "java.lang.IllegalArgumentException"
    );
    assert_eq!(
        error.get_message(),
        Some("Expression context cannot be null")
    );

    let result = builder
        .build_link(context.as_ref(), Some(&java("path")), None)
        .expect("abstract subclass closure")
        .expect("closure handles the link");
    assert_eq!(result.to_string_lossy(), "path");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn link_builder_is_safely_shared_by_concurrent_render_threads() {
    let configuration = configuration();
    let context = ExpressionContext::new(Some(configuration)).expect("non-web expression context");
    let builder: Arc<dyn ILinkBuilder> = Arc::new(StandardLinkBuilder::new());
    let barrier = Arc::new(Barrier::new(8));
    let mut threads = Vec::new();

    for thread_index in 0..8 {
        let builder = Arc::clone(&builder);
        let context = Arc::clone(&context);
        let barrier = Arc::clone(&barrier);
        threads.push(std::thread::spawn(move || {
            let parameters = map("id", text_value(&format!("item {thread_index}")));
            barrier.wait();
            builder
                .build_link(
                    context.as_ref(),
                    Some(&java("orders/{id}")),
                    Some(&parameters),
                )
                .expect("concurrent link build")
                .expect("standard builder handles the link")
        }));
    }

    for (thread_index, thread) in threads.into_iter().enumerate() {
        assert_eq!(
            thread
                .join()
                .expect("link-builder worker")
                .to_string_lossy(),
            format!("orders/item%20{thread_index}")
        );
    }
}

fn export_abstract_state(output: &mut String) {
    let mut builder = StandardLinkBuilder::new();
    emit_optional_java(output, "state.name.default", builder.get_name().cloned());
    emit_optional_i32(output, "state.order.default", builder.get_order());
    builder.set_name(None);
    builder.set_order(Some(-17));
    emit_optional_java(output, "state.name.null", builder.get_name().cloned());
    emit_optional_i32(output, "state.order.negative", builder.get_order());
}

fn export_validation_and_classification(output: &mut String, context: &dyn IExpressionContext) {
    let builder = StandardLinkBuilder::new();
    let error = builder
        .build_link_nullable(None, Some(&java("/x")), None)
        .expect_err("null context");
    let validation = error
        .downcast_ref::<ValidateError>()
        .expect("typed validation error");
    emit(
        output,
        "validation.context.null",
        &format!(
            "{}|{}",
            validation.java_class_name(),
            validation.get_message().unwrap_or("null")
        ),
    );
    emit_build(
        output,
        "validation.base.null",
        &builder,
        context,
        None,
        None,
    );
    emit_build(
        output,
        "classification.empty",
        &builder,
        context,
        Some(""),
        None,
    );
    emit_build(
        output,
        "classification.base",
        &builder,
        context,
        Some("relative/path"),
        None,
    );
    emit_build(
        output,
        "classification.absolute.http",
        &builder,
        context,
        Some("https://example.org/x"),
        None,
    );
    emit_build(
        output,
        "classification.absolute.embedded_scheme",
        &builder,
        context,
        Some("prefix:https://example.org/x"),
        None,
    );
    emit_build(
        output,
        "classification.absolute.protocol_relative",
        &builder,
        context,
        Some("//example.org/x"),
        None,
    );
    emit_build(
        output,
        "classification.absolute.mailto_case",
        &builder,
        context,
        Some("MaIlTo:user@example.org"),
        None,
    );
    emit_build(
        output,
        "classification.server_relative",
        &builder,
        context,
        Some("~/root/path"),
        None,
    );
    let fragment_parameters = map("q", text_value("x"));
    emit_build(
        output,
        "classification.fragment.last",
        &builder,
        context,
        Some("path#first#last"),
        Some(&fragment_parameters),
    );
    emit_build(
        output,
        "classification.fragment.zero",
        &builder,
        context,
        Some("#fragment"),
        Some(&fragment_parameters),
    );
    emit_processing_failure(
        output,
        "security.javascript.lower",
        builder.build_link(context, Some(&java("javascript:alert(1)")), None),
    );
    emit_processing_failure(
        output,
        "security.javascript.mixed",
        builder.build_link(context, Some(&java("JaVaScRiPt:alert(1)")), None),
    );
    emit_build(
        output,
        "security.javascript.leading_space",
        &builder,
        context,
        Some(" javascript:alert(1)"),
        None,
    );
    emit_build(
        output,
        "security.javascript_similar",
        &builder,
        context,
        Some("javascriptx:alert(1)"),
        None,
    );
    emit_processing_failure(
        output,
        "classification.context_non_web",
        builder.build_link(context, Some(&java("/context/path")), None),
    );
}

fn export_query_parameters(output: &mut String, context: &dyn IExpressionContext) {
    let builder = StandardLinkBuilder::new();
    let scalar = map("name", text_value("a b"));
    emit_build(
        output,
        "query.scalar",
        &builder,
        context,
        Some("path"),
        Some(&scalar),
    );
    let existing = map("name", text_value("a=b&c+d#e"));
    emit_build(
        output,
        "query.existing",
        &builder,
        context,
        Some("path?fixed=yes"),
        Some(&existing),
    );
    let null_value = map("flag", None);
    emit_build(
        output,
        "query.null_value",
        &builder,
        context,
        Some("path"),
        Some(&null_value),
    );
    let empty_string = map("empty", text_value(""));
    emit_build(
        output,
        "query.empty_string",
        &builder,
        context,
        Some("path"),
        Some(&empty_string),
    );
    let list = map(
        "item",
        list_value(vec![text_value("one"), None, text_value("two")]),
    );
    emit_build(
        output,
        "query.list",
        &builder,
        context,
        Some("path"),
        Some(&list),
    );
    let empty_list = map("item", list_value(Vec::new()));
    emit_build(
        output,
        "query.empty_list",
        &builder,
        context,
        Some("path"),
        Some(&empty_list),
    );
    let mut empty_then_value = map("empty", list_value(Vec::new()));
    empty_then_value.insert(Some(java("next")), text_value("value"));
    emit_build(
        output,
        "query.empty_list_then_value",
        &builder,
        context,
        Some("path"),
        Some(&empty_then_value),
    );
    let mut null_key = LinkParameters::new();
    null_key.insert(None, text_value("value"));
    emit_build(
        output,
        "query.null_key",
        &builder,
        context,
        Some("path"),
        Some(&null_key),
    );
    let mut ordered = map("first", text_value("1"));
    ordered.insert(Some(java("second")), None);
    ordered.insert(
        Some(java("third")),
        list_value(vec![text_value("3"), text_value("4")]),
    );
    emit_build(
        output,
        "query.insertion_order",
        &builder,
        context,
        Some("path"),
        Some(&ordered),
    );
    let defensive = map("id", text_value("7"));
    emit_build(
        output,
        "query.defensive.result",
        &builder,
        context,
        Some("path/{id}"),
        Some(&defensive),
    );
    emit(output, "query.defensive.size", &defensive.len().to_string());
    emit(
        output,
        "query.defensive.value",
        &parameter_text(&defensive, "id"),
    );
    let numbers = map(
        "n",
        list_value(vec![
            Some(Arc::new(TemplateValue::Number(JavaNumber::Long(i64::MAX)))),
            Some(Arc::new(TemplateValue::Number(JavaNumber::BigInteger(
                "123456789012345678901234567890"
                    .parse::<BigInt>()
                    .expect("big integer"),
            )))),
            Some(Arc::new(TemplateValue::Number(JavaNumber::BigDecimal(
                JavaBigDecimal::parse("1.2300").expect("big decimal"),
            )))),
            Some(Arc::new(TemplateValue::Boolean(true))),
        ]),
    );
    emit_build(
        output,
        "query.number_types",
        &builder,
        context,
        Some("path"),
        Some(&numbers),
    );
}

fn export_template_parameters(output: &mut String, context: &dyn IExpressionContext) {
    let builder = StandardLinkBuilder::new();
    let path = map("id", text_value("a/b c"));
    emit_build(
        output,
        "template.path",
        &builder,
        context,
        Some("orders/{id}"),
        Some(&path),
    );
    emit_build(
        output,
        "template.segment",
        &builder,
        context,
        Some("orders{/id}"),
        Some(&path),
    );
    let query = map("id", text_value("a=b&c+d#e"));
    emit_build(
        output,
        "template.query",
        &builder,
        context,
        Some("orders?item={id}"),
        Some(&query),
    );
    let repeated = map("id", text_value("a b"));
    emit_build(
        output,
        "template.repeated",
        &builder,
        context,
        Some("{id}/x/{id}"),
        Some(&repeated),
    );
    let direct = map("id", text_value("a/b"));
    emit_build(
        output,
        "template.direct_preferred",
        &builder,
        context,
        Some("{id}/x{/id}"),
        Some(&direct),
    );
    let leading_nulls = map(
        "id",
        list_value(vec![None, text_value(""), text_value("x")]),
    );
    emit_build(
        output,
        "template.list.leading_nulls",
        &builder,
        context,
        Some("{id}"),
        Some(&leading_nulls),
    );
    let middle_null = map(
        "id",
        list_value(vec![text_value("a"), None, text_value("b")]),
    );
    emit_build(
        output,
        "template.list.middle_null",
        &builder,
        context,
        Some("{id}"),
        Some(&middle_null),
    );
    let null = map("id", None);
    emit_build(
        output,
        "template.null",
        &builder,
        context,
        Some("{id}"),
        Some(&null),
    );
    let replacement = map("id", text_value("{id}"));
    emit_build(
        output,
        "template.replacement_contains_template",
        &builder,
        context,
        Some("{id}/tail"),
        Some(&replacement),
    );
    let mut path_and_remaining = map("id", text_value("a/b"));
    path_and_remaining.insert(Some(java("q")), text_value("x y"));
    emit_build(
        output,
        "template.path_and_remaining",
        &builder,
        context,
        Some("orders/{id}"),
        Some(&path_and_remaining),
    );
}

fn export_escaping(output: &mut String, context: &dyn IExpressionContext) {
    let builder = StandardLinkBuilder::new();
    let ascii = map("v", text_value("-._~!$&'()*+,;=:@/ ?#[]"));
    emit_build(
        output,
        "escape.path_ascii",
        &builder,
        context,
        Some("{v}"),
        Some(&ascii),
    );
    emit_build(
        output,
        "escape.segment_ascii",
        &builder,
        context,
        Some("{/v}"),
        Some(&ascii),
    );
    emit_build(
        output,
        "escape.query_ascii",
        &builder,
        context,
        Some("path"),
        Some(&ascii),
    );
    let unicode = map("v", text_value("中文😀"));
    emit_build(
        output,
        "escape.unicode",
        &builder,
        context,
        Some("path/{v}"),
        Some(&unicode),
    );
    let isolated = Utf16String::from_utf16(vec![u16::from(b'a'), 0xd800, u16::from(b'b'), 0xdc00]);
    emit(output, "escape.isolated.input_units", &utf16_hex(&isolated));
    let isolated_parameter = map("v", Some(Arc::new(TemplateValue::string(isolated.clone()))));
    emit_build_units(
        output,
        "escape.isolated.path_units",
        &builder,
        context,
        "{v}",
        &isolated_parameter,
    );
    emit_build_units(
        output,
        "escape.isolated.query_units",
        &builder,
        context,
        "path",
        &isolated_parameter,
    );
}

fn export_web_and_extension_points(
    output: &mut String,
    configuration: Arc<dyn IEngineConfiguration>,
    non_web: &dyn IExpressionContext,
) {
    let builder = StandardLinkBuilder::new();
    for (key, application_path) in [
        ("web.null_application_path", None),
        ("web.empty_application_path", Some("")),
        ("web.root_application_path", Some("/")),
    ] {
        let context = web_context(Arc::clone(&configuration), application_path);
        emit_build(output, key, &builder, context.as_ref(), Some("/x"), None);
    }
    let context = web_context(Arc::clone(&configuration), Some("/app"));
    let query = map("q", text_value("v"));
    emit_build(
        output,
        "web.application_path",
        &builder,
        context.as_ref(),
        Some("/x"),
        Some(&query),
    );
    emit_build(
        output,
        "web.absolute_transformed",
        &builder,
        context.as_ref(),
        Some("https://example.org/x"),
        None,
    );

    let context_calls = Arc::new(AtomicUsize::new(0));
    let process_calls = Arc::new(AtomicUsize::new(0));
    let original_identity = Arc::new(AtomicUsize::new(0));
    let original_size = Arc::new(AtomicUsize::new(0));
    let process_input = Arc::new(Mutex::new(None::<Utf16String>));
    let context_calls_hook = Arc::clone(&context_calls);
    let original_identity_hook = Arc::clone(&original_identity);
    let original_size_hook = Arc::clone(&original_size);
    let process_calls_hook = Arc::clone(&process_calls);
    let process_input_hook = Arc::clone(&process_input);
    let probe = StandardLinkBuilder::new()
        .with_context_path_hook(move |_context, _base, parameters| {
            context_calls_hook.fetch_add(1, Ordering::SeqCst);
            original_identity_hook.store(
                parameters.map_or(0, |value| value as *const LinkParameters as usize),
                Ordering::SeqCst,
            );
            original_size_hook.store(parameters.map_or(0, IndexMap::len), Ordering::SeqCst);
            Ok(Some(java("/hook")))
        })
        .with_process_link_hook(move |_context, link| {
            process_calls_hook.fetch_add(1, Ordering::SeqCst);
            *process_input_hook.lock().expect("process-input lock") = Some(link.clone());
            Ok(Some(java(&format!("P[{}]", link.to_string_lossy()))))
        });
    let original = map("id", text_value("7"));
    let original_pointer = &original as *const LinkParameters as usize;
    emit_build(
        output,
        "hooks.result",
        &probe,
        non_web,
        Some("/x/{id}"),
        Some(&original),
    );
    emit(
        output,
        "hooks.context_calls",
        &context_calls.load(Ordering::SeqCst).to_string(),
    );
    emit(
        output,
        "hooks.process_calls",
        &process_calls.load(Ordering::SeqCst).to_string(),
    );
    emit(
        output,
        "hooks.original_identity",
        if original_identity.load(Ordering::SeqCst) == original_pointer {
            "true"
        } else {
            "false"
        },
    );
    emit(
        output,
        "hooks.original_size",
        &original_size.load(Ordering::SeqCst).to_string(),
    );
    emit_optional_java(
        output,
        "hooks.process_input",
        process_input.lock().expect("process-input lock").clone(),
    );

    let null_process = StandardLinkBuilder::new()
        .with_context_path_hook(|_context, _base, _parameters| Ok(Some(java("/hook"))))
        .with_process_link_hook(|_context, _link| Ok(None));
    emit_build(
        output,
        "hooks.process_null",
        &null_process,
        non_web,
        Some("/x"),
        None,
    );
}

fn emit_build(
    output: &mut String,
    key: &str,
    builder: &StandardLinkBuilder,
    context: &dyn IExpressionContext,
    base: Option<&str>,
    parameters: Option<&LinkParameters>,
) {
    let base = base.map(java);
    let result = builder
        .build_link(context, base.as_ref(), parameters)
        .unwrap_or_else(|error| panic!("{key}: {error}"));
    emit_optional_java(output, key, result);
}

fn emit_build_units(
    output: &mut String,
    key: &str,
    builder: &StandardLinkBuilder,
    context: &dyn IExpressionContext,
    base: &str,
    parameters: &LinkParameters,
) {
    let result = builder
        .build_link(context, Some(&java(base)), Some(parameters))
        .unwrap_or_else(|error| panic!("{key}: {error}"));
    match result {
        Some(value) => emit(output, key, &utf16_hex(&value)),
        None => emit(output, key, "null"),
    }
}

fn emit_processing_failure(
    output: &mut String,
    key: &str,
    result: Result<Option<Utf16String>, thymeleaf::TemplateProcessingException>,
) {
    let error = result.expect_err("Java contract requires processing failure");
    emit(
        output,
        key,
        &format!("org.thymeleaf.exceptions.TemplateProcessingException|{error}"),
    );
}

fn configuration() -> Arc<dyn IEngineConfiguration> {
    TemplateEngine::new()
        .get_configuration()
        .expect("initialized engine configuration")
}

fn web_context(
    configuration: Arc<dyn IEngineConfiguration>,
    application_path: Option<&str>,
) -> Arc<WebExpressionContext> {
    let exchange: Arc<dyn IWebExchange> = Arc::new(TestWebExchange::new(application_path));
    WebExpressionContext::new(Some(configuration), Some(exchange)).expect("web expression context")
}

fn java(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn text_value(value: &str) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::string(java(value))))
}

fn list_value(values: Vec<Option<Arc<TemplateValue>>>) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::List(Arc::new(
        values
            .into_iter()
            .map(|value| value.unwrap_or_else(|| Arc::new(TemplateValue::Null)))
            .collect(),
    ))))
}

fn map(name: &str, value: Option<Arc<TemplateValue>>) -> LinkParameters {
    let mut parameters = LinkParameters::new();
    parameters.insert(Some(java(name)), value);
    parameters
}

fn parameter_text(parameters: &LinkParameters, name: &str) -> String {
    parameters
        .get(&Some(java(name)))
        .and_then(Option::as_deref)
        .and_then(TemplateValue::to_utf16_string)
        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy())
}

fn emit(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('=');
    output.push_str(value);
    output.push('\n');
}

fn emit_optional_java(output: &mut String, key: &str, value: Option<Utf16String>) {
    emit(
        output,
        key,
        &value.map_or_else(|| "null".to_owned(), |value| value.to_string_lossy()),
    );
}

fn emit_optional_i32(output: &mut String, key: &str, value: Option<i32>) {
    emit(
        output,
        key,
        &value.map_or_else(|| "null".to_owned(), |value| value.to_string()),
    );
}

fn utf16_hex(value: &Utf16String) -> String {
    value
        .as_utf16()
        .iter()
        .map(|unit| format!("{unit:04X}"))
        .collect::<Vec<_>>()
        .join(",")
}

struct TestWebApplication;

impl IWebApplication for TestWebApplication {
    fn contains_attribute(&self, _name: Option<&Utf16String>) -> bool {
        false
    }

    fn get_attribute_count(&self) -> i32 {
        0
    }

    fn get_all_attribute_names(&self) -> Vec<Option<Utf16String>> {
        Vec::new()
    }

    fn get_attribute_map(&self) -> LinkParameters {
        LinkParameters::new()
    }

    fn get_attribute_value(&self, _name: Option<&Utf16String>) -> Option<Arc<TemplateValue>> {
        None
    }

    fn set_attribute_value(&self, _name: Option<Utf16String>, _value: Option<Arc<TemplateValue>>) {}

    fn remove_attribute(&self, _name: Option<&Utf16String>) {}

    fn resource_exists(&self, _path: Option<&Utf16String>) -> bool {
        false
    }

    fn get_resource_as_stream(&self, _path: Option<&Utf16String>) -> Option<Box<dyn Read + Send>> {
        None
    }
}

struct TestWebRequest {
    application_path: Option<Utf16String>,
}

impl IWebRequest for TestWebRequest {
    fn get_method(&self) -> Option<Utf16String> {
        None
    }

    fn get_scheme(&self) -> Option<Utf16String> {
        None
    }

    fn get_server_name(&self) -> Option<Utf16String> {
        None
    }

    fn get_server_port(&self) -> Option<i32> {
        None
    }

    fn get_application_path(&self) -> Option<Utf16String> {
        self.application_path.clone()
    }

    fn get_path_within_application(&self) -> Option<Utf16String> {
        None
    }

    fn get_query_string(&self) -> Option<Utf16String> {
        None
    }

    fn contains_header(&self, _name: Option<&Utf16String>) -> bool {
        false
    }

    fn get_header_count(&self) -> i32 {
        0
    }

    fn get_all_header_names(&self) -> Vec<Option<Utf16String>> {
        Vec::new()
    }

    fn get_header_map(&self) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
        IndexMap::new()
    }

    fn get_header_values(&self, _name: Option<&Utf16String>) -> Option<Vec<Option<Utf16String>>> {
        None
    }

    fn contains_parameter(&self, _name: Option<&Utf16String>) -> bool {
        false
    }

    fn get_parameter_count(&self) -> i32 {
        0
    }

    fn get_all_parameter_names(&self) -> Vec<Option<Utf16String>> {
        Vec::new()
    }

    fn get_parameter_map(&self) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
        IndexMap::new()
    }

    fn get_parameter_values(
        &self,
        _name: Option<&Utf16String>,
    ) -> Option<Vec<Option<Utf16String>>> {
        None
    }

    fn contains_cookie(&self, _name: Option<&Utf16String>) -> bool {
        false
    }

    fn get_cookie_count(&self) -> i32 {
        0
    }

    fn get_all_cookie_names(&self) -> Vec<Option<Utf16String>> {
        Vec::new()
    }

    fn get_cookie_map(&self) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
        IndexMap::new()
    }

    fn get_cookie_values(&self, _name: Option<&Utf16String>) -> Option<Vec<Option<Utf16String>>> {
        None
    }
}

struct TestWebExchange {
    request: TestWebRequest,
    application: TestWebApplication,
}

impl TestWebExchange {
    fn new(application_path: Option<&str>) -> Self {
        Self {
            request: TestWebRequest {
                application_path: application_path.map(java),
            },
            application: TestWebApplication,
        }
    }
}

impl IWebExchange for TestWebExchange {
    fn get_request(&self) -> &dyn IWebRequest {
        &self.request
    }

    fn get_session(&self) -> Option<&dyn IWebSession> {
        None
    }

    fn get_application(&self) -> &dyn IWebApplication {
        &self.application
    }

    fn get_principal(&self) -> Option<Arc<TemplateValue>> {
        None
    }

    fn get_locale(&self) -> Option<Locale> {
        None
    }

    fn get_content_type(&self) -> Option<Utf16String> {
        None
    }

    fn get_character_encoding(&self) -> Option<Utf16String> {
        None
    }

    fn contains_attribute(&self, _name: Option<&Utf16String>) -> bool {
        false
    }

    fn get_attribute_count(&self) -> i32 {
        0
    }

    fn get_all_attribute_names(&self) -> Vec<Option<Utf16String>> {
        Vec::new()
    }

    fn get_attribute_map(&self) -> LinkParameters {
        LinkParameters::new()
    }

    fn get_attribute_value(&self, _name: Option<&Utf16String>) -> Option<Arc<TemplateValue>> {
        None
    }

    fn set_attribute_value(&self, _name: Option<Utf16String>, _value: Option<Arc<TemplateValue>>) {}

    fn remove_attribute(&self, _name: Option<&Utf16String>) {}

    fn transform_url(&self, url: Option<&Utf16String>) -> Option<Utf16String> {
        url.map(|url| java(&format!("T[{}]", url.to_string_lossy())))
    }
}
