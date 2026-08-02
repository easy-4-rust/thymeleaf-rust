//! `AbstractProcessorDialect` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use thymeleaf::{
    AbstractDialectError, AbstractProcessorDialect, IDialect, IProcessorDialect, ProcessorSet,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/abstract_processor_dialect_golden.txt");

struct ProbeDialect {
    base: AbstractProcessorDialect,
    calls: AtomicUsize,
    last_prefix: Mutex<Option<Option<String>>>,
}

impl ProbeDialect {
    fn new(
        name: Option<&str>,
        prefix: Option<&str>,
        processor_precedence: i32,
    ) -> Result<Self, AbstractDialectError> {
        Ok(Self {
            base: AbstractProcessorDialect::new(name, prefix, processor_precedence)?,
            calls: AtomicUsize::new(0),
            last_prefix: Mutex::new(None),
        })
    }
}

impl IDialect for ProbeDialect {
    fn get_name(&self) -> Option<&str> {
        Some(self.base.get_name())
    }
}

impl IProcessorDialect for ProbeDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.base.get_prefix()
    }

    fn get_dialect_processor_precedence(&self) -> i32 {
        self.base.get_dialect_processor_precedence()
    }

    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.last_prefix.lock().expect("last prefix lock") =
            Some(dialect_prefix.map(str::to_owned));
        Some(ProcessorSet::new())
    }
}

#[test]
fn abstract_processor_dialect_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    match ProbeDialect::new(None, Some("ignored"), i32::MAX) {
        Ok(_) => emit(&mut output, "nullName", "<none>"),
        Err(error) => emit(
            &mut output,
            "nullName",
            format!("ERR:java.lang.IllegalArgumentException:{error}"),
        ),
    }

    for (key, name, prefix, precedence, actual_prefix) in [
        ("nullPrefix", "", None, i32::MIN, None),
        ("emptyPrefix", "empty-prefix", Some(""), 0, Some("")),
        ("unicode", "方言", Some("前缀"), i32::MAX, Some("用户覆盖")),
    ] {
        emit_case(&mut output, key, name, prefix, precedence, actual_prefix);
    }

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_case(
    output: &mut String,
    key: &str,
    name: &str,
    prefix: Option<&str>,
    processor_precedence: i32,
    actual_prefix: Option<&str>,
) {
    let implementation = ProbeDialect::new(Some(name), prefix, processor_precedence)
        .expect("non-null name is valid");
    let dialect: &dyn IDialect = &implementation;
    let processor_dialect: &dyn IProcessorDialect = &implementation;
    let processors = processor_dialect
        .get_processors(actual_prefix)
        .expect("probe returns an empty set");

    emit(
        output,
        &format!("case.{key}"),
        format!(
            "name={},prefix={},precedence={},dialectName={},interfacePrefix={},interfacePrecedence={},processorsSize={},lastPrefix={},calls={},stable={}",
            implementation.base.get_name(),
            format_nullable(implementation.base.get_prefix()),
            implementation.base.get_dialect_processor_precedence(),
            format_nullable(dialect.get_name()),
            format_nullable(processor_dialect.get_prefix()),
            processor_dialect.get_dialect_processor_precedence(),
            processors.len(),
            format_nullable(
                implementation
                    .last_prefix
                    .lock()
                    .expect("last prefix lock")
                    .as_ref()
                    .and_then(Option::as_deref)
            ),
            implementation.calls.load(Ordering::SeqCst),
            std::ptr::eq(
                implementation.base.get_name(),
                implementation.base.get_name()
            ) && match (
                implementation.base.get_prefix(),
                implementation.base.get_prefix(),
            ) {
                (Some(left), Some(right)) => std::ptr::eq(left, right),
                (None, None) => true,
                _ => false,
            } && implementation.base.get_dialect_processor_precedence()
                == processor_dialect.get_dialect_processor_precedence()
        ),
    );
}

fn format_nullable(value: Option<&str>) -> &str {
    value.unwrap_or("null")
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to string");
}
