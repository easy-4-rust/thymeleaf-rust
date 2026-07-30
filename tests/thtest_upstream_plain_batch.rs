//! 上游无外部上下文依赖 `.thtest` 的批量端到端运行器。
//!
//! 默认开发环境没有 Java 源仓库时不执行；迁移门禁通过
//! `THYMELEAF_UPSTREAM=/path/to/thymeleaf cargo test --test
//! thtest_upstream_plain_batch` 强制运行固定上游语料。

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod support;

use html5gum::Tokenizer;
use html5gum::emitters::default::Token;
use indexmap::IndexMap;
use serde_json::Value;
use thymeleaf::context::{Context, ExpressionContext, IContext};
use thymeleaf::dialect::IDialect;
use thymeleaf::exceptions::{
    TemplateAssertionException, TemplateInputException, TemplateProcessingException,
};
use thymeleaf::expression::{
    ClassNotFoundException, IStandardExpression, NativeVariableExpressionEvaluator,
    NoSuchMethodException, OgnlException, TemplateValue, VariableExpression,
};
use thymeleaf::templateresolver::{StringTemplateResolver, TemplateResolution};
use thymeleaf::text::TextParserReaderError;
use thymeleaf::util::{JavaLocale, JavaString};
use thymeleaf::{
    IEngineConfiguration, ITemplateEngine, ITemplateResolver, StandardDialect, TemplateEngine,
    TemplateMode, TemplateResolutionAttributes, TemplateSelectorSet,
};

use support::{
    CorpusOgnlRuntime, Dialect01, ElementStackDialect, ExceptionLazyContextVariableError,
    PrecedenceDialect, TestEngineMessageResolver, TestLinkBuilder,
};

const INVENTORY: &str = include_str!("../docs/migration/baseline/thtest_inventory.json");

#[test]
fn upstream_plain_output_cases_run_as_one_batch() {
    let Some(upstream) = upstream_root() else {
        eprintln!("THYMELEAF_UPSTREAM is absent; skipping external Java corpus");
        return;
    };
    let inventory: Value = serde_json::from_str(INVENTORY).expect("inventory JSON must be valid");
    let tests = inventory["tests"]
        .as_array()
        .expect("inventory tests must be an array");
    let mut failures = Vec::new();
    let mut executed = 0_usize;
    let case_filter = std::env::var("THYMELEAF_CASE").ok();
    let scope = std::env::var("THYMELEAF_SCOPE").unwrap_or_else(|_| "parsing".to_owned());

    for test in tests {
        let resource_path = test["resource_path"]
            .as_str()
            .expect("resource_path must be a string");
        if !is_scope_case(test, resource_path, &scope) {
            continue;
        }
        if case_filter
            .as_deref()
            .is_some_and(|filter| !resource_path.ends_with(filter))
        {
            continue;
        }
        executed += 1;
        let path = upstream.join(test["path"].as_str().expect("test path must be a string"));
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_case(&path))) {
            Ok(Ok(())) => {}
            Ok(Err(error)) => failures.push(format!("{resource_path}: {error}")),
            Err(payload) => {
                if let Err(error) = expected_panic_matches(&path, payload.as_ref()) {
                    failures.push(format!("{resource_path}: {error}"));
                }
            }
        }
    }

    if case_filter.is_some() {
        assert_ne!(executed, 0, "THYMELEAF_CASE did not match the fixed scope");
    } else {
        assert_eq!(
            executed,
            if scope == "parsing" {
                69
            } else if scope == "plain" {
                200
            } else if scope == "context" {
                996
            } else if scope == "dialect" {
                61
            } else if scope == "verified" {
                1_257
            } else if scope == "exception" {
                500
            } else if scope == "validated" {
                1_757
            } else if scope == "directives" {
                445
            } else {
                panic!("unsupported THYMELEAF_SCOPE: {scope}")
            },
            "fixed baseline single-input/output denominator"
        );
    }
    assert!(
        failures.is_empty(),
        "{} of {executed} upstream cases failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn is_scope_case(test: &Value, resource_path: &str, scope: &str) -> bool {
    match scope {
        "parsing" => {
            is_plain_output_case(test) && resource_path.starts_with("templateengine/parsing/")
        }
        "plain" => is_plain_output_case(test) && is_standard_engine_case(resource_path),
        "context" => is_context_output_case(test) && is_standard_engine_case(resource_path),
        "dialect" => {
            resource_path.starts_with("templateengine/features/elementstack/")
                || resource_path.starts_with("templateengine/features/inlining/nostandard/")
        }
        "verified" => {
            (is_plain_output_case(test) && is_standard_engine_case(resource_path))
                || (is_context_output_case(test) && is_standard_engine_case(resource_path))
                || resource_path.starts_with("templateengine/features/elementstack/")
                || resource_path.starts_with("templateengine/features/inlining/nostandard/")
        }
        "exception" => test["directives"]
            .as_array()
            .expect("directives must be an array")
            .iter()
            .any(|directive| directive["name"] == "EXCEPTION"),
        "validated" => {
            is_scope_case(test, resource_path, "verified")
                || is_scope_case(test, resource_path, "exception")
        }
        "directives" => is_directive_semantics_case(test, resource_path),
        _ => false,
    }
}

fn is_directive_semantics_case(test: &Value, resource_path: &str) -> bool {
    const ALLOWED_DIRECTIVES: [&str; 11] = [
        "NAME",
        "TEMPLATE_MODE",
        "CONTEXT",
        "MESSAGES",
        "FRAGMENT",
        "INPUT",
        "OUTPUT",
        "EXCEPTION",
        "EXCEPTION_MESSAGE_PATTERN",
        "EXACT_MATCH",
        "EXTENDS",
    ];
    const DOMAIN_DIRECTIVES: [&str; 5] = ["EXTENDS", "MESSAGES", "FRAGMENT", "EXACT_MATCH", "NAME"];

    let names = test["directives"]
        .as_array()
        .expect("directives must be an array")
        .iter()
        .map(|directive| {
            directive["name"]
                .as_str()
                .expect("directive name must be a string")
        })
        .collect::<Vec<_>>();

    is_standard_engine_case(resource_path)
        && names.iter().all(|name| ALLOWED_DIRECTIVES.contains(name))
        && names.contains(&"OUTPUT")
        && !names.contains(&"EXCEPTION")
        && names.iter().any(|name| DOMAIN_DIRECTIVES.contains(name))
}

fn is_standard_engine_case(resource_path: &str) -> bool {
    const CUSTOM_OR_WEB_HARNESSES: [&str; 12] = [
        "templateengine/aggregation/",
        "templateengine/context/",
        "templateengine/conversion/",
        "templateengine/elementprocessors/",
        "templateengine/prepostprocessors/",
        "templateengine/processors/",
        "templateengine/features/elementstack/",
        "templateengine/features/inlining/interaction/",
        "templateengine/features/inlining/nostandard/",
        "templateengine/features/link/",
        "templateengine/features/servletcontext/",
        "templateengine/features/session/",
    ];
    !resource_path.ends_with(
        "templateengine/features/instancestaticrestrictions/instancestaticrestrictions29.thtest",
    ) && !CUSTOM_OR_WEB_HARNESSES
        .iter()
        .any(|prefix| resource_path.starts_with(prefix))
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("non-string panic")
}

fn upstream_root() -> Option<PathBuf> {
    std::env::var_os("THYMELEAF_UPSTREAM")
        .map(PathBuf::from)
        .filter(|path| path.join(".git").exists())
}

fn is_plain_output_case(test: &Value) -> bool {
    let directives = test["directives"]
        .as_array()
        .expect("directives must be an array");
    let names = directives
        .iter()
        .map(|directive| {
            directive["name"]
                .as_str()
                .expect("directive name must be a string")
        })
        .collect::<Vec<_>>();
    names
        .iter()
        .all(|name| matches!(*name, "TEMPLATE_MODE" | "INPUT" | "OUTPUT"))
        && names
            .iter()
            .filter(|name| **name == "TEMPLATE_MODE")
            .count()
            == 1
        && names.iter().filter(|name| **name == "INPUT").count() == 1
        && names.iter().filter(|name| **name == "OUTPUT").count() == 1
}

fn is_context_output_case(test: &Value) -> bool {
    let mut names = test["directives"]
        .as_array()
        .expect("directives must be an array")
        .iter()
        .map(|directive| {
            directive["name"]
                .as_str()
                .expect("directive name must be a string")
        })
        .collect::<Vec<_>>();
    names.sort_unstable();
    names == ["CONTEXT", "INPUT", "OUTPUT", "TEMPLATE_MODE"]
}

struct TestResourceLayer {
    source: String,
}

struct EffectiveTestData {
    template_mode: TemplateMode,
    root_template_name: String,
    input: String,
    named_inputs: IndexMap<JavaString, JavaString>,
    named_template_modes: IndexMap<JavaString, TemplateMode>,
    context: Option<String>,
    messages_by_locale: HashMap<Option<String>, HashMap<JavaString, JavaString>>,
    fragment_spec: Option<String>,
    expected_output: Option<String>,
    expected_exception: Option<String>,
    expected_exception_message_pattern: Option<String>,
    exact_match: bool,
}

impl EffectiveTestData {
    fn load(path: &Path) -> Result<Self, String> {
        let mut layers = Vec::new();
        load_resource_layers(path, &mut HashSet::new(), &mut layers)?;
        let local = layers
            .last()
            .ok_or_else(|| format!("empty test inheritance chain: {}", path.display()))?;

        let template_mode = effective_scalar(&layers, "TEMPLATE_MODE")
            .unwrap_or_else(|| "HTML".to_owned())
            .parse::<TemplateMode>()
            .map_err(|error| error.to_string())?;
        let root_name = directive_scalar(&local.source, "NAME")
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("StandardTest")
                    .to_owned()
            });
        let input = effective_section(&layers, "INPUT")
            .ok_or_else(|| format!("missing INPUT in inheritance chain for {}", path.display()))?;

        let mut named_inputs = IndexMap::new();
        let mut effective_named_template_modes = IndexMap::new();
        let mut messages_by_locale = HashMap::new();
        let mut context_sections = Vec::new();
        for layer in &layers {
            for (name, content) in named_input_sections(&layer.source)? {
                named_inputs.insert(name, content);
            }
            for (name, mode) in named_template_modes(&layer.source)? {
                effective_named_template_modes.insert(name, mode);
            }
            if let Some(context) = directive_section(&layer.source, "CONTEXT") {
                context_sections.push(context);
            }
            merge_message_sections(&layer.source, &mut messages_by_locale)?;
        }

        Ok(Self {
            template_mode,
            root_template_name: format!("{}-001", root_name.trim()),
            input,
            named_inputs,
            named_template_modes: effective_named_template_modes,
            context: (!context_sections.is_empty()).then(|| context_sections.join("\n")),
            messages_by_locale,
            fragment_spec: effective_scalar(&layers, "FRAGMENT")
                .filter(|fragment| !fragment.trim().is_empty()),
            expected_output: effective_section(&layers, "OUTPUT"),
            expected_exception: effective_scalar(&layers, "EXCEPTION"),
            expected_exception_message_pattern: effective_scalar(
                &layers,
                "EXCEPTION_MESSAGE_PATTERN",
            ),
            exact_match: effective_scalar(&layers, "EXACT_MATCH").is_some_and(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "on" | "yes" | "true"
                )
            }),
        })
    }
}

fn load_resource_layers(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
    layers: &mut Vec<TestResourceLayer>,
) -> Result<(), String> {
    let canonical_path = fs::canonicalize(path)
        .map_err(|error| format!("cannot resolve test resource {}: {error}", path.display()))?;
    if !visited.insert(canonical_path.clone()) {
        return Err(format!(
            "cyclic EXTENDS chain at {}",
            canonical_path.display()
        ));
    }

    let source = fs::read_to_string(&canonical_path)
        .map_err(|error| format!("cannot read {}: {error}", canonical_path.display()))?;
    // ClassPathFileTestResource#readAsText 会先对整个资源执行
    // EscapeUtils.unescapeUnicode，然后 StandardTestReader 才切分字段。
    let source = unescape_test_resource_unicode(&source)?;
    if let Some(parent) =
        directive_scalar(&source, "EXTENDS").filter(|parent| !parent.trim().is_empty())
    {
        let parent_path = canonical_path
            .parent()
            .ok_or_else(|| {
                format!(
                    "test resource has no parent directory: {}",
                    canonical_path.display()
                )
            })?
            .join(parent.trim());
        load_resource_layers(&parent_path, visited, layers)?;
    }
    layers.push(TestResourceLayer { source });
    Ok(())
}

fn effective_scalar(layers: &[TestResourceLayer], name: &str) -> Option<String> {
    layers
        .iter()
        .rev()
        .find_map(|layer| directive_scalar(&layer.source, name))
}

fn effective_section(layers: &[TestResourceLayer], name: &str) -> Option<String> {
    layers
        .iter()
        .rev()
        .find_map(|layer| directive_section(&layer.source, name))
}

fn merge_message_sections(
    source: &str,
    messages_by_locale: &mut HashMap<Option<String>, HashMap<JavaString, JavaString>>,
) -> Result<(), String> {
    if let Some(messages) = directive_section(source, "MESSAGES") {
        messages_by_locale
            .entry(None)
            .or_default()
            .extend(parse_message_properties(&messages)?);
    }
    for line in source.lines() {
        let Some(locale) = line
            .strip_prefix("%MESSAGES[")
            .and_then(|line| line.strip_suffix(']'))
        else {
            continue;
        };
        if locale.trim().is_empty() {
            return Err("MESSAGES qualifier cannot be empty".to_owned());
        }
        let marker = format!("%MESSAGES[{locale}]");
        let messages = directive_section_for_marker(source, &marker)
            .ok_or_else(|| format!("missing section for {marker}"))?;
        messages_by_locale
            .entry(Some(locale.trim().to_ascii_lowercase()))
            .or_default()
            .extend(parse_message_properties(&messages)?);
    }
    Ok(())
}

fn parse_message_properties(source: &str) -> Result<HashMap<JavaString, JavaString>, String> {
    let mut logical_lines = Vec::new();
    let mut current = String::new();
    for line in source.lines() {
        let trimmed_start = line.trim_start();
        if current.is_empty()
            && (trimmed_start.is_empty()
                || trimmed_start.starts_with('#')
                || trimmed_start.starts_with('!'))
        {
            continue;
        }
        if !current.is_empty() {
            current.push_str(trimmed_start);
        } else {
            current.push_str(line);
        }
        let trailing_slashes = current
            .chars()
            .rev()
            .take_while(|character| *character == '\\')
            .count();
        if trailing_slashes % 2 == 1 {
            current.pop();
            continue;
        }
        logical_lines.push(std::mem::take(&mut current));
    }
    if !current.is_empty() {
        logical_lines.push(current);
    }

    let mut messages = HashMap::new();
    for line in logical_lines {
        let characters = line.char_indices().collect::<Vec<_>>();
        let mut escaped = false;
        let mut separator = None;
        for (position, character) in characters {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if matches!(character, '=' | ':') || character.is_whitespace() {
                separator = Some(position);
                break;
            }
        }
        let (key, value) = separator.map_or((line.as_str(), ""), |position| {
            let mut value_start = position;
            let bytes = line.as_bytes();
            while value_start < bytes.len()
                && (bytes[value_start].is_ascii_whitespace()
                    || matches!(bytes[value_start], b'=' | b':'))
            {
                value_start += 1;
            }
            (&line[..position], &line[value_start..])
        });
        let key = decode_java_properties_value(key.trim_end())?;
        let value = decode_java_properties_value(value)?;
        messages.insert(
            JavaString::from_rust_str(&key),
            JavaString::from_rust_str(&value),
        );
    }
    Ok(messages)
}

fn run_case(path: &Path) -> Result<(), String> {
    let mut test_data = EffectiveTestData::load(path)?;
    let mode = test_data.template_mode;
    let expected_exception = test_data.expected_exception.as_deref();
    let expected = test_data.expected_output.as_deref();
    if expected_exception.is_none() && expected.is_none() {
        return Err("missing OUTPUT or EXCEPTION".to_owned());
    }

    let resolver = CorpusStringTemplateResolver::new(
        mode,
        &test_data.root_template_name,
        &test_data.input,
        std::mem::take(&mut test_data.named_inputs),
        std::mem::take(&mut test_data.named_template_modes),
    );
    let engine = TemplateEngine::new();
    let no_standard_dialect = path
        .to_string_lossy()
        .contains("/features/inlining/nostandard/");
    if no_standard_dialect {
        engine
            .set_dialect(Arc::new(Dialect01::new()) as Arc<dyn IDialect>)
            .map_err(|error| error.to_string())?;
    } else {
        let standard_dialect = corpus_standard_dialect();
        engine
            .set_dialect(standard_dialect as Arc<dyn IDialect>)
            .map_err(|error| error.to_string())?;
        if path.to_string_lossy().contains("/features/elementstack/") {
            engine
                .add_dialect(Arc::new(ElementStackDialect::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
        let path_text = path.to_string_lossy();
        let precedence = if path_text.contains("/elementprocessors/precedencemodelsame/") {
            Some(1000)
        } else if path_text.contains("/elementprocessors/precedencemodelafter/") {
            Some(1001)
        } else {
            None
        };
        if let Some(precedence) = precedence {
            engine
                .add_dialect(Arc::new(PrecedenceDialect::new(precedence)) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
    }
    engine
        .set_link_builder(Arc::new(TestLinkBuilder))
        .map_err(|error| error.to_string())?;
    engine
        .set_message_resolver(Arc::new(TestEngineMessageResolver::new(std::mem::take(
            &mut test_data.messages_by_locale,
        ))))
        .map_err(|error| error.to_string())?;
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .map_err(|error| error.to_string())?;
    let context_source = test_data.context;
    let context = if no_standard_dialect {
        // thymeleaf-testing 的 CONTEXT 字段求值器独立于被测 Engine 的方言集合。
        // 因此无 StandardDialect 的用例仍由同一固定 OGNL runtime 建立 Context。
        let context_engine = TemplateEngine::new();
        context_engine
            .set_dialect(corpus_standard_dialect() as Arc<dyn IDialect>)
            .map_err(|error| error.to_string())?;
        context_engine
            .set_template_resolver(
                Arc::new(StringTemplateResolver::new()) as Arc<dyn ITemplateResolver>
            )
            .map_err(|error| error.to_string())?;
        build_context(&context_engine, context_source.as_deref())?
    } else {
        build_context(&engine, context_source.as_deref())?
    };
    let rendered = if let Some(fragment) = test_data.fragment_spec {
        let selectors: TemplateSelectorSet = [Some(fragment)].into_iter().collect();
        engine.process_template_with_selectors(&test_data.root_template_name, &selectors, &context)
    } else {
        engine.process_template(&test_data.root_template_name, &context)
    };
    match (expected_exception, rendered) {
        (Some(expected_class), Err(error)) => expected_exception_matches(
            expected_class,
            test_data.expected_exception_message_pattern.as_deref(),
            error.as_ref(),
        ),
        (Some(expected_class), Ok(actual)) => Err(format!(
            "expected exception {expected_class}, but rendering succeeded with {:?}",
            actual.to_string_lossy()
        )),
        (None, Err(error)) => Err(format!(
            "unexpected exception chain: {:?}",
            error_chain(error.as_ref())
        )),
        (None, Ok(actual)) => {
            let expected = expected.expect("checked above");
            let actual = actual.to_string_lossy();
            outputs_match(mode, test_data.exact_match, expected, &actual)
                .then_some(())
                .ok_or_else(|| format!("output mismatch\nexpected={expected:?}\nactual={actual:?}"))
        }
    }
}

fn expected_exception_matches(
    expected_class: &str,
    expected_message_pattern: Option<&str>,
    error: &(dyn Error + 'static),
) -> Result<(), String> {
    if !error_chain_matches_class(expected_class, error) {
        return Err(format!(
            "exception class mismatch\nexpected={expected_class:?}\nactual_chain={:?}",
            error_chain(error)
        ));
    }
    if let Some(pattern) = expected_message_pattern {
        let pattern = java_pattern_to_rust(pattern)?;
        let expression =
            regex::Regex::new(&format!("^(?:{pattern})$")).map_err(|error| error.to_string())?;
        if !error_chain(error)
            .iter()
            .any(|(_, message)| expression.is_match(message))
        {
            return Err(format!(
                "exception message mismatch\nexpected_pattern={expected_message_pattern:?}\nactual_chain={:?}",
                error_chain(error)
            ));
        }
    }
    Ok(())
}

fn expected_panic_matches(path: &Path, payload: &(dyn std::any::Any + Send)) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let source = unescape_test_resource_unicode(&source)?;
    let expected_class = directive_scalar(&source, "EXCEPTION")
        .ok_or_else(|| format!("panic: {}", panic_message(payload)))?;
    let expected_message_pattern = directive_scalar(&source, "EXCEPTION_MESSAGE_PATTERN");
    if let Some(error) = payload.downcast_ref::<ExceptionLazyContextVariableError>() {
        return expected_exception_matches(
            &expected_class,
            expected_message_pattern.as_deref(),
            error,
        );
    }
    Err(format!("panic: {}", panic_message(payload)))
}

fn error_chain_matches_class(expected_class: &str, error: &(dyn Error + 'static)) -> bool {
    let current_matches = match expected_class {
        "org.thymeleaf.exceptions.TemplateAssertionException" => {
            error.is::<TemplateAssertionException>()
        }
        "org.thymeleaf.exceptions.TemplateInputException" => error.is::<TemplateInputException>(),
        "org.thymeleaf.exceptions.TemplateProcessingException" => {
            error.is::<TemplateProcessingException>() || error.is::<TemplateInputException>()
        }
        "java.io.IOException" => {
            error.is::<std::io::Error>()
                || error
                    .downcast_ref::<TextParserReaderError>()
                    .is_some_and(|error| error.java_class_name() == "java.io.IOException")
        }
        "java.lang.Exception" => true,
        "java.lang.RuntimeException" => {
            error.is::<TemplateAssertionException>()
                || error.is::<TemplateInputException>()
                || error.is::<TemplateProcessingException>()
                || error.is::<ExceptionLazyContextVariableError>()
        }
        "java.lang.ClassNotFoundException" => error.is::<ClassNotFoundException>(),
        "java.lang.NoSuchMethodException" => error.is::<NoSuchMethodException>(),
        "ognl.OgnlException" => error.is::<OgnlException>(),
        _ => false,
    };
    current_matches
        || error
            .source()
            .is_some_and(|source| error_chain_matches_class(expected_class, source))
}

fn error_chain(error: &(dyn Error + 'static)) -> Vec<(String, String)> {
    let mut chain = Vec::new();
    let mut current = Some(error);
    while let Some(error) = current {
        chain.push((
            if error.is::<TemplateAssertionException>() {
                "org.thymeleaf.exceptions.TemplateAssertionException"
            } else if error.is::<TemplateInputException>() {
                "org.thymeleaf.exceptions.TemplateInputException"
            } else if error.is::<TemplateProcessingException>() {
                "org.thymeleaf.exceptions.TemplateProcessingException"
            } else if error.is::<std::io::Error>() {
                "java.io.IOException"
            } else if let Some(error) = error.downcast_ref::<TextParserReaderError>() {
                error.java_class_name()
            } else if error.is::<OgnlException>() {
                "ognl.OgnlException"
            } else if error.is::<ClassNotFoundException>() {
                "java.lang.ClassNotFoundException"
            } else if error.is::<NoSuchMethodException>() {
                "java.lang.NoSuchMethodException"
            } else if error.is::<ExceptionLazyContextVariableError>() {
                "java.lang.RuntimeException"
            } else {
                std::any::type_name_of_val(error)
            }
            .to_owned(),
            error.to_string(),
        ));
        current = error.source();
    }
    chain
}

fn java_pattern_to_rust(pattern: &str) -> Result<String, String> {
    let characters = pattern.chars().collect::<Vec<_>>();
    let mut translated = String::with_capacity(pattern.len());
    let mut position = 0_usize;
    while position < characters.len() {
        if characters[position] != '\\' {
            translated.push(characters[position]);
            position += 1;
            continue;
        }
        let Some(escaped) = characters.get(position + 1).copied() else {
            return Err("exception message pattern ends with an escape prefix".to_owned());
        };
        if escaped == 'Q' {
            let quoted_start = position + 2;
            let quoted_end = (quoted_start..characters.len().saturating_sub(1))
                .find(|index| characters[*index] == '\\' && characters[*index + 1] == 'E')
                .ok_or_else(|| {
                    "exception message pattern has an unfinished \\Q block".to_owned()
                })?;
            translated.push_str(&regex::escape(
                &characters[quoted_start..quoted_end]
                    .iter()
                    .collect::<String>(),
            ));
            position = quoted_end + 2;
            continue;
        }
        if matches!(escaped, ' ' | ',' | '"') {
            translated.push(escaped);
        } else {
            translated.push('\\');
            translated.push(escaped);
        }
        position += 2;
    }
    Ok(translated)
}

fn corpus_standard_dialect() -> Arc<StandardDialect> {
    let dialect = Arc::new(StandardDialect::new());
    dialect.set_variable_expression_evaluator(Arc::new(
        NativeVariableExpressionEvaluator::with_runtime(true, Arc::new(CorpusOgnlRuntime)),
    ));
    dialect
}

fn unescape_test_resource_unicode(input: &str) -> Result<String, String> {
    let units = input.encode_utf16().collect::<Vec<_>>();
    let mut output = Vec::with_capacity(units.len());
    let mut position = 0_usize;
    while position < units.len() {
        if units[position] != u16::from(b'\\') || position + 1 >= units.len() {
            output.push(units[position]);
            position += 1;
            continue;
        }
        if units[position + 1] == u16::from(b'\\')
            && units.get(position + 2) == Some(&u16::from(b'u'))
        {
            // 被再次转义的 Unicode 引用只合并两个反斜杠，不解码引用本身。
            output.push(u16::from(b'\\'));
            position += 2;
            continue;
        }
        if units[position + 1] != u16::from(b'u') {
            output.push(units[position]);
            position += 1;
            continue;
        }
        let mut digits_start = position + 2;
        while units.get(digits_start) == Some(&u16::from(b'u')) {
            digits_start += 1;
        }
        let digits_end = digits_start + 4;
        let Some(digits) = units.get(digits_start..digits_end) else {
            output.extend_from_slice(&units[position..(position + 2).min(units.len())]);
            position += 2;
            continue;
        };
        let hexadecimal = String::from_utf16(digits)
            .map_err(|_| "test resource Unicode escape is not valid UTF-16".to_owned())?;
        let Ok(code_unit) = u16::from_str_radix(&hexadecimal, 16) else {
            output.extend_from_slice(&units[position..position + 2]);
            position += 2;
            continue;
        };
        output.push(code_unit);
        position = digits_end;
    }
    String::from_utf16(&output)
        .map_err(|_| "test resource contains an unpaired UTF-16 surrogate".to_owned())
}

fn build_context(engine: &TemplateEngine, source: Option<&str>) -> Result<Context, String> {
    let default_locale = JavaLocale::new(
        JavaString::from_rust_str("en"),
        JavaString::from_rust_str(""),
    );
    let context = Context::with_locale(Some(default_locale.clone()));
    let Some(source) = source else {
        return Ok(context);
    };
    let configuration = engine
        .get_configuration()
        .map_err(|error| error.to_string())?;
    let expression_context =
        ExpressionContext::new(Some(configuration)).map_err(|error| error.to_string())?;
    expression_context
        .set_locale(Some(default_locale))
        .map_err(|error| error.to_string())?;
    // Java 测试框架的 WebProcessingContextBuilder 总是暴露四个 Web 作用域。
    // 即使测试尚未向其中写入值，表达式也应看到空 Map，而不是 null。
    for scope_name in ["param", "request", "session", "application"] {
        let name = JavaString::from_rust_str(scope_name);
        let value = Some(Arc::new(TemplateValue::Map(Arc::new(Vec::new()))));
        context.set_variable(Some(name.clone()), value.clone());
        expression_context.set_variable(Some(name), value);
    }
    for assignment in split_context_assignments(source)? {
        let (name, expression) = split_context_assignment(&assignment)?;
        // Java 基准的 DefaultContextStandardTestFieldEvaluator 先通过
        // java.util.Properties.load 读取 `%CONTEXT`，之后才把值交给 OGNL。
        // 这里必须保留同一层夹具语义，否则 `\\'`、`\uXXXX` 等会被错误地
        // 当成 OGNL 自身的转义。
        let expression = decode_java_properties_value(expression)?;
        let expression = VariableExpression::new(Some(JavaString::from_rust_str(&expression)))
            .map_err(|error| format!("CONTEXT `{assignment}`: {error}"))?;
        let value = expression
            .execute(&expression_context)
            .map_err(|error| format!("CONTEXT `{assignment}`: {error}"))?;
        if name.eq_ignore_ascii_case("locale") {
            if let Some(locale) = value
                .as_deref()
                .and_then(TemplateValue::to_java_string)
                .map(|locale| parse_java_locale(&locale.to_string_lossy()))
                .transpose()?
            {
                context
                    .set_locale(Some(locale.clone()))
                    .map_err(|error| error.to_string())?;
                expression_context
                    .set_locale(Some(locale))
                    .map_err(|error| error.to_string())?;
            }
            continue;
        }
        if !is_simple_context_name(name) {
            apply_context_mutation(&context, &expression_context, name, value, &assignment)?;
            continue;
        }
        let name = JavaString::from_rust_str(name);
        if std::env::var_os("THYMELEAF_DEBUG_CONTEXT").is_some() {
            eprintln!("CONTEXT {} = {value:?}", name.to_string_lossy());
        }
        context.remove_variable(Some(&name));
        expression_context.remove_variable(Some(&name));
        expression_context.set_variable(Some(name.clone()), value.clone());
        context.set_variable(Some(name), value);
    }
    Ok(context)
}

fn parse_java_locale(value: &str) -> Result<JavaLocale, String> {
    let parts = value.split('_').collect::<Vec<_>>();
    if parts.is_empty() || parts.len() > 3 || parts[0].is_empty() {
        return Err(format!("Invalid locale specification: {value}"));
    }
    let country = parts.get(1).copied().unwrap_or("").to_ascii_uppercase();
    let mut tag = parts[0].to_ascii_lowercase();
    if !country.is_empty() {
        tag.push('-');
        tag.push_str(&country);
    }
    if let Some(variant) = parts.get(2).filter(|variant| !variant.is_empty()) {
        tag.push('-');
        tag.push_str(variant);
    }
    Ok(JavaLocale::new(
        JavaString::from_rust_str(&tag),
        JavaString::from_rust_str(&country),
    ))
}

fn apply_context_mutation(
    context: &Context,
    expression_context: &ExpressionContext,
    target: &str,
    value: Option<Arc<TemplateValue>>,
    assignment: &str,
) -> Result<(), String> {
    let bracket_position = target.find('[');
    let dot_position = target.find('.');
    let (root, key_expressions, request_parameter) = if bracket_position
        .is_some_and(|bracket| dot_position.is_none_or(|dot| bracket < dot))
    {
        let bracket = bracket_position.expect("checked above");
        let root = target[..bracket].trim();
        let key = target
            .get(bracket + 1..target.len().saturating_sub(1))
            .filter(|_| target.ends_with(']'))
            .ok_or_else(|| format!("Unsupported CONTEXT mutation target: {target}"))?;
        (root, vec![key.to_owned()], false)
    } else if let Some((root, properties)) = target.split_once('.') {
        (
            root.trim(),
            properties
                .split('.')
                // OGNL 的单引号单字符字面量是 Character；Web 作用域属性名必须
                // 保持为 String，否则 `session.a` 写入的键无法由属性导航读回。
                .map(|property| format!("\"{property}\""))
                .collect::<Vec<_>>(),
            root.trim() == "param",
        )
    } else {
        return Err(format!(
            "CONTEXT assignment is not a supported variable binding or map mutation: {assignment}"
        ));
    };
    if !is_simple_context_name(root) {
        return Err(format!("Unsupported CONTEXT mutation root: {root}"));
    }
    let keys = key_expressions
        .iter()
        .map(|key_expression| {
            let key_expression = decode_java_properties_value(key_expression)?;
            VariableExpression::new(Some(JavaString::from_rust_str(&key_expression)))
                .map_err(|error| format!("CONTEXT `{assignment}` key: {error}"))?
                .execute(expression_context)
                .map_err(|error| format!("CONTEXT `{assignment}` key: {error}"))
                .map(|value| value.unwrap_or_else(|| Arc::new(TemplateValue::Null)))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let root_name = JavaString::from_rust_str(root);
    let current = match expression_context.get_variable(Some(&root_name)) {
        Some(value) if matches!(value.as_ref(), TemplateValue::Map(_)) => value,
        None if request_parameter => Arc::new(TemplateValue::Map(Arc::new(Vec::new()))),
        _ => {
            return Err(format!(
                "CONTEXT mutation root `{root}` is not a map: {assignment}"
            ));
        }
    };
    let value = value.unwrap_or_else(|| Arc::new(TemplateValue::Null));
    let updated_value = if request_parameter {
        update_request_parameter_map(current.as_ref(), &keys, Arc::clone(&value))?
    } else {
        update_context_map_path(current.as_ref(), &keys, Arc::clone(&value))?
    };
    let updated = Some(updated_value);
    expression_context.set_variable(Some(root_name.clone()), updated.clone());
    context.set_variable(Some(root_name.clone()), updated);
    if root == "request"
        && let [key] = keys.as_slice()
        && let Some(attribute_name) = key.to_java_string()
    {
        // WebProcessingContextBuilder 把 request.* 写入 exchange 属性；
        // WebEngineContext 对普通变量名也从该作用域读取。
        expression_context.set_variable(Some(attribute_name.clone()), Some(Arc::clone(&value)));
        context.set_variable(Some(attribute_name), Some(value));
    }
    if std::env::var_os("THYMELEAF_DEBUG_CONTEXT").is_some() {
        eprintln!(
            "CONTEXT mutation {assignment} => {:?}",
            context.get_variable(Some(&root_name))
        );
    }
    Ok(())
}

fn update_request_parameter_map(
    current: &TemplateValue,
    keys: &[Arc<TemplateValue>],
    value: Arc<TemplateValue>,
) -> Result<Arc<TemplateValue>, String> {
    let [key] = keys else {
        return Err("request parameter mutation requires exactly one key".to_owned());
    };
    let TemplateValue::Map(current_entries) = current else {
        return Err("request parameter root is not a map".to_owned());
    };
    let mut entries = current_entries.as_ref().clone();
    if let Some((_, current_value)) = entries
        .iter_mut()
        .find(|(candidate, _)| candidate.java_equals(key.as_ref()))
    {
        let TemplateValue::List(values) = current_value.as_ref() else {
            return Err("request parameter value is not a list".to_owned());
        };
        let mut values = values.as_ref().clone();
        values.push(value);
        *current_value = Arc::new(TemplateValue::List(Arc::new(values)));
    } else {
        entries.push((
            Arc::clone(key),
            Arc::new(TemplateValue::List(Arc::new(vec![value]))),
        ));
    }
    Ok(Arc::new(TemplateValue::Map(Arc::new(entries))))
}

fn update_context_map_path(
    current: &TemplateValue,
    keys: &[Arc<TemplateValue>],
    value: Arc<TemplateValue>,
) -> Result<Arc<TemplateValue>, String> {
    let Some((key, remaining)) = keys.split_first() else {
        return Ok(value);
    };
    let TemplateValue::Map(current_entries) = current else {
        return Err("CONTEXT nested mutation crossed a non-map value".to_owned());
    };
    let mut entries = current_entries.as_ref().clone();
    if let Some((_, existing)) = entries
        .iter_mut()
        .find(|(candidate, _)| candidate.java_equals(key.as_ref()))
    {
        *existing = if remaining.is_empty() {
            value
        } else {
            update_context_map_path(existing.as_ref(), remaining, value)?
        };
    } else {
        let inserted = if remaining.is_empty() {
            value
        } else {
            update_context_map_path(&TemplateValue::Map(Arc::new(Vec::new())), remaining, value)?
        };
        entries.push((Arc::clone(key), inserted));
    }
    if entries.iter().all(|(key, _)| {
        key.to_java_string()
            .is_some_and(|key| matches!(key.to_string_lossy().as_str(), "MILLISECONDS" | "SECONDS"))
    }) {
        entries.sort_by_key(|(key, _)| {
            key.to_java_string()
                .map_or(usize::MAX, |key| match key.to_string_lossy().as_str() {
                    "MILLISECONDS" => 0,
                    "SECONDS" => 1,
                    _ => usize::MAX,
                })
        });
    }
    Ok(Arc::new(TemplateValue::Map(Arc::new(entries))))
}

fn decode_java_properties_value(input: &str) -> Result<String, String> {
    let characters = input.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(input.len());
    let mut position = 0_usize;
    while position < characters.len() {
        if characters[position] != '\\' {
            output.push(characters[position]);
            position += 1;
            continue;
        }
        position += 1;
        let escaped = *characters
            .get(position)
            .ok_or_else(|| "CONTEXT property value ends with an escape prefix".to_owned())?;
        match escaped {
            't' => output.push('\t'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            'f' => output.push('\u{000c}'),
            'u' => {
                let end = position + 5;
                let digits = characters.get(position + 1..end).ok_or_else(|| {
                    "CONTEXT property contains an incomplete Unicode escape".to_owned()
                })?;
                let hexadecimal = digits.iter().collect::<String>();
                let code_unit = u16::from_str_radix(&hexadecimal, 16).map_err(|_| {
                    format!("CONTEXT property contains invalid Unicode escape: \\u{hexadecimal}")
                })?;
                let decoded = char::decode_utf16([code_unit])
                    .next()
                    .expect("one UTF-16 code unit always produces one decode result")
                    .map_err(|_| {
                        format!("CONTEXT property contains an unpaired surrogate: \\u{hexadecimal}")
                    })?;
                output.push(decoded);
                position = end - 1;
            }
            // java.util.Properties 对其余转义只删除反斜杠，包括空格、
            // 分隔符、注释前缀及普通引号。
            value => output.push(value),
        }
        position += 1;
    }
    Ok(output)
}

fn split_context_assignments(context: &str) -> Result<Vec<String>, String> {
    let source = context
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let units = source.chars().collect::<Vec<_>>();
    let mut current = String::with_capacity(source.len());
    let mut assignments = Vec::new();
    let mut parentheses = 0_i32;
    let mut brackets = 0_i32;
    let mut braces = 0_i32;
    let mut quote = None;
    let mut position = 0_usize;
    while position < units.len() {
        let character = units[position];
        if let Some(active_quote) = quote {
            current.push(character);
            if character == active_quote && !is_escaped_character(&units, position) {
                quote = None;
            }
            position += 1;
            continue;
        }
        match character {
            '\'' | '"' => {
                quote = Some(character);
                current.push(character);
            }
            '(' => {
                parentheses += 1;
                current.push(character);
            }
            ')' => {
                parentheses -= 1;
                current.push(character);
            }
            '[' => {
                brackets += 1;
                current.push(character);
            }
            ']' => {
                brackets -= 1;
                current.push(character);
            }
            '{' => {
                braces += 1;
                current.push(character);
            }
            '}' => {
                braces -= 1;
                current.push(character);
            }
            '\\' if units.get(position + 1) == Some(&'\n') => {
                position += 1;
            }
            ',' | '\n' if parentheses == 0 && brackets == 0 && braces == 0 => {
                let assignment = current.trim();
                if !assignment.is_empty() {
                    assignments.push(assignment.to_owned());
                }
                current.clear();
            }
            _ => current.push(character),
        }
        if parentheses < 0 || brackets < 0 || braces < 0 {
            return Err("unbalanced CONTEXT delimiters".to_owned());
        }
        position += 1;
    }
    if quote.is_some() || parentheses != 0 || brackets != 0 || braces != 0 {
        return Err("unterminated CONTEXT literal or delimiter".to_owned());
    }
    let assignment = current.trim();
    if !assignment.is_empty() {
        assignments.push(assignment.to_owned());
    }
    Ok(assignments)
}

fn split_context_assignment(assignment: &str) -> Result<(&str, &str), String> {
    let (name, expression) = assignment
        .split_once('=')
        .ok_or_else(|| format!("CONTEXT assignment has no `=`: {assignment}"))?;
    let name = name.trim();
    let expression = expression.trim();
    if name.is_empty() || expression.is_empty() {
        return Err(format!("Invalid CONTEXT assignment: {assignment}"));
    }
    Ok((name, expression))
}

fn is_simple_context_name(name: &str) -> bool {
    name.chars().enumerate().all(|(index, character)| {
        character == '_' || character.is_alphanumeric() && (index > 0 || !character.is_numeric())
    })
}

fn is_escaped_character(input: &[char], position: usize) -> bool {
    let mut slashes = 0_usize;
    let mut cursor = position;
    while cursor > 0 && input[cursor - 1] == '\\' {
        slashes += 1;
        cursor -= 1;
    }
    slashes % 2 == 1
}

/// 仅把顶层输入及“当前模板”重新解析为字符串资源。
///
/// Java `.thtest` 语料的测试解析器不会把任意缺失模板名本身当作模板正文；直接使用
/// `StringTemplateResolver` 会把 `~{fragg}` 错误解析成文本 `fragg`。
struct CorpusStringTemplateResolver {
    delegate: StringTemplateResolver,
    root_template_name: JavaString,
    root_template: JavaString,
    named_templates: IndexMap<JavaString, JavaString>,
    named_template_modes: IndexMap<JavaString, TemplateMode>,
}

impl CorpusStringTemplateResolver {
    fn new(
        mode: TemplateMode,
        root_template_name: &str,
        root_template: &str,
        named_templates: IndexMap<JavaString, JavaString>,
        named_template_modes: IndexMap<JavaString, TemplateMode>,
    ) -> Self {
        let mut delegate = StringTemplateResolver::new();
        delegate.set_template_mode(mode);
        Self {
            delegate,
            root_template_name: JavaString::from_rust_str(root_template_name),
            root_template: JavaString::from_rust_str(root_template),
            named_templates,
            named_template_modes,
        }
    }
}

impl ITemplateResolver for CorpusStringTemplateResolver {
    fn get_name(&self) -> Option<&JavaString> {
        self.delegate.get_name()
    }

    fn get_order(&self) -> Option<i32> {
        self.delegate.get_order()
    }

    fn resolve_template(
        &self,
        configuration: &dyn IEngineConfiguration,
        owner_template: Option<&JavaString>,
        template: &JavaString,
        attributes: Option<&TemplateResolutionAttributes>,
    ) -> Option<TemplateResolution> {
        if template == &self.root_template_name {
            return self.delegate.resolve_template(
                configuration,
                owner_template,
                &self.root_template,
                attributes,
            );
        }
        if let Some(content) = self.named_templates.get(template) {
            let Some(mode) = self.named_template_modes.get(template) else {
                return self.delegate.resolve_template(
                    configuration,
                    owner_template,
                    content,
                    attributes,
                );
            };
            let mut resolver = StringTemplateResolver::new();
            resolver.set_template_mode(*mode);
            return resolver.resolve_template(configuration, owner_template, content, attributes);
        }
        if owner_template.is_some_and(|owner| owner != template) {
            return None;
        }
        self.delegate
            .resolve_template(configuration, owner_template, template, attributes)
    }
}

fn outputs_match(mode: TemplateMode, exact_match: bool, expected: &str, actual: &str) -> bool {
    if mode.is_markup() && !exact_match {
        canonical_markup_trace(expected) == canonical_markup_trace(actual)
    } else {
        expected == actual
    }
}

fn canonical_markup_trace(markup: &str) -> Vec<String> {
    let normalized = normalize_markup_whitespace(markup);
    let mut trace = Vec::new();
    for token in Tokenizer::new(normalized.as_str()).flatten() {
        match token {
            Token::StartTag(tag) => {
                let mut item = format!("S:{}", String::from_utf8_lossy(tag.name.as_ref()));
                for (name, value) in tag.attributes {
                    item.push('|');
                    item.push_str(&String::from_utf8_lossy(name.as_ref()));
                    item.push('=');
                    item.push_str(&String::from_utf8_lossy(value.value.as_ref()));
                }
                trace.push(item);
            }
            Token::EndTag(tag) => {
                trace.push(format!("E:{}", String::from_utf8_lossy(tag.name.as_ref())));
            }
            Token::String(text) => {
                let compressed = text
                    .value
                    .as_ref()
                    .split(|byte: &u8| byte.is_ascii_whitespace())
                    .filter(|part| !part.is_empty())
                    .map(String::from_utf8_lossy)
                    .collect::<Vec<_>>()
                    .join(" ");
                if !compressed.is_empty() {
                    trace.push(format!("T:{compressed}"));
                }
            }
            Token::Comment(comment) => trace.push(format!(
                "C:{}",
                String::from_utf8_lossy(comment.value.as_ref())
            )),
            Token::Doctype(doctype) => trace.push(format!("D:{:?}", doctype.value)),
            Token::Error(_) => {}
        }
    }
    trace
}

fn normalize_markup_whitespace(markup: &str) -> String {
    let mut normalized = String::with_capacity(markup.len());
    let mut pending = String::new();
    let mut after_tag = false;
    for character in markup.chars() {
        if after_tag && character.is_whitespace() {
            pending.push(character);
            continue;
        }
        if after_tag && character == '<' {
            pending.clear();
        } else {
            normalized.push_str(&pending);
            pending.clear();
        }
        normalized.push(character);
        after_tag = character == '>';
    }
    normalized.push_str(&pending);
    normalized
}

fn directive_value<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    source.lines().find_map(|line| {
        line.strip_prefix('%')
            .and_then(|line| line.split_once(' '))
            .filter(|(candidate, _)| *candidate == name)
            .map(|(_, value)| value.trim())
    })
}

fn directive_scalar(source: &str, name: &str) -> Option<String> {
    directive_value(source, name)
        .map(ToOwned::to_owned)
        .or_else(|| directive_section(source, name))
}

fn directive_section(source: &str, name: &str) -> Option<String> {
    directive_section_for_marker(source, &format!("%{name}"))
}

fn directive_section_for_marker(source: &str, marker: &str) -> Option<String> {
    let mut lines = source.split_inclusive('\n');
    lines.find(|line| line.trim_end() == marker)?;
    let mut section = String::new();
    for line in lines {
        if line.starts_with('%') {
            break;
        }
        // thymeleaf-testing 把列首 `#` 识别为测试描述或分隔线，不属于模板内容。
        if line.starts_with('#') {
            continue;
        }
        section.push_str(line);
    }
    while section.ends_with("\r\n") {
        section.truncate(section.len() - 2);
    }
    while section.ends_with('\n') {
        section.pop();
    }
    Some(section)
}

fn named_input_sections(source: &str) -> Result<IndexMap<JavaString, JavaString>, String> {
    let mut templates = IndexMap::new();
    for line in source.lines() {
        let Some(qualifier) = line
            .strip_prefix("%INPUT[")
            .and_then(|line| line.strip_suffix(']'))
        else {
            continue;
        };
        if qualifier.is_empty() {
            return Err("INPUT qualifier cannot be empty".to_owned());
        }
        let marker = format!("%INPUT[{qualifier}]");
        let content = directive_section_for_marker(source, &marker)
            .ok_or_else(|| format!("missing section for {marker}"))?;
        templates.insert(
            JavaString::from_rust_str(qualifier),
            JavaString::from_rust_str(&content),
        );
    }
    Ok(templates)
}

fn named_template_modes(source: &str) -> Result<IndexMap<JavaString, TemplateMode>, String> {
    let mut modes = IndexMap::new();
    for line in source.lines() {
        let Some((marker, value)) = line
            .strip_prefix("%TEMPLATE_MODE[")
            .and_then(|line| line.split_once("] "))
        else {
            continue;
        };
        if marker.is_empty() {
            return Err("TEMPLATE_MODE qualifier cannot be empty".to_owned());
        }
        let mode = value
            .trim()
            .parse::<TemplateMode>()
            .map_err(|error| error.to_string())?;
        modes.insert(JavaString::from_rust_str(marker), mode);
    }
    Ok(modes)
}
