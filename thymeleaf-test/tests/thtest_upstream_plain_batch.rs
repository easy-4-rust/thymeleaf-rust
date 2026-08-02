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
use thymeleaf::context::{ExpressionContext, IContext, WebContext};
use thymeleaf::dialect::IDialect;
use thymeleaf::exceptions::{
    TemplateAssertionException, TemplateInputException, TemplateProcessingException,
};
use thymeleaf::expression::{
    ClassNotFoundException, IStandardExpression, NativeVariableExpressionEvaluator,
    NoSuchMethodException, OgnlException, VariableExpression,
};
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::text::TextParserReaderError;
use thymeleaf::util::{JavaLocale, JavaString};
use thymeleaf::web::IWebExchange;
use thymeleaf::{
    ITemplateEngine, ITemplateResolver, StandardDialect, TemplateEngine, TemplateMode,
    TemplateSelectorSet,
};

use support::thtest_harness::{
    CorpusStringTemplateResolver, build_context, decode_java_properties_value,
    directive_section_for_marker, is_simple_context_name, named_input_sections,
    named_template_modes, split_context_assignment, split_context_assignments,
};
use support::{
    ContextDialect, ContextVarTestDialect, ConversionTestDialect1, ConversionTestDialect4,
    CorpusOgnlRuntime, CorpusWebExchange, Dialect01, Dialect02, ElementStackDialect,
    ExceptionLazyContextVariableError, InteractionDialect01, MarkupDialect, NoOpDialect,
    PrePostProcessorsDialect01, PrecedenceDialect, RemoveDialect, ReplaceWithNonProcessableDialect,
    ReplaceWithProcessableDialect, SurroundDialect, TestEngineMessageResolver, TestLinkBuilder,
};

const INVENTORY: &str = include_str!("../../docs/migration/baseline/thtest_inventory.json");
const SEMANTIC_SCOPES: [&str; 15] = [
    "validated",
    "directives",
    "prepostprocessors",
    "multiinput",
    "link",
    "inlining_interaction",
    "conversion",
    "block",
    "context_vartest",
    "noop",
    "aggregation",
    "markup",
    "precedence",
    "web_context",
    "processor_remaining",
];

#[test]
fn semantic_inventory_is_fully_disposed() {
    let inventory: Value = serde_json::from_str(INVENTORY).expect("inventory JSON must be valid");
    let tests = inventory["tests"]
        .as_array()
        .expect("inventory tests must be an array");
    let executable = tests
        .iter()
        .filter(|test| test["kind"] == "EXECUTABLE")
        .collect::<Vec<_>>();
    let verified = executable
        .iter()
        .filter(|test| {
            let resource_path = test["resource_path"]
                .as_str()
                .expect("resource_path must be a string");
            is_scope_case(test, resource_path, "semantic_all")
        })
        .count();
    let dispositions = executable
        .iter()
        .filter(|test| {
            let resource_path = test["resource_path"]
                .as_str()
                .expect("resource_path must be a string");
            !is_scope_case(test, resource_path, "semantic_all")
                && is_named_semantic_disposition(resource_path)
        })
        .count();
    let unexplained = executable
        .iter()
        .filter_map(|test| {
            let resource_path = test["resource_path"].as_str()?;
            (!is_scope_case(test, resource_path, "semantic_all")
                && !is_named_semantic_disposition(resource_path))
            .then_some(resource_path)
        })
        .collect::<Vec<_>>();

    assert_eq!(executable.len(), 2_608);
    assert_eq!(verified, 2_595);
    assert_eq!(dispositions, 13);
    assert!(
        unexplained.is_empty(),
        "unexplained semantic resources: {unexplained:?}"
    );
}

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
                433
            } else if scope == "prepostprocessors" {
                10
            } else if scope == "multiinput" {
                237
            } else if scope == "link" {
                35
            } else if scope == "inlining_interaction" {
                18
            } else if scope == "conversion" {
                28
            } else if scope == "processor_misc" {
                23
            } else if scope == "block" {
                8
            } else if scope == "context_vartest" {
                38
            } else if scope == "noop" {
                4
            } else if scope == "aggregation" {
                3
            } else if scope == "markup" {
                11
            } else if scope == "precedence" {
                6
            } else if scope == "web_context" {
                5
            } else if scope == "processor_remaining" {
                4
            } else if scope == "semantic_all" {
                2_595
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
        "semantic_all" => SEMANTIC_SCOPES
            .iter()
            .any(|scope| is_scope_case(test, resource_path, scope)),
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
        "prepostprocessors" => {
            resource_path.starts_with("templateengine/prepostprocessors/")
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        "multiinput" => is_multi_input_standard_case(test, resource_path),
        "link" => {
            resource_path.starts_with("templateengine/features/link/")
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        "inlining_interaction" => {
            resource_path.starts_with("templateengine/features/inlining/interaction/")
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        "conversion" => {
            (resource_path.starts_with("templateengine/conversion/conversion1/")
                || resource_path.starts_with("templateengine/conversion/conversion4/"))
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        "processor_misc" => {
            (resource_path.starts_with("templateengine/elementprocessors/markup/")
                || resource_path.starts_with("templateengine/elementprocessors/block/")
                || resource_path.starts_with("templateengine/processors/noop/"))
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        "block" => {
            resource_path.starts_with("templateengine/elementprocessors/block/")
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        "context_vartest" => {
            resource_path.starts_with("templateengine/context/vartest/")
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        "noop" => {
            resource_path.starts_with("templateengine/processors/noop/")
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        "aggregation" => {
            resource_path.starts_with("templateengine/aggregation/")
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        "markup" => {
            resource_path.starts_with("templateengine/elementprocessors/markup/")
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        "precedence" => {
            resource_path.starts_with("templateengine/elementprocessors/precedence")
                && (test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
                    || test["directives"]
                        .as_array()
                        .expect("directives must be an array")
                        .iter()
                        .any(|directive| directive["name"] == "EXCEPTION"))
        }
        "web_context" => {
            (resource_path.starts_with("templateengine/context/base/")
                || resource_path.starts_with("templateengine/features/session/")
                || resource_path.starts_with("templateengine/features/servletcontext/"))
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        "processor_remaining" => {
            (resource_path.starts_with("templateengine/processors/remove/")
                || resource_path
                    .starts_with("templateengine/processors/replacewithnonprocessable/")
                || resource_path.starts_with("templateengine/processors/replacewithprocessable/")
                || resource_path.starts_with("templateengine/processors/surround/"))
                && test["directives"]
                    .as_array()
                    .expect("directives must be an array")
                    .iter()
                    .any(|directive| directive["name"] == "OUTPUT")
        }
        _ => false,
    }
}

fn is_named_semantic_disposition(resource_path: &str) -> bool {
    resource_path.starts_with("templateengine/features/execinfo/")
        || resource_path
            == "templateengine/features/instancestaticrestrictions/instancestaticrestrictions29.thtest"
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
        && !is_disabled_restricted_exec_info_resource(resource_path)
        && names.iter().all(|name| ALLOWED_DIRECTIVES.contains(name))
        && names.contains(&"OUTPUT")
        && !names.contains(&"EXCEPTION")
        && names.iter().any(|name| DOMAIN_DIRECTIVES.contains(name))
}

fn is_disabled_restricted_exec_info_resource(resource_path: &str) -> bool {
    const DISABLED_CASES: [&str; 12] = [
        "execinfo06.thtest",
        "execinfo08.thtest",
        "execinfo09.thtest",
        "execinfo10.thtest",
        "execinfo11.thtest",
        "execinfo12.thtest",
        "execinfo13.thtest",
        "execinfo14.thtest",
        "execinfo15.thtest",
        "execinfo20.thtest",
        "execinfo21.thtest",
        "execinfo22.thtest",
    ];

    // Java FeaturesTest#testExecInfo 已注释整个目录；这些 TEXT 资源又通过
    // StandardTextTagProcessor 的 RESTRICTED 上下文访问 #execInfo，与 Java
    // OGNLVariableExpressionEvaluator#checkRestrictedVariables 明确冲突。
    // 保留安全拒绝语义，不把上游未执行的历史期望伪装成可通过基线。
    resource_path.starts_with("templateengine/features/execinfo/")
        && DISABLED_CASES
            .iter()
            .any(|case| resource_path.ends_with(case))
}

fn is_multi_input_standard_case(test: &Value, resource_path: &str) -> bool {
    const ALLOWED_DIRECTIVES: [&str; 9] = [
        "NAME",
        "TEMPLATE_MODE",
        "CONTEXT",
        "MESSAGES",
        "FRAGMENT",
        "INPUT",
        "OUTPUT",
        "EXACT_MATCH",
        "EXTENDS",
    ];
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
        && !is_disabled_restricted_exec_info_resource(resource_path)
        && !is_directive_semantics_case(test, resource_path)
        && names.iter().all(|name| ALLOWED_DIRECTIVES.contains(name))
        && names.contains(&"OUTPUT")
        && names.iter().filter(|name| **name == "INPUT").count() > 1
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
    let path_text = path.to_string_lossy();
    let no_standard_dialect = path_text.contains("/features/inlining/nostandard/");
    if path_text.contains("/conversion/conversion1/") {
        engine
            .set_dialect(Arc::new(ConversionTestDialect1::new()) as Arc<dyn IDialect>)
            .map_err(|error| error.to_string())?;
    } else if path_text.contains("/conversion/conversion4/") {
        engine
            .set_dialect(Arc::new(ConversionTestDialect4::new()) as Arc<dyn IDialect>)
            .map_err(|error| error.to_string())?;
    } else if path_text.contains("/processors/noop/") {
        engine
            .set_dialect(Arc::new(NoOpDialect::new()) as Arc<dyn IDialect>)
            .map_err(|error| error.to_string())?;
    } else if no_standard_dialect {
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
        if path
            .to_string_lossy()
            .contains("/templateengine/prepostprocessors/")
        {
            engine
                .add_dialect(Arc::new(PrePostProcessorsDialect01::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
        if path
            .to_string_lossy()
            .contains("/features/inlining/interaction/")
        {
            engine
                .add_dialect(Arc::new(InteractionDialect01::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
        if path_text.contains("/templateengine/aggregation/") {
            engine
                .add_dialect(Arc::new(Dialect01::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
            engine
                .add_dialect(Arc::new(Dialect02::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
        if path_text.contains("/elementprocessors/markup/") {
            engine
                .add_dialect(Arc::new(MarkupDialect::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
        if path_text.contains("/context/vartest/") {
            engine
                .add_dialect(Arc::new(ContextVarTestDialect::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
        if path_text.contains("/context/base/") {
            engine
                .add_dialect(Arc::new(ContextDialect::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
        if path_text.contains("/processors/remove/") {
            engine
                .add_dialect(Arc::new(RemoveDialect::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
        if path_text.contains("/processors/replacewithnonprocessable/") {
            engine
                .add_dialect(Arc::new(ReplaceWithNonProcessableDialect::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
        if path_text.contains("/processors/replacewithprocessable/") {
            engine
                .add_dialect(Arc::new(ReplaceWithProcessableDialect::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
        if path_text.contains("/processors/surround/") {
            engine
                .add_dialect(Arc::new(SurroundDialect::new()) as Arc<dyn IDialect>)
                .map_err(|error| error.to_string())?;
        }
        let precedence = if path_text.contains("/elementprocessors/precedencemodelbefore/")
            || path_text.contains("/elementprocessors/precedencetagbefore/")
        {
            Some(999)
        } else if path_text.contains("/elementprocessors/precedencemodelsame/")
            || path_text.contains("/elementprocessors/precedencetagsame/")
        {
            Some(1000)
        } else if path_text.contains("/elementprocessors/precedencemodelafter/")
            || path_text.contains("/elementprocessors/precedencetagafter/")
        {
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
    let context: Box<dyn IContext> = if path_text.contains("/context/base/")
        || path_text.contains("/features/session/")
        || path_text.contains("/features/servletcontext/")
    {
        Box::new(build_web_context(&engine, context_source.as_deref())?)
    } else if no_standard_dialect {
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
        Box::new(build_context(&context_engine, context_source.as_deref())?)
    } else {
        Box::new(build_context(&engine, context_source.as_deref())?)
    };
    let rendered = if let Some(fragment) = test_data.fragment_spec {
        let selectors: TemplateSelectorSet = [Some(fragment)].into_iter().collect();
        engine.process_template_with_selectors(
            &test_data.root_template_name,
            &selectors,
            context.as_ref(),
        )
    } else {
        engine.process_template(&test_data.root_template_name, context.as_ref())
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

fn build_web_context(engine: &TemplateEngine, source: Option<&str>) -> Result<WebContext, String> {
    let default_locale = JavaLocale::new(
        JavaString::from_rust_str("en"),
        JavaString::from_rust_str(""),
    );
    let exchange = Arc::new(CorpusWebExchange::new());
    let web_exchange: Arc<dyn IWebExchange> = exchange.clone();
    let context = WebContext::with_locale(Some(web_exchange), Some(default_locale.clone()))
        .map_err(|error| error.to_string())?;
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

    for assignment in split_context_assignments(source)? {
        let (target, expression) = split_context_assignment(&assignment)?;
        let expression = decode_java_properties_value(expression)?;
        let expression = VariableExpression::new(Some(JavaString::from_rust_str(&expression)))
            .map_err(|error| format!("CONTEXT `{assignment}`: {error}"))?;
        let value = expression
            .execute(expression_context.as_ref())
            .map_err(|error| format!("CONTEXT `{assignment}`: {error}"))?;
        if let Some(name) = target.strip_prefix("session.") {
            let name = Some(JavaString::from_rust_str(name.trim()));
            exchange
                .get_session()
                .expect("corpus web exchange always has a session")
                .set_attribute_value(name, value);
        } else if let Some(name) = target.strip_prefix("application.") {
            exchange
                .get_application()
                .set_attribute_value(Some(JavaString::from_rust_str(name.trim())), value);
        } else if is_simple_context_name(target) {
            let name = Some(JavaString::from_rust_str(target));
            context.set_variable(name.clone(), value.clone());
            expression_context.set_variable(name, value);
        } else {
            return Err(format!("Unsupported Web CONTEXT mutation target: {target}"));
        }
    }
    Ok(context)
}

/// 仅把顶层输入及“当前模板”重新解析为字符串资源。
///
/// Java `.thtest` 语料的测试解析器不会把任意缺失模板名本身当作模板正文；直接使用
/// `StringTemplateResolver` 会把 `~{fragg}` 错误解析成文本 `fragg`。
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
