//! 固定上游 `.thtest` 清单的结构与分母门禁。

use std::collections::HashSet;

use serde_json::Value;

const INVENTORY: &str = include_str!("../docs/migration/baseline/thtest_inventory.json");

#[test]
fn upstream_thtest_inventory_is_complete_unique_and_structurally_valid() {
    let inventory: Value = serde_json::from_str(INVENTORY).expect("inventory must be valid JSON");
    let summary = inventory
        .get("summary")
        .and_then(Value::as_object)
        .expect("inventory summary must be an object");
    assert_eq!(summary["thtest_files"], 2_609);
    assert_eq!(summary["executable_tests"], 2_608);
    assert_eq!(summary["support_thtest_files"], 1);
    assert_eq!(summary["referenced_common_files"], 20);
    assert_eq!(summary["violations"], 0);

    let tests = inventory["tests"]
        .as_array()
        .expect("inventory tests must be an array");
    assert_eq!(tests.len(), 2_609);

    let mut paths = HashSet::with_capacity(tests.len());
    let mut executable = 0_usize;
    let mut support = 0_usize;
    for test in tests {
        let path = test["path"].as_str().expect("test path must be a string");
        assert!(paths.insert(path), "duplicate thtest path: {path}");
        assert!(path.ends_with(".thtest"), "invalid thtest path: {path}");

        let digest = test["sha256"]
            .as_str()
            .expect("test digest must be a string");
        assert_eq!(digest.len(), 64, "invalid SHA-256 digest for {path}");
        assert!(
            digest.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "non-hex SHA-256 digest for {path}"
        );

        match test["kind"].as_str().expect("test kind must be a string") {
            "EXECUTABLE" => {
                executable += 1;
                assert_eq!(test["verification_status"], "PENDING");
            }
            "SUPPORT" => {
                support += 1;
                assert_eq!(test["verification_status"], "NOT_APPLICABLE");
            }
            kind => panic!("unknown thtest kind {kind} for {path}"),
        }
    }

    assert_eq!(executable, 2_608);
    assert_eq!(support, 1);
}
