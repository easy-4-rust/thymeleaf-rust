//! `IProcessorDialect` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use thymeleaf::{IDialect, IProcessor, IProcessorDialect, ProcessorSet, TemplateMode};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/processor_dialect_contract_golden.txt");

struct ProbeProcessor {
    template_mode: Option<TemplateMode>,
    precedence: i32,
}

impl IProcessor for ProbeProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.template_mode
    }

    fn get_precedence(&self) -> i32 {
        self.precedence
    }
}

struct ProbeDialect {
    name: String,
    prefix: Option<String>,
    precedence: i32,
    calls: AtomicUsize,
    last_prefix: Mutex<Option<Option<String>>>,
    duplicate_added: AtomicBool,
}

impl ProbeDialect {
    fn new(name: &str, prefix: Option<&str>, precedence: i32) -> Self {
        Self {
            name: name.to_owned(),
            prefix: prefix.map(str::to_owned),
            precedence,
            calls: AtomicUsize::new(0),
            last_prefix: Mutex::new(None),
            duplicate_added: AtomicBool::new(false),
        }
    }
}

impl IDialect for ProbeDialect {
    fn get_name(&self) -> Option<&str> {
        Some(&self.name)
    }
}

impl IProcessorDialect for ProbeDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    fn get_dialect_processor_precedence(&self) -> i32 {
        self.precedence
    }

    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.last_prefix.lock().expect("last prefix lock") =
            Some(dialect_prefix.map(str::to_owned));
        if dialect_prefix == Some("return-null") {
            return None;
        }

        let mut processors = ProcessorSet::new();
        let first: Arc<dyn IProcessor> = Arc::new(ProbeProcessor {
            template_mode: Some(TemplateMode::HTML),
            precedence: i32::MIN,
        });
        processors.insert(None);
        processors.insert(Some(Arc::clone(&first)));
        self.duplicate_added
            .store(processors.insert(Some(first)), Ordering::SeqCst);
        processors.insert(Some(Arc::new(ProbeProcessor {
            template_mode: None,
            precedence: i32::MAX,
        })));
        Some(processors)
    }
}

#[test]
fn processor_dialect_contract_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    let null_prefix = ProbeDialect::new("probe-null", None, i32::MIN);
    let empty_prefix = ProbeDialect::new("probe-empty", Some(""), 0);
    let unicode_prefix = ProbeDialect::new("方言", Some("前缀"), i32::MAX);

    emit_getters(&mut output, "null", &null_prefix);
    emit_getters(&mut output, "empty", &empty_prefix);
    emit_getters(&mut output, "unicode", &unicode_prefix);

    emit_processors(&mut output, "null", &null_prefix, None);
    emit_processors(&mut output, "empty", &null_prefix, Some(""));
    emit_processors(&mut output, "unicode", &null_prefix, Some("用户前缀"));
    emit_processors(&mut output, "nullSet", &null_prefix, Some("return-null"));

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_getters(output: &mut String, key: &str, implementation: &ProbeDialect) {
    let dialect: &dyn IProcessorDialect = implementation;
    emit(
        output,
        &format!("getters.{key}"),
        format!(
            "name={},prefix={},precedence={}",
            format_nullable(dialect.get_name()),
            format_nullable(dialect.get_prefix()),
            dialect.get_dialect_processor_precedence()
        ),
    );
}

fn emit_processors(
    output: &mut String,
    key: &str,
    implementation: &ProbeDialect,
    dialect_prefix: Option<&str>,
) {
    let dialect: &dyn IProcessorDialect = implementation;
    let Some(processors) = dialect.get_processors(dialect_prefix) else {
        emit(
            output,
            &format!("processors.{key}"),
            format!(
                "set=null,lastPrefix={},calls={}",
                format_nullable(
                    implementation
                        .last_prefix
                        .lock()
                        .expect("last prefix lock")
                        .as_ref()
                        .and_then(Option::as_deref)
                ),
                implementation.calls.load(Ordering::SeqCst)
            ),
        );
        return;
    };

    let values = processors
        .iter()
        .map(|processor| match processor {
            None => "null".to_owned(),
            Some(processor) => format!(
                "{}:{}",
                processor
                    .get_template_mode()
                    .map_or_else(|| "null".to_owned(), |mode| mode.to_string()),
                processor.get_precedence()
            ),
        })
        .collect::<Vec<_>>()
        .join("|");
    emit(
        output,
        &format!("processors.{key}"),
        format!(
            "size={},values={},duplicateAdded={},lastPrefix={},calls={}",
            processors.len(),
            values,
            implementation.duplicate_added.load(Ordering::SeqCst),
            format_nullable(
                implementation
                    .last_prefix
                    .lock()
                    .expect("last prefix lock")
                    .as_ref()
                    .and_then(Option::as_deref)
            ),
            implementation.calls.load(Ordering::SeqCst)
        ),
    );
}

fn format_nullable(value: Option<&str>) -> &str {
    value.unwrap_or("null")
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to string");
}
