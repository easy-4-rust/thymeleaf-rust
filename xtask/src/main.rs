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
    status: String,
}

#[derive(Debug, Serialize)]
struct MigrationReport {
    baseline: BaselineReport,
    objects: ObjectReport,
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
struct ResultReport {
    exact: usize,
    equivalent: usize,
    behavior_verified: usize,
    missing: usize,
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

    let rust_files = collect_rust_files(&project_root.join("src"))?;
    let red_lines = scan_red_lines(project_root, &rust_files, &mut violations)?;
    let verified = rows
        .iter()
        .filter(|row| row.status == "BEHAVIOR_VERIFIED")
        .count();
    validate_verified_objects(project_root, &inventory, &rows, &mut violations)?;

    let exact = rows
        .iter()
        .filter(|row| row.status == "BEHAVIOR_VERIFIED" && row.target_file.starts_with("src/"))
        .count();
    let equivalent = rows
        .iter()
        .filter(|row| row.status == "JAVA_ONLY_EXEMPT")
        .count();
    let missing = inventory.summary.primary_objects - verified - equivalent;
    let expected_files = rows
        .iter()
        .filter(|row| row.target_file.starts_with("src/"))
        .map(|row| row.target_file.as_str())
        .collect::<BTreeSet<_>>();
    let extra = rust_files
        .iter()
        .filter(|path| {
            let relative = relative_path(project_root, path);
            relative != "src/lib.rs"
                && !relative.ends_with("/mod.rs")
                && !expected_files.contains(relative.as_str())
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
        result: ResultReport {
            exact,
            equivalent,
            behavior_verified: verified,
            missing,
            extra,
            path_collisions,
            stubs: red_lines,
        },
        violations,
    })
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

fn validate_verified_objects(
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

    for row in rows.iter().filter(|row| row.status == "BEHAVIOR_VERIFIED") {
        if !row.target_file.starts_with("src/") {
            violations.push(format!(
                "verified object {} has non-core target {}",
                row.java_name, row.target_file
            ));
            continue;
        }
        let path = project_root.join(&row.target_file);
        if !path.is_file() {
            violations.push(format!(
                "verified object {} is missing {}",
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
        "objects: primary={}, nested={}, verified={}, missing={}",
        report.objects.java_primary,
        report.objects.java_nested,
        report.result.behavior_verified,
        report.result.missing
    );
    println!(
        "result: exact={}, equivalent={}, extra={}, collisions={}, stubs={}",
        report.result.exact,
        report.result.equivalent,
        report.result.extra,
        report.result.path_collisions,
        report.result.stubs
    );
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
    use super::{contains_type_declaration, parse_cli, parse_object_rows, path_collision_count};

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
            "| 2 | `Beta` | class | `B.java` | `src/alpha.rs` | `Beta` | — | 1:1 | NOT_STARTED |\n"
        );
        let rows = parse_object_rows(markdown).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].java_name, "Alpha");
        assert_eq!(path_collision_count(&rows), 1);
        assert!(parse_object_rows("not a table").is_err());
    }

    #[test]
    fn recognizes_supported_type_declarations() {
        assert!(contains_type_declaration("pub struct Alpha {}", "Alpha"));
        assert!(contains_type_declaration("pub enum Alpha {}", "Alpha"));
        assert!(contains_type_declaration("pub trait Alpha {}", "Alpha"));
        assert!(!contains_type_declaration("pub fn alpha() {}", "Alpha"));
    }
}
