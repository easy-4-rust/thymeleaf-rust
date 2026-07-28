//! Thymeleaf 3.1.5 基础对象的 Java/Rust Golden 差分测试。

use std::error::Error;
use std::io;

use thymeleaf::{
    AlreadyInitializedException, CacheConfigurationException, ConfigurationException,
    ParserInitializationException, TemplateAssertionException, TemplateInputException,
    TemplateMode, TemplateOutputException, TemplateProcessingException,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/foundation_golden.txt");

#[test]
fn foundation_objects_match_java_golden() {
    assert_eq!(rust_foundation_golden(), JAVA_GOLDEN);
}

fn rust_foundation_golden() -> String {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_template_modes(&mut output);
    emit_simple_exceptions(&mut output);
    emit_template_processing_exception(&mut output);
    emit_template_assertion_exception(&mut output);
    emit_template_input_exception(&mut output);
    emit_template_output_exception(&mut output);
    output
}

fn emit_template_modes(output: &mut String) {
    for mode in [
        TemplateMode::HTML,
        TemplateMode::XML,
        TemplateMode::TEXT,
        TemplateMode::JAVASCRIPT,
        TemplateMode::CSS,
        TemplateMode::RAW,
    ] {
        emit(
            output,
            &format!("mode.{mode}.flags"),
            &format!(
                "{},{},{}",
                mode.is_markup(),
                mode.is_text(),
                mode.is_case_sensitive()
            ),
        );
        emit(output, &format!("mode.{mode}.display"), &mode.to_string());
    }

    for (key, value) in [
        ("null", None),
        ("empty", Some("")),
        ("blank", Some(" \n\t")),
        ("nul_control", Some("\0")),
        ("nbsp", Some("\u{00A0}")),
        ("html", Some("html")),
        ("XML", Some("XML")),
        ("Text", Some("Text")),
        ("javascript", Some("javascript")),
        ("Css", Some("Css")),
        ("raw", Some("raw")),
        ("unknown", Some("MARKDOWN")),
        ("padded_xml", Some(" XML ")),
    ] {
        let value = match TemplateMode::parse(value) {
            Ok(mode) => mode.to_string(),
            Err(error) => format!("IllegalArgumentException:{error}"),
        };
        emit(output, &format!("parse.{key}"), &value);
    }
}

fn emit_simple_exceptions(output: &mut String) {
    let already = AlreadyInitializedException::with_cause(
        Some("initialized".to_owned()),
        io::Error::other("cause"),
    );
    emit(output, "already.message", already.get_message().unwrap());
    emit(
        output,
        "already.cause",
        &already.source().unwrap().to_string(),
    );
    emit(
        output,
        "already.null",
        nullable_message(AlreadyInitializedException::new(None).get_message()),
    );

    let configuration = ConfigurationException::with_cause(
        Some("configuration".to_owned()),
        io::Error::other("cause"),
    );
    emit(
        output,
        "configuration.message",
        configuration.get_message().unwrap(),
    );
    emit(
        output,
        "configuration.cause",
        &configuration.source().unwrap().to_string(),
    );
    emit(
        output,
        "configuration.null",
        nullable_message(ConfigurationException::new(None).get_message()),
    );

    let cache = CacheConfigurationException::with_cause(
        Some("cache".to_owned()),
        io::Error::other("cause"),
    );
    emit(output, "cache.message", cache.get_message().unwrap());
    emit(output, "cache.cause", &cache.source().unwrap().to_string());
    emit(
        output,
        "cache.null",
        nullable_message(CacheConfigurationException::new(None).get_message()),
    );

    let parser = ParserInitializationException::with_cause(
        Some("parser".to_owned()),
        io::Error::other("cause"),
    );
    emit(output, "parser.message", parser.get_message().unwrap());
    emit(
        output,
        "parser.cause",
        &parser.source().unwrap().to_string(),
    );
    emit(
        output,
        "parser.null",
        nullable_message(ParserInitializationException::new(None).get_message()),
    );
}

fn emit_template_processing_exception(output: &mut String) {
    let plain = TemplateProcessingException::new(Some("problem".to_owned()));
    emit(output, "processing.plain.message", &plain.get_message());
    emit(
        output,
        "processing.plain.template",
        nullable_message(plain.get_template_name()),
    );
    emit(
        output,
        "processing.plain.has_template",
        &plain.has_template_name().to_string(),
    );
    emit(
        output,
        "processing.plain.line",
        &nullable_number(plain.get_line()),
    );
    emit(
        output,
        "processing.plain.col",
        &nullable_number(plain.get_col()),
    );
    emit(
        output,
        "processing.plain.has_line_col",
        &plain.has_line_and_col().to_string(),
    );

    emit(
        output,
        "processing.null.message",
        &TemplateProcessingException::new(None).get_message(),
    );

    let caused = TemplateProcessingException::with_cause(
        Some("problem".to_owned()),
        io::Error::other("cause"),
    );
    emit(
        output,
        "processing.caused.cause",
        &caused.source().unwrap().to_string(),
    );

    let template_caused = TemplateProcessingException::with_template_and_cause(
        Some("problem".to_owned()),
        Some("index.html".to_owned()),
        io::Error::other("cause"),
    );
    emit(
        output,
        "processing.template_cause.message",
        &template_caused.get_message(),
    );
    emit(
        output,
        "processing.template_cause.cause",
        &template_caused.source().unwrap().to_string(),
    );

    emit_processing_location(output, "complete", 7, 11);
    emit_processing_location(output, "line_only", 7, -1);
    emit_processing_location(output, "col_only", -1, 11);
    emit_processing_location(output, "no_location", -1, -1);

    let hidden_location =
        TemplateProcessingException::with_location(Some("problem".to_owned()), None, 1, 2);
    emit(
        output,
        "processing.hidden_location.message",
        &hidden_location.get_message(),
    );

    let mut mutable = TemplateProcessingException::with_location_and_cause(
        Some("problem".to_owned()),
        Some("old.html".to_owned()),
        1,
        2,
        io::Error::other("cause"),
    );
    mutable.set_template_name(Some("new.html".to_owned()));
    mutable.set_line_and_col(-1, 9);
    emit(output, "processing.mutable.message", &mutable.get_message());
    emit(
        output,
        "processing.mutable.template",
        nullable_message(mutable.get_template_name()),
    );
    emit(
        output,
        "processing.mutable.line",
        &nullable_number(mutable.get_line()),
    );
    emit(
        output,
        "processing.mutable.col",
        &nullable_number(mutable.get_col()),
    );
    emit(
        output,
        "processing.mutable.has_line_col",
        &mutable.has_line_and_col().to_string(),
    );
    emit(
        output,
        "processing.mutable.cause",
        &mutable.source().unwrap().to_string(),
    );
}

fn emit_processing_location(output: &mut String, key: &str, line: i32, col: i32) {
    let exception = TemplateProcessingException::with_location(
        Some("problem".to_owned()),
        Some("index.html".to_owned()),
        line,
        col,
    );
    emit(
        output,
        &format!("processing.{key}.message"),
        &exception.get_message(),
    );
    emit(
        output,
        &format!("processing.{key}.line"),
        &nullable_number(exception.get_line()),
    );
    emit(
        output,
        &format!("processing.{key}.col"),
        &nullable_number(exception.get_col()),
    );
    emit(
        output,
        &format!("processing.{key}.has_line_col"),
        &exception.has_line_and_col().to_string(),
    );
}

fn emit_template_assertion_exception(output: &mut String) {
    emit(
        output,
        "assertion.plain",
        TemplateAssertionException::new(Some("${user != null}"), Some("index.html")).get_message(),
    );
    emit(
        output,
        "assertion.located",
        TemplateAssertionException::with_location(
            Some("${user != null}"),
            Some("index.html"),
            7,
            3,
        )
        .get_message(),
    );
    emit(
        output,
        "assertion.null",
        TemplateAssertionException::new(None, None).get_message(),
    );
}

fn emit_template_input_exception(output: &mut String) {
    emit(
        output,
        "input.plain",
        &TemplateInputException::new(Some("input".to_owned())).get_message(),
    );
    let caused =
        TemplateInputException::with_cause(Some("input".to_owned()), io::Error::other("cause"));
    emit(
        output,
        "input.caused.cause",
        &caused.source().unwrap().to_string(),
    );
    emit(
        output,
        "input.template_cause",
        &TemplateInputException::with_template_and_cause(
            Some("input".to_owned()),
            Some("index.html".to_owned()),
            io::Error::other("cause"),
        )
        .get_message(),
    );
    emit(
        output,
        "input.location",
        &TemplateInputException::with_location(
            Some("input".to_owned()),
            Some("index.html".to_owned()),
            3,
            4,
        )
        .get_message(),
    );
    let located_cause = TemplateInputException::with_location_and_cause(
        Some("input".to_owned()),
        Some("index.html".to_owned()),
        5,
        6,
        io::Error::other("cause"),
    );
    emit(
        output,
        "input.location_cause.message",
        &located_cause.get_message(),
    );
    emit(
        output,
        "input.location_cause.cause",
        &located_cause.source().unwrap().to_string(),
    );
}

fn emit_template_output_exception(output: &mut String) {
    let output_error = TemplateOutputException::new(
        Some("output".to_owned()),
        Some("index.html".to_owned()),
        9,
        10,
        io::Error::other("writer"),
    );
    emit(output, "output.message", &output_error.get_message());
    emit(
        output,
        "output.template",
        nullable_message(output_error.get_template_name()),
    );
    emit(
        output,
        "output.line",
        &nullable_number(output_error.get_line()),
    );
    emit(
        output,
        "output.col",
        &nullable_number(output_error.get_col()),
    );
    emit(
        output,
        "output.has_line_col",
        &output_error.has_line_and_col().to_string(),
    );
    emit(
        output,
        "output.cause",
        &output_error.source().unwrap().to_string(),
    );
}

fn emit(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('=');
    output.push_str(value);
    output.push('\n');
}

fn nullable_message(value: Option<&str>) -> &str {
    value.unwrap_or("null")
}

fn nullable_number(value: Option<i32>) -> String {
    value.map_or_else(|| "null".to_owned(), |number| number.to_string())
}
