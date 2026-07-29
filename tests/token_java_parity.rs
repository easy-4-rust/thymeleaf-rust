//! `Token` 与 `TokenParsingTracer` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::Write;
use std::ptr;
use std::sync::Arc;

use thymeleaf::expression::{
    JavaTokenStringResult, JavaTokenValue, Token, TokenError, TokenParsingTracer,
};
use thymeleaf::util::JavaString;

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/token_golden.txt");
const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[test]
fn token_and_tracer_match_java_golden() {
    cover_public_adapter_contracts();

    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_value_cases(&mut output);
    emit_exception_cases(&mut output);
    emit_readable_character_cases(&mut output);
    emit_exhaustive_character_cases(&mut output);
    emit_trace_cases(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn cover_public_adapter_contracts() {
    let owned = Token::new(Some(OwnedProbe));
    assert!(matches!(
        owned.get_string_representation(),
        Ok(JavaTokenStringResult::Owned(value))
            if value == JavaString::from_rust_str("owned")
    ));

    let runtime = TokenError::runtime("example.TokenException", "failure");
    assert_eq!(runtime.java_class_name(), "example.TokenException");
    assert_eq!(runtime.to_string(), "failure");

    let one = JavaString::from_rust_str("a");
    let index_error =
        Token::<JavaString>::is_token_char(Some(&one), i32::MAX).expect_err("index error");
    assert_eq!(
        index_error.java_class_name(),
        "java.lang.StringIndexOutOfBoundsException"
    );
    assert_eq!(
        index_error.to_string(),
        format!("String index out of range: {}", i32::MAX)
    );
}

fn emit_value_cases(output: &mut String) {
    let source = Arc::new(JavaString::from_rust_str("source"));
    let string_token = Token::new(Some(Arc::clone(&source)));
    let shared_probe = Arc::new(SharedStringProbe {
        shared: JavaString::from_rust_str("shared"),
    });
    let object_token = Token::new(Some(Arc::clone(&shared_probe)));

    emit(
        output,
        "value.identity",
        Arc::ptr_eq(string_token.get_value().expect("string value"), &source),
    );
    emit(
        output,
        "value.stringRepresentationIdentity",
        matches!(
            string_token.get_string_representation(),
            Ok(JavaTokenStringResult::Borrowed(value))
                if ptr::eq(value, source.as_ref())
        ),
    );
    emit(
        output,
        "value.toStringIdentity",
        matches!(
            string_token.to_string(),
            Ok(JavaTokenStringResult::Borrowed(value))
                if ptr::eq(value, source.as_ref())
        ),
    );
    emit(
        output,
        "value.objectIdentity",
        Arc::ptr_eq(
            object_token.get_value().expect("object value"),
            &shared_probe,
        ),
    );
    emit(
        output,
        "value.sharedRepresentationIdentity",
        matches!(
            object_token.get_string_representation(),
            Ok(JavaTokenStringResult::Borrowed(value))
                if ptr::eq(
                    value,
                    &shared_probe.shared
                )
        ),
    );
    emit(
        output,
        "value.sharedToStringIdentity",
        matches!(
            object_token.to_string(),
            Ok(JavaTokenStringResult::Borrowed(value))
                if ptr::eq(
                    value,
                    &shared_probe.shared
                )
        ),
    );

    let null_token = Token::<JavaString>::new(None);
    emit(
        output,
        "value.nullGet",
        if null_token.get_value().is_none() {
            "null"
        } else {
            "unexpected"
        },
    );
    let null_string_token = Token::new(Some(NullStringProbe));
    emit(
        output,
        "value.nullToStringResult",
        match null_string_token.to_string() {
            Ok(JavaTokenStringResult::Null) => "null",
            _ => "unexpected",
        },
    );
    emit_class_outcome(
        output,
        "value.nullFailure",
        null_token.get_string_representation(),
    );
    emit_outcome(
        output,
        "value.runtimeFailure",
        Token::new(Some(ThrowingProbe)).get_string_representation(),
    );
}

fn emit_exception_cases(output: &mut String) {
    emit_boolean_outcome(
        output,
        "char.null",
        Token::<JavaString>::is_token_char(None, 0),
    );
    let empty = JavaString::from_rust_str("");
    emit_boolean_outcome(
        output,
        "char.negative",
        Token::<JavaString>::is_token_char(Some(&empty), -1),
    );
    emit_boolean_outcome(
        output,
        "char.empty",
        Token::<JavaString>::is_token_char(Some(&empty), 0),
    );
    let one = JavaString::from_rust_str("a");
    emit_boolean_outcome(
        output,
        "char.afterEnd",
        Token::<JavaString>::is_token_char(Some(&one), 1),
    );
    match TokenParsingTracer::trace(None) {
        Ok(_) => emit(output, "trace.null", "OK:unexpected"),
        Err(error) => emit(
            output,
            "trace.null",
            format!("ERR:{}", error.java_class_name()),
        ),
    }
}

fn emit_readable_character_cases(output: &mut String) {
    let boundaries = [
        0x0000, 0x000A, 0x0020, 0x002D, 0x002E, 0x0030, 0x0039, 0x0041, 0x005A, 0x005B, 0x005D,
        0x005F, 0x0061, 0x007A, 0x00B6, 0x00B7, 0x00B8, 0x00BF, 0x00C0, 0x00D6, 0x00D7, 0x00D8,
        0x00F6, 0x00F7, 0x00F8, 0x02FF, 0x0300, 0x036F, 0x0370, 0x037D, 0x037E, 0x037F, 0x1FFF,
        0x2000, 0x200C, 0x200D, 0x203E, 0x203F, 0x2040, 0x2041, 0x206F, 0x2070, 0x218F, 0x2190,
        0x2BFF, 0x2C00, 0x2FEF, 0x2FF0, 0x3000, 0x3001, 0xD7FF, 0xD800, 0xF8FF, 0xF900, 0xFDCF,
        0xFDD0, 0xFDEF, 0xFDF0, 0xFFFD, 0xFFFE, 0xFFFF,
    ];
    let mut result = String::with_capacity(boundaries.len());
    for boundary in boundaries {
        let context = JavaString::from_utf16([boundary]);
        result.push(
            if Token::<JavaString>::is_token_char(Some(&context), 0).expect("valid boundary") {
                '1'
            } else {
                '0'
            },
        );
    }
    emit(output, "char.boundaries", result);

    let dash_contexts = [
        "-", "a-", "1-", "-a", "-1", "1-2", "a-1", "1-a", "--", "1--2", ".-.", "é-1", "1-é",
        "a - b", "a-+b", "foo-bar", "12.3-4", "12.-x", "x-.12",
    ];
    for (index, context) in dash_contexts.iter().enumerate() {
        let traced = TokenParsingTracer::trace(Some(&JavaString::from_rust_str(context)))
            .expect("dash trace");
        emit(
            output,
            &format!("dash.trace.{index}"),
            traced.to_string_lossy(),
        );
    }
}

fn emit_exhaustive_character_cases(output: &mut String) {
    let mut single_hash = FNV_OFFSET;
    let mut left_dash_hash = FNV_OFFSET;
    let mut right_dash_hash = FNV_OFFSET;
    let mut all_bmp = Vec::with_capacity(usize::from(u16::MAX) + 1);

    for code_unit in u16::MIN..=u16::MAX {
        all_bmp.push(code_unit);
        let single = JavaString::from_utf16([code_unit]);
        single_hash = mix_boolean(
            single_hash,
            Token::<JavaString>::is_token_char(Some(&single), 0).expect("single BMP"),
        );
        let left_dash = JavaString::from_utf16([code_unit, u16::from(b'-')]);
        left_dash_hash = mix_boolean(
            left_dash_hash,
            Token::<JavaString>::is_token_char(Some(&left_dash), 1).expect("left dash BMP"),
        );
        let right_dash = JavaString::from_utf16([u16::from(b'-'), code_unit]);
        right_dash_hash = mix_boolean(
            right_dash_hash,
            Token::<JavaString>::is_token_char(Some(&right_dash), 0).expect("right dash BMP"),
        );
    }

    emit(output, "exhaustive.singleBmpHash", hex(single_hash));
    emit(output, "exhaustive.leftDashBmpHash", hex(left_dash_hash));
    emit(output, "exhaustive.rightDashBmpHash", hex(right_dash_hash));

    let all_bmp_string = JavaString::from_utf16(all_bmp);
    let traced_bmp = TokenParsingTracer::trace(Some(&all_bmp_string)).expect("BMP trace");
    emit(
        output,
        "exhaustive.traceBmpHash",
        hex(hash_string(&traced_bmp)),
    );

    let mut decision_hash = FNV_OFFSET;
    let mut trace_hash = FNV_OFFSET;
    let mut state = 0x4d595df4d0f33173_u64;
    let pool = [
        0x002D, 0x0030, 0x0031, 0x0039, 0x002E, 0x0061, 0x005A, 0x005F, 0x005B, 0x005D, 0x0020,
        0x000A, 0x002B, 0x0023, 0x00B7, 0x00C0, 0x037E, 0x200C, 0xD800, 0xF900, 0xFFFD, 0xFFFF,
    ];
    for _sample in 0..20_000 {
        state = next(state);
        let length = usize::try_from(state >> 60).expect("nibble") + 1;
        let mut units = Vec::with_capacity(length);
        for _ in 0..length {
            state = next(state);
            let index = usize::try_from(state % u64::try_from(pool.len()).expect("pool length"))
                .expect("pool index");
            units.push(pool[index]);
        }
        let context = JavaString::from_utf16(units);
        for position in 0..length {
            decision_hash = mix_boolean(
                decision_hash,
                Token::<JavaString>::is_token_char(
                    Some(&context),
                    i32::try_from(position).expect("position"),
                )
                .expect("generated position"),
            );
        }
        trace_hash = mix_string(
            trace_hash,
            &TokenParsingTracer::trace(Some(&context)).expect("generated trace"),
        );
    }
    emit(output, "exhaustive.contextDecisionHash", hex(decision_hash));
    emit(output, "exhaustive.contextTraceHash", hex(trace_hash));
}

fn emit_trace_cases(output: &mut String) {
    emit(
        output,
        "trace.substitute",
        TokenParsingTracer::TOKEN_SUBSTITUTE,
    );
    emit(
        output,
        "trace.empty",
        TokenParsingTracer::trace(Some(&JavaString::from_rust_str("")))
            .expect("empty trace")
            .to_string_lossy(),
    );
    emit(
        output,
        "trace.mixed",
        TokenParsingTracer::trace(Some(&JavaString::from_rust_str(
            "foo-bar + 12-3 -- .-. ${x}",
        )))
        .expect("mixed trace")
        .to_string_lossy(),
    );
    let utf16_trace = TokenParsingTracer::trace(Some(&JavaString::from_utf16([
        0x00B7, 0x037E, 0xD800, 0xF900,
    ])))
    .expect("UTF-16 trace");
    emit(output, "trace.utf16", to_utf16_hex(&utf16_trace));
}

struct SharedStringProbe {
    shared: JavaString,
}

impl JavaTokenValue for SharedStringProbe {
    fn java_token_to_string(&self) -> Result<JavaTokenStringResult<'_>, TokenError> {
        Ok(JavaTokenStringResult::Borrowed(&self.shared))
    }
}

struct NullStringProbe;

impl JavaTokenValue for NullStringProbe {
    fn java_token_to_string(&self) -> Result<JavaTokenStringResult<'_>, TokenError> {
        Ok(JavaTokenStringResult::Null)
    }
}

struct ThrowingProbe;

impl JavaTokenValue for ThrowingProbe {
    fn java_token_to_string(&self) -> Result<JavaTokenStringResult<'_>, TokenError> {
        Err(TokenError::runtime(
            "java.lang.IllegalStateException",
            "boom",
        ))
    }
}

struct OwnedProbe;

impl JavaTokenValue for OwnedProbe {
    fn java_token_to_string(&self) -> Result<JavaTokenStringResult<'_>, TokenError> {
        Ok(JavaTokenStringResult::Owned(JavaString::from_rust_str(
            "owned",
        )))
    }
}

fn emit_boolean_outcome(output: &mut String, key: &str, result: Result<bool, TokenError>) {
    match result {
        Ok(value) => emit(output, key, format!("OK:{value}")),
        Err(error) => emit(output, key, format!("ERR:{}", error.java_class_name())),
    }
}

fn emit_class_outcome(
    output: &mut String,
    key: &str,
    result: Result<JavaTokenStringResult<'_>, TokenError>,
) {
    match result {
        Ok(value) => emit(output, key, format!("OK:{}", describe_string_result(value))),
        Err(error) => emit(output, key, format!("ERR:{}", error.java_class_name())),
    }
}

fn emit_outcome(
    output: &mut String,
    key: &str,
    result: Result<JavaTokenStringResult<'_>, TokenError>,
) {
    match result {
        Ok(value) => emit(output, key, format!("OK:{}", describe_string_result(value))),
        Err(error) => emit(
            output,
            key,
            format!("ERR:{}:{}", error.java_class_name(), error),
        ),
    }
}

fn describe_string_result(result: JavaTokenStringResult<'_>) -> String {
    match result {
        JavaTokenStringResult::Null => "null".to_owned(),
        JavaTokenStringResult::Borrowed(value) => value.to_string_lossy(),
        JavaTokenStringResult::Owned(value) => value.to_string_lossy(),
    }
}

const fn next(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

const fn mix_boolean(hash: u64, value: bool) -> u64 {
    (hash ^ if value { 1 } else { 0 }).wrapping_mul(FNV_PRIME)
}

fn mix_string(mut hash: u64, value: &JavaString) -> u64 {
    for unit in value.as_utf16() {
        hash = (hash ^ u64::from(unit & 0x00FF)).wrapping_mul(FNV_PRIME);
        hash = (hash ^ u64::from(unit >> 8)).wrapping_mul(FNV_PRIME);
    }
    hash
}

fn hash_string(value: &JavaString) -> u64 {
    mix_string(FNV_OFFSET, value)
}

fn hex(value: u64) -> String {
    format!("{value:016x}")
}

fn to_utf16_hex(value: &JavaString) -> String {
    value
        .as_utf16()
        .iter()
        .map(|unit| format!("{unit:04x}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to String");
}
