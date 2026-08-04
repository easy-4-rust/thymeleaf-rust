//! `CharArrayWrapperSequence` 与固定 Thymeleaf Java 基线的逐记录语义对照测试。

use std::fmt::Write;
use std::sync::{Arc, RwLock};

use thymeleaf::util::{
    CharArrayWrapperSequence, CharArrayWrapperSequenceError, SharedCharArray, Utf16String,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/char_array_wrapper_sequence_golden.txt");
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x100_0000_01b3;

#[test]
fn char_array_wrapper_sequence_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    constructor_cases(&mut output);
    mutation_and_clone_cases(&mut output);
    access_cases(&mut output);
    subsequence_cases(&mut output);
    equality_hash_and_string_cases(&mut output);
    exhaustive_cases(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn constructor_cases(output: &mut String) {
    emit_sequence(
        output,
        "constructor.full",
        CharArrayWrapperSequence::new(Some(chars())),
    );
    emit_sequence(
        output,
        "constructor.null",
        CharArrayWrapperSequence::new(None),
    );
    emit_sequence(
        output,
        "constructor.empty",
        CharArrayWrapperSequence::new(Some(shared(Vec::new()))),
    );
    emit_sequence(
        output,
        "constructor.range",
        CharArrayWrapperSequence::with_range(Some(chars()), 1, 2),
    );
    emit_sequence(
        output,
        "constructor.zero",
        CharArrayWrapperSequence::with_range(Some(chars()), 3, 0),
    );
    emit_sequence(
        output,
        "constructor.negativeLength",
        CharArrayWrapperSequence::with_range(Some(chars()), 1, -2),
    );
    emit_result(
        output,
        "constructor.negativeLengthStored",
        CharArrayWrapperSequence::with_range(Some(chars()), 1, -2)
            .map(|sequence| sequence.length().to_string()),
    );
    emit_sequence(
        output,
        "constructor.negativeOffset",
        CharArrayWrapperSequence::with_range(Some(chars()), -1, 1),
    );
    emit_sequence(
        output,
        "constructor.offsetAtEnd",
        CharArrayWrapperSequence::with_range(Some(chars()), 4, 0),
    );
    emit_sequence(
        output,
        "constructor.long",
        CharArrayWrapperSequence::with_range(Some(chars()), 2, 3),
    );
    emit_sequence(
        output,
        "constructor.overflow",
        CharArrayWrapperSequence::with_range(Some(chars()), 1, i32::MAX),
    );
    emit_result(
        output,
        "constructor.overflowStored",
        CharArrayWrapperSequence::with_range(Some(chars()), 1, i32::MAX)
            .map(|sequence| sequence.length().to_string()),
    );
    emit_sequence(
        output,
        "constructor.minimumLength",
        CharArrayWrapperSequence::with_range(Some(chars()), 1, i32::MIN),
    );
    emit_result(
        output,
        "constructor.minimumLengthStored",
        CharArrayWrapperSequence::with_range(Some(chars()), 1, i32::MIN)
            .map(|sequence| sequence.length().to_string()),
    );
    emit_sequence(
        output,
        "constructor.nullRange",
        CharArrayWrapperSequence::with_range(None, -1, i32::MAX),
    );
}

fn mutation_and_clone_cases(output: &mut String) {
    let buffer = chars();
    let sequence =
        CharArrayWrapperSequence::with_range(Some(Arc::clone(&buffer)), 1, 2).expect("view");
    let clone = sequence.clone();
    emit(output, "clone.distinct", !std::ptr::eq(&clone, &sequence));
    emit(output, "clone.equals", clone == sequence);
    emit(
        output,
        "clone.hash",
        clone.hash_code() == sequence.hash_code(),
    );
    buffer.write().expect("write lock")[1] = 0x005A;
    emit(
        output,
        "mutation.original",
        describe(&sequence).expect("valid"),
    );
    emit(output, "mutation.clone", describe(&clone).expect("valid"));
    emit(output, "mutation.equals", clone == sequence);
}

fn access_cases(output: &mut String) {
    let sequence =
        CharArrayWrapperSequence::with_range(Some(chars()), 1, 2).expect("valid sequence");
    emit_char_at(output, "charAt.zero", &sequence, 0);
    emit_char_at(output, "charAt.one", &sequence, 1);
    emit_char_at(output, "charAt.negative", &sequence, -1);
    emit_char_at(output, "charAt.atLength", &sequence, 2);

    let overflow =
        CharArrayWrapperSequence::with_range(Some(chars()), 1, i32::MAX).expect("overflow view");
    emit_char_at(output, "charAt.overflowViewZero", &overflow, 0);
    emit_char_at(output, "charAt.overflowViewLast", &overflow, i32::MAX - 1);
    let negative_overflow = CharArrayWrapperSequence::with_range(Some(chars()), 2, i32::MAX)
        .expect("negative overflow view");
    emit_char_at(
        output,
        "charAt.overflowViewNegativeIndex",
        &negative_overflow,
        i32::MAX - 1,
    );

    let negative =
        CharArrayWrapperSequence::with_range(Some(chars()), 1, -2).expect("negative view");
    emit_char_at(output, "charAt.negativeView", &negative, 0);
}

fn subsequence_cases(output: &mut String) {
    let sequence = CharArrayWrapperSequence::new(Some(chars())).expect("sequence");
    emit_sequence(output, "sub.full", sequence.sub_sequence(0, 4));
    emit_sequence(output, "sub.middle", sequence.sub_sequence(1, 3));
    emit_sequence(output, "sub.zeroAtStart", sequence.sub_sequence(0, 0));
    emit_sequence(output, "sub.zeroAtLast", sequence.sub_sequence(3, 3));
    emit_sequence(output, "sub.zeroAtEnd", sequence.sub_sequence(4, 4));
    emit_sequence(output, "sub.negativeStart", sequence.sub_sequence(-1, 1));
    emit_sequence(output, "sub.endAfter", sequence.sub_sequence(1, 5));
    emit_sequence(output, "sub.reversed", sequence.sub_sequence(2, 1));
    emit_result(
        output,
        "sub.reversedLength",
        sequence
            .sub_sequence(2, 1)
            .map(|subsequence| subsequence.length().to_string()),
    );
    emit_sequence(output, "sub.negativeEnd", sequence.sub_sequence(1, -1));
    emit_result(
        output,
        "sub.negativeEndLength",
        sequence
            .sub_sequence(1, -1)
            .map(|subsequence| subsequence.length().to_string()),
    );
}

fn equality_hash_and_string_cases(output: &mut String) {
    let first_buffer = chars();
    let first =
        CharArrayWrapperSequence::with_range(Some(Arc::clone(&first_buffer)), 1, 2).expect("first");
    let same_content = CharArrayWrapperSequence::with_range(
        Some(shared(vec![0x0058, 0xD800, 0x0042, 0x0059])),
        1,
        2,
    )
    .expect("same");
    let different_content = CharArrayWrapperSequence::with_range(
        Some(shared(vec![0x0058, 0xD800, 0x0043, 0x0059])),
        1,
        2,
    )
    .expect("different");
    let different_length =
        CharArrayWrapperSequence::with_range(Some(first_buffer), 1, 1).expect("length");
    emit(output, "equals.identity", first.equals_object(Some(&first)));
    emit(output, "equals.null", first.equals_object(None));
    emit(
        output,
        "equals.string",
        first.equals_object(Some(&first.to_utf16_string().expect("string"))),
    );
    emit(
        output,
        "equals.sameContent",
        first.equals_object(Some(&same_content)),
    );
    emit(
        output,
        "equals.differentContent",
        first.equals_object(Some(&different_content)),
    );
    emit(
        output,
        "equals.differentLength",
        first.equals_object(Some(&different_length)),
    );
    emit(output, "hash.first", first.hash_code());
    emit(output, "hash.sameContent", same_content.hash_code());
    emit(
        output,
        "hash.stringCompatible",
        first.hash_code() == utf16_string_hash(&first.to_utf16_string().expect("string")),
    );
    emit(
        output,
        "toString.first",
        to_utf16_hex(&first.to_utf16_string().expect("string")),
    );

    let negative = CharArrayWrapperSequence::with_range(Some(chars()), 1, -2).expect("negative");
    emit_result(
        output,
        "negative.hash",
        Ok(negative.hash_code().to_string()),
    );
    emit_result(
        output,
        "negative.toString",
        negative
            .to_utf16_string()
            .map(|value| value.to_string_lossy()),
    );

    let overflow =
        CharArrayWrapperSequence::with_range(Some(chars()), 1, i32::MAX).expect("overflow");
    emit_result(
        output,
        "overflow.hash",
        Ok(overflow.hash_code().to_string()),
    );
    emit_result(
        output,
        "overflow.toString",
        overflow
            .to_utf16_string()
            .map(|value| value.to_string_lossy()),
    );
}

fn exhaustive_cases(output: &mut String) {
    let special_lengths = [-2, -1, 0, 1, 2, 3, 4, 5, i32::MAX, i32::MIN];
    let mut constructor_hash = FNV_OFFSET;
    for size in 0..=6 {
        let buffer = (0..size)
            .map(|index| 0xD7FE_u16.wrapping_add(index as u16))
            .collect::<Vec<_>>();
        for offset in -2..=8 {
            for length in special_lengths {
                match CharArrayWrapperSequence::with_range(
                    Some(shared(buffer.clone())),
                    offset,
                    length,
                ) {
                    Ok(value) => {
                        constructor_hash = mix(constructor_hash, 1);
                        constructor_hash = mix(constructor_hash, value.length());
                        constructor_hash = mix(constructor_hash, value.hash_code());
                        constructor_hash = match value.to_utf16_string() {
                            Ok(string) => mix_string(constructor_hash, &string),
                            Err(error) => mix_error(constructor_hash, &error),
                        };
                    }
                    Err(error) => {
                        constructor_hash = mix(constructor_hash, 0);
                        constructor_hash = mix_error(constructor_hash, &error);
                    }
                }
            }
        }
    }
    emit(
        output,
        "exhaustive.constructorHash",
        format!("{constructor_hash:016x}"),
    );

    let sequence = CharArrayWrapperSequence::new(Some(chars())).expect("sequence");
    let mut subsequence_hash = FNV_OFFSET;
    for start in -2..=7 {
        for end in -2..=7 {
            match sequence.sub_sequence(start, end) {
                Ok(subsequence) => {
                    subsequence_hash = mix(subsequence_hash, 1);
                    subsequence_hash = mix(subsequence_hash, subsequence.length());
                    subsequence_hash = mix(subsequence_hash, subsequence.hash_code());
                    subsequence_hash = match subsequence.to_utf16_string() {
                        Ok(string) => mix_string(subsequence_hash, &string),
                        Err(error) => mix_error(subsequence_hash, &error),
                    };
                }
                Err(error) => {
                    subsequence_hash = mix(subsequence_hash, 0);
                    subsequence_hash = mix_error(subsequence_hash, &error);
                }
            }
        }
    }
    emit(
        output,
        "exhaustive.subsequenceHash",
        format!("{subsequence_hash:016x}"),
    );
}

fn emit_char_at(output: &mut String, key: &str, sequence: &CharArrayWrapperSequence, index: i32) {
    emit_result(
        output,
        key,
        sequence.char_at(index).map(|unit| format!("{unit:04x}")),
    );
}

fn emit_sequence(
    output: &mut String,
    key: &str,
    result: Result<CharArrayWrapperSequence, CharArrayWrapperSequenceError>,
) {
    emit_result(output, key, result.and_then(|sequence| describe(&sequence)));
}

fn describe(sequence: &CharArrayWrapperSequence) -> Result<String, CharArrayWrapperSequenceError> {
    Ok(format!(
        "{}:{}:{}",
        sequence.length(),
        sequence.hash_code(),
        to_utf16_hex(&sequence.to_utf16_string()?)
    ))
}

fn emit_result(
    output: &mut String,
    key: &str,
    result: Result<String, CharArrayWrapperSequenceError>,
) {
    match result {
        Ok(value) => emit(output, key, format!("OK:{value}")),
        Err(error) => emit(
            output,
            key,
            format!(
                "ERR:{}:{}",
                error.java_class_name(),
                to_utf16_hex(&error.message())
            ),
        ),
    }
}

fn chars() -> SharedCharArray {
    shared(vec![0x0041, 0xD800, 0x0042, 0x0043])
}

fn shared(value: Vec<u16>) -> SharedCharArray {
    Arc::new(RwLock::new(value))
}

fn to_utf16_hex(value: &Utf16String) -> String {
    value
        .as_utf16()
        .iter()
        .map(|unit| format!("{unit:04x}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn utf16_string_hash(value: &Utf16String) -> i32 {
    value.as_utf16().iter().fold(0_i32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(i32::from(*unit))
    })
}

fn mix_error(hash: u64, error: &CharArrayWrapperSequenceError) -> u64 {
    let hash = mix_rust_str(hash, error.java_class_name());
    mix_string(hash, &error.message())
}

fn mix_rust_str(hash: u64, value: &str) -> u64 {
    mix_string(hash, &Utf16String::from_rust_str(value))
}

fn mix_string(mut hash: u64, value: &Utf16String) -> u64 {
    for unit in value.as_utf16() {
        hash = mix(hash, i32::from(unit & 0x00FF));
        hash = mix(hash, i32::from(unit >> 8));
    }
    hash
}

fn mix(hash: u64, value: i32) -> u64 {
    (hash ^ value as u64).wrapping_mul(FNV_PRIME)
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to string");
}
