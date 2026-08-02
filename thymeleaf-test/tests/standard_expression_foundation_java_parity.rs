//! `LiteralValue` 与 `StandardExpressionExecutionContext` 的 Java/Rust Golden 差分测试。

use std::any::Any;
use std::fmt::Write;
use std::ptr;

use thymeleaf::expression::{LiteralValue, StandardExpressionExecutionContext};
use thymeleaf::util::JavaString;

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/standard_expression_foundation_golden.txt");

#[test]
fn standard_expression_foundation_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_literal_value_cases(&mut output);
    emit_execution_context_cases(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_literal_value_cases(output: &mut String) {
    let literal = LiteralValue::new(Some(JavaString::from_rust_str("4")));
    let same_text = LiteralValue::new(Some(JavaString::from_rust_str("4")));
    let null_literal = LiteralValue::new(None);
    let other = String::from("object");
    let other_object = &other as &dyn Any;

    emit(
        output,
        "literal.value",
        literal
            .get_value()
            .expect("literal value")
            .to_string_lossy(),
    );
    emit(output, "literal.nullValue", "null");
    let literal_alias = &literal;
    emit(output, "literal.identityEquals", literal.eq(literal_alias));
    emit(output, "literal.distinctEquals", literal == same_text);
    emit(output, "unwrap.null", "null");
    emit(
        output,
        "unwrap.literal",
        LiteralValue::unwrap(Some(&literal as &dyn Any))
            .and_then(|value| value.downcast_ref::<JavaString>())
            .expect("unwrapped string")
            .to_string_lossy(),
    );
    emit(
        output,
        "unwrap.literalNull",
        if LiteralValue::unwrap(Some(&null_literal as &dyn Any)).is_none() {
            "null"
        } else {
            "unexpected"
        },
    );
    emit(
        output,
        "unwrap.otherIdentity",
        ptr::eq(
            LiteralValue::unwrap(Some(other_object)).expect("other object"),
            other_object,
        ),
    );
}

fn emit_execution_context_cases(output: &mut String) {
    let contexts = [
        ("normal", StandardExpressionExecutionContext::NORMAL),
        ("restricted", StandardExpressionExecutionContext::RESTRICTED),
        (
            "forbid",
            StandardExpressionExecutionContext::RESTRICTED_FORBID_UNSAFE_EXP_RESULTS,
        ),
    ];

    for (name, context) in contexts {
        emit(
            output,
            &format!("context.{name}.flags"),
            format!(
                "{},{},{},{}",
                context.get_restrict_variable_access(),
                context.get_restrict_external_access(),
                context.get_forbid_unsafe_expression_results(),
                context.get_perform_type_conversion()
            ),
        );
    }

    for (name, context) in contexts {
        let converted = context.with_type_conversion();
        emit(
            output,
            &format!("context.{name}.withoutSame"),
            ptr::eq(context.without_type_conversion(), context),
        );
        emit(
            output,
            &format!("context.{name}.converted"),
            converted.get_perform_type_conversion(),
        );
        emit(
            output,
            &format!("context.{name}.withSame"),
            ptr::eq(converted.with_type_conversion(), converted),
        );
        emit(
            output,
            &format!("context.{name}.roundTrip"),
            ptr::eq(converted.without_type_conversion(), context),
        );
        emit(
            output,
            &format!("context.{name}.canonical"),
            ptr::eq(context.with_type_conversion(), converted),
        );
    }
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to String");
}
