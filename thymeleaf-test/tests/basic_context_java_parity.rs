//! 基础 Context 六个 Java 对象的固定 Golden 差分与 Rust 并发义务测试。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};

use thymeleaf::context::{
    AbstractExpressionContext, Context, ExpressionContext, IContext, IExpressionContext,
};
use thymeleaf::dialect::{IDialect, IExpressionObjectDialect};
use thymeleaf::expression::{
    ExpressionObjectNames, IExpressionObjectFactory, IExpressionObjects, StandardExpressionResult,
    TemplateValue,
};
use thymeleaf::util::{JavaLocale, JavaNumber, JavaString, ValidateError};
use thymeleaf::{IEngineConfiguration, ITemplateEngine, TemplateEngine};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/basic_context_golden.txt");

#[test]
fn basic_context_semantics_match_java_golden() {
    let expected = parse_golden(JAVA_GOLDEN);
    assert_eq!(
        expected.get("baseline").map(String::as_str),
        Some(JAVA_BASELINE)
    );
    assert_shape_inventory(&expected);

    let mut actual = BTreeMap::new();
    export_constructors_and_variables(&mut actual);
    export_live_variable_names(&mut actual);
    export_mutations_and_errors(&mut actual);
    export_abstract_context(&mut actual);
    export_expression_contexts(&mut actual);

    for (key, value) in actual {
        assert_eq!(
            expected.get(&key),
            Some(&value),
            "Java Golden mismatch for {key}"
        );
    }
}

#[test]
fn expression_objects_lazy_initialization_is_singleton_under_rust_concurrency() {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("default configuration");
    let context = ExpressionContext::new(Some(configuration))
        .expect("valid expression context configuration");
    let barrier = Arc::new(Barrier::new(13));
    let mut workers = Vec::new();
    for _ in 0..12 {
        let context = Arc::clone(&context);
        let barrier = Arc::clone(&barrier);
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            let objects = context.get_expression_objects();
            (objects as *const dyn IExpressionObjects as *const ()) as usize
        }));
    }
    barrier.wait();
    let expected_identity = {
        let objects = context.get_expression_objects();
        (objects as *const dyn IExpressionObjects as *const ()) as usize
    };
    for worker in workers {
        assert_eq!(
            worker.join().expect("expression-object reader"),
            expected_identity
        );
    }
}

fn assert_shape_inventory(expected: &BTreeMap<String, String>) {
    let expected_counts = [
        ("IContext", "4"),
        ("AbstractContext", "12"),
        ("Context", "3"),
        ("IExpressionContext", "2"),
        ("AbstractExpressionContext", "5"),
        ("ExpressionContext", "3"),
    ];
    let mut declarations = 0usize;
    for (object, count) in expected_counts {
        assert_eq!(
            expected
                .get(&format!("shape.{object}.count"))
                .map(String::as_str),
            Some(count),
            "unexpected Java declaration count for {object}"
        );
        let signatures = expected
            .get(&format!("shape.{object}.signatures"))
            .expect("shape signatures");
        assert!(!signatures.is_empty(), "empty Java shape for {object}");
        declarations += count.parse::<usize>().expect("shape count");
    }
    assert_eq!(declarations, 29);
}

fn export_constructors_and_variables(output: &mut BTreeMap<String, String>) {
    let original_default = JavaLocale::get_default();
    JavaLocale::set_default(locale("fr-CA", "CA"));
    let default_context = Context::new();
    let null_locale_context = Context::with_locale(None);
    JavaLocale::set_default(locale("ja-JP", "JP"));
    emit(
        output,
        "context.default.locale.snapshot",
        default_context.get_locale(),
    );
    emit(
        output,
        "context.null.locale.snapshot",
        null_locale_context.get_locale(),
    );
    emit(
        output,
        "context.new.default.locale",
        Context::new().get_locale(),
    );
    JavaLocale::set_default(original_default);

    let marker = string_value("Marker(shared)");
    let variables = vec![
        (Some(js("first")), Some(Arc::clone(&marker))),
        (None, Some(string_value("null-key"))),
        (Some(js("nullable")), None),
    ];
    let context = Context::with_locale_and_variables(Some(locale("de-DE", "DE")), Some(&variables));

    emit(output, "context.explicit.locale", context.get_locale());
    emit_names(output, "context.copy.names", &context);
    emit(
        output,
        "context.copy.source.independent",
        !context.contains_variable(Some(&js("later"))),
    );
    emit(
        output,
        "context.copy.value.identity",
        context
            .get_variable(Some(&js("first")))
            .is_some_and(|value| Arc::ptr_eq(&value, &marker)),
    );
    emit(
        output,
        "context.contains.null.key",
        context.contains_variable(None),
    );
    emit(
        output,
        "context.contains.null.value",
        context.contains_variable(Some(&js("nullable"))),
    );
    emit(
        output,
        "context.contains.absent",
        context.contains_variable(Some(&js("absent"))),
    );
    emit_value(output, "context.get.null.key", context.get_variable(None));
    emit_value(
        output,
        "context.get.null.value",
        context.get_variable(Some(&js("nullable"))),
    );
    emit_value(
        output,
        "context.get.absent",
        context.get_variable(Some(&js("absent"))),
    );
}

fn export_live_variable_names(output: &mut BTreeMap<String, String>) {
    let variables = vec![
        (Some(js("one")), Some(int_value(1))),
        (Some(js("two")), Some(int_value(2))),
        (Some(js("three")), Some(int_value(3))),
    ];
    let context = Context::with_locale_and_variables(Some(locale("en-US", "US")), Some(&variables));
    let names = context.get_variable_names();
    let second_names = context.get_variable_names();
    emit(output, "names.identity", Arc::ptr_eq(&names, &second_names));
    emit_snapshot(output, "names.initial", names.snapshot());

    context.set_variable(Some(js("four")), Some(int_value(4)));
    emit_snapshot(output, "names.after.set", names.snapshot());
    emit(
        output,
        "names.remove.changed",
        names.remove(Some(&js("two"))),
    );
    emit(
        output,
        "names.remove.backing",
        context.contains_variable(Some(&js("two"))),
    );
    emit(
        output,
        "names.remove.absent",
        names.remove(Some(&js("absent"))),
    );
    emit(
        output,
        "names.contains.all",
        names.contains_all(&[Some(js("one")), Some(js("three")), Some(js("four"))]),
    );
    emit(
        output,
        "names.remove.all.changed",
        names.remove_all(&[Some(js("one")), Some(js("absent"))]),
    );
    emit_snapshot(output, "names.after.remove.all", names.snapshot());
    emit(
        output,
        "names.retain.all.changed",
        names.retain_all(&[Some(js("four"))]),
    );
    emit_snapshot(output, "names.after.retain.all", names.snapshot());
    names.clear();
    emit_snapshot(output, "names.after.clear", names.snapshot());
    emit(
        output,
        "names.clear.backing.empty",
        context.get_variable_names().is_empty(),
    );
    // Rust 的只读类型系统不暴露 Java Set#add；该 Java 运行时失败由编译期禁止替代。
}

fn export_mutations_and_errors(output: &mut BTreeMap<String, String>) {
    let context = Context::new();
    context.set_variable(Some(js("first")), Some(int_value(1)));
    context.set_variable(Some(js("second")), Some(int_value(2)));
    context.set_variable(Some(js("first")), Some(int_value(11)));
    emit_names(output, "mutate.replace.order", &context);
    emit_value(
        output,
        "mutate.replace.value",
        context.get_variable(Some(&js("first"))),
    );

    let additions = vec![
        (Some(js("second")), Some(int_value(22))),
        (Some(js("third")), Some(int_value(3))),
    ];
    context.set_variables(Some(&additions));
    context.set_variables(None);
    emit_names(output, "mutate.put.all.order", &context);
    emit_value(
        output,
        "mutate.put.all.second",
        context.get_variable(Some(&js("second"))),
    );
    context.remove_variable(Some(&js("absent")));
    context.remove_variable(Some(&js("first")));
    emit_names(output, "mutate.after.remove", &context);
    context.clear_variables();
    emit_names(output, "mutate.after.clear", &context);

    emit_validate_error(
        output,
        "context.null.locale.error",
        context.set_locale(None),
    );
    context
        .set_locale(Some(locale("it-IT", "IT")))
        .expect("valid locale");
    emit(output, "context.changed.locale", context.get_locale());
}

fn export_abstract_context(output: &mut BTreeMap<String, String>) {
    // Rust 不继承抽象类；Context 组合的同一 AbstractContext 路径覆盖三个 protected 构造器。
    let empty = Context::new();
    let locale_context = Context::with_locale(Some(locale("en-GB", "GB")));
    let variables = vec![
        (Some(js("alpha")), Some(string_value("a"))),
        (Some(js("beta")), Some(string_value("b"))),
    ];
    let populated =
        Context::with_locale_and_variables(Some(locale("ko-KR", "KR")), Some(&variables));
    emit(
        output,
        "abstract.default.locale.nonnull",
        !empty.get_locale().to_string().is_empty(),
    );
    emit(output, "abstract.locale", locale_context.get_locale());
    emit_names(output, "abstract.variables", &populated);
    emit_value(
        output,
        "abstract.variable.beta",
        populated.get_variable(Some(&js("beta"))),
    );
}

fn export_expression_contexts(output: &mut BTreeMap<String, String>) {
    let factory = Arc::new(ProbeFactory::new());
    let configuration = configuration_with_factory(Arc::clone(&factory));
    let variables = vec![
        (Some(js("first")), Some(string_value("one"))),
        (Some(js("nullable")), None),
    ];
    let context = ExpressionContext::with_locale_and_variables(
        Some(Arc::clone(&configuration)),
        Some(locale("fr-FR", "FR")),
        Some(&variables),
    )
    .expect("expression context");
    emit(
        output,
        "expression.configuration.identity",
        std::ptr::eq(context.get_configuration(), configuration.as_ref()),
    );
    emit(output, "expression.before.builds", factory.builds());
    let first_objects = context.get_expression_objects();
    let second_objects = context.get_expression_objects();
    emit(
        output,
        "expression.objects.identity",
        std::ptr::eq(first_objects, second_objects),
    );
    emit(
        output,
        "expression.names.contains.probe",
        first_objects.contains_object(Some(&js("probe"))),
    );
    let first = first_objects
        .get_object(Some(&js("probe")))
        .expect("probe object")
        .expect("non-null probe");
    let second = first_objects
        .get_object(Some(&js("probe")))
        .expect("probe object")
        .expect("non-null probe");
    emit_value(output, "expression.object.value", Some(Arc::clone(&first)));
    emit(
        output,
        "expression.object.identity",
        Arc::ptr_eq(&first, &second),
    );
    emit(output, "expression.object.builds", factory.builds());
    let context_trait: Arc<dyn IExpressionContext> = context.clone();
    emit(
        output,
        "expression.factory.context.identity",
        factory
            .last_context()
            .is_some_and(|built| Arc::ptr_eq(&built, &context_trait)),
    );
    emit(
        output,
        "expression.factory.context.class",
        if factory
            .last_context()
            .is_some_and(|built| built.as_any().is::<ExpressionContext>())
        {
            "org.thymeleaf.context.ExpressionContext"
        } else {
            "WRONG"
        },
    );
    emit(output, "expression.locale", context.get_locale());
    emit_names(output, "expression.variables", context.as_ref());

    let abstract_factory = Arc::new(ProbeFactory::new());
    let abstract_configuration = configuration_with_factory(Arc::clone(&abstract_factory));
    let abstract_context = AbstractExpressionContext::with_locale_and_variables(
        Some(Arc::clone(&abstract_configuration)),
        Some(locale("zh-TW", "TW")),
        Some(&variables),
    )
    .expect("abstract expression context");
    let abstract_objects = abstract_context.get_expression_objects();
    abstract_objects
        .get_object(Some(&js("probe")))
        .expect("probe object");
    emit(
        output,
        "abstract.expression.configuration.identity",
        std::ptr::eq(
            abstract_context.get_configuration(),
            abstract_configuration.as_ref(),
        ),
    );
    emit(
        output,
        "abstract.expression.objects.identity",
        std::ptr::eq(abstract_objects, abstract_context.get_expression_objects()),
    );
    let abstract_trait: Arc<dyn IExpressionContext> = abstract_context.clone();
    emit(
        output,
        "abstract.expression.factory.context.identity",
        abstract_factory
            .last_context()
            .is_some_and(|built| Arc::ptr_eq(&built, &abstract_trait)),
    );
    emit(
        output,
        "abstract.expression.factory.context.class",
        if abstract_factory
            .last_context()
            .is_some_and(|built| built.as_any().is::<AbstractExpressionContext>())
        {
            "org.thymeleaf.context.BasicContextGolden$ProbeExpressionContext"
        } else {
            "WRONG"
        },
    );

    emit_constructor_error(
        output,
        "expression.null.config.default",
        ExpressionContext::new(None),
    );
    emit_constructor_error(
        output,
        "expression.null.config.locale",
        ExpressionContext::with_locale(None, Some(locale("en-US", "US"))),
    );
    emit_constructor_error(
        output,
        "expression.null.config.variables",
        ExpressionContext::with_locale_and_variables(
            None,
            Some(locale("en-US", "US")),
            Some(&variables),
        ),
    );
    emit_constructor_error(
        output,
        "abstract.expression.null.config",
        AbstractExpressionContext::with_locale_and_variables(
            None,
            Some(locale("en-US", "US")),
            Some(&variables),
        ),
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

fn emit_snapshot(output: &mut BTreeMap<String, String>, key: &str, names: Vec<Option<JavaString>>) {
    let rendered = names
        .iter()
        .map(|name| {
            name.as_ref()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy)
        })
        .collect::<Vec<_>>()
        .join(", ");
    emit(output, key, format!("[{rendered}]"));
}

fn emit_value(output: &mut BTreeMap<String, String>, key: &str, value: Option<Arc<TemplateValue>>) {
    let rendered = value
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy());
    emit(output, key, rendered);
}

fn emit_validate_error(
    output: &mut BTreeMap<String, String>,
    key: &str,
    result: Result<(), ValidateError>,
) {
    match result {
        Ok(()) => emit(output, key, "NONE"),
        Err(error) => emit(output, key, format!("{}:{error}", error.java_class_name())),
    }
}

fn emit_constructor_error<T>(
    output: &mut BTreeMap<String, String>,
    key: &str,
    result: Result<Arc<T>, ValidateError>,
) {
    match result {
        Ok(_) => emit(output, key, "NONE"),
        Err(error) => emit(output, key, format!("{}:{error}", error.java_class_name())),
    }
}

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn locale(language_tag: &str, country: &str) -> JavaLocale {
    JavaLocale::new(js(language_tag), js(country))
}

fn string_value(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::string(js(value)))
}

fn int_value(value: i32) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Number(JavaNumber::Integer(value)))
}

struct ProbeDialect {
    factory: Arc<ProbeFactory>,
}

impl IDialect for ProbeDialect {
    fn as_expression_object_dialect(&self) -> Option<&dyn IExpressionObjectDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some("probe")
    }
}

impl IExpressionObjectDialect for ProbeDialect {
    fn get_expression_object_factory(&self) -> Option<Arc<dyn IExpressionObjectFactory>> {
        Some(self.factory.clone())
    }
}

struct ProbeFactory {
    names: ExpressionObjectNames,
    builds: AtomicUsize,
    last_context: Mutex<Option<Arc<dyn IExpressionContext>>>,
}

impl ProbeFactory {
    fn new() -> Self {
        Self {
            names: vec![Some(js("probe"))].into(),
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
        expression_object_name: Option<&JavaString>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        self.builds.fetch_add(1, Ordering::SeqCst);
        *self
            .last_context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(context);
        let name =
            expression_object_name.map_or_else(|| "null".to_owned(), JavaString::to_string_lossy);
        Ok(Some(string_value(&format!("Marker({name})"))))
    }

    fn is_cacheable(&self, _expression_object_name: Option<&JavaString>) -> bool {
        true
    }
}
