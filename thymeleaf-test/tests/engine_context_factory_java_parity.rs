//! 固定 Java 上游的 Engine Context 工厂差分与 Rust 并发义务。

#[allow(dead_code, unused_imports)]
mod support;

use std::collections::BTreeMap;
use std::sync::{Arc, Barrier, Mutex};

use indexmap::IndexMap;
use support::CorpusWebExchange;
use thymeleaf::cache::AlwaysValidCacheEntryValidity;
use thymeleaf::context::{
    Context, EngineContext, IContext, IContextVariableNames, IEngineContextFactory,
    StandardEngineContextFactory, WebEngineContext, WebExpressionContext,
};
use thymeleaf::engine::TemplateData;
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresource::StringTemplateResource;
use thymeleaf::util::{Locale, NumberValue, Utf16String};
use thymeleaf::{
    ITemplateEngine, TemplateEngine, TemplateMode, TemplateResolutionAttributeValue,
    TemplateResolutionAttributes,
};

fn golden() -> BTreeMap<String, String> {
    include_str!("../../thymeleaf/tests/fixtures/engine_context_factory_golden.txt")
        .lines()
        .map(|line| {
            let (key, value) = line.split_once('=').expect("golden key/value");
            (key.to_owned(), value.to_owned())
        })
        .collect()
}

fn utf16_string(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn locale(language_tag: &str, country: &str) -> Locale {
    Locale::new(utf16_string(language_tag), utf16_string(country))
}

fn string_value(value: &str) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::string(utf16_string(value))))
}

fn integer_value(value: i32) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::Number(NumberValue::Integer(value))))
}

fn template_data(name: &str) -> TemplateData {
    TemplateData::new(
        Some(utf16_string(name)),
        None,
        Some(Arc::new(
            StringTemplateResource::new(Some(name)).expect("string resource"),
        )),
        Some(TemplateMode::HTML),
        Some(Arc::new(AlwaysValidCacheEntryValidity::new())),
    )
}

fn resolution_attributes() -> TemplateResolutionAttributes {
    let mut attributes = TemplateResolutionAttributes::new();
    attributes.insert(
        Some("second".to_owned()),
        TemplateResolutionAttributeValue::new(2_i32),
    );
    attributes.insert(
        Some("first".to_owned()),
        TemplateResolutionAttributeValue::new("one".to_owned()),
    );
    attributes
}

fn variable_names(context: &dyn IContext) -> String {
    let mut names = context
        .get_variable_names()
        .snapshot()
        .into_iter()
        .map(|name| name.map_or_else(|| "null".to_owned(), |name| name.to_string_lossy()))
        .collect::<Vec<_>>();
    names.sort();
    format!("[{}]", names.join(", "))
}

fn variable_text(context: &dyn IContext, name: &str) -> String {
    context
        .get_variable(Some(&utf16_string(name)))
        .and_then(|value| value.to_utf16_string())
        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy())
}

#[test]
fn standard_engine_context_factory_matches_java_golden() {
    let fixture = golden();
    assert_eq!(
        fixture["baseline"],
        "10f9dd2eb8cbd98515ce14b149d115e0287d0add"
    );
    assert_eq!(
        fixture["shape.factory.interface"],
        "public abstract org.thymeleaf.context.IEngineContext createEngineContext(\
         org.thymeleaf.IEngineConfiguration,org.thymeleaf.engine.TemplateData,java.util.Map,\
         org.thymeleaf.context.IContext)"
    );
    assert!(fixture["shape.factory.standard"].contains("public <init>()"));
    assert!(fixture["shape.factory.standard"].contains("createEngineContext("));

    let engine = TemplateEngine::new();
    let configuration = engine.get_configuration().expect("engine configuration");
    let factory = StandardEngineContextFactory::new();
    let attributes = resolution_attributes();

    let empty = TraceContext::new(locale("fr-CA", "CA"), IndexMap::new());
    let empty_result = factory.create_engine_context(
        Arc::clone(&configuration),
        template_data("empty"),
        Some(&attributes),
        &empty,
    );
    assert!(empty_result.as_any().is::<EngineContext>());
    assert_eq!(
        fixture["plain.empty.class"],
        "org.thymeleaf.context.EngineContext"
    );
    assert_eq!(empty.trace(), fixture["plain.empty.trace"]);
    assert_eq!(
        variable_names(empty_result.as_ref()),
        fixture["plain.empty.names"]
    );
    assert_eq!(
        empty_result.get_locale().to_string(),
        fixture["plain.empty.locale"]
    );
    assert_eq!(
        empty_result.level().to_string(),
        fixture["plain.empty.level"]
    );
    assert_eq!(
        empty_result
            .get_template_data()
            .get_template()
            .expect("template")
            .to_string_lossy(),
        fixture["plain.empty.template"]
    );
    let result_attributes = empty_result
        .get_template_resolution_attributes()
        .expect("resolution attributes");
    assert_eq!(
        result_attributes.len().to_string(),
        fixture["plain.empty.attributes.size"]
    );
    assert_eq!(
        result_attributes[&Some("second".to_owned())].to_string(),
        fixture["plain.empty.attributes.second"]
    );
    assert_eq!(
        result_attributes[&Some("first".to_owned())].to_string(),
        fixture["plain.empty.attributes.first"]
    );

    let mut variables = IndexMap::new();
    variables.insert(Some(utf16_string("second")), integer_value(2));
    variables.insert(Some(utf16_string("first")), string_value("one"));
    variables.insert(
        Some(utf16_string("nullable")),
        Some(Arc::new(TemplateValue::Null)),
    );
    let populated = TraceContext::new(locale("ja-JP", "JP"), variables);
    let populated_result = factory.create_engine_context(
        Arc::clone(&configuration),
        template_data("plain"),
        Some(&attributes),
        &populated,
    );
    assert!(populated_result.as_any().is::<EngineContext>());
    assert_eq!(populated.trace(), fixture["plain.vars.trace"]);
    assert_eq!(
        variable_names(populated_result.as_ref()),
        fixture["plain.vars.names"]
    );
    assert_eq!(
        variable_text(populated_result.as_ref(), "second"),
        fixture["plain.vars.second"]
    );
    assert_eq!(
        variable_text(populated_result.as_ref(), "first"),
        fixture["plain.vars.first"]
    );
    assert_eq!(
        variable_text(populated_result.as_ref(), "nullable"),
        fixture["plain.vars.nullable"]
    );
    assert_eq!(
        populated_result.level().to_string(),
        fixture["plain.vars.level"]
    );
}

#[test]
fn every_builtin_web_context_capability_creates_web_engine_context() {
    let fixture = golden();
    let engine = TemplateEngine::new();
    let configuration = engine.get_configuration().expect("engine configuration");
    let factory = StandardEngineContextFactory::new();
    let exchange: Arc<dyn thymeleaf::web::IWebExchange> = Arc::new(CorpusWebExchange::new());
    let variables = vec![
        (Some(utf16_string("webSecond")), integer_value(22)),
        (Some(utf16_string("webFirst")), string_value("one")),
    ];

    let web_context = thymeleaf::context::WebContext::with_locale_and_variables(
        Some(Arc::clone(&exchange)),
        Some(locale("de-DE", "DE")),
        Some(variables.as_slice()),
    )
    .expect("web context");
    let web_result = factory.create_engine_context(
        Arc::clone(&configuration),
        template_data("web"),
        Some(&resolution_attributes()),
        &web_context,
    );
    assert!(web_result.as_any().is::<WebEngineContext>());
    assert_eq!(
        fixture["web.context.class"],
        "org.thymeleaf.context.WebEngineContext"
    );
    assert_eq!(fixture["web.context.exchange.same"], "true");
    assert_eq!(
        variable_names(web_result.as_ref()),
        fixture["web.context.names"]
    );
    assert_eq!(
        variable_text(web_result.as_ref(), "webSecond"),
        fixture["web.context.second"]
    );
    assert_eq!(
        variable_text(web_result.as_ref(), "webFirst"),
        fixture["web.context.first"]
    );

    let expression_context = WebExpressionContext::with_locale_and_variables(
        Some(Arc::clone(&configuration)),
        Some(Arc::clone(&exchange)),
        Some(locale("it-IT", "IT")),
        Some(variables.as_slice()),
    )
    .expect("web expression context");
    let expression_result = factory.create_engine_context(
        configuration,
        template_data("web-expression"),
        None,
        expression_context.as_ref(),
    );
    assert!(expression_result.as_any().is::<WebEngineContext>());
    assert_eq!(
        fixture["web.expression.class"],
        "org.thymeleaf.context.WebEngineContext"
    );
    assert_eq!(fixture["web.expression.exchange.same"], "true");
    assert_eq!(
        expression_result.get_locale().to_string(),
        fixture["web.expression.locale"]
    );
    assert_eq!(
        variable_names(expression_result.as_ref()),
        fixture["web.expression.names"]
    );
}

#[test]
fn stateless_standard_factory_is_thread_safe_and_creates_fresh_contexts() {
    let engine = TemplateEngine::new();
    let configuration = engine.get_configuration().expect("engine configuration");
    let factory = StandardEngineContextFactory::new();
    let variables = vec![(Some(utf16_string("shared")), string_value("value"))];
    let context = Arc::new(Context::with_locale_and_variables(
        Some(locale("en-US", "US")),
        Some(variables.as_slice()),
    ));
    let barrier = Arc::new(Barrier::new(12));

    let handles = (0..12)
        .map(|_| {
            let configuration = Arc::clone(&configuration);
            let context = Arc::clone(&context);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                factory.create_engine_context(
                    configuration,
                    template_data("concurrent"),
                    None,
                    context.as_ref(),
                )
            })
        })
        .collect::<Vec<_>>();

    let contexts = handles
        .into_iter()
        .map(|handle| handle.join().expect("factory thread"))
        .collect::<Vec<_>>();
    for context in &contexts {
        assert_eq!(context.level(), 0);
        assert_eq!(variable_text(context.as_ref(), "shared"), "value");
    }
    for left in 0..contexts.len() {
        for right in (left + 1)..contexts.len() {
            assert!(!Arc::ptr_eq(&contexts[left], &contexts[right]));
        }
    }
}

struct TraceContext {
    locale: Locale,
    variables: IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>,
    trace: Mutex<Vec<String>>,
}

impl TraceContext {
    fn new(
        locale: Locale,
        variables: IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>,
    ) -> Self {
        Self {
            locale,
            variables,
            trace: Mutex::new(Vec::new()),
        }
    }

    fn trace(&self) -> String {
        self.trace
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .join(",")
    }
}

impl IContext for TraceContext {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_locale(&self) -> Locale {
        self.trace
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push("locale".to_owned());
        self.locale.clone()
    }

    fn contains_variable(&self, name: Option<&Utf16String>) -> bool {
        self.variables.contains_key(&name.cloned())
    }

    fn get_variable_names(&self) -> Arc<dyn IContextVariableNames + '_> {
        self.trace
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push("names".to_owned());
        Arc::new(TraceVariableNames {
            names: self.variables.keys().cloned().collect(),
        })
    }

    fn get_variable(&self, name: Option<&Utf16String>) -> Option<Arc<TemplateValue>> {
        let rendered_name = name.map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy);
        self.trace
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(format!("get:{rendered_name}"));
        self.variables.get(&name.cloned()).cloned().flatten()
    }
}

struct TraceVariableNames {
    names: Vec<Option<Utf16String>>,
}

impl IContextVariableNames for TraceVariableNames {
    fn len(&self) -> usize {
        self.names.len()
    }

    fn contains(&self, name: Option<&Utf16String>) -> bool {
        self.names.contains(&name.cloned())
    }

    fn snapshot(&self) -> Vec<Option<Utf16String>> {
        self.names.clone()
    }

    fn remove(&self, _name: Option<&Utf16String>) -> bool {
        false
    }
}
