//! `TextParseException` 与固定 Thymeleaf Java 基线的逐记录语义对照测试。

use std::error::Error;
use std::fmt::{Display, Formatter, Write};

use thymeleaf::text::{TextParseCause, TextParseException};
use thymeleaf::util::Utf16String;

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/text_parse_exception_golden.txt");
const PLAIN_CLASS: &str =
    "org.thymeleaf.templateparser.text.TextParseExceptionGolden$PlainThrowable";

#[test]
fn text_parse_exception_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    basic_constructors(&mut output);
    cause_constructors(&mut output);
    location_constructors(&mut output);
    inherited_location_constructors(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn basic_constructors(output: &mut String) {
    emit_exception(output, "default", &TextParseException::new());
    emit_exception(
        output,
        "message",
        &TextParseException::with_message(Some(java("problem"))),
    );
    emit_exception(
        output,
        "nullMessage",
        &TextParseException::with_message(None),
    );
    emit_exception(
        output,
        "surrogateMessage",
        &TextParseException::with_message(Some(Utf16String::from_utf16([0xD800]))),
    );
}

fn cause_constructors(output: &mut String) {
    emit_exception(
        output,
        "messageCause",
        &TextParseException::with_message_and_cause(
            Some(java("outer")),
            Some(plain_cause(Some("cause"))),
        ),
    );
    emit_exception(
        output,
        "nullMessageCause",
        &TextParseException::with_message_and_cause(None, Some(plain_cause(Some("cause")))),
    );
    emit_exception(
        output,
        "messageNullCause",
        &TextParseException::with_message_and_cause(Some(java("outer")), None),
    );
    emit_exception(
        output,
        "nullMessageNullCause",
        &TextParseException::with_message_and_cause(None, None),
    );
    emit_exception(
        output,
        "cause",
        &TextParseException::with_cause(Some(plain_cause(Some("cause")))),
    );
    emit_exception(output, "nullCause", &TextParseException::with_cause(None));
    emit_exception(
        output,
        "nullCauseMessage",
        &TextParseException::with_cause(Some(plain_cause(None))),
    );

    let error: Box<dyn Error + Send + Sync> = Box::new(PlainError);
    let identity = error.as_ref() as *const dyn Error as *const ();
    let cause = TextParseCause::with_java_metadata(error, PLAIN_CLASS, Some(java("cause")));
    let caused = TextParseException::with_message_and_cause(Some(java("outer")), Some(cause));
    let source_identity = caused.source().expect("source") as *const dyn Error as *const ();
    emit(output, "cause.identity", identity == source_identity);
}

fn location_constructors(output: &mut String) {
    emit_exception(
        output,
        "location",
        &TextParseException::with_location(7, 11),
    );
    emit_exception(
        output,
        "negativeLocation",
        &TextParseException::with_location(-1, i32::MIN),
    );
    emit_exception(
        output,
        "messageLocation",
        &TextParseException::with_message_at(Some(&java("problem")), 7, 11),
    );
    emit_exception(
        output,
        "nullMessageLocation",
        &TextParseException::with_message_at(None, 7, 11),
    );
    emit_exception(
        output,
        "causeLocation",
        &TextParseException::with_cause_at(Some(plain_cause(Some("cause"))), 7, 11),
    );
    emit_exception(
        output,
        "nullCauseLocation",
        &TextParseException::with_cause_at(None, 7, 11),
    );
    emit_exception(
        output,
        "messageCauseLocation",
        &TextParseException::with_message_and_cause_at(
            Some(&java("problem")),
            Some(plain_cause(Some("cause"))),
            7,
            11,
        ),
    );
    emit_exception(
        output,
        "nullMessageCauseLocation",
        &TextParseException::with_message_and_cause_at(
            None,
            Some(plain_cause(Some("cause"))),
            7,
            11,
        ),
    );
}

fn inherited_location_constructors(output: &mut String) {
    let located = TextParseException::with_message_at(Some(&java("inner")), 3, 5);
    emit_exception(
        output,
        "inherit.messageCause",
        &TextParseException::with_message_and_cause(
            Some(java("outer")),
            Some(TextParseCause::from_text_parse(located)),
        ),
    );
    let located = TextParseException::with_message_at(Some(&java("inner")), 3, 5);
    emit_exception(
        output,
        "inherit.nullMessageCause",
        &TextParseException::with_message_and_cause(
            None,
            Some(TextParseCause::from_text_parse(located)),
        ),
    );
    let located = TextParseException::with_message_at(Some(&java("inner")), 3, 5);
    emit_exception(
        output,
        "inherit.cause",
        &TextParseException::with_cause(Some(TextParseCause::from_text_parse(located))),
    );

    let unlocated = TextParseException::with_message(Some(java("inner")));
    emit_exception(
        output,
        "inherit.unlocatedMessage",
        &TextParseException::with_message_and_cause(
            Some(java("outer")),
            Some(TextParseCause::from_text_parse(unlocated)),
        ),
    );
    let unlocated = TextParseException::with_message(Some(java("inner")));
    emit_exception(
        output,
        "inherit.unlocatedCause",
        &TextParseException::with_cause(Some(TextParseCause::from_text_parse(unlocated))),
    );

    let null_located = TextParseException::with_message_at(None, 3, 5);
    emit_exception(
        output,
        "inherit.nullInnerMessage",
        &TextParseException::with_message_and_cause(
            None,
            Some(TextParseCause::from_text_parse(null_located)),
        ),
    );
}

fn emit_exception(output: &mut String, key: &str, exception: &TextParseException) {
    let message = exception
        .get_message()
        .cloned()
        .unwrap_or_else(|| java("null"));
    let line = exception
        .get_line()
        .map_or_else(|| "null".to_owned(), |value| value.to_string());
    let col = exception
        .get_col()
        .map_or_else(|| "null".to_owned(), |value| value.to_string());
    let cause = exception
        .get_cause()
        .map_or("null", TextParseCause::class_name);
    emit(
        output,
        key,
        format!("{}:{line}:{col}:{cause}", to_utf16_hex(&message)),
    );
}

fn plain_cause(message: Option<&str>) -> TextParseCause {
    TextParseCause::with_java_metadata(Box::new(PlainError), PLAIN_CLASS, message.map(java))
}

fn java(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn to_utf16_hex(value: &Utf16String) -> String {
    value
        .as_utf16()
        .iter()
        .map(|unit| format!("{unit:04x}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to string");
}

#[derive(Debug)]
struct PlainError;

impl Display for PlainError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("cause")
    }
}

impl Error for PlainError {}
