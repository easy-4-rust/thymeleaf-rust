//! `ITemplateEngine`、`TemplateEngine` 与 `IThrottledTemplateProcessor` 的 Java Golden 差分测试。

use std::io::{self, Write};
use std::sync::{Arc, Barrier, Mutex, MutexGuard, mpsc};

use thymeleaf::context::Context;
use thymeleaf::expression::TemplateValue;
use thymeleaf::linkbuilder::StandardLinkBuilder;
use thymeleaf::messageresolver::StandardMessageResolver;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::{Charset, JavaString, JavaWriter};
use thymeleaf::{ITemplateEngine, TemplateEngine, TemplateSelectorSet, TemplateSpec};

const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/template_engine_execution_golden.txt");
const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

#[test]
fn template_engine_execution_matches_java_golden() {
    assert_eq!(rust_golden(), JAVA_GOLDEN);
}

#[test]
fn throttled_completion_status_supports_a_concurrent_observer() {
    let barrier = Arc::new(Barrier::new(2));
    let worker_barrier = Arc::clone(&barrier);
    let (status_sender, status_receiver) = mpsc::channel();

    let worker = std::thread::spawn(move || {
        let mut processor = string_engine()
            .process_throttled_template("<p>concurrent</p>", &Context::new())
            .expect("throttled processor");
        status_sender
            .send(processor.get_completion_status())
            .expect("observer receives status before processing");
        worker_barrier.wait();

        let writer = TrackingWriter::new();
        while !processor.is_finished() {
            processor
                .process_writer(1, Box::new(writer.clone()))
                .expect("single processing thread advances one UTF-16 unit");
            std::thread::yield_now();
        }
        writer.text()
    });

    let status = status_receiver
        .recv()
        .expect("processing thread publishes completion status");
    assert!(!status.is_finished());
    barrier.wait();
    while !status.is_finished() {
        std::thread::yield_now();
    }

    assert_eq!(
        worker.join().expect("processing thread completes"),
        "<p>concurrent</p>"
    );
    assert!(status.is_finished());
}

fn rust_golden() -> String {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_initialization_and_ordering(&mut output);
    emit_processing_overloads(&mut output);
    emit_throttled_characters(&mut output);
    emit_throttled_bytes(&mut output);
    emit_mode_switch_failure(&mut output);
    output
}

fn emit_initialization_and_ordering(output: &mut String) {
    let engine = TemplateEngine::new();
    engine
        .set_template_resolvers(vec![
            Arc::new(string_resolver(Some(20))),
            Arc::new(string_resolver(None)),
            Arc::new(string_resolver(Some(-1))),
        ])
        .expect("configuration is mutable before initialization");
    engine
        .set_message_resolvers(vec![
            Arc::new(message_resolver(Some(20))),
            Arc::new(message_resolver(None)),
            Arc::new(message_resolver(Some(-1))),
        ])
        .expect("configuration is mutable before initialization");
    engine
        .set_link_builders(vec![
            Arc::new(link_builder(Some(20))),
            Arc::new(link_builder(None)),
            Arc::new(link_builder(Some(-1))),
        ])
        .expect("configuration is mutable before initialization");

    emit(
        output,
        "initialization.before",
        &engine.is_initialized().to_string(),
    );
    emit(
        output,
        "ordering.template.before",
        &orders(
            engine
                .get_template_resolvers()
                .iter()
                .map(|value| value.get_order()),
        ),
    );
    emit(
        output,
        "ordering.message.before",
        &orders(
            engine
                .get_message_resolvers()
                .iter()
                .map(|value| value.get_order()),
        ),
    );
    emit(
        output,
        "ordering.link.before",
        &orders(
            engine
                .get_link_builders()
                .iter()
                .map(|value| value.get_order()),
        ),
    );

    engine
        .get_configuration()
        .expect("initialization must succeed");
    emit(
        output,
        "initialization.after",
        &engine.is_initialized().to_string(),
    );
    emit(
        output,
        "ordering.template.after",
        &orders(
            engine
                .get_template_resolvers()
                .iter()
                .map(|value| value.get_order()),
        ),
    );
    emit(
        output,
        "ordering.message.after",
        &orders(
            engine
                .get_message_resolvers()
                .iter()
                .map(|value| value.get_order()),
        ),
    );
    emit(
        output,
        "ordering.link.after",
        &orders(
            engine
                .get_link_builders()
                .iter()
                .map(|value| value.get_order()),
        ),
    );

    let freeze = engine
        .add_template_resolver(Arc::new(StringTemplateResolver::new()))
        .map_or_else(
            |error| format!("java.lang.IllegalStateException:{error}"),
            |()| "NO_ERROR".to_owned(),
        );
    emit(output, "initialization.freeze", &freeze);
}

fn emit_processing_overloads(output: &mut String) {
    let engine = string_engine();
    let context = Context::new();
    context.set_variable(
        Some(JavaString::from_rust_str("name")),
        Some(Arc::new(TemplateValue::string(JavaString::from_rust_str(
            "Rust",
        )))),
    );
    let template = "<p th:text=\"${name}\">fallback</p>";

    let rendered = engine
        .process_template(template, &context)
        .expect("string overload");
    emit(output, "process.string", &rendered.to_string_lossy());

    let spec = TemplateSpec::with_template_mode(Some(template), None).expect("valid template");
    let rendered = thymeleaf::ITemplateEngine::process(&engine, &spec, &context)
        .expect("TemplateSpec overload");
    emit(output, "process.spec", &rendered.to_string_lossy());

    let writer = TrackingWriter::new();
    ITemplateEngine::process_template_to_writer(
        &engine,
        template,
        &context,
        Box::new(writer.clone()),
    )
    .expect("writer overload");
    emit(output, "process.writer.output", &writer.text());
    emit(
        output,
        "process.writer.flush_count",
        &writer.flush_count().to_string(),
    );

    let selected = engine
        .process_template_with_selectors(
            "<main><p id=\"a\">A</p><p id=\"b\">B</p></main>",
            &selectors("#b"),
            &context,
        )
        .expect("selector overload");
    emit(output, "process.selectors", &selected.to_string_lossy());

    let selected_writer = TrackingWriter::new();
    ITemplateEngine::process_template_with_selectors_to_writer(
        &engine,
        "<main><p id=\"a\">A</p><p id=\"b\">B</p></main>",
        &selectors("#a"),
        &context,
        Box::new(selected_writer.clone()),
    )
    .expect("selector writer overload");
    emit(
        output,
        "process.selectors_writer.output",
        &selected_writer.text(),
    );
    emit(
        output,
        "process.selectors_writer.flush_count",
        &selected_writer.flush_count().to_string(),
    );

    let mut selected_processor = ITemplateEngine::process_throttled_template_with_selectors(
        &engine,
        "<main><p id=\"a\">A</p><p id=\"b\">B</p></main>",
        &selectors("#b"),
        &context,
    )
    .expect("selector throttled overload");
    let throttled_selected_writer = TrackingWriter::new();
    let selected_count = selected_processor
        .process_all_writer(Box::new(throttled_selected_writer.clone()))
        .expect("process selected fragment");
    emit(
        output,
        "process.selectors_throttled.count",
        &selected_count.to_string(),
    );
    emit(
        output,
        "process.selectors_throttled.output",
        &throttled_selected_writer.text(),
    );

    let empty = string_engine()
        .process_template("", &Context::new())
        .expect("empty templates are valid");
    emit(output, "process.empty", &empty.to_string_lossy());

    let failure = string_engine()
        .process_template_to_writer("output", &Context::new(), Box::new(FlushFailingWriter))
        .map_or_else(
            |error| format!("org.thymeleaf.exceptions.TemplateOutputException:{error}"),
            |()| "NO_ERROR".to_owned(),
        );
    emit(output, "process.flush_failure", &failure);
}

fn emit_throttled_characters(output: &mut String) {
    let mut processor = string_engine()
        .process_throttled_template("<p>abcdef</p>", &Context::new())
        .expect("throttled processor");
    let writer = TrackingWriter::new();

    emit(
        output,
        "throttle.chars.identifier_nonempty",
        &(!processor.get_processor_identifier().is_empty()).to_string(),
    );
    emit(
        output,
        "throttle.chars.spec",
        &processor.get_template_spec().to_string(),
    );
    emit(
        output,
        "throttle.chars.initial_finished",
        &processor.is_finished().to_string(),
    );
    let zero = processor
        .process_writer(0, Box::new(writer.clone()))
        .expect("zero does not advance");
    emit(output, "throttle.chars.zero", &zero.to_string());

    let mut counts = Vec::new();
    let mut guard = 0;
    while !processor.is_finished() && guard < 100 {
        counts.push(
            processor
                .process_writer(3, Box::new(writer.clone()))
                .expect("bounded character output"),
        );
        guard += 1;
    }
    emit(output, "throttle.chars.counts", &join_counts(&counts));
    emit(output, "throttle.chars.output", &writer.text());
    emit(
        output,
        "throttle.chars.final_finished",
        &processor.is_finished().to_string(),
    );
    let after = processor
        .process_writer(3, Box::new(writer.clone()))
        .expect("finished processor returns zero");
    emit(output, "throttle.chars.after_finished", &after.to_string());

    let mut all = string_engine()
        .process_throttled_template("all-at-once", &Context::new())
        .expect("unlimited processor");
    let all_writer = TrackingWriter::new();
    let count = all
        .process_all_writer(Box::new(all_writer.clone()))
        .expect("process all chars");
    emit(output, "throttle.chars.all_count", &count.to_string());
    emit(output, "throttle.chars.all_output", &all_writer.text());
}

fn emit_throttled_bytes(output: &mut String) {
    let charset = Charset::for_name("UTF-8").expect("UTF-8 is mandatory");
    let mut processor = string_engine()
        .process_throttled_template("Aé中B", &Context::new())
        .expect("byte processor");
    let bytes = SharedBytes::new();
    emit(
        output,
        "throttle.bytes.initial_finished",
        &processor.is_finished().to_string(),
    );

    let mut counts = Vec::new();
    let mut guard = 0;
    while !processor.is_finished() && guard < 100 {
        counts.push(
            processor
                .process_output_stream(3, Box::new(bytes.clone()), &charset)
                .expect("bounded byte output"),
        );
        guard += 1;
    }
    emit(output, "throttle.bytes.counts", &join_counts(&counts));
    emit(output, "throttle.bytes.output", &bytes.text());
    emit(
        output,
        "throttle.bytes.final_finished",
        &processor.is_finished().to_string(),
    );

    let mut all = string_engine()
        .process_throttled_template("Aé中B", &Context::new())
        .expect("unlimited byte processor");
    let all_bytes = SharedBytes::new();
    let count = all
        .process_all_output_stream(Box::new(all_bytes.clone()), &charset)
        .expect("process all bytes");
    emit(output, "throttle.bytes.all_count", &count.to_string());
    emit(output, "throttle.bytes.all_output", &all_bytes.text());
}

fn emit_mode_switch_failure(output: &mut String) {
    let charset = Charset::for_name("UTF-8").expect("UTF-8 is mandatory");
    let mut processor = string_engine()
        .process_throttled_template("abcdef", &Context::new())
        .expect("mode switch processor");
    processor
        .process_writer(1, Box::new(TrackingWriter::new()))
        .expect("first character step");
    let result = processor
        .process_output_stream(1, Box::new(SharedBytes::new()), &charset)
        .map_or_else(
            |error| format!("org.thymeleaf.exceptions.TemplateOutputException:{error}"),
            |_| "NO_ERROR".to_owned(),
        );
    emit(output, "throttle.mode_switch", &result);
}

fn string_engine() -> TemplateEngine {
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(StringTemplateResolver::new()))
        .expect("resolver set before initialization");
    engine
}

fn string_resolver(order: Option<i32>) -> StringTemplateResolver {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_order(order);
    resolver
}

fn message_resolver(order: Option<i32>) -> StandardMessageResolver {
    let mut resolver = StandardMessageResolver::new();
    resolver.set_order(order);
    resolver
}

fn link_builder(order: Option<i32>) -> StandardLinkBuilder {
    let mut builder = StandardLinkBuilder::new();
    builder.set_order(order);
    builder
}

fn selectors(selector: &str) -> TemplateSelectorSet {
    [Some(selector.to_owned())].into_iter().collect()
}

fn orders(values: impl Iterator<Item = Option<i32>>) -> String {
    values
        .map(|value| value.map_or_else(|| "null".to_owned(), |value| value.to_string()))
        .collect::<Vec<_>>()
        .join(",")
}

fn join_counts(counts: &[i32]) -> String {
    counts
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn emit(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('\t');
    output.push_str(&escape(value));
    output.push('\n');
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Clone, Default)]
struct TrackingWriter {
    state: Arc<Mutex<TrackingWriterState>>,
}

#[derive(Default)]
struct TrackingWriterState {
    utf16: Vec<u16>,
    flush_count: usize,
}

impl TrackingWriter {
    fn new() -> Self {
        Self::default()
    }

    fn text(&self) -> String {
        String::from_utf16_lossy(&lock(&self.state).utf16)
    }

    fn flush_count(&self) -> usize {
        lock(&self.state).flush_count
    }
}

impl JavaWriter for TrackingWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        lock(&self.state).utf16.extend_from_slice(characters);
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        lock(&self.state).flush_count += 1;
        Ok(())
    }
}

struct FlushFailingWriter;

impl JavaWriter for FlushFailingWriter {
    fn write_utf16(&mut self, _characters: &[u16]) -> io::Result<()> {
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("golden flush failure"))
    }
}

#[derive(Clone, Default)]
struct SharedBytes {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl SharedBytes {
    fn new() -> Self {
        Self::default()
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&lock(&self.bytes)).into_owned()
    }
}

impl Write for SharedBytes {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        lock(&self.bytes).extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
