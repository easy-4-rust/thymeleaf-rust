//! Web Context 三个 Java 对象的固定 Golden 差分与 Rust 并发/capability 义务测试。

// 共享 Web corpus 同时服务于完整 Web SPI 批次；本批次只消费 exchange 身份语义，
// 因而显式关闭未使用支撑对象告警，避免复制一套弱化的 IWebExchange 测试实现。
#![allow(dead_code, unused_imports)]

mod support;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};

use support::CorpusWebExchange;
use thymeleaf::context::{
    IContext, IExpressionContext, IWebContext, WebContext, WebExpressionContext,
};
use thymeleaf::dialect::{IDialect, IExpressionObjectDialect};
use thymeleaf::expression::{
    ExpressionObjectNames, IExpressionObjectFactory, IExpressionObjects, StandardExpressionResult,
    TemplateValue,
};
use thymeleaf::util::{JavaLocale, Utf16String, ValidateError};
use thymeleaf::web::IWebExchange;
use thymeleaf::{IEngineConfiguration, ITemplateEngine, TemplateEngine};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/web_context_golden.txt");

#[test]
fn web_context_semantics_match_java_golden() {
    let expected = parse_golden(JAVA_GOLDEN);
    assert_eq!(
        expected.get("baseline").map(String::as_str),
        Some(JAVA_BASELINE)
    );
    assert_shape_inventory(&expected);

    let mut actual = BTreeMap::new();
    export_web_context(&mut actual);
    export_web_expression_context(&mut actual);
    export_validation(&mut actual);

    for (key, value) in actual {
        assert_eq!(
            expected.get(&key),
            Some(&value),
            "Java Golden mismatch for {key}"
        );
    }
}

#[test]
fn web_expression_context_keeps_exchange_and_lazy_objects_under_concurrency() {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("default configuration");
    let context = WebExpressionContext::new(Some(configuration), Some(exchange.clone()))
        .expect("valid web expression context");
    let barrier = Arc::new(Barrier::new(13));
    let mut workers = Vec::new();
    for _ in 0..12 {
        let context = Arc::clone(&context);
        let exchange = Arc::clone(&exchange);
        let barrier = Arc::clone(&barrier);
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            let objects = context.get_expression_objects();
            let object_identity = (objects as *const dyn IExpressionObjects as *const ()) as usize;
            let web_context = context.as_web_context().expect("web capability");
            let exchange_identity = std::ptr::eq(web_context.get_exchange(), exchange.as_ref());
            (object_identity, exchange_identity)
        }));
    }
    barrier.wait();
    let expected_identity = {
        let objects = context.get_expression_objects();
        (objects as *const dyn IExpressionObjects as *const ()) as usize
    };
    for worker in workers {
        let (identity, exchange_identity) = worker.join().expect("web context reader");
        assert_eq!(identity, expected_identity);
        assert!(exchange_identity);
    }
}

fn assert_shape_inventory(expected: &BTreeMap<String, String>) {
    let expected_counts = [
        ("IWebContext", "1"),
        ("WebContext", "4"),
        ("WebExpressionContext", "4"),
    ];
    let mut declarations = 0usize;
    for (object, count) in expected_counts {
        assert_eq!(
            expected
                .get(&format!("shape.{object}.count"))
                .map(String::as_str),
            Some(count)
        );
        assert!(
            !expected
                .get(&format!("shape.{object}.signatures"))
                .expect("shape signatures")
                .is_empty()
        );
        declarations += count.parse::<usize>().expect("shape count");
    }
    assert_eq!(declarations, 9);
}

fn export_web_context(output: &mut BTreeMap<String, String>) {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let marker = string_value("Marker(shared)");
    let variables = vec![
        (Some(js("first")), Some(marker.clone())),
        (None, Some(string_value("null-key"))),
        (Some(js("nullable")), None),
    ];
    let context = WebContext::with_locale_and_variables(
        Some(exchange.clone()),
        Some(locale("de-DE", "DE")),
        Some(&variables),
    )
    .expect("valid web context");

    emit(
        output,
        "web.exchange.identity",
        std::ptr::eq(context.get_exchange(), exchange.as_ref()),
    );
    let web_context: &dyn IWebContext = &context;
    emit(
        output,
        "web.interface.exchange.identity",
        std::ptr::eq(web_context.get_exchange(), exchange.as_ref()),
    );
    emit(output, "web.locale", context.get_locale());
    emit_names(output, "web.names", &context);
    emit(
        output,
        "web.value.identity",
        context
            .get_variable(Some(&js("first")))
            .is_some_and(|value| Arc::ptr_eq(&value, &marker)),
    );
    emit(
        output,
        "web.contains.null.key",
        context.contains_variable(None),
    );
    emit(
        output,
        "web.contains.null.value",
        context.contains_variable(Some(&js("nullable"))),
    );
    let names = context.get_variable_names();
    emit(
        output,
        "web.names.identity",
        Arc::ptr_eq(&names, &context.get_variable_names()),
    );
    context.set_variable(Some(js("later")), Some(string_value("value")));
    emit_snapshot(output, "web.names.live", names.snapshot());
    emit(
        output,
        "web.names.remove.changed",
        names.remove(Some(&js("first"))),
    );
    emit(
        output,
        "web.names.remove.backing",
        context.contains_variable(Some(&js("first"))),
    );
}

fn export_web_expression_context(output: &mut BTreeMap<String, String>) {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let factory = Arc::new(ProbeFactory::new(exchange.clone()));
    let configuration = configuration_with_factory(factory.clone());
    let variables = vec![
        (Some(js("first")), Some(string_value("one"))),
        (Some(js("nullable")), None),
    ];
    let context = WebExpressionContext::with_locale_and_variables(
        Some(configuration.clone()),
        Some(exchange.clone()),
        Some(locale("fr-FR", "FR")),
        Some(&variables),
    )
    .expect("valid web expression context");

    emit(
        output,
        "web.expression.configuration.identity",
        std::ptr::eq(context.get_configuration(), configuration.as_ref()),
    );
    emit(
        output,
        "web.expression.exchange.identity",
        std::ptr::eq(context.get_exchange(), exchange.as_ref()),
    );
    let web_context: &dyn IWebContext = context.as_ref();
    emit(
        output,
        "web.expression.interface.exchange.identity",
        std::ptr::eq(web_context.get_exchange(), exchange.as_ref()),
    );
    emit(output, "web.expression.before.builds", factory.builds());
    let first_objects = context.get_expression_objects();
    let second_objects = context.get_expression_objects();
    emit(
        output,
        "web.expression.objects.identity",
        std::ptr::eq(first_objects, second_objects),
    );
    emit(
        output,
        "web.expression.names.contains.probe",
        first_objects.contains_object(Some(&js("probe"))),
    );
    let first = first_objects
        .get_object(Some(&js("probe")))
        .expect("probe object")
        .expect("non-null probe object");
    let second = first_objects
        .get_object(Some(&js("probe")))
        .expect("probe object")
        .expect("non-null probe object");
    emit_value(output, "web.expression.object.value", Some(first.clone()));
    emit(
        output,
        "web.expression.object.identity",
        Arc::ptr_eq(&first, &second),
    );
    emit(output, "web.expression.object.builds", factory.builds());
    let context_trait: Arc<dyn IExpressionContext> = context.clone();
    emit(
        output,
        "web.expression.factory.context.identity",
        factory
            .last_context()
            .is_some_and(|built| Arc::ptr_eq(&built, &context_trait)),
    );
    emit(
        output,
        "web.expression.factory.context.class",
        if factory
            .last_context()
            .is_some_and(|built| built.as_any().is::<WebExpressionContext>())
        {
            "org.thymeleaf.context.WebExpressionContext"
        } else {
            "WRONG"
        },
    );
    emit(
        output,
        "web.expression.factory.context.web",
        factory
            .last_context()
            .is_some_and(|built| built.as_web_context().is_some()),
    );
    emit(
        output,
        "web.expression.factory.exchange.identity",
        factory.last_context().is_some_and(|built| {
            built
                .as_web_context()
                .is_some_and(|web| std::ptr::eq(web.get_exchange(), exchange.as_ref()))
        }),
    );
    emit(output, "web.expression.locale", context.get_locale());
    emit_names(output, "web.expression.names", context.as_ref());
}

fn export_validation(output: &mut BTreeMap<String, String>) {
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("default configuration");
    let variables = vec![(Some(js("first")), Some(string_value("one")))];

    emit_error(output, "web.null.exchange.default", WebContext::new(None));
    emit_error(
        output,
        "web.null.exchange.locale",
        WebContext::with_locale(None, Some(locale("en-US", "US"))),
    );
    emit_error(
        output,
        "web.null.exchange.variables",
        WebContext::with_locale_and_variables(None, Some(locale("en-US", "US")), Some(&variables)),
    );

    emit_error(
        output,
        "web.expression.null.configuration.default",
        WebExpressionContext::new(None, Some(exchange.clone())),
    );
    emit_error(
        output,
        "web.expression.null.configuration.locale",
        WebExpressionContext::with_locale(
            None,
            Some(exchange.clone()),
            Some(locale("en-US", "US")),
        ),
    );
    emit_error(
        output,
        "web.expression.null.configuration.variables",
        WebExpressionContext::with_locale_and_variables(
            None,
            Some(exchange.clone()),
            Some(locale("en-US", "US")),
            Some(&variables),
        ),
    );
    emit_error(
        output,
        "web.expression.null.exchange.default",
        WebExpressionContext::new(Some(configuration.clone()), None),
    );
    emit_error(
        output,
        "web.expression.null.exchange.locale",
        WebExpressionContext::with_locale(
            Some(configuration.clone()),
            None,
            Some(locale("en-US", "US")),
        ),
    );
    emit_error(
        output,
        "web.expression.null.exchange.variables",
        WebExpressionContext::with_locale_and_variables(
            Some(configuration),
            None,
            Some(locale("en-US", "US")),
            Some(&variables),
        ),
    );
    emit_error(
        output,
        "web.expression.both.null.precedence",
        WebExpressionContext::new(None, None),
    );
}

fn configuration_with_factory(factory: Arc<ProbeFactory>) -> Arc<dyn IEngineConfiguration> {
    let engine = TemplateEngine::new();
    engine
        .add_dialect(Arc::new(ProbeDialect { factory }) as Arc<dyn IDialect>)
        .expect("dialect registration");
    engine.get_configuration().expect("engine configuration")
}

fn parse_golden(source: &str) -> BTreeMap<String, String> {
    source
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn emit(output: &mut BTreeMap<String, String>, key: &str, value: impl ToString) {
    output.insert(key.to_owned(), value.to_string());
}

fn emit_names(output: &mut BTreeMap<String, String>, key: &str, context: &dyn IContext) {
    emit_snapshot(output, key, context.get_variable_names().snapshot());
}

fn emit_snapshot(
    output: &mut BTreeMap<String, String>,
    key: &str,
    names: Vec<Option<Utf16String>>,
) {
    let rendered = names
        .iter()
        .map(|name| {
            name.as_ref()
                .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy)
        })
        .collect::<Vec<_>>()
        .join(", ");
    emit(output, key, format!("[{rendered}]"));
}

fn emit_value(output: &mut BTreeMap<String, String>, key: &str, value: Option<Arc<TemplateValue>>) {
    let rendered = value
        .as_deref()
        .and_then(TemplateValue::to_utf16_string)
        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy());
    emit(output, key, rendered);
}

fn emit_error<T>(
    output: &mut BTreeMap<String, String>,
    key: &str,
    result: Result<T, ValidateError>,
) {
    match result {
        Ok(_) => emit(output, key, "NONE"),
        Err(error) => emit(output, key, format!("{}:{error}", error.java_class_name())),
    }
}

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn locale(language_tag: &str, country: &str) -> JavaLocale {
    JavaLocale::new(js(language_tag), js(country))
}

fn string_value(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::string(js(value)))
}

struct ProbeDialect {
    factory: Arc<ProbeFactory>,
}

impl IDialect for ProbeDialect {
    fn as_expression_object_dialect(&self) -> Option<&dyn IExpressionObjectDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some("web-probe")
    }
}

impl IExpressionObjectDialect for ProbeDialect {
    fn get_expression_object_factory(&self) -> Option<Arc<dyn IExpressionObjectFactory>> {
        Some(self.factory.clone())
    }
}

struct ProbeFactory {
    names: ExpressionObjectNames,
    expected_exchange: Arc<dyn IWebExchange>,
    builds: AtomicUsize,
    last_context: Mutex<Option<Arc<dyn IExpressionContext>>>,
}

impl ProbeFactory {
    fn new(expected_exchange: Arc<dyn IWebExchange>) -> Self {
        Self {
            names: vec![Some(js("probe"))].into(),
            expected_exchange,
            builds: AtomicUsize::new(0),
            last_context: Mutex::new(None),
        }
    }

    fn builds(&self) -> usize {
        self.builds.load(Ordering::SeqCst)
    }

    fn last_context(&self) -> Option<Arc<dyn IExpressionContext>> {
        self.last_context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl IExpressionObjectFactory for ProbeFactory {
    fn get_all_expression_object_names(&self) -> Option<ExpressionObjectNames> {
        Some(self.names.clone())
    }

    fn build_object(
        &self,
        context: Arc<dyn IExpressionContext>,
        expression_object_name: Option<&Utf16String>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let Some(web_context) = context.as_web_context() else {
            panic!("factory lost IWebContext capability");
        };
        assert!(std::ptr::eq(
            web_context.get_exchange(),
            self.expected_exchange.as_ref()
        ));
        self.builds.fetch_add(1, Ordering::SeqCst);
        *self
            .last_context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(context);
        let name =
            expression_object_name.map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy);
        Ok(Some(string_value(&format!("Marker({name})"))))
    }

    fn is_cacheable(&self, _expression_object_name: Option<&Utf16String>) -> bool {
        true
    }
}
