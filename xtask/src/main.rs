use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

const INVENTORY_PATH: &str = "docs/migration/baseline/java_api_inventory.json";
const OBJECT_TABLE_PATH: &str = "docs/migration/对象级对照表.md";
const TODO_MACRO: &str = concat!("todo", "!(");
const UNIMPLEMENTED_MACRO: &str = concat!("unimplemented", "!(");
const APPROVED_RUST_EXTENSION_FILES: [&str; 2] = [
    "thymeleaf/src/processor/processor_set.rs",
    "thymeleaf/src/templateresource/template_resource_reader.rs",
];

#[derive(Debug, Deserialize)]
struct Inventory {
    java: JavaBaseline,
    objects: Vec<InventoryObject>,
    summary: InventorySummary,
}

#[derive(Debug, Deserialize)]
struct JavaBaseline {
    baseline: String,
}

#[derive(Debug, Deserialize)]
struct InventoryObject {
    name: String,
    qualified_name: String,
    nested_types: Vec<String>,
    methods: Vec<InventoryMethod>,
}

#[derive(Debug, Deserialize)]
struct InventoryMethod {
    name: String,
    qualified_name: String,
    signature: String,
    visibility: String,
}

#[derive(Debug, Deserialize)]
struct InventorySummary {
    primary_objects: usize,
    nested_types: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ObjectRow {
    number: usize,
    java_name: String,
    target_file: String,
    rust_object: String,
    mapping: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct MigrationReport {
    baseline: BaselineReport,
    objects: ObjectReport,
    methods: MethodReport,
    result: ResultReport,
    violations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BaselineReport {
    version: &'static str,
    commit: String,
}

#[derive(Debug, Serialize)]
struct ObjectReport {
    java_primary: usize,
    java_nested: usize,
    exact_expected: usize,
    equivalent_expected: usize,
}

#[derive(Debug, Serialize)]
struct MethodReport {
    java_declared: usize,
    explicit_rust_name: usize,
    dynamic_dispatch: usize,
    rust_idiom_or_constructor: usize,
    trait_or_flow_merged: usize,
    private_merged: usize,
    review_required: usize,
    review_required_methods: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ResultReport {
    /// 工作树中名称直接对齐且类型声明存在的对象数。
    exact: usize,
    /// 工作树中批准等价映射且类型声明存在的对象数。
    equivalent: usize,
    /// 冻结文档声明为已验证的对象数。
    behavior_verified: usize,
    /// 冻结文档声明为已实现待验证的对象数。
    implemented_unverified: usize,
    /// 工作树中存在真实对象、但冻结文档尚未登记为已实现的对象数。
    present_unverified: usize,
    /// 工作树中尚无目标对象文件的对象数。
    missing: usize,
    /// 工作树中尚无目标对象文件的 Java 对象名。
    missing_objects: Vec<String>,
    /// 目标文件存在但缺少预期类型声明的对象数。
    type_mismatches: usize,
    /// 目标文件存在但缺少预期类型声明的 Java 对象名。
    type_mismatch_objects: Vec<String>,
    extra: usize,
    path_collisions: usize,
    stubs: usize,
}

#[derive(Debug)]
struct CliError(String);

impl Display for CliError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for CliError {}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = parse_cli(env::args().skip(1))?;
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| CliError("xtask must live below the project root".to_owned()))?
        .to_owned();
    let report = migration_check(&project_root, &cli.upstream, &cli.baseline)?;
    print_human_report(&report);

    let json = serde_json::to_string_pretty(&report)?;
    if let Some(json_path) = cli.json {
        let json_path = if json_path.is_absolute() {
            json_path
        } else {
            project_root.join(json_path)
        };
        if let Some(parent) = json_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&json_path, format!("{json}\n"))?;
        println!("JSON report: {}", json_path.display());
    } else {
        println!("{json}");
    }

    if report.violations.is_empty() {
        Ok(())
    } else {
        Err(Box::new(CliError(format!(
            "migration check failed with {} violation(s)",
            report.violations.len()
        ))))
    }
}

struct Cli {
    upstream: PathBuf,
    baseline: String,
    json: Option<PathBuf>,
}

fn parse_cli(arguments: impl Iterator<Item = String>) -> Result<Cli, CliError> {
    let mut arguments = arguments;
    if arguments.next().as_deref() != Some("migration-check") {
        return Err(CliError(
            "usage: cargo xtask migration-check --upstream PATH --baseline SHA [--json PATH]"
                .to_owned(),
        ));
    }

    let mut upstream = None;
    let mut baseline = None;
    let mut json = None;
    while let Some(argument) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| CliError(format!("missing value for {argument}")))?;
        match argument.as_str() {
            "--upstream" => upstream = Some(PathBuf::from(value)),
            "--baseline" => baseline = Some(value),
            "--json" => json = Some(PathBuf::from(value)),
            _ => return Err(CliError(format!("unknown argument: {argument}"))),
        }
    }

    Ok(Cli {
        upstream: upstream.ok_or_else(|| CliError("--upstream is required".to_owned()))?,
        baseline: baseline.ok_or_else(|| CliError("--baseline is required".to_owned()))?,
        json,
    })
}

fn migration_check(
    project_root: &Path,
    upstream: &Path,
    baseline: &str,
) -> Result<MigrationReport, Box<dyn Error>> {
    let inventory: Inventory =
        serde_json::from_str(&fs::read_to_string(project_root.join(INVENTORY_PATH))?)?;
    let rows = parse_object_rows(&fs::read_to_string(project_root.join(OBJECT_TABLE_PATH))?)?;
    let mut violations = Vec::new();

    validate_baseline(upstream, baseline, &inventory, &mut violations)?;
    validate_manifest(&inventory, &rows, &mut violations);

    let rust_files = collect_rust_files(&project_root.join("thymeleaf").join("src"))?;
    let red_lines = scan_red_lines(project_root, &rust_files, &mut violations)?;
    let live = inspect_live_objects(project_root, &inventory, &rows, &mut violations)?;
    let methods = inspect_live_methods(project_root, &inventory, &rows)?;
    let verified = rows
        .iter()
        .filter(|row| row.status == "BEHAVIOR_VERIFIED")
        .count();
    let implemented_unverified = rows
        .iter()
        .filter(|row| row.status == "IMPLEMENTED_UNVERIFIED")
        .count();
    validate_implemented_objects(project_root, &inventory, &rows, &mut violations)?;

    let expected_files = rows
        .iter()
        .filter(|row| row.target_file.starts_with("thymeleaf/src/"))
        .map(|row| row.target_file.as_str())
        .collect::<BTreeSet<_>>();
    let extra = rust_files
        .iter()
        .filter(|path| {
            let relative = relative_path(project_root, path);
            relative != "thymeleaf/src/lib.rs"
                && !relative.ends_with("/mod.rs")
                && !expected_files.contains(relative.as_str())
                && !is_approved_rust_extension_file(&relative)
        })
        .count();
    let path_collisions = path_collision_count(&rows);

    Ok(MigrationReport {
        baseline: BaselineReport {
            version: "3.1.5.RELEASE",
            commit: baseline.to_owned(),
        },
        objects: ObjectReport {
            java_primary: inventory.summary.primary_objects,
            java_nested: inventory.summary.nested_types,
            exact_expected: 540,
            equivalent_expected: 20,
        },
        methods,
        result: ResultReport {
            exact: live.exact,
            equivalent: live.equivalent,
            behavior_verified: verified,
            implemented_unverified,
            present_unverified: live
                .present
                .saturating_sub(verified + implemented_unverified),
            missing: live.missing,
            missing_objects: live.missing_objects,
            type_mismatches: live.type_mismatches,
            type_mismatch_objects: live.type_mismatch_objects,
            extra,
            path_collisions,
            stubs: red_lines,
        },
        violations,
    })
}

fn inspect_live_methods(
    project_root: &Path,
    inventory: &Inventory,
    rows: &[ObjectRow],
) -> Result<MethodReport, Box<dyn Error>> {
    let rows_by_name = rows
        .iter()
        .map(|row| (row.java_name.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut report = MethodReport {
        java_declared: 0,
        explicit_rust_name: 0,
        dynamic_dispatch: 0,
        rust_idiom_or_constructor: 0,
        trait_or_flow_merged: 0,
        private_merged: 0,
        review_required: 0,
        review_required_methods: Vec::new(),
    };

    for object in &inventory.objects {
        let Some(row) = rows_by_name.get(object.name.as_str()) else {
            continue;
        };
        let source = live_source_for_row(project_root, row)?.unwrap_or_default();
        let sibling_source = live_sibling_source_for_row(project_root, row)?;
        for method in &object.methods {
            report.java_declared += 1;
            if is_constructor(method)
                || method.qualified_name.contains("$anon@")
                || is_rust_idiom_method(&method.name)
                || !row.target_file.starts_with("thymeleaf/src/")
            {
                report.rust_idiom_or_constructor += 1;
            } else if contains_rust_method(&source, &method.name) {
                report.explicit_rust_name += 1;
            } else if contains_dynamic_method(&source, &method.name) {
                report.dynamic_dispatch += 1;
            } else if contains_rust_method(&sibling_source, &method.name)
                || is_trait_or_flow_merged(method, &source, &sibling_source)
            {
                report.trait_or_flow_merged += 1;
            } else if method.visibility == "private" {
                report.private_merged += 1;
            } else {
                report.review_required += 1;
                report.review_required_methods.push(format!(
                    "{} {}",
                    method.qualified_name,
                    method.signature.replace('\n', " ")
                ));
            }
        }
    }
    Ok(report)
}

fn live_sibling_source_for_row(
    project_root: &Path,
    row: &ObjectRow,
) -> Result<String, Box<dyn Error>> {
    if !row.target_file.starts_with("thymeleaf/src/") {
        return Ok(String::new());
    }
    let Some(parent) = project_root
        .join(&row.target_file)
        .parent()
        .map(Path::to_owned)
    else {
        return Ok(String::new());
    };
    let mut source = String::new();
    for entry in fs::read_dir(parent)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            source.push_str(&fs::read_to_string(path)?);
            source.push('\n');
        }
    }
    Ok(source)
}

fn is_trait_or_flow_merged(method: &InventoryMethod, source: &str, sibling_source: &str) -> bool {
    if method.name.starts_with("execute")
        && (source.contains("fn execute_with_context(")
            || source.contains("fn execute_raw(")
            || (matches!(method.name.as_str(), "executeComplex" | "executeSimple")
                && sibling_source.contains("fn execute_with_context(")))
    {
        return true;
    }
    if method.name.starts_with("compose")
        && method
            .qualified_name
            .contains("org.thymeleaf.standard.expression")
        && sibling_source.contains("fn parse_range(")
    {
        return true;
    }
    if method.name.starts_with("parse")
        && method
            .qualified_name
            .contains("org.thymeleaf.standard.expression")
        && sibling_source.contains("fn parse_range(")
    {
        return true;
    }
    if method.name.starts_with("internalParse") && sibling_source.contains("ExpressionParsingUtil")
    {
        return true;
    }
    if method.name.starts_with("doProcess")
        && (source.contains("move |")
            || source.contains("do_process:")
            || source.contains("fn process("))
    {
        return true;
    }
    if method.name.starts_with("handle")
        && (source.contains("pub fn parse(")
            || source.contains("pub(crate) fn element_start(")
            || source.contains("process_injected_attributes"))
    {
        return true;
    }
    if method.name.starts_with("asEngine")
        && (source.contains("IEngineTemplateEvent")
            || source.contains("into_engine_")
            || source.contains("fn be_handled("))
    {
        return true;
    }
    if method.name.starts_with("startGathering") && source.contains("fn start_gathering_") {
        return true;
    }
    if method.name == "performTearDownChecks" && sibling_source.contains("perform_teardown_checks")
    {
        return true;
    }
    if method.name.starts_with("doProcessTemplate") && source.contains("fn process_template_") {
        return true;
    }
    if matches!(
        method.name.as_str(),
        "computeValue"
            | "getListProperty"
            | "getArrayProperty"
            | "getEnumerationProperty"
            | "getIteratorProperty"
    ) && (source.contains("JavaBigDecimal::parse") || source.contains("fn read_property("))
    {
        return true;
    }
    if matches!(
        method.name.as_str(),
        "getSourceAccessor" | "getSourceSetter"
    ) && source.contains("NativeContextPropertyAccessor")
    {
        // OGNL 的 JVM 源码生成器在 Rust 中由直接属性访问替代，不生成 Java 源码。
        return true;
    }
    if matches!(
        method.name.as_str(),
        "validateSelectionValue" | "computeAdditionalLocalVariables"
    ) && source.contains("validate_selection_value")
        && source.contains("compute_additional_local_variables")
    {
        return true;
    }
    if method.name == "setParseSelection" && source.contains("selectors:") {
        return true;
    }
    if method.name == "computeTemplateResource"
        && (source.contains("fn resolve_template(")
            || sibling_source.contains("fn resolve_template("))
    {
        return true;
    }
    if method.name == "getClassLoader" && sibling_source.contains("ResourceLoaderUtils") {
        return true;
    }
    if method.name == "parseAndCompose" && source.contains("fn parse_expression(") {
        return true;
    }
    if matches!(
        method.name.as_str(),
        "decompose" | "unnest" | "parseAsSimpleIndexPlaceholder"
    ) && source.contains("fn parse_range(")
    {
        return true;
    }
    matches!(
        method.name.as_str(),
        "writeUnresolved" | "asEngineEvent" | "beHandled"
    ) && (source.contains("IWritableCharSequence")
        || source.contains("IEngineTemplateEvent")
        || sibling_source.contains("be_handled"))
}

fn live_source_for_row(
    project_root: &Path,
    row: &ObjectRow,
) -> Result<Option<String>, Box<dyn Error>> {
    if row.target_file.starts_with("thymeleaf/src/") {
        let path = project_root.join(&row.target_file);
        return path
            .is_file()
            .then(|| fs::read_to_string(path))
            .transpose()
            .map_err(Into::into);
    }
    let Some((file_name, _)) = host_integration_target(&row.java_name) else {
        return Ok(None);
    };
    let path = project_root
        .join("integrations/thymeleaf-hyper/src")
        .join(file_name);
    path.is_file()
        .then(|| fs::read_to_string(path))
        .transpose()
        .map_err(Into::into)
}

fn contains_rust_method(source: &str, java_name: &str) -> bool {
    rust_method_candidates(java_name)
        .into_iter()
        .any(|candidate| {
            source.contains(&format!("fn {candidate}("))
                || source.contains(&format!("fn {candidate}<"))
                || source.contains(&format!("fn {candidate}\n"))
                || source.contains(&format!("fn {candidate}_"))
                || source.contains(&format!("fn java_{candidate}("))
                || source.contains(&format!("{candidate}: F"))
                || source.contains(&format!("{candidate}: Arc<dyn Fn"))
        })
}

fn contains_dynamic_method(source: &str, java_name: &str) -> bool {
    if source.contains(&format!("\"{java_name}\"")) {
        return true;
    }
    for prefix in ["array", "list", "set"] {
        if let Some(scalar_name) = java_name.strip_prefix(prefix)
            && !scalar_name.is_empty()
        {
            let mut characters = scalar_name.chars();
            let scalar_name = characters
                .next()
                .map(char::to_lowercase)
                .into_iter()
                .flatten()
                .chain(characters)
                .collect::<String>();
            if source.contains("collection_method")
                && (source.contains(&format!("\"{scalar_name}\""))
                    || contains_rust_method(source, &scalar_name))
            {
                return true;
            }
        }
    }
    false
}

fn rust_method_candidates(java_name: &str) -> Vec<String> {
    let snake = to_snake_case(java_name);
    let compact_javascript = snake.replace("java_script", "javascript");
    let mut candidates = vec![snake];
    if !candidates.contains(&compact_javascript) {
        candidates.push(compact_javascript);
    }
    let aliases = match java_name {
        "length" => &["len", "java_length"][..],
        "charAt" => &["char_at", "java_char_at"][..],
        "subSequence" => &["sub_sequence", "java_sub_sequence"][..],
        "getHandlerClass" => &["get_handler_factory"][..],
        "processAll" => &["process_all_to_writer", "process_all_to_stream"][..],
        "setOutput" => &["set_output_writer", "set_output_stream"][..],
        _ => &[][..],
    };
    for alias in aliases {
        if !candidates.iter().any(|candidate| candidate == alias) {
            candidates.push((*alias).to_owned());
        }
    }
    // JavaBean getter 在 Rust 中通常直接使用字段语义名；这仍然是一一对应的公开
    // 行为，而不是缺失方法。例如 getMimeType -> mime_type。
    for prefix in ["get", "is"] {
        if let Some(property_name) = java_name.strip_prefix(prefix)
            && property_name
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_uppercase())
        {
            let property_name = to_snake_case(property_name);
            if !candidates.contains(&property_name) {
                candidates.push(property_name);
            }
        }
    }
    // AttoParser 的 handleX 回调在 Rust parser adapter 中去掉 handle 前缀。
    if let Some(event_name) = java_name.strip_prefix("handle")
        && event_name
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_uppercase())
    {
        let event_name = to_snake_case(event_name);
        if !candidates.contains(&event_name) {
            candidates.push(event_name.clone());
        }
        if event_name == "cdata_section" && !candidates.iter().any(|name| name == "cdata") {
            candidates.push("cdata".to_owned());
        }
    }
    candidates
}

fn is_constructor(method: &InventoryMethod) -> bool {
    let mut segments = method.qualified_name.rsplit("::");
    matches!(
        (segments.next(), segments.next()),
        (Some(method_name), Some(owner_name)) if method_name == owner_name
    )
}

fn to_snake_case(value: &str) -> String {
    let characters = value.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(value.len() + 8);
    for (index, character) in characters.iter().copied().enumerate() {
        if character.is_ascii_uppercase() {
            let previous_is_lower = index > 0 && characters[index - 1].is_ascii_lowercase();
            let next_is_lower = characters
                .get(index + 1)
                .is_some_and(char::is_ascii_lowercase);
            if index > 0 && (previous_is_lower || next_is_lower) {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
        } else {
            output.push(character);
        }
    }
    output
}

fn is_rust_idiom_method(method_name: &str) -> bool {
    matches!(
        method_name,
        "toString"
            | "hashCode"
            | "equals"
            | "compareTo"
            | "clone"
            | "iterator"
            | "spliterator"
            | "isEmpty"
            | "containsKey"
            | "containsValue"
            | "keySet"
            | "values"
            | "entrySet"
            | "put"
            | "putAll"
            | "clear"
            | "toArray"
    )
}

#[derive(Default)]
struct LiveObjectReport {
    exact: usize,
    equivalent: usize,
    present: usize,
    missing: usize,
    type_mismatches: usize,
    missing_objects: Vec<String>,
    type_mismatch_objects: Vec<String>,
}

fn inspect_live_objects(
    project_root: &Path,
    inventory: &Inventory,
    rows: &[ObjectRow],
    violations: &mut Vec<String>,
) -> Result<LiveObjectReport, Box<dyn Error>> {
    let qualified_names = inventory
        .objects
        .iter()
        .map(|object| {
            (
                object.name.as_str(),
                (
                    object.qualified_name.replace("::", "."),
                    object.nested_types.as_slice(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut report = LiveObjectReport::default();

    for row in rows {
        if !row.target_file.starts_with("thymeleaf/src/") {
            if inspect_host_integration_object(project_root, row)? {
                report.present += 1;
                report.equivalent += 1;
            } else {
                report.missing += 1;
                report.missing_objects.push(row.java_name.clone());
            }
            continue;
        }
        let path = project_root.join(&row.target_file);
        if !path.is_file() {
            report.missing += 1;
            report.missing_objects.push(row.java_name.clone());
            continue;
        }
        let source = fs::read_to_string(&path)?;
        if !contains_type_declaration(&source, &row.rust_object) {
            report.type_mismatches += 1;
            report.type_mismatch_objects.push(row.java_name.clone());
            violations.push(format!(
                "live object {} at {} does not define expected Rust object {}",
                row.java_name, row.target_file, row.rust_object
            ));
            continue;
        }

        report.present += 1;
        if row.mapping.contains('🔶') {
            report.equivalent += 1;
        } else {
            report.exact += 1;
        }
        if let Some((qualified_name, nested_types)) = qualified_names.get(row.java_name.as_str()) {
            if !source.contains(qualified_name) {
                violations.push(format!(
                    "live object {} does not document Java source {}",
                    row.target_file, qualified_name
                ));
            }
            for nested_type in *nested_types {
                // Java 编译器生成的匿名 Iterator 没有稳定源对象名；Rust 由
                // TemplateValue→VecDeque 的穷举归一化路径承接，不伪造命名类型。
                if is_java_anonymous_type(nested_type) {
                    continue;
                }
                if !source.contains(nested_type) {
                    violations.push(format!(
                        "live object {} is missing nested type {}",
                        row.target_file, nested_type
                    ));
                }
            }
        }
    }
    Ok(report)
}

fn inspect_host_integration_object(
    project_root: &Path,
    row: &ObjectRow,
) -> Result<bool, Box<dyn Error>> {
    let Some((file_name, rust_object)) = host_integration_target(&row.java_name) else {
        return Ok(false);
    };
    let path = project_root
        .join("integrations/thymeleaf-hyper/src")
        .join(file_name);
    if !path.is_file() {
        return Ok(false);
    }
    let source = fs::read_to_string(path)?;
    Ok(contains_type_declaration(&source, rust_object) && source.contains(&row.java_name))
}

fn host_integration_target(java_name: &str) -> Option<(&'static str, &'static str)> {
    if java_name.ends_with("WebApplication") {
        Some(("host_web_application.rs", "HostWebApplication"))
    } else if java_name.ends_with("WebExchange") {
        Some(("host_web_exchange.rs", "HostWebExchange"))
    } else if java_name.ends_with("WebRequest") {
        Some(("host_web_request.rs", "HostWebRequest"))
    } else if java_name.ends_with("WebSession") {
        Some(("host_web_session.rs", "HostWebSession"))
    } else {
        None
    }
}

fn validate_baseline(
    upstream: &Path,
    baseline: &str,
    inventory: &Inventory,
    violations: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    if inventory.java.baseline != baseline {
        violations.push(format!(
            "inventory baseline {} does not match requested {baseline}",
            inventory.java.baseline
        ));
    }
    let output = Command::new("git")
        .args(["-C"])
        .arg(upstream)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !output.status.success() {
        return Err(Box::new(CliError(format!(
            "cannot read upstream git HEAD at {}",
            upstream.display()
        ))));
    }
    let actual = String::from_utf8(output.stdout)?.trim().to_owned();
    if actual != baseline {
        violations.push(format!("upstream HEAD {actual} does not match {baseline}"));
    }
    if inventory.summary.primary_objects != 491 || inventory.summary.nested_types != 69 {
        violations.push(format!(
            "inventory totals changed: primary={}, nested={}",
            inventory.summary.primary_objects, inventory.summary.nested_types
        ));
    }
    Ok(())
}

fn validate_manifest(inventory: &Inventory, rows: &[ObjectRow], violations: &mut Vec<String>) {
    if rows.len() != inventory.objects.len() {
        violations.push(format!(
            "object table has {} rows, inventory has {}",
            rows.len(),
            inventory.objects.len()
        ));
    }

    let inventory_names = inventory
        .objects
        .iter()
        .map(|object| object.name.as_str())
        .collect::<BTreeSet<_>>();
    let row_names = rows
        .iter()
        .map(|row| row.java_name.as_str())
        .collect::<BTreeSet<_>>();
    for missing in inventory_names.difference(&row_names) {
        violations.push(format!("manifest missing Java object {missing}"));
    }
    for extra in row_names.difference(&inventory_names) {
        violations.push(format!("manifest has unknown Java object {extra}"));
    }

    for (expected, row) in (1..=rows.len()).zip(rows) {
        if row.number != expected {
            violations.push(format!(
                "object row number {} appears where {expected} was expected",
                row.number
            ));
        }
    }
}

fn validate_implemented_objects(
    project_root: &Path,
    inventory: &Inventory,
    rows: &[ObjectRow],
    violations: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let qualified_names = inventory
        .objects
        .iter()
        .map(|object| {
            (
                object.name.as_str(),
                (
                    object.qualified_name.replace("::", "."),
                    object.nested_types.as_slice(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();

    for row in rows.iter().filter(|row| is_implemented(row)) {
        if !row.target_file.starts_with("thymeleaf/src/") {
            violations.push(format!(
                "implemented object {} has non-core target {}",
                row.java_name, row.target_file
            ));
            continue;
        }
        let path = project_root.join(&row.target_file);
        if !path.is_file() {
            violations.push(format!(
                "implemented object {} is missing {}",
                row.java_name, row.target_file
            ));
            continue;
        }
        let source = fs::read_to_string(&path)?;
        if !contains_type_declaration(&source, &row.rust_object) {
            violations.push(format!(
                "{} does not define expected Rust object {}",
                row.target_file, row.rust_object
            ));
        }
        if let Some((qualified_name, nested_types)) = qualified_names.get(row.java_name.as_str()) {
            if !source.contains(qualified_name) {
                violations.push(format!(
                    "{} does not document Java source {}",
                    row.target_file, qualified_name
                ));
            }
            for nested_type in *nested_types {
                if is_java_anonymous_type(nested_type) {
                    continue;
                }
                if !source.contains(nested_type) {
                    violations.push(format!(
                        "{} is missing nested type {}",
                        row.target_file, nested_type
                    ));
                }
            }
        }
    }
    Ok(())
}

fn is_java_anonymous_type(nested_type: &str) -> bool {
    nested_type.starts_with('<') && nested_type.ends_with('>') && nested_type.contains("$anon@")
}

fn is_implemented(row: &ObjectRow) -> bool {
    matches!(
        row.status.as_str(),
        "BEHAVIOR_VERIFIED" | "IMPLEMENTED_UNVERIFIED"
    )
}

fn scan_red_lines(
    project_root: &Path,
    rust_files: &[PathBuf],
    violations: &mut Vec<String>,
) -> Result<usize, Box<dyn Error>> {
    let mut count = 0;
    for path in rust_files {
        let relative = relative_path(project_root, path);
        let source = fs::read_to_string(path)?;
        if path.file_name().and_then(|name| name.to_str()) == Some("compat.rs") {
            violations.push(format!("forbidden compatibility bucket: {relative}"));
            count += 1;
        }
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            let forbidden = trimmed.contains(TODO_MACRO)
                || trimmed.contains(UNIMPLEMENTED_MACRO)
                || (trimmed.starts_with("use ") && trimmed.contains("::*"));
            if forbidden {
                violations.push(format!("forbidden source at {relative}:{}", index + 1));
                count += 1;
            }
            if (relative == "src/lib.rs" || relative.ends_with("/mod.rs"))
                && ["struct ", "enum ", "trait "].iter().any(|keyword| {
                    trimmed.starts_with(keyword) || trimmed.starts_with(&format!("pub {keyword}"))
                })
            {
                violations.push(format!(
                    "business type declared in module index at {relative}:{}",
                    index + 1
                ));
                count += 1;
            }
        }
    }
    Ok(count)
}

fn parse_object_rows(markdown: &str) -> Result<Vec<ObjectRow>, CliError> {
    let mut rows = Vec::new();
    for line in markdown.lines() {
        let columns = line.split('|').map(str::trim).collect::<Vec<_>>();
        if columns.len() < 11 {
            continue;
        }
        let Ok(number) = columns[1].parse::<usize>() else {
            continue;
        };
        rows.push(ObjectRow {
            number,
            java_name: strip_code(columns[2]),
            target_file: strip_code(columns[5]),
            rust_object: strip_code(columns[6]),
            mapping: strip_code(columns[8]),
            status: strip_code(columns[9]),
        });
    }
    if rows.is_empty() {
        return Err(CliError(
            "object table contains no parseable rows".to_owned(),
        ));
    }
    Ok(rows)
}

fn strip_code(value: &str) -> String {
    value.trim_matches('`').to_owned()
}

fn contains_type_declaration(source: &str, expected: &str) -> bool {
    ["struct", "enum", "trait"]
        .iter()
        .any(|kind| source.contains(&format!("{kind} {expected}")))
}

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    collect_rust_files_recursive(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_rust_files_recursive(
    root: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files_recursive(&path, files)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn path_collision_count(rows: &[ObjectRow]) -> usize {
    let mut counts = BTreeMap::new();
    for path in rows
        .iter()
        .map(|row| row.target_file.as_str())
        .filter(|path| path.starts_with("src/"))
    {
        *counts.entry(path).or_insert(0_usize) += 1;
    }
    counts.values().map(|count| count.saturating_sub(1)).sum()
}

fn is_approved_rust_extension_file(relative: &str) -> bool {
    APPROVED_RUST_EXTENSION_FILES.contains(&relative)
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn print_human_report(report: &MigrationReport) {
    println!(
        "Thymeleaf migration check\nbaseline: {} ({})",
        report.baseline.version, report.baseline.commit
    );
    println!(
        "objects: primary={}, nested={}, verified={}, implemented_unverified={}, missing={}",
        report.objects.java_primary,
        report.objects.java_nested,
        report.result.behavior_verified,
        report.result.implemented_unverified,
        report.result.missing
    );
    println!(
        "live: exact={}, equivalent={}, present_unverified={}, missing={}, type_mismatches={}",
        report.result.exact,
        report.result.equivalent,
        report.result.present_unverified,
        report.result.missing,
        report.result.type_mismatches
    );
    println!(
        "layout: extra={}, collisions={}, stubs={}",
        report.result.extra, report.result.path_collisions, report.result.stubs
    );
    println!(
        "methods: declared={}, explicit={}, dynamic={}, idiom_or_constructor={}, \
         trait_or_flow_merged={}, private_merged={}, review_required={}",
        report.methods.java_declared,
        report.methods.explicit_rust_name,
        report.methods.dynamic_dispatch,
        report.methods.rust_idiom_or_constructor,
        report.methods.trait_or_flow_merged,
        report.methods.private_merged,
        report.methods.review_required
    );
    if !report.result.missing_objects.is_empty() {
        println!(
            "missing objects: {}",
            report.result.missing_objects.join(", ")
        );
    }
    if !report.result.type_mismatch_objects.is_empty() {
        println!(
            "type mismatches: {}",
            report.result.type_mismatch_objects.join(", ")
        );
    }
    if report.violations.is_empty() {
        println!("status: PASS");
    } else {
        println!("status: FAIL");
        for violation in &report.violations {
            println!("- {violation}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        contains_type_declaration, is_approved_rust_extension_file, is_implemented, parse_cli,
        parse_object_rows, path_collision_count,
    };

    #[test]
    fn parses_cli_and_rejects_invalid_shapes() {
        let cli = parse_cli(
            [
                "migration-check",
                "--upstream",
                "../thymeleaf",
                "--baseline",
                "abc",
                "--json",
                "target/report.json",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(cli.baseline, "abc");
        assert!(cli.upstream.ends_with("thymeleaf"));
        assert!(cli.json.unwrap().ends_with("report.json"));

        assert!(parse_cli(std::iter::empty()).is_err());
        assert!(
            parse_cli(
                ["migration-check", "--upstream"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .is_err()
        );
        assert!(
            parse_cli(
                ["migration-check", "--unknown", "value"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .is_err()
        );
        assert!(
            parse_cli(
                ["migration-check", "--upstream", "path"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .is_err()
        );
    }

    #[test]
    fn parses_object_table_and_counts_collisions() {
        let markdown = concat!(
            "| 1 | `Alpha` | class | `A.java` | `src/alpha.rs` | `Alpha` | — | 1:1 | BEHAVIOR_VERIFIED |\n",
            "| 2 | `Beta` | class | `B.java` | `src/alpha.rs` | `Beta` | — | 1:1 | IMPLEMENTED_UNVERIFIED |\n",
            "| 3 | `Gamma` | class | `C.java` | `src/gamma.rs` | `Gamma` | — | 1:1 | NOT_STARTED |\n"
        );
        let rows = parse_object_rows(markdown).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].java_name, "Alpha");
        assert_eq!(path_collision_count(&rows), 1);
        assert!(is_implemented(&rows[0]));
        assert!(is_implemented(&rows[1]));
        assert!(!is_implemented(&rows[2]));
        assert!(parse_object_rows("not a table").is_err());
    }

    #[test]
    fn recognizes_supported_type_declarations() {
        assert!(contains_type_declaration("pub struct Alpha {}", "Alpha"));
        assert!(contains_type_declaration("pub enum Alpha {}", "Alpha"));
        assert!(contains_type_declaration("pub trait Alpha {}", "Alpha"));
        assert!(!contains_type_declaration("pub fn alpha() {}", "Alpha"));
    }

    #[test]
    fn recognizes_only_explicit_rust_extension_files() {
        assert!(is_approved_rust_extension_file(
            "thymeleaf/src/processor/processor_set.rs"
        ));
        assert!(is_approved_rust_extension_file(
            "thymeleaf/src/templateresource/template_resource_reader.rs"
        ));
        assert!(!is_approved_rust_extension_file(
            "src/processor/unregistered_helper.rs"
        ));
    }
}
