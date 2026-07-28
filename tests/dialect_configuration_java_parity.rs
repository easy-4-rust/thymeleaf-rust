//! 方言基础对象与方言配置的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::sync::Arc;

use thymeleaf::{AbstractDialect, DialectConfiguration, DialectConfigurationError, IDialect};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/dialect_configuration_golden.txt");

struct NullableNameDialect;

impl IDialect for NullableNameDialect {
    fn get_name(&self) -> Option<&str> {
        None
    }
}

#[test]
fn dialect_objects_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    export_abstract_dialect(&mut output);
    export_dialect_configuration(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn export_abstract_dialect(output: &mut String) {
    emit(
        output,
        "abstract.name",
        AbstractDialect::new(Some("Test"))
            .expect("valid name")
            .get_name(),
    );
    emit(
        output,
        "abstract.empty",
        AbstractDialect::new(Some(""))
            .expect("empty name is legal")
            .get_name(),
    );
    emit(
        output,
        "abstract.unicode",
        AbstractDialect::new(Some("标准方言"))
            .expect("unicode name is legal")
            .get_name(),
    );
    match AbstractDialect::new(None) {
        Ok(_) => emit_no_failure(output, "abstract.null"),
        Err(error) => emit_failure(
            output,
            "abstract.null",
            "IllegalArgumentException",
            &error.to_string(),
        ),
    }

    let nullable_name_dialect = NullableNameDialect;
    emit(
        output,
        "interface.null_name",
        nullable_name_dialect.get_name().unwrap_or("null"),
    );
}

fn export_dialect_configuration(output: &mut String) {
    let dialect: Arc<dyn IDialect> =
        Arc::new(AbstractDialect::new(Some("Test")).expect("valid dialect"));

    let defaults =
        DialectConfiguration::new(Some(Arc::clone(&dialect))).expect("valid configuration");
    emit(
        output,
        "default.specified",
        &defaults.is_prefix_specified().to_string(),
    );
    emit(
        output,
        "default.prefix",
        defaults.get_prefix().unwrap_or("null"),
    );
    emit(
        output,
        "default.dialect_identity",
        &std::ptr::eq(defaults.get_dialect(), dialect.as_ref()).to_string(),
    );
    emit(
        output,
        "default.dialect_name",
        defaults.get_dialect().get_name().unwrap_or("null"),
    );

    let explicit_null = DialectConfiguration::with_prefix(None, Some(Arc::clone(&dialect)))
        .expect("explicit null prefix is legal");
    emit(
        output,
        "explicit_null.specified",
        &explicit_null.is_prefix_specified().to_string(),
    );
    emit(
        output,
        "explicit_null.prefix",
        explicit_null.get_prefix().unwrap_or("null"),
    );
    emit(
        output,
        "explicit_null.dialect_identity",
        &std::ptr::eq(explicit_null.get_dialect(), dialect.as_ref()).to_string(),
    );

    let empty = DialectConfiguration::with_prefix(Some(""), Some(Arc::clone(&dialect)))
        .expect("empty prefix is legal");
    emit(
        output,
        "empty.specified",
        &empty.is_prefix_specified().to_string(),
    );
    emit(output, "empty.prefix", empty.get_prefix().unwrap_or("null"));

    let custom = DialectConfiguration::with_prefix(Some("th"), Some(Arc::clone(&dialect)))
        .expect("custom prefix is legal");
    emit(
        output,
        "custom.specified",
        &custom.is_prefix_specified().to_string(),
    );
    emit(
        output,
        "custom.prefix",
        custom.get_prefix().unwrap_or("null"),
    );

    let whitespace = DialectConfiguration::with_prefix(Some(" \t"), Some(dialect))
        .expect("whitespace prefix is legal");
    emit(
        output,
        "whitespace.prefix",
        whitespace.get_prefix().unwrap_or("null"),
    );

    export_null_dialect_failure(output, "null.default", DialectConfiguration::new(None));
    export_null_dialect_failure(
        output,
        "null.explicit",
        DialectConfiguration::with_prefix(Some("th"), None),
    );
}

fn export_null_dialect_failure(
    output: &mut String,
    key: &str,
    result: Result<DialectConfiguration, DialectConfigurationError>,
) {
    match result {
        Ok(_) => emit_no_failure(output, key),
        Err(error) => emit_failure(output, key, "IllegalArgumentException", &error.to_string()),
    }
}

fn emit_failure(output: &mut String, key: &str, class: &str, message: &str) {
    emit(output, &format!("{key}.class"), class);
    emit(output, &format!("{key}.message"), message);
}

fn emit_no_failure(output: &mut String, key: &str) {
    emit_failure(output, key, "<none>", "<none>");
}

fn emit(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('=');
    output.push_str(&value.replace('\\', "\\\\").replace('\t', "\\t"));
    output.push('\n');
}
