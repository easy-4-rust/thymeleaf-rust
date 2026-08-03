//! 固定 Java 测试入口与 Rust 证据的 SOURCE_PARITY 完整性门禁。
//!
//! 本测试不以 Rust 测试函数数量冒充 Java JUnit 数量，而是逐项验证固定上游
//! 413 个源码测试入口、1,154 个运行时 case（仅 core 模块；Spring 集成按对象级对照表范围声明排除） 及对应证据是否仍然可追溯。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const STATIC_INVENTORY: &str =
    include_str!("../../docs/migration/baseline/migration_test_static_inventory.json");
const SOURCE_PARITY_INVENTORY: &str =
    include_str!("../../docs/migration/baseline/source_parity_inventory.json");
const CORE_MODULE: &str = "tests/thymeleaf-tests-core";
const BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

#[test]
fn every_java_source_test_and_runtime_case_has_live_rust_disposition() {
    let static_inventory: Value =
        serde_json::from_str(STATIC_INVENTORY).expect("static inventory must be valid JSON");
    let parity_inventory: Value =
        serde_json::from_str(SOURCE_PARITY_INVENTORY).expect("parity inventory must be valid JSON");
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("thymeleaf-test must sit directly under the workspace root")
        .to_path_buf();

    assert_eq!(parity_inventory["schema_version"], 1);
    assert_eq!(parity_inventory["upstream"]["baseline"], BASELINE);
    assert_eq!(parity_inventory["summary"]["source_methods"], 413);  // 移除 spring 后
    assert_eq!(parity_inventory["summary"]["core_source_methods"], 413);  // core unchanged
    assert_eq!(
        parity_inventory["summary"]["integration_source_methods"],
        0
    );  // Spring 集成语义不入清单
    assert_eq!(parity_inventory["summary"]["core_runtime_cases"], 1_154);
    assert_eq!(
        parity_inventory["summary"]["integration_runtime_cases"],
        0
    );  // Spring 集成无 runtime case
    assert_eq!(parity_inventory["summary"]["runtime_cases"], 1_154);  // 移除 spring 后仅 core
    assert_eq!(parity_inventory["summary"]["missing"], 0);

    let static_tests = static_inventory["java_tests"]
        .as_array()
        .expect("static java_tests must be an array");
    let parity_entries = parity_inventory["entries"]
        .as_array()
        .expect("parity entries must be an array");
    assert_eq!(static_tests.len(), 413);  // 移除 spring 后
    assert_eq!(parity_entries.len(), static_tests.len());

    let expected = static_tests
        .iter()
        .map(|test| {
            (
                test["file"].as_str().expect("static file").to_owned(),
                test["name"].as_str().expect("static name").to_owned(),
            )
        })
        .collect::<HashSet<_>>();
    let actual = parity_entries
        .iter()
        .map(|entry| {
            (
                entry["file"].as_str().expect("parity file").to_owned(),
                entry["method"].as_str().expect("parity method").to_owned(),
            )
        })
        .collect::<HashSet<_>>();
    assert_eq!(
        actual, expected,
        "SOURCE_PARITY entries drifted from source inventory"
    );

    let mut ids = HashSet::new();
    let mut dispositions = HashMap::<&str, usize>::new();
    let mut runtime_cases_seen = HashSet::new();
    let mut core_runtime_case_count = 0_usize;
    for entry in parity_entries {
        let id = entry["id"].as_str().expect("entry id");
        assert!(ids.insert(id), "duplicate SOURCE_PARITY id: {id}");
        let disposition = entry["disposition"].as_str().expect("entry disposition");
        assert_ne!(disposition, "MISSING", "unclosed SOURCE_PARITY entry: {id}");
        assert!(
            matches!(
                disposition,
                "MAPPED" | "MERGED" | "SPLIT" | "NOT_APPLICABLE" | "POLICY_DIFFERENCE"
            ),
            "unsupported SOURCE_PARITY disposition {disposition}: {id}"
        );
        *dispositions.entry(disposition).or_default() += 1;

        let rationale = entry["rationale"].as_str().expect("entry rationale");
        assert!(
            !rationale.trim().is_empty(),
            "SOURCE_PARITY rationale is empty: {id}"
        );
        let evidence = entry["evidence"].as_array().expect("entry evidence");
        assert!(
            !evidence.is_empty(),
            "SOURCE_PARITY evidence is empty: {id}"
        );
        for item in evidence {
            validate_live_evidence(&project_root, item, id);
        }

        let runtime_cases = entry["runtime_cases"]
            .as_array()
            .expect("runtime_cases must be an array");
        assert!(
            !runtime_cases.is_empty(),
            "source test lacks expanded runtime case: {id}"
        );
        for runtime_case in runtime_cases {
            let runtime_case = runtime_case.as_str().expect("runtime case name");
            assert!(
                runtime_cases_seen.insert(format!(
                    "{}:{}#{runtime_case}",
                    entry["module"].as_str().expect("entry module"),
                    entry["class"].as_str().expect("entry class")
                )),
                "duplicate runtime case for {id}: {runtime_case}"
            );
            if entry["module"] == CORE_MODULE {
                core_runtime_case_count += 1;
            }
        }
    }

    assert_eq!(core_runtime_case_count, 1_154);  // unchanged
    assert_eq!(
        runtime_cases_seen.len() as u64,
        parity_inventory["summary"]["runtime_cases"]
            .as_u64()
            .expect("runtime_cases summary")
    );
    assert_eq!(dispositions.values().sum::<usize>(), 413);  // 移除 spring 后

    if let Ok(upstream) = std::env::var("THYMELEAF_UPSTREAM") {
        let upstream = Path::new(&upstream);
        for entry in parity_entries {
            let relative = entry["file"].as_str().expect("entry file");
            assert!(
                upstream.join(relative).is_file(),
                "fixed upstream source test disappeared: {relative}"
            );
        }
    }
}

fn validate_live_evidence(project_root: &Path, evidence: &Value, id: &str) {
    let relative = evidence["path"].as_str().expect("evidence path");
    let marker = evidence["marker"].as_str().expect("evidence marker");
    let path = project_root.join(relative);
    assert!(
        path.exists(),
        "SOURCE_PARITY evidence path is absent for {id}: {relative}"
    );
    assert!(
        path_contains_marker(&path, marker),
        "SOURCE_PARITY evidence marker is absent for {id}: {relative}::{marker}"
    );
}

fn path_contains_marker(path: &Path, marker: &str) -> bool {
    if path.is_file() {
        return fs::read_to_string(path).is_ok_and(|content| content.contains(marker));
    }
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    entries.filter_map(Result::ok).any(|entry| {
        let child = entry.path();
        child
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains(marker))
            || path_contains_marker(&child, marker)
    })
}
