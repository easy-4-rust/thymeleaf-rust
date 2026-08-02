//! Thymeleaf 3.1.5 正式制品版本元数据的 Java/Rust Golden 差分测试。

use thymeleaf::Thymeleaf;

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/thymeleaf_version_golden.txt");

#[test]
fn thymeleaf_version_metadata_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit(&mut output, "version", Thymeleaf::get_version());
    emit(
        &mut output,
        "build_timestamp",
        Thymeleaf::get_build_timestamp().unwrap_or("null"),
    );
    emit(
        &mut output,
        "major",
        &Thymeleaf::get_version_major().to_string(),
    );
    emit(
        &mut output,
        "minor",
        &Thymeleaf::get_version_minor().to_string(),
    );
    emit(
        &mut output,
        "patch",
        &Thymeleaf::get_version_patch().to_string(),
    );
    emit(
        &mut output,
        "qualifier",
        Thymeleaf::get_version_qualifier().unwrap_or("null"),
    );
    emit(
        &mut output,
        "stable",
        &Thymeleaf::is_version_stable_release().to_string(),
    );
    assert_eq!(output, JAVA_GOLDEN);
}

fn emit(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('=');
    output.push_str(value);
    output.push('\n');
}
