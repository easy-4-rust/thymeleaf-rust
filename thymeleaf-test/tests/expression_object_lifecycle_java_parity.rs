//! 表达式对象工厂、生命周期容器与原生 Map 包装器的固定 Java Golden 差分测试。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};

use thymeleaf::context::{EngineContext, ExpressionContext, IEngineContext, IExpressionContext};
use thymeleaf::engine::TemplateData;
use thymeleaf::expression::{
    ExpressionObjectNames, ExpressionObjects, IExpressionObjectFactory, IExpressionObjects,
    NativeContextPropertyAccessor, NativeExpressionObjectsWrapper,
    NativeExpressionObjectsWrapperError, StandardExpressionObjectFactory, StandardExpressionResult,
    TemplateValue,
};
use thymeleaf::util::{JavaLocale, JavaString, ValidateError};
use thymeleaf::{ITemplateEngine, TemplateEngine, TemplateMode};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/expression_objects_golden.txt");

#[test]
fn expression_object_lifecycle_and_wrapper_match_java_golden() {
    let expected = parse_golden(JAVA_GOLDEN);
    assert_eq!(
        expected.get("baseline").map(String::as_str),
        Some(JAVA_BASELINE)
    );

    let mut actual = BTreeMap::new();
    export_expression_objects(&mut actual);
    export_standard_factory(&mut actual);
    export_wrapper(&mut actual);

    for (key, value) in actual {
        assert_eq!(
            expected.get(&key),
            Some(&value),
            "Java Golden mismatch for {key}"
        );
    }
}

#[test]
fn expression_object_cache_is_thread_safe_and_weak_context_breaks_arc_cycles() {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("default configuration");
    let context: Arc<dyn IExpressionContext> =
        ExpressionContext::new(Some(configuration)).expect("valid expression context");
    let factory = Arc::new(ProbeFactory::new());
    let objects = Arc::new(
        ExpressionObjects::new(
            Some(Arc::downgrade(&context)),
            Some(factory.clone() as Arc<dyn IExpressionObjectFactory>),
        )
        .expect("valid expression objects"),
    );
    let cached_name = js("cached");
    let warmed = objects
        .get_object(Some(&cached_name))
        .expect("cached object")
        .expect("non-null cached object");
    let barrier = Arc::new(Barrier::new(13));
    let mut workers = Vec::new();
    for _ in 0..12 {
        let objects = Arc::clone(&objects);
        let barrier = Arc::clone(&barrier);
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            objects
                .get_object(Some(&js("cached")))
                .expect("cached read")
                .expect("non-null cached value")
        }));
    }
    barrier.wait();
    for worker in workers {
        assert!(Arc::ptr_eq(&warmed, &worker.join().expect("reader thread")));
    }
    assert_eq!(factory.builds_for("cached"), 1);

    let first_fresh = objects
        .get_object(Some(&js("fresh")))
        .expect("fresh object")
        .expect("non-null fresh object");
    let second_fresh = objects
        .get_object(Some(&js("fresh")))
        .expect("fresh object")
        .expect("non-null fresh object");
    assert!(!Arc::ptr_eq(&first_fresh, &second_fresh));

    *factory.last_context.lock().expect("last context") = None;
    drop(context);
    assert!(
        objects
            .get_object(Some(&js("cachedNull")))
            .expect("dead context produces null")
            .is_none()
    );
    assert_eq!(
        factory.builds_for("cachedNull"),
        0,
        "a dead Weak context must neither leak nor invoke the factory"
    );
}

/// 回归：标准方言缓存的表达式对象不得反向拥有请求 Context。
///
/// 这里必须使用真实 `StandardExpressionObjectFactory`，而不是不持有 Context 的测试
/// 工厂；覆盖 `#ctx`、`#root`、`#vars`、`#conversions`、`#messages`、`#ids` 与
/// `#execInfo` 七个会读取 Context 的可缓存对象。
#[test]
fn standard_expression_object_cache_does_not_retain_context() {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("default configuration");
    let context = EngineContext::new(
        configuration,
        TemplateData::new(None, None, None, Some(TemplateMode::HTML), None),
        None,
        JavaLocale::new(js("en"), js("US")),
        None,
    );
    let weak = Arc::downgrade(&context);

    for name in [
        "ctx",
        "root",
        "vars",
        "conversions",
        "messages",
        "ids",
        "execInfo",
    ] {
        let name = js(name);
        let value = context
            .get_expression_objects()
            .get_object(Some(&name))
            .expect("standard expression object builds");
        assert!(
            value.is_some(),
            "standard object {} must be available",
            name.to_string_lossy()
        );
    }

    drop(context);
    assert!(
        weak.upgrade().is_none(),
        "cached standard expression objects must not retain their Context"
    );
}

fn export_expression_objects(output: &mut BTreeMap<String, String>) {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("default configuration");
    let context: Arc<dyn IExpressionContext> = ExpressionContext::with_locale(
        Some(configuration),
        Some(JavaLocale::new(js("en-US"), js("US"))),
    )
    .expect("valid expression context");
    let factory = Arc::new(ProbeFactory::new());
    let objects = ExpressionObjects::new(
        Some(Arc::downgrade(&context)),
        Some(factory.clone() as Arc<dyn IExpressionObjectFactory>),
    )
    .expect("valid expression objects");

    emit(
        output,
        "container.names.identity",
        Arc::ptr_eq(&objects.get_object_names(), &factory.names),
    );
    emit(
        output,
        "container.names",
        join_names(&objects.get_object_names()),
    );
    emit(output, "container.size", objects.size());
    emit(
        output,
        "container.contains.cached",
        objects.contains_object(Some(&js("cached"))),
    );
    emit(
        output,
        "container.contains.null",
        objects.contains_object(None),
    );
    emit(
        output,
        "container.contains.unknown",
        objects.contains_object(Some(&js("unknown"))),
    );
    emit_value(
        output,
        "container.unknown.value",
        objects.get_object(Some(&js("unknown"))).expect("unknown"),
    );
    emit(output, "container.unknown.builds", factory.builds());

    let cached_one = objects
        .get_object(Some(&js("cached")))
        .expect("cached one")
        .expect("cached value");
    let cached_two = objects
        .get_object(Some(&js("cached")))
        .expect("cached two")
        .expect("cached value");
    emit_arc_value(output, "container.cached.value", Some(&cached_one));
    emit(
        output,
        "container.cached.same",
        Arc::ptr_eq(&cached_one, &cached_two),
    );
    emit(
        output,
        "container.cached.builds",
        factory.builds_for("cached"),
    );
    emit(
        output,
        "container.cached.cache_checks",
        factory.cache_checks_for("cached"),
    );
    emit(
        output,
        "container.cached.context.same",
        factory
            .last_context
            .lock()
            .expect("last context")
            .as_ref()
            .is_some_and(|built| Arc::ptr_eq(built, &context)),
    );

    let fresh_one = objects
        .get_object(Some(&js("fresh")))
        .expect("fresh one")
        .expect("fresh value");
    let fresh_two = objects
        .get_object(Some(&js("fresh")))
        .expect("fresh two")
        .expect("fresh value");
    emit_arc_value(output, "container.fresh.first", Some(&fresh_one));
    emit_arc_value(output, "container.fresh.second", Some(&fresh_two));
    emit(
        output,
        "container.fresh.same",
        Arc::ptr_eq(&fresh_one, &fresh_two),
    );
    emit(
        output,
        "container.fresh.builds",
        factory.builds_for("fresh"),
    );
    emit(
        output,
        "container.fresh.cache_checks",
        factory.cache_checks_for("fresh"),
    );

    emit_value(
        output,
        "container.null.first",
        objects
            .get_object(Some(&js("cachedNull")))
            .expect("cached null first"),
    );
    emit_value(
        output,
        "container.null.second",
        objects
            .get_object(Some(&js("cachedNull")))
            .expect("cached null second"),
    );
    emit(
        output,
        "container.null.builds",
        factory.builds_for("cachedNull"),
    );
    emit(
        output,
        "container.null.cache_checks",
        factory.cache_checks_for("cachedNull"),
    );
    emit(
        output,
        "container.null.context",
        match ExpressionObjects::new(
            None,
            Some(factory.clone() as Arc<dyn IExpressionObjectFactory>),
        ) {
            Ok(_) => "NONE".to_owned(),
            Err(error) => format!("java.lang.IllegalArgumentException:{error}"),
        },
    );
    emit(
        output,
        "container.null.factory",
        match ExpressionObjects::new(Some(Arc::downgrade(&context)), None) {
            Ok(_) => "NONE".to_owned(),
            Err(error) => format!("java.lang.IllegalArgumentException:{error}"),
        },
    );
}

fn export_standard_factory(output: &mut BTreeMap<String, String>) {
    let configuration = TemplateEngine::new()
        .get_configuration()
        .expect("default configuration");
    let context: Arc<dyn IExpressionContext> = ExpressionContext::with_locale(
        Some(Arc::clone(&configuration)),
        Some(JavaLocale::new(js("fr-CA"), js("CA"))),
    )
    .expect("valid expression context");
    let first = StandardExpressionObjectFactory::new();
    let second = StandardExpressionObjectFactory::new();
    let names = first
        .get_all_expression_object_names()
        .expect("standard names");
    emit(output, "standard.names", join_names(&names));
    emit(output, "standard.names.count", names.len());
    emit(
        output,
        "standard.names.identity",
        Arc::ptr_eq(
            &names,
            &second
                .get_all_expression_object_names()
                .expect("standard names"),
        ),
    );
    for (key, name) in [
        ("standard.cache.null", None),
        ("standard.cache.object", Some(js("object"))),
        ("standard.cache.unknown", Some(js("unknown"))),
    ] {
        emit(output, key, first.is_cacheable(name.as_ref()));
    }

    let ctx = build(&first, &context, Some("ctx"));
    let root = build(&first, &context, Some("root"));
    let vars = build(&first, &context, Some("vars"));
    let fallback = build(&first, &context, Some("object"));
    emit(output, "standard.ctx.same", same_context_value(&ctx, &root));
    emit(
        output,
        "standard.root.same",
        same_context_value(&root, &ctx),
    );
    emit(
        output,
        "standard.vars.same",
        same_context_value(&vars, &ctx),
    );
    emit(
        output,
        "standard.object.fallback.same",
        same_context_value(&fallback, &ctx),
    );
    emit_value(
        output,
        "standard.locale",
        build(&first, &context, Some("locale")),
    );
    emit_value(
        output,
        "standard.unknown",
        build(&first, &context, Some("unknown")),
    );
    emit_value(output, "standard.null", build(&first, &context, None));

    for name in [
        "conversions",
        "uris",
        "temporals",
        "calendars",
        "dates",
        "bools",
        "numbers",
        "objects",
        "strings",
        "arrays",
        "lists",
        "sets",
        "maps",
        "aggregates",
        "messages",
        "ids",
        "execInfo",
    ] {
        let value = build(&first, &context, Some(name));
        emit(
            output,
            &format!("standard.ordinary.{name}"),
            value.map_or_else(
                || "null".to_owned(),
                |value| value.java_class_name().to_owned(),
            ),
        );
    }

    let template_context = EngineContext::new(
        configuration,
        TemplateData::new(None, None, None, Some(TemplateMode::HTML), None),
        None,
        JavaLocale::new(js("ja-JP"), js("JP")),
        None,
    );
    let selection = Arc::new(TemplateValue::string(js("selection")));
    template_context.set_selection_target(Some(Arc::clone(&selection)));
    let template_expression_context: Arc<dyn IExpressionContext> = template_context;
    emit(
        output,
        "standard.template.object.same",
        build(&first, &template_expression_context, Some("object"))
            .is_some_and(|value| Arc::ptr_eq(&value, &selection)),
    );
    for name in ["messages", "ids", "execInfo"] {
        let value =
            build(&first, &template_expression_context, Some(name)).expect("template-only object");
        emit(
            output,
            &format!("standard.template.{name}"),
            value.java_class_name(),
        );
    }

    for name in [
        "uris",
        "bools",
        "objects",
        "arrays",
        "lists",
        "sets",
        "maps",
        "aggregates",
    ] {
        let left = build(&first, &context, Some(name)).expect("singleton");
        let right = build(&second, &context, Some(name)).expect("singleton");
        emit(
            output,
            &format!("standard.singleton.{name}"),
            Arc::ptr_eq(&left, &right),
        );
    }
    for name in ["strings", "numbers", "dates", "calendars"] {
        let left = build(&first, &context, Some(name)).expect("fresh");
        let right = build(&second, &context, Some(name)).expect("fresh");
        emit(
            output,
            &format!("standard.fresh.{name}"),
            Arc::ptr_eq(&left, &right),
        );
    }
    for name in ["request", "response", "session", "servletContext"] {
        let error = first
            .build_object(Arc::clone(&context), Some(&js(name)))
            .expect_err("removed servlet object");
        let class = error.downcast_ref::<ValidateError>().map_or(
            "java.lang.IllegalArgumentException",
            ValidateError::java_class_name,
        );
        emit(
            output,
            &format!("standard.removed.{name}"),
            format!("{class}:{error}"),
        );
    }
}

fn export_wrapper(output: &mut BTreeMap<String, String>) {
    let expression_objects = WrapperObjects::new();
    let mut wrapper = NativeExpressionObjectsWrapper::new(&expression_objects);
    emit(
        output,
        "wrapper.restricted.names",
        ["ctx", "vars", "root", "this", "execInfo", "custom"]
            .into_iter()
            .filter(|name| NativeExpressionObjectsWrapper::is_restricted(Some(&js(name))))
            .collect::<Vec<_>>()
            .join(","),
    );
    emit(output, "wrapper.initial.size", wrapper.size());
    emit(output, "wrapper.initial.empty", wrapper.is_empty());
    emit(
        output,
        "wrapper.initial.keys.identity",
        Arc::ptr_eq(&wrapper.key_set(), &expression_objects.names),
    );
    emit(
        output,
        "wrapper.initial.keys",
        join_names(&wrapper.key_set()),
    );
    emit_wrapper_result(
        output,
        "wrapper.contains.custom",
        wrapper.contains_key(Some(&js("custom"))),
    );
    emit_wrapper_result(
        output,
        "wrapper.contains.missing",
        wrapper.contains_key(Some(&js("missing"))),
    );
    emit_wrapper_value(
        output,
        "wrapper.get.custom",
        wrapper.get(Some(&js("custom"))),
    );
    emit(
        output,
        "wrapper.custom.builds",
        expression_objects.gets.load(Ordering::SeqCst),
    );

    emit_wrapper_value(
        output,
        "wrapper.put.first",
        wrapper.put(Some(js("local")), Some(string_value("one"))),
    );
    emit_wrapper_value(
        output,
        "wrapper.put.second",
        wrapper.put(Some(js("local")), Some(string_value("two"))),
    );
    emit_wrapper_value(output, "wrapper.get.local", wrapper.get(Some(&js("local"))));
    emit(output, "wrapper.size.local", wrapper.size());
    emit(output, "wrapper.keys.local", join_names(&wrapper.key_set()));
    let mut values = wrapper
        .values()
        .iter()
        .map(value_string)
        .collect::<Vec<_>>();
    values.sort();
    emit(output, "wrapper.values.local", values.join(","));
    emit_wrapper_value(
        output,
        "wrapper.put.expression",
        wrapper.put(Some(js("custom")), Some(string_value("bad"))),
    );
    emit_wrapper_value(
        output,
        "wrapper.remove.expression",
        wrapper.remove(Some(&js("custom"))),
    );
    emit_wrapper_value(
        output,
        "wrapper.remove.local",
        wrapper.remove(Some(&js("local"))),
    );

    let put_all = wrapper.put_all([
        (Some(js("batch")), Some(string_value("value"))),
        (Some(js("custom")), Some(string_value("forbidden"))),
    ]);
    emit_wrapper_unit(output, "wrapper.putAll", put_all);
    emit_wrapper_value(
        output,
        "wrapper.putAll.batch",
        wrapper.get(Some(&js("batch"))),
    );
    emit_wrapper_value(
        output,
        "wrapper.putAll.custom",
        wrapper.get(Some(&js("custom"))),
    );

    wrapper
        .put(
            Some(js(
                NativeContextPropertyAccessor::RESTRICT_EXPRESSION_OBJECTS,
            )),
            Some(Arc::new(TemplateValue::Boolean(true))),
        )
        .expect("restriction flag");
    emit_wrapper_value(
        output,
        "wrapper.restricted.ctx",
        wrapper.get(Some(&js("ctx"))),
    );
    emit_wrapper_value(
        output,
        "wrapper.restricted.custom",
        wrapper.get(Some(&js("custom"))),
    );

    emit_wrapper_value(output, "wrapper.null.get", wrapper.get(None));
    emit_wrapper_result(output, "wrapper.null.contains", wrapper.contains_key(None));
    emit_wrapper_value(
        output,
        "wrapper.null.put",
        wrapper.put(None, Some(string_value("value"))),
    );
    emit_wrapper_value(output, "wrapper.null.remove", wrapper.remove(None));
    emit_wrapper_unit(output, "wrapper.clear", wrapper.clear());
    emit_wrapper_result(
        output,
        "wrapper.containsValue",
        wrapper.contains_value(Some(&string_value("value"))),
    );
    emit_wrapper_unit(output, "wrapper.clone", wrapper.clone_map());
    emit_wrapper_unit(output, "wrapper.entrySet", wrapper.entry_set());
    let other_objects = WrapperObjects::new();
    let other = NativeExpressionObjectsWrapper::new(&other_objects);
    emit_wrapper_result(output, "wrapper.equals", wrapper.equals_map(&other));
    emit_wrapper_result(output, "wrapper.hashCode", wrapper.hash_code());
    emit(output, "wrapper.toString", wrapper.to_string());
}

fn build(
    factory: &StandardExpressionObjectFactory,
    context: &Arc<dyn IExpressionContext>,
    name: Option<&str>,
) -> Option<Arc<TemplateValue>> {
    let name = name.map(js);
    factory
        .build_object(Arc::clone(context), name.as_ref())
        .expect("standard object build")
}

fn same_context_value(
    left: &Option<Arc<TemplateValue>>,
    right: &Option<Arc<TemplateValue>>,
) -> bool {
    left.as_ref()
        .zip(right.as_ref())
        .is_some_and(|(left, right)| left.java_equals(right))
}

fn parse_golden(input: &str) -> BTreeMap<String, String> {
    input
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn join_names(names: &ExpressionObjectNames) -> String {
    names
        .iter()
        .map(|name| {
            name.as_ref()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy)
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn emit(output: &mut BTreeMap<String, String>, key: &str, value: impl ToString) {
    output.insert(key.to_owned(), value.to_string());
}

fn emit_value(output: &mut BTreeMap<String, String>, key: &str, value: Option<Arc<TemplateValue>>) {
    emit_arc_value(output, key, value.as_ref());
}

fn emit_arc_value(
    output: &mut BTreeMap<String, String>,
    key: &str,
    value: Option<&Arc<TemplateValue>>,
) {
    emit(output, key, value_string(&value.cloned()));
}

fn value_string(value: &Option<Arc<TemplateValue>>) -> String {
    value.as_ref().map_or_else(
        || "null".to_owned(),
        |value| {
            value
                .to_java_string()
                .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy())
        },
    )
}

fn emit_wrapper_value(
    output: &mut BTreeMap<String, String>,
    key: &str,
    result: Result<Option<Arc<TemplateValue>>, NativeExpressionObjectsWrapperError>,
) {
    match result {
        Ok(value) => emit(output, key, value_string(&value)),
        Err(error) => emit(output, key, format!("{}:{error}", error.java_class_name())),
    }
}

fn emit_wrapper_result<T: ToString>(
    output: &mut BTreeMap<String, String>,
    key: &str,
    result: Result<T, NativeExpressionObjectsWrapperError>,
) {
    match result {
        Ok(value) => emit(output, key, value),
        Err(error) => emit(output, key, format!("{}:{error}", error.java_class_name())),
    }
}

fn emit_wrapper_unit(
    output: &mut BTreeMap<String, String>,
    key: &str,
    result: Result<(), NativeExpressionObjectsWrapperError>,
) {
    match result {
        Ok(()) => emit(output, key, "NONE"),
        Err(error) => emit(output, key, format!("{}:{error}", error.java_class_name())),
    }
}

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn string_value(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::string(js(value)))
}

struct ProbeFactory {
    names: ExpressionObjectNames,
    builds: AtomicUsize,
    builds_by_name: Mutex<BTreeMap<String, usize>>,
    cache_checks_by_name: Mutex<BTreeMap<String, usize>>,
    last_context: Mutex<Option<Arc<dyn IExpressionContext>>>,
}

impl ProbeFactory {
    fn new() -> Self {
        Self {
            names: vec![
                Some(js("cached")),
                Some(js("fresh")),
                Some(js("cachedNull")),
                None,
            ]
            .into(),
            builds: AtomicUsize::new(0),
            builds_by_name: Mutex::new(BTreeMap::new()),
            cache_checks_by_name: Mutex::new(BTreeMap::new()),
            last_context: Mutex::new(None),
        }
    }

    fn builds(&self) -> usize {
        self.builds.load(Ordering::SeqCst)
    }

    fn builds_for(&self, name: &str) -> usize {
        self.builds_by_name
            .lock()
            .expect("build counters")
            .get(name)
            .copied()
            .unwrap_or(0)
    }

    fn cache_checks_for(&self, name: &str) -> usize {
        self.cache_checks_by_name
            .lock()
            .expect("cache counters")
            .get(name)
            .copied()
            .unwrap_or(0)
    }
}

impl IExpressionObjectFactory for ProbeFactory {
    fn get_all_expression_object_names(&self) -> Option<ExpressionObjectNames> {
        Some(Arc::clone(&self.names))
    }

    fn build_object(
        &self,
        context: Arc<dyn IExpressionContext>,
        expression_object_name: Option<&JavaString>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        *self.last_context.lock().expect("last context") = Some(context);
        let sequence = self.builds.fetch_add(1, Ordering::SeqCst) + 1;
        let name =
            expression_object_name.map_or_else(|| "null".to_owned(), JavaString::to_string_lossy);
        *self
            .builds_by_name
            .lock()
            .expect("build counters")
            .entry(name.clone())
            .or_default() += 1;
        if name == "cachedNull" {
            return Ok(None);
        }
        Ok(Some(string_value(&format!("{name}-{sequence}"))))
    }

    fn is_cacheable(&self, expression_object_name: Option<&JavaString>) -> bool {
        let name =
            expression_object_name.map_or_else(|| "null".to_owned(), JavaString::to_string_lossy);
        *self
            .cache_checks_by_name
            .lock()
            .expect("cache counters")
            .entry(name.clone())
            .or_default() += 1;
        name != "fresh"
    }
}

struct WrapperObjects {
    names: ExpressionObjectNames,
    gets: AtomicUsize,
}

impl WrapperObjects {
    fn new() -> Self {
        Self {
            names: ["ctx", "vars", "root", "this", "execInfo", "custom"]
                .into_iter()
                .map(|name| Some(js(name)))
                .collect::<Vec<_>>()
                .into(),
            gets: AtomicUsize::new(0),
        }
    }
}

impl IExpressionObjects for WrapperObjects {
    fn size(&self) -> i32 {
        i32::try_from(self.names.len()).expect("small names collection")
    }

    fn contains_object(&self, name: Option<&JavaString>) -> bool {
        self.names
            .iter()
            .any(|candidate| candidate.as_ref() == name)
    }

    fn get_object_names(&self) -> ExpressionObjectNames {
        Arc::clone(&self.names)
    }

    fn get_object(
        &self,
        name: Option<&JavaString>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        self.gets.fetch_add(1, Ordering::SeqCst);
        let name = name.map_or_else(|| "null".to_owned(), JavaString::to_string_lossy);
        Ok(Some(string_value(&format!("object:{name}"))))
    }
}
