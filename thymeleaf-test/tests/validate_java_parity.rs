//! `Validate` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::{Display, Write};

use thymeleaf::util::{Validate, ValidateError};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/validate_golden.txt");

#[test]
fn validate_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    emit_outcome(
        &mut output,
        "not_null.value",
        Validate::not_null(Some(&"value"), Some("failure")),
    );
    emit_outcome(
        &mut output,
        "not_null.null",
        Validate::not_null::<str>(None, Some("failure")),
    );
    emit_outcome(
        &mut output,
        "not_null.null_message",
        Validate::not_null::<str>(None, None),
    );

    emit_outcome(
        &mut output,
        "not_empty_str.value",
        Validate::not_empty_str(Some("value"), Some("empty")),
    );
    for (key, value) in [
        ("null", None),
        ("empty", Some("")),
        ("space", Some(" ")),
        ("control", Some("\u{001C}")),
        ("ogham", Some("\u{1680}")),
        ("punctuation_space", Some("\u{2008}")),
        ("line_separator", Some("\u{2028}")),
        ("medium_space", Some("\u{205F}")),
        ("ideographic_space", Some("\u{3000}")),
        ("nbsp", Some("\u{00A0}")),
    ] {
        emit_outcome(
            &mut output,
            &format!("not_empty_str.{key}"),
            Validate::not_empty_str(value, Some("empty")),
        );
    }

    let empty_collection: Vec<String> = Vec::new();
    let value_collection = vec!["value".to_owned()];
    emit_outcome(
        &mut output,
        "not_empty_collection.null",
        Validate::not_empty_collection(None::<&Vec<String>>, Some("empty")),
    );
    emit_outcome(
        &mut output,
        "not_empty_collection.empty",
        Validate::not_empty_collection(Some(&empty_collection), Some("empty")),
    );
    emit_outcome(
        &mut output,
        "not_empty_collection.value",
        Validate::not_empty_collection(Some(&value_collection), Some("empty")),
    );

    emit_outcome(
        &mut output,
        "not_empty_array.null",
        Validate::not_empty_array::<String>(None, Some("empty")),
    );
    emit_outcome(
        &mut output,
        "not_empty_array.empty",
        Validate::not_empty_array::<String>(Some(&[]), Some("empty")),
    );
    emit_outcome(
        &mut output,
        "not_empty_array.value",
        Validate::not_empty_array(Some(&["value".to_owned()]), Some("empty")),
    );

    let no_nulls = vec![Some("one".to_owned()), Some("2".to_owned())];
    let with_null = vec![Some("one".to_owned()), None, Some("three".to_owned())];
    emit_outcome(
        &mut output,
        "no_nulls_iterable.value",
        Validate::contains_no_nulls_iterable(Some(&no_nulls), Some("null")),
    );
    emit_outcome(
        &mut output,
        "no_nulls_iterable.element",
        Validate::contains_no_nulls_iterable(Some(&with_null), Some("null")),
    );
    emit_outcome(
        &mut output,
        "no_nulls_iterable.null_message",
        Validate::contains_no_nulls_iterable(Some(&with_null), None),
    );
    emit_implicit(
        &mut output,
        "no_nulls_iterable.null_container",
        Validate::contains_no_nulls_iterable::<Vec<Option<String>>, String>(None, Some("ignored")),
    );

    let no_empties = vec![Some("value".to_owned()), Some("\u{00A0}".to_owned())];
    let with_empty = vec![Some("value".to_owned()), Some(String::new())];
    let with_whitespace = vec![Some("value".to_owned()), Some("\u{2008}".to_owned())];
    let with_null_string = vec![Some("value".to_owned()), None];
    emit_outcome(
        &mut output,
        "no_empties.value",
        Validate::contains_no_empties(Some(&no_empties), Some("empty")),
    );
    emit_outcome(
        &mut output,
        "no_empties.empty",
        Validate::contains_no_empties(Some(&with_empty), Some("empty")),
    );
    emit_outcome(
        &mut output,
        "no_empties.whitespace",
        Validate::contains_no_empties(Some(&with_whitespace), Some("empty")),
    );
    emit_outcome(
        &mut output,
        "no_empties.null_element",
        Validate::contains_no_empties(Some(&with_null_string), Some("empty")),
    );
    emit_implicit(
        &mut output,
        "no_empties.null_container",
        Validate::contains_no_empties::<Vec<Option<String>>, String>(None, Some("ignored")),
    );

    emit_outcome(
        &mut output,
        "no_nulls_array.value",
        Validate::contains_no_nulls_array(
            Some(&[Some("one".to_owned()), Some("2".to_owned())]),
            Some("null"),
        ),
    );
    emit_outcome(
        &mut output,
        "no_nulls_array.element",
        Validate::contains_no_nulls_array(Some(&[Some("one".to_owned()), None]), Some("null")),
    );
    emit_implicit(
        &mut output,
        "no_nulls_array.null_container",
        Validate::contains_no_nulls_array::<String>(None, Some("ignored")),
    );

    emit_outcome(
        &mut output,
        "is_true.true",
        Validate::is_true(true, Some("failure")),
    );
    emit_outcome(
        &mut output,
        "is_true.false",
        Validate::is_true(false, Some("failure")),
    );
    emit_outcome(
        &mut output,
        "is_true.null_message",
        Validate::is_true(false, None),
    );

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_outcome(output: &mut String, key: &str, result: Result<(), ValidateError>) {
    match result {
        Ok(()) => emit(output, key, "OK"),
        Err(error) => {
            let _display = error.to_string();
            emit(
                output,
                key,
                format!(
                    "{}:{}",
                    error.java_class_name(),
                    error.get_message().unwrap_or("null")
                ),
            );
        }
    }
}

fn emit_implicit(output: &mut String, key: &str, result: Result<(), ValidateError>) {
    match result {
        Ok(()) => emit(output, key, "OK"),
        Err(error) => emit(output, key, error.java_class_name()),
    }
}

fn emit(output: &mut String, key: &str, value: impl Display) {
    writeln!(output, "{key}={value}").expect("string output");
}
