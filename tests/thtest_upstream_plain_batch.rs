//! 上游无外部上下文依赖 `.thtest` 的批量端到端运行器。
//!
//! 默认开发环境没有 Java 源仓库时不执行；迁移门禁通过
//! `THYMELEAF_UPSTREAM=/path/to/thymeleaf cargo test --test
//! thtest_upstream_plain_batch` 强制运行固定上游语料。

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
use thymeleaf::util::JavaString;
use thymeleaf::{
    IEngineConfiguration, ITemplateEngine, ITemplateResolver, StandardDialect, TemplateEngine,
    TemplateMode, TemplateResolutionAttributes,
};

use support::{
    CorpusOgnlRuntime, Dialect01, ElementStackDialect, ExceptionLazyContextVariableError,
    PrecedenceDialect, TestLinkBuilder,
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
        _ => false,
    }
}

fn is_standard_engine_case(resource_path: &str) -> bool {
    const CUSTOM_OR_WEB_HARNESSES: [&str; 11] = [
        "templateengine/aggregation/",
        "templateengine/context/",
        "templateengine/conversion/",
        "templateengine/elementprocessors/",
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

fn run_case(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
    // ClassPathFileTestResource#readAsText 会先对整个 `.thtest` 执行
    // EscapeUtils.unescapeUnicode，然后 StandardTestReader 才切分各指令段。
    let source = unescape_test_resource_unicode(&source)?;
    let mode = inherited_directive_value(path, &source, "TEMPLATE_MODE")?
        .ok_or_else(|| "missing TEMPLATE_MODE".to_owned())?
        .parse::<TemplateMode>()
        .map_err(|error| error.to_string())?;
    let input = directive_section(&source, "INPUT").ok_or_else(|| "missing INPUT".to_owned())?;
    let expected_exception = directive_scalar(&source, "EXCEPTION");
    let expected = directive_section(&source, "OUTPUT");
    if expected_exception.is_none() && expected.is_none() {
        return Err("missing OUTPUT or EXCEPTION".to_owned());
    }

    let root_template_name = format!(
        "{}-001",
        path.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "test resource file name is not valid UTF-8".to_owned())?
    );
    let resolver = CorpusStringTemplateResolver::new(
        mode,
        &root_template_name,
        &input,
        named_input_sections(&source)?,
        named_template_modes(&source)?,
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
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .map_err(|error| error.to_string())?;
    let context_source = directive_section(&source, "CONTEXT");
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
    match (
        expected_exception.as_deref(),
        engine.process_template(&root_template_name, &context),
    ) {
        (Some(expected_class), Err(error)) => {
            let expected_message_pattern = directive_scalar(&source, "EXCEPTION_MESSAGE_PATTERN");
            expected_exception_matches(
                expected_class,
                expected_message_pattern.as_deref(),
                error.as_ref(),
            )
        }
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
            outputs_match(mode, &expected, &actual)
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
    let context = Context::new();
    let Some(source) = source else {
        return Ok(context);
    };
    let configuration = engine
        .get_configuration()
        .map_err(|error| error.to_string())?;
    let expression_context =
        ExpressionContext::new(Some(configuration)).map_err(|error| error.to_string())?;
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
        if !is_simple_context_name(name) {
            apply_context_mutation(&context, &expression_context, name, value, &assignment)?;
            continue;
        }
        let name = JavaString::from_rust_str(name);
        if std::env::var_os("THYMELEAF_DEBUG_CONTEXT").is_some() {
            eprintln!("CONTEXT {} = {value:?}", name.to_string_lossy());
        }
        expression_context.set_variable(Some(name.clone()), value.clone());
        context.set_variable(Some(name), value);
    }
    Ok(context)
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
                .map(|property| format!("'{property}'"))
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
    let value = if request_parameter {
        Arc::new(TemplateValue::List(Arc::new(vec![value])))
    } else {
        value
    };
    let updated = Some(update_context_map_path(current.as_ref(), &keys, value)?);
    expression_context.set_variable(Some(root_name.clone()), updated.clone());
    context.set_variable(Some(root_name), updated);
    Ok(())
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

fn outputs_match(mode: TemplateMode, expected: &str, actual: &str) -> bool {
    if mode.is_markup() {
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

fn inherited_directive_value(
    path: &Path,
    source: &str,
    name: &str,
) -> Result<Option<String>, String> {
    if let Some(value) = directive_value(source, name) {
        return Ok(Some(value.to_owned()));
    }
    let Some(parent) = directive_value(source, "EXTENDS") else {
        return Ok(None);
    };
    let parent_path = path
        .parent()
        .ok_or_else(|| format!("test resource has no parent directory: {}", path.display()))?
        .join(parent);
    let parent_source = fs::read_to_string(&parent_path)
        .map_err(|error| format!("cannot read EXTENDS {}: {error}", parent_path.display()))?;
    let parent_source = unescape_test_resource_unicode(&parent_source)?;
    inherited_directive_value(&parent_path, &parent_source, name)
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
