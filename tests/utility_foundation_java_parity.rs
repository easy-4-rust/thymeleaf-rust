//! `NumberPointType` 与 `IdentityCounter` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::Display;
use std::fmt::Write;
use std::rc::Rc;

use thymeleaf::util::{IdentityCounter, IdentityCounterError, NumberPointType};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/utility_foundation_golden.txt");

#[test]
fn utility_foundation_objects_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    let values = [
        NumberPointType::Point,
        NumberPointType::Comma,
        NumberPointType::Whitespace,
        NumberPointType::None,
        NumberPointType::Default,
    ];
    for (ordinal, value) in values.into_iter().enumerate() {
        emit(
            &mut output,
            &format!("point.{ordinal}.name"),
            value.get_name(),
        );
        emit(&mut output, &format!("point.{ordinal}.display"), value);
        emit(
            &mut output,
            &format!("point.{ordinal}.identity"),
            NumberPointType::match_name(Some(value.get_name())) == Some(value),
        );
    }
    emit_match(&mut output, "null", None);
    emit_match(&mut output, "empty", Some(""));
    emit_match(&mut output, "lower", Some("point"));
    emit_match(&mut output, "leading_space", Some(" POINT"));
    emit_match(&mut output, "unknown", Some("UNKNOWN"));

    emit_failure(
        &mut output,
        "identity.negative",
        IdentityCounter::<String>::new(-1),
    );
    emit_failure(
        &mut output,
        "identity.too_large",
        IdentityCounter::<String>::new(i32::MAX),
    );
    emit(
        &mut output,
        "identity.zero",
        IdentityCounter::<String>::new(0).is_ok(),
    );

    let mut counter = IdentityCounter::new(2).expect("counter");
    let first = Rc::new("same".to_owned());
    let first_alias = Rc::clone(&first);
    let equal_but_distinct = Rc::new("same".to_owned());
    emit(
        &mut output,
        "identity.first.before",
        counter.is_already_counted(Some(&first)),
    );
    counter.count(Some(Rc::clone(&first)));
    emit(
        &mut output,
        "identity.first.after",
        counter.is_already_counted(Some(&first)),
    );
    emit(
        &mut output,
        "identity.alias",
        counter.is_already_counted(Some(&first_alias)),
    );
    emit(
        &mut output,
        "identity.equal_distinct.before",
        counter.is_already_counted(Some(&equal_but_distinct)),
    );
    counter.count(Some(Rc::clone(&equal_but_distinct)));
    emit(
        &mut output,
        "identity.equal_distinct.after",
        counter.is_already_counted(Some(&equal_but_distinct)),
    );
    counter.count(Some(first));

    emit(
        &mut output,
        "identity.null.before",
        counter.is_already_counted(None),
    );
    counter.count(None);
    counter.count(None);
    emit(
        &mut output,
        "identity.null.after",
        counter.is_already_counted(None),
    );
    emit(
        &mut output,
        "identity.unseen",
        counter.is_already_counted(Some(&Rc::new("same".to_owned()))),
    );

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_match(output: &mut String, key: &str, input: Option<&str>) {
    let value = NumberPointType::match_name(input)
        .map(NumberPointType::get_name)
        .unwrap_or("null");
    emit(output, &format!("match.{key}"), value);
}

fn emit_failure(
    output: &mut String,
    key: &str,
    result: Result<IdentityCounter<String>, IdentityCounterError>,
) {
    match result {
        Ok(_) => emit(output, key, "NO_ERROR"),
        Err(error) => emit(
            output,
            key,
            format!("java.lang.IllegalArgumentException:{error}"),
        ),
    }
}

fn emit(output: &mut String, key: &str, value: impl Display) {
    writeln!(output, "{key}={value}").expect("string output");
}
