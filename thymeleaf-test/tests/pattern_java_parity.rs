//! `PatternUtils` 与 `PatternSpec` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::{Display, Write};

use indexmap::IndexSet;
use thymeleaf::util::{PatternSpec, PatternSpecError, PatternUtils};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/pattern_golden.txt");

#[test]
fn pattern_utils_and_spec_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    emit_pattern(
        &mut output,
        "glob",
        "*.html",
        &["index.html", "path/view.html", "index.htm"],
    );
    emit_pattern(
        &mut output,
        "escaped",
        "a.(b)[c]?$+*",
        &["a.(b)[c]?$+tail", "xa.(b)[c]?$+tail"],
    );
    emit_pattern(
        &mut output,
        "alternation",
        "foo|bar",
        &["foo", "bar", "foobar"],
    );
    emit_pattern(&mut output, "quantifier", "a{2}", &["aa", "a", "aaa"]);
    emit_pattern(&mut output, "digit", r"\d*", &["1tail", "١tail", "xtail"]);
    emit_pattern(
        &mut output,
        "star",
        "*",
        &[
            "plain",
            "line\nbreak",
            "line\rbreak",
            "line\u{0085}break",
            "line\u{2028}break",
        ],
    );
    emit_pattern(&mut output, "trailing_escape", r"abc\", &["abc$", "abc"]);
    emit_pattern(&mut output, "empty", "", &["", "value"]);
    emit_pattern(
        &mut output,
        "quoted",
        r"\Qfoo|*\E",
        &["foo|(?:.*?)", "foo|anything", "foo|*"],
    );
    emit_pattern(
        &mut output,
        "unterminated_quote",
        r"\Qfoo",
        &["foo$", "foo"],
    );

    for (regex_class, input) in [
        (r"\D", "x"),
        (r"\w", "_"),
        (r"\W", "-"),
        (r"\s", "\t"),
        (r"\S", "x"),
        (r"\h", "\u{3000}"),
        (r"\H", "x"),
        (r"\v", "\u{2028}"),
        (r"\V", "x"),
        (r"\R", "\r\n"),
    ] {
        let pattern =
            PatternUtils::str_pattern_to_pattern(Some(regex_class)).expect("class pattern");
        emit(
            &mut output,
            &format!("pattern.class.{}", &regex_class[1..]),
            pattern.matches(Some(input)).expect("input"),
        );
    }

    let null = PatternUtils::str_pattern_to_pattern(None).expect_err("null");
    emit(&mut output, "pattern.null", null.class_name());
    let syntax = PatternUtils::str_pattern_to_pattern(Some("{")).expect_err("syntax");
    emit(
        &mut output,
        "pattern.syntax",
        format!(
            "{}:{}",
            syntax.class_name(),
            syntax.get_pattern().expect("pattern")
        ),
    );

    let empty = PatternSpec::new();
    emit(&mut output, "spec.new.empty", empty.is_empty());
    emit(
        &mut output,
        "spec.new.patterns",
        format_patterns(empty.get_patterns()),
    );
    emit(
        &mut output,
        "spec.new.null_match",
        empty.matches(None).expect("empty"),
    );
    let _: &IndexSet<Option<String>> = empty.get_patterns();
    emit(
        &mut output,
        "spec.new.unmodifiable",
        "java.lang.UnsupportedOperationException",
    );

    let mut set = PatternSpec::new();
    set.set_patterns(Some(&[Some("*.html"), Some("admin/*")]))
        .expect("ordered");
    emit(
        &mut output,
        "spec.set.patterns",
        format_patterns(set.get_patterns()),
    );
    emit(
        &mut output,
        "spec.set.html",
        set.matches(Some("index.html")).expect("match"),
    );
    emit(
        &mut output,
        "spec.set.admin",
        set.matches(Some("admin/users")).expect("match"),
    );
    emit(
        &mut output,
        "spec.set.miss",
        set.matches(Some("index.htm")).expect("match"),
    );

    set.add_pattern(Some("*.html")).expect("duplicate");
    emit(
        &mut output,
        "spec.duplicate.patterns",
        format_patterns(set.get_patterns()),
    );

    let mut validation = PatternSpec::new();
    emit_spec_error(
        &mut output,
        "spec.add.null",
        validation.add_pattern(None),
        true,
    );
    emit_spec_error(
        &mut output,
        "spec.add.empty",
        validation.add_pattern(Some("")),
        true,
    );
    emit_spec_error(
        &mut output,
        "spec.add.whitespace",
        validation.add_pattern(Some("\u{2008}")),
        true,
    );
    emit(
        &mut output,
        "spec.add.validation_patterns",
        format_patterns(validation.get_patterns()),
    );

    let mut add_syntax = PatternSpec::new();
    emit_spec_error(
        &mut output,
        "spec.add.syntax",
        add_syntax.add_pattern(Some("{")),
        false,
    );
    emit(
        &mut output,
        "spec.add.syntax_patterns",
        format_patterns(add_syntax.get_patterns()),
    );
    emit(&mut output, "spec.add.syntax_empty", add_syntax.is_empty());

    let mut set_syntax = PatternSpec::new();
    emit_spec_error(
        &mut output,
        "spec.set.syntax",
        set_syntax.set_patterns(Some(&[Some("*.html"), Some("{"), Some("*.txt")])),
        false,
    );
    emit(
        &mut output,
        "spec.set.syntax_patterns",
        format_patterns(set_syntax.get_patterns()),
    );
    emit(
        &mut output,
        "spec.set.syntax_html",
        set_syntax.matches(Some("view.html")).expect("prefix"),
    );
    emit(
        &mut output,
        "spec.set.syntax_txt",
        set_syntax
            .matches(Some("view.txt"))
            .expect("missing suffix"),
    );

    let mut set_null = PatternSpec::new();
    emit_spec_error(
        &mut output,
        "spec.set.null_element",
        set_null.set_patterns(Some(&[Some("*.html"), None, Some("*.txt")])),
        false,
    );
    emit(
        &mut output,
        "spec.set.null_patterns",
        format_patterns(set_null.get_patterns()),
    );

    let mut null_match = PatternSpec::new();
    null_match.add_pattern(Some("*")).expect("pattern");
    emit_spec_result_class(&mut output, "spec.matches.null", null_match.matches(None));
    null_match.clear_patterns();
    emit(&mut output, "spec.clear.empty", null_match.is_empty());
    emit(
        &mut output,
        "spec.clear.patterns",
        format_patterns(null_match.get_patterns()),
    );
    emit(
        &mut output,
        "spec.clear.null_match",
        null_match.matches(None).expect("empty"),
    );
    null_match
        .set_patterns(Some(&[Some("*.html"), Some("admin/*")]))
        .expect("patterns");
    null_match.set_patterns(None).expect("clear");
    emit(&mut output, "spec.set_null.empty", null_match.is_empty());
    emit(
        &mut output,
        "spec.set_null.patterns",
        format_patterns(null_match.get_patterns()),
    );

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_pattern(output: &mut String, key: &str, source: &str, inputs: &[&str]) {
    let pattern = PatternUtils::str_pattern_to_pattern(Some(source)).expect("pattern");
    emit(output, &format!("pattern.{key}.source"), pattern.as_str());
    for (index, input) in inputs.iter().enumerate() {
        emit(
            output,
            &format!("pattern.{key}.{index}"),
            pattern.matches(Some(input)).expect("input"),
        );
    }
}

fn format_patterns(patterns: &IndexSet<Option<String>>) -> String {
    let values = patterns
        .iter()
        .map(|pattern| pattern.as_deref().unwrap_or("null"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn emit_spec_error(
    output: &mut String,
    key: &str,
    result: Result<(), PatternSpecError>,
    include_message: bool,
) {
    match result {
        Ok(()) => emit(output, key, "OK"),
        Err(error) if include_message => emit(
            output,
            key,
            format!(
                "{}:{}",
                error.class_name(),
                error.get_message().unwrap_or("null")
            ),
        ),
        Err(error) => emit(output, key, error.class_name()),
    }
}

fn emit_spec_result_class(output: &mut String, key: &str, result: Result<bool, PatternSpecError>) {
    match result {
        Ok(value) => emit(output, key, value),
        Err(error) => emit(output, key, error.class_name()),
    }
}

fn emit(output: &mut String, key: &str, value: impl Display) {
    writeln!(output, "{key}={value}").expect("string output");
}
