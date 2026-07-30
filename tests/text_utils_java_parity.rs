//! `TextUtils` 的 Thymeleaf 3.1.5 Java/Rust UTF-16 Golden 差分测试。

use std::fmt::{Display, Write};
use std::sync::{Arc, RwLock};

use thymeleaf::util::{
    CharArrayWrapperSequence, JavaCharSequence, JavaString, TextUtils, TextUtilsError,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/text_utils_golden.txt");

#[test]
fn text_utils_matches_all_java_overloads_and_utf16_corpora() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_all_overloads(&mut output);
    emit_errors(&mut output);
    emit_dynamic_sequence_traces(&mut output);
    emit_case_fold_digest(&mut output);
    emit_contains_corpus(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

#[test]
#[allow(clippy::too_many_lines)]
fn text_utils_public_failure_and_short_circuit_contracts_are_exhaustive() {
    macro_rules! error {
        ($expression:expr) => {
            assert!($expression.is_err())
        };
    }

    let good = java("a");
    let chars = ['a' as u16];
    let length_failure = LengthFailure;
    let char_failure = CharFailure;
    assert!(length_failure.as_java_string().is_none());
    assert!(char_failure.as_java_string().is_none());

    let string_error = good.java_char_at(-1).unwrap_err();
    assert_eq!(
        string_error.java_class_name(),
        "java.lang.StringIndexOutOfBoundsException"
    );
    assert!(good.java_char_at(1).is_err());
    assert!(!string_error.to_string().is_empty());
    let dynamic = TextUtilsError::SequenceAccess {
        class_name: "example.Failure".to_owned(),
        message: Some(java("failure")),
    };
    assert_eq!(dynamic.java_class_name(), "example.Failure");
    assert_eq!(dynamic.message().unwrap().to_string_lossy(), "failure");
    assert_eq!(dynamic.to_string(), "failure");
    let no_message = TextUtilsError::SequenceAccess {
        class_name: "example.Empty".to_owned(),
        message: None,
    };
    assert_eq!(no_message.message(), None);
    assert_eq!(no_message.to_string(), "");
    assert_eq!(TextUtilsError::NullPointer.message(), None);
    assert_eq!(TextUtilsError::NullPointer.to_string(), "");
    let illegal = TextUtils::equals_chars_range(true, None, 0, 0, Some(&[]), 0, 0).unwrap_err();
    assert_eq!(
        illegal.java_class_name(),
        "java.lang.IllegalArgumentException"
    );
    assert!(illegal.message().is_some());
    let array = TextUtils::hash_chars_range(Some(&[]), 0, 1).unwrap_err();
    assert_eq!(
        array.java_class_name(),
        "java.lang.ArrayIndexOutOfBoundsException"
    );
    assert!(array.message().is_some());

    let shared = Arc::new(RwLock::new(vec!['a' as u16]));
    let wrapper = CharArrayWrapperSequence::with_range(Some(shared), 0, 1).unwrap();
    assert_eq!(wrapper.java_length(), Ok(1));
    assert_eq!(wrapper.java_char_at(0), Ok('a' as u16));
    assert!(wrapper.as_java_string().is_none());
    error!(wrapper.java_char_at(1));

    error!(TextUtils::equals_sequences(false, None, Some(&good)));
    error!(TextUtils::equals_sequences(false, Some(&good), None));
    error!(TextUtils::equals_sequences(
        false,
        Some(&length_failure),
        Some(&good)
    ));
    error!(TextUtils::equals_sequences(
        true,
        Some(&length_failure),
        Some(&good)
    ));
    error!(TextUtils::equals_sequences(
        false,
        Some(&good),
        Some(&length_failure)
    ));
    error!(TextUtils::equals_sequence_and_chars(
        false,
        Some(&good),
        None
    ));
    error!(TextUtils::equals_chars(false, None, Some(&chars)));
    error!(TextUtils::equals_chars(false, Some(&chars), None));
    error!(TextUtils::equals_chars_range(
        false,
        None,
        0,
        1,
        Some(&chars),
        0,
        1
    ));
    error!(TextUtils::equals_chars_range(
        false,
        Some(&chars),
        0,
        1,
        None,
        0,
        1
    ));
    error!(TextUtils::equals_sequence_and_chars_range(
        false,
        None,
        0,
        1,
        Some(&chars),
        0,
        1
    ));
    error!(TextUtils::equals_sequence_and_chars_range(
        false,
        Some(&char_failure),
        0,
        1,
        Some(&chars),
        0,
        1
    ));
    error!(TextUtils::equals_sequence_and_chars_range(
        false,
        Some(&good),
        0,
        1,
        None,
        0,
        1
    ));
    error!(TextUtils::equals_sequences_range(
        false,
        None,
        0,
        1,
        Some(&good),
        0,
        1
    ));
    error!(TextUtils::equals_sequences_range(
        false,
        Some(&good),
        0,
        1,
        None,
        0,
        1
    ));
    error!(TextUtils::equals_sequences_range(
        false,
        Some(&good),
        0,
        1,
        Some(&char_failure),
        0,
        1
    ));
    assert_eq!(
        TextUtils::equals_sequences(true, Some(&good), Some(&good)),
        Ok(true)
    );
    cover_public_success_branches();

    exercise_prefix_failures(&good, &chars, &length_failure, &char_failure);
    exercise_suffix_failures(&good, &chars, &length_failure, &char_failure);
    exercise_contains_failures(&good, &chars, &length_failure, &char_failure);
    exercise_compare_failures(&good, &chars, &length_failure, &char_failure);
    exercise_binary_failures(&good, &chars, &length_failure);

    error!(TextUtils::hash_chars_range(Some(&[]), -1, 1));
    error!(TextUtils::hash_sequence(Some(&length_failure)));
    error!(TextUtils::hash_sequence_range(Some(&length_failure), 0, 1));
    error!(TextUtils::hash_sequence_range(Some(&char_failure), 1, 2));
    error!(TextUtils::hash_pair(None, Some(&good)));
    error!(TextUtils::hash_pair(Some(&good), None));
    error!(TextUtils::hash_triple(None, Some(&good), Some(&good)));
    error!(TextUtils::hash_triple(Some(&good), None, Some(&good)));
    error!(TextUtils::hash_triple(Some(&good), Some(&good), None));
    error!(TextUtils::hash_quadruple(
        None,
        Some(&good),
        Some(&good),
        Some(&good)
    ));
    error!(TextUtils::hash_quadruple(
        Some(&good),
        None,
        Some(&good),
        Some(&good)
    ));
    error!(TextUtils::hash_quadruple(
        Some(&good),
        Some(&good),
        None,
        Some(&good)
    ));
    error!(TextUtils::hash_quadruple(
        Some(&good),
        Some(&good),
        Some(&good),
        None
    ));
    for null_index in 0..5 {
        let values: [Option<&dyn JavaCharSequence>; 5] = [
            (null_index != 0).then_some(&good),
            (null_index != 1).then_some(&good),
            (null_index != 2).then_some(&good),
            (null_index != 3).then_some(&good),
            (null_index != 4).then_some(&good),
        ];
        error!(TextUtils::hash_quintuple(
            values[0], values[1], values[2], values[3], values[4]
        ));
    }
}

#[allow(clippy::too_many_lines)]
fn cover_public_success_branches() {
    let a = java("a");
    let aa = java("aa");
    let c = java("c");
    let e = java("e");
    assert_eq!(
        TextUtils::starts_with_sequences(true, Some(&aa), Some(&a)),
        Ok(true)
    );
    assert_eq!(
        TextUtils::ends_with_sequences(true, Some(&aa), Some(&a)),
        Ok(true)
    );
    assert_eq!(
        TextUtils::contains_sequences(true, Some(&aa), Some(&a)),
        Ok(true)
    );
    let dynamic_a = ProbeSequence::new("a");
    let dynamic_aa = ProbeSequence::new("aa");
    assert_eq!(
        TextUtils::hash_sequence_range(Some(&dynamic_aa), 0, 1),
        Ok('a' as i32)
    );
    assert!(TextUtils::hash_sequence_range(None, 1, 2).is_err());
    assert_eq!(
        TextUtils::starts_with_sequences(true, Some(&aa), Some(&dynamic_a)),
        Ok(true)
    );
    assert_eq!(
        TextUtils::ends_with_sequences(true, Some(&aa), Some(&dynamic_a)),
        Ok(true)
    );
    assert_eq!(
        TextUtils::contains_sequences(true, Some(&aa), Some(&dynamic_a)),
        Ok(true)
    );

    let same = aa.as_utf16();
    assert_eq!(
        TextUtils::equals_chars_range(true, Some(same), 0, 1, Some(same), 0, 0),
        Ok(false)
    );
    assert_eq!(
        TextUtils::equals_chars_range(true, Some(same), 0, 1, Some(same), 1, 1),
        Ok(true)
    );
    assert_eq!(
        TextUtils::equals_chars_range(true, Some(same), 0, 1, Some(same), 0, 1),
        Ok(true)
    );
    assert_eq!(
        TextUtils::equals_sequence_and_chars_range(true, Some(&aa), 0, 1, Some(same), 0, 0,),
        Ok(false)
    );
    assert_eq!(
        TextUtils::equals_sequences_range(true, Some(&aa), 0, 1, Some(&aa), 0, 0),
        Ok(false)
    );
    assert_eq!(
        TextUtils::equals_sequences_range(true, Some(&aa), 0, 1, Some(&aa), 1, 1),
        Ok(true)
    );
    assert_eq!(
        TextUtils::equals_sequences_range(true, Some(&aa), 0, 1, Some(&aa), 0, 1),
        Ok(true)
    );

    assert_eq!(
        TextUtils::ends_with_chars_range(true, Some(&[]), 0, 0, Some(&['a' as u16]), 0, 1),
        Ok(false)
    );
    assert_eq!(
        TextUtils::ends_with_chars_range(
            true,
            Some(&['a' as u16]),
            0,
            1,
            Some(&['b' as u16]),
            0,
            1,
        ),
        Ok(false)
    );
    assert_eq!(
        TextUtils::contains_chars_range(true, Some(&[]), 0, 0, Some(&['a' as u16]), 0, 1),
        Ok(false)
    );
    assert_eq!(
        TextUtils::contains_chars_range(true, Some(&['a' as u16]), 0, 1, Some(&[]), 0, 0),
        Ok(true)
    );
    assert_eq!(
        TextUtils::contains_chars_range(
            true,
            Some(&['a' as u16, 'b' as u16]),
            0,
            2,
            Some(&['z' as u16]),
            0,
            1,
        ),
        Ok(false)
    );

    assert_eq!(
        TextUtils::compare_chars_range(true, Some(same), 0, 1, Some(same), 0, 1),
        Ok(0)
    );
    assert_eq!(
        TextUtils::compare_sequences_range(true, Some(&aa), 0, 1, Some(&aa), 0, 1),
        Ok(0)
    );
    assert_eq!(
        TextUtils::compare_chars(true, Some(&['a' as u16]), Some(&['z' as u16])),
        Ok(-25)
    );
    assert_eq!(
        TextUtils::compare_chars(false, Some(&[0x0130]), Some(&['j' as u16])),
        Ok(-1)
    );

    let char_values = [Some(a.as_utf16()), Some(c.as_utf16()), Some(e.as_utf16())];
    let sequence_values: [Option<&dyn JavaCharSequence>; 3] = [Some(&a), Some(&c), Some(&e)];
    for key in ["0", "b", "c", "d", "z"] {
        let key = java(key);
        let expected = TextUtils::binary_search_chars_values_and_chars(
            true,
            Some(&char_values),
            Some(key.as_utf16()),
            0,
            key.len() as i32,
        )
        .unwrap();
        assert_eq!(
            TextUtils::binary_search_chars_values_and_sequence(
                true,
                Some(&char_values),
                Some(&key),
                0,
                key.len() as i32,
            ),
            Ok(expected)
        );
        assert_eq!(
            TextUtils::binary_search_sequence_values_and_chars(
                true,
                Some(&sequence_values),
                Some(key.as_utf16()),
                0,
                key.len() as i32,
            ),
            Ok(expected)
        );
        assert_eq!(
            TextUtils::binary_search_sequence_values_and_sequence(
                true,
                Some(&sequence_values),
                Some(&key),
                0,
                key.len() as i32,
            ),
            Ok(expected)
        );
    }
}

fn emit_all_overloads(output: &mut String) {
    let text = JavaString::from_rust_str("xxAb\u{0131}\u{03C2}zzyy");
    let fragment = JavaString::from_rust_str("aB\u{0049}\u{03C3}");
    let text_chars = text.as_utf16();
    let fragment_chars = fragment.as_utf16();

    emit_result(
        output,
        "equals.sequence_sequence",
        TextUtils::equals_sequences(false, Some(&java("Ab\u{0131}\u{03C2}")), Some(&fragment)),
    );
    emit_result(
        output,
        "equals.sequence_chars",
        TextUtils::equals_sequence_and_chars(
            false,
            Some(&java("Ab\u{0131}\u{03C2}")),
            Some(fragment_chars),
        ),
    );
    emit_result(
        output,
        "equals.chars_chars",
        TextUtils::equals_chars(
            false,
            Some(java("Ab\u{0131}\u{03C2}").as_utf16()),
            Some(fragment_chars),
        ),
    );
    emit_result(
        output,
        "equals.chars_range",
        TextUtils::equals_chars_range(false, Some(text_chars), 2, 4, Some(fragment_chars), 0, 4),
    );
    emit_result(
        output,
        "equals.sequence_chars_range",
        TextUtils::equals_sequence_and_chars_range(
            false,
            Some(&text),
            2,
            4,
            Some(fragment_chars),
            0,
            4,
        ),
    );
    emit_result(
        output,
        "equals.sequence_sequence_range",
        TextUtils::equals_sequences_range(false, Some(&text), 2, 4, Some(&fragment), 0, 4),
    );

    let starts_text = java("Ab\u{0131}\u{03C2}-tail");
    emit_result(
        output,
        "starts.sequence_sequence",
        TextUtils::starts_with_sequences(false, Some(&starts_text), Some(&fragment)),
    );
    emit_result(
        output,
        "starts.sequence_chars",
        TextUtils::starts_with_sequence_and_chars(false, Some(&starts_text), Some(fragment_chars)),
    );
    emit_result(
        output,
        "starts.chars_chars",
        TextUtils::starts_with_chars(false, Some(starts_text.as_utf16()), Some(fragment_chars)),
    );
    emit_result(
        output,
        "starts.chars_range",
        TextUtils::starts_with_chars_range(
            false,
            Some(text_chars),
            2,
            6,
            Some(fragment_chars),
            0,
            4,
        ),
    );
    emit_result(
        output,
        "starts.sequence_chars_range",
        TextUtils::starts_with_sequence_and_chars_range(
            false,
            Some(&text),
            2,
            6,
            Some(fragment_chars),
            0,
            4,
        ),
    );
    emit_result(
        output,
        "starts.chars_sequence_range",
        TextUtils::starts_with_chars_and_sequence_range(
            false,
            Some(text_chars),
            2,
            6,
            Some(&fragment),
            0,
            4,
        ),
    );
    emit_result(
        output,
        "starts.sequence_sequence_range",
        TextUtils::starts_with_sequences_range(false, Some(&text), 2, 6, Some(&fragment), 0, 4),
    );

    let ends_text = java("head-Ab\u{0131}\u{03C2}");
    emit_result(
        output,
        "ends.sequence_sequence",
        TextUtils::ends_with_sequences(false, Some(&ends_text), Some(&fragment)),
    );
    emit_result(
        output,
        "ends.sequence_chars",
        TextUtils::ends_with_sequence_and_chars(false, Some(&ends_text), Some(fragment_chars)),
    );
    emit_result(
        output,
        "ends.chars_chars",
        TextUtils::ends_with_chars(false, Some(ends_text.as_utf16()), Some(fragment_chars)),
    );
    emit_result(
        output,
        "ends.chars_range",
        TextUtils::ends_with_chars_range(false, Some(text_chars), 0, 6, Some(fragment_chars), 0, 4),
    );
    emit_result(
        output,
        "ends.sequence_chars_range",
        TextUtils::ends_with_sequence_and_chars_range(
            false,
            Some(&text),
            0,
            6,
            Some(fragment_chars),
            0,
            4,
        ),
    );
    emit_result(
        output,
        "ends.chars_sequence_range",
        TextUtils::ends_with_chars_and_sequence_range(
            false,
            Some(text_chars),
            0,
            6,
            Some(&fragment),
            0,
            4,
        ),
    );
    emit_result(
        output,
        "ends.sequence_sequence_range",
        TextUtils::ends_with_sequences_range(false, Some(&text), 0, 6, Some(&fragment), 0, 4),
    );

    emit_result(
        output,
        "contains.sequence_sequence",
        TextUtils::contains_sequences(false, Some(&text), Some(&fragment)),
    );
    emit_result(
        output,
        "contains.sequence_chars",
        TextUtils::contains_sequence_and_chars(false, Some(&text), Some(fragment_chars)),
    );
    emit_result(
        output,
        "contains.chars_chars",
        TextUtils::contains_chars(false, Some(text_chars), Some(fragment_chars)),
    );
    emit_result(
        output,
        "contains.chars_range",
        TextUtils::contains_chars_range(
            false,
            Some(text_chars),
            0,
            text_chars.len() as i32,
            Some(fragment_chars),
            0,
            4,
        ),
    );
    emit_result(
        output,
        "contains.sequence_chars_range",
        TextUtils::contains_sequence_and_chars_range(
            false,
            Some(&text),
            0,
            text.len() as i32,
            Some(fragment_chars),
            0,
            4,
        ),
    );
    emit_result(
        output,
        "contains.chars_sequence_range",
        TextUtils::contains_chars_and_sequence_range(
            false,
            Some(text_chars),
            0,
            text_chars.len() as i32,
            Some(&fragment),
            0,
            4,
        ),
    );
    emit_result(
        output,
        "contains.sequence_sequence_range",
        TextUtils::contains_sequences_range(
            false,
            Some(&text),
            0,
            text.len() as i32,
            Some(&fragment),
            0,
            4,
        ),
    );

    let compared = java("Ab\u{0131}\u{03C2}");
    emit_result(
        output,
        "compare.sequence_sequence",
        TextUtils::compare_sequences(false, Some(&compared), Some(&fragment)),
    );
    emit_result(
        output,
        "compare.sequence_chars",
        TextUtils::compare_sequence_and_chars(false, Some(&compared), Some(fragment_chars)),
    );
    emit_result(
        output,
        "compare.chars_chars",
        TextUtils::compare_chars(false, Some(compared.as_utf16()), Some(fragment_chars)),
    );
    emit_result(
        output,
        "compare.chars_range",
        TextUtils::compare_chars_range(false, Some(text_chars), 2, 4, Some(fragment_chars), 0, 4),
    );
    emit_result(
        output,
        "compare.sequence_chars_range",
        TextUtils::compare_sequence_and_chars_range(
            false,
            Some(&text),
            2,
            4,
            Some(fragment_chars),
            0,
            4,
        ),
    );
    emit_result(
        output,
        "compare.sequence_sequence_range",
        TextUtils::compare_sequences_range(false, Some(&text), 2, 4, Some(&fragment), 0, 4),
    );

    let value_a = java("A");
    let value_ab = java("ab");
    let value_b = java("b");
    let value_z = java("z");
    let char_values = [
        Some(value_a.as_utf16()),
        Some(value_ab.as_utf16()),
        Some(value_b.as_utf16()),
        Some(value_z.as_utf16()),
    ];
    let sequence_values: [Option<&dyn JavaCharSequence>; 4] = [
        Some(&value_a),
        Some(&value_ab),
        Some(&value_b),
        Some(&value_z),
    ];
    let search_chars = java("--AB--");
    let search_sequence = ProbeSequence::new("--AB--");
    emit_result(
        output,
        "binary.char_values_chars",
        TextUtils::binary_search_chars_values_and_chars(
            false,
            Some(&char_values),
            Some(search_chars.as_utf16()),
            2,
            2,
        ),
    );
    emit_result(
        output,
        "binary.char_values_sequence",
        TextUtils::binary_search_chars_values_and_sequence(
            false,
            Some(&char_values),
            Some(&search_sequence),
            2,
            2,
        ),
    );
    emit_result(
        output,
        "binary.sequence_values_chars",
        TextUtils::binary_search_sequence_values_and_chars(
            false,
            Some(&sequence_values),
            Some(search_chars.as_utf16()),
            2,
            2,
        ),
    );
    emit_result(
        output,
        "binary.sequence_values_sequence",
        TextUtils::binary_search_sequence_values_and_sequence(
            false,
            Some(&sequence_values),
            Some(&search_sequence),
            2,
            2,
        ),
    );
    emit_result(
        output,
        "binary.char_values_chars_range",
        TextUtils::binary_search_chars_values_and_chars_range(
            false,
            Some(&char_values),
            1,
            2,
            Some(search_chars.as_utf16()),
            2,
            2,
        ),
    );
    emit_result(
        output,
        "binary.char_values_sequence_range",
        TextUtils::binary_search_chars_values_and_sequence_range(
            false,
            Some(&char_values),
            1,
            2,
            Some(&search_sequence),
            2,
            2,
        ),
    );
    emit_result(
        output,
        "binary.sequence_values_chars_range",
        TextUtils::binary_search_sequence_values_and_chars_range(
            false,
            Some(&sequence_values),
            1,
            2,
            Some(search_chars.as_utf16()),
            2,
            2,
        ),
    );
    emit_result(
        output,
        "binary.sequence_values_sequence_range",
        TextUtils::binary_search_sequence_values_and_sequence_range(
            false,
            Some(&sequence_values),
            1,
            2,
            Some(&search_sequence),
            2,
            2,
        ),
    );

    emit_result(
        output,
        "hash.chars_range",
        TextUtils::hash_chars_range(Some(text_chars), 2, 4),
    );
    emit_result(
        output,
        "hash.sequence",
        TextUtils::hash_sequence(Some(&compared)),
    );
    emit_result(
        output,
        "hash.sequence_range",
        TextUtils::hash_sequence_range(Some(&text), 2, 6),
    );
    emit_result(
        output,
        "hash.pair",
        TextUtils::hash_pair(Some(&java("Ab")), Some(&java("\u{0131}\u{03C2}"))),
    );
    emit_result(
        output,
        "hash.triple",
        TextUtils::hash_triple(
            Some(&java("A")),
            Some(&java("b")),
            Some(&java("\u{0131}\u{03C2}")),
        ),
    );
    emit_result(
        output,
        "hash.quadruple",
        TextUtils::hash_quadruple(
            Some(&java("A")),
            Some(&java("b")),
            Some(&java("\u{0131}")),
            Some(&java("\u{03C2}")),
        ),
    );
    emit_result(
        output,
        "hash.quintuple",
        TextUtils::hash_quintuple(
            Some(&java("")),
            Some(&java("A")),
            Some(&java("b")),
            Some(&java("\u{0131}")),
            Some(&java("\u{03C2}")),
        ),
    );
}

fn emit_errors(output: &mut String) {
    emit_result(
        output,
        "error.equals.short_null_first",
        TextUtils::equals_sequence_and_chars(true, None, Some(&[])),
    );
    emit_result(
        output,
        "error.equals.range_null_first",
        TextUtils::equals_chars_range(true, None, 0, 0, Some(&[]), 0, 0),
    );
    emit_result(
        output,
        "error.starts.short_null_prefix",
        TextUtils::starts_with_sequence_and_chars(true, Some(&java("a")), None),
    );
    emit_result(
        output,
        "error.ends.range_null_suffix",
        TextUtils::ends_with_sequence_and_chars_range(true, Some(&java("a")), 0, 1, None, 0, 0),
    );
    emit_result(
        output,
        "error.contains.range_invalid_text",
        TextUtils::contains_chars_range(true, Some(&[]), 1, 1, Some(&['a' as u16]), 0, 1),
    );
    emit_result(
        output,
        "error.compare.range_invalid_second",
        TextUtils::compare_chars_range(true, Some(&['a' as u16]), 0, 1, Some(&[]), 0, 1),
    );
    emit_result(
        output,
        "error.binary.null_values",
        TextUtils::binary_search_chars_values_and_chars(true, None, Some(&[]), 0, 0),
    );
    let no_char_values: [Option<&[u16]>; 0] = [];
    emit_result(
        output,
        "error.binary.null_text",
        TextUtils::binary_search_chars_values_and_chars_range(
            true,
            Some(&no_char_values),
            0,
            0,
            None,
            0,
            0,
        ),
    );
    let null_mid = [None];
    emit_result(
        output,
        "error.binary.null_mid",
        TextUtils::binary_search_chars_values_and_chars(true, Some(&null_mid), Some(&[]), 0, 0),
    );
    emit_result(
        output,
        "error.binary.outer_index",
        TextUtils::binary_search_chars_values_and_chars_range(
            true,
            Some(&no_char_values),
            1,
            1,
            Some(&[]),
            0,
            0,
        ),
    );
    emit_result(
        output,
        "error.hash.null_chars_empty",
        TextUtils::hash_chars_range(None, 0, 0),
    );
    emit_result(
        output,
        "error.hash.null_chars_one",
        TextUtils::hash_chars_range(None, 0, 1),
    );
    emit_result(
        output,
        "error.hash.null_sequence_empty_range",
        TextUtils::hash_sequence_range(None, 1, 1),
    );
    emit_result(
        output,
        "error.hash.null_sequence_zero_range",
        TextUtils::hash_sequence_range(None, 0, 0),
    );
}

fn emit_dynamic_sequence_traces(output: &mut String) {
    let left = ProbeSequence::new("Ab");
    let right = ProbeSequence::new("aB");
    emit_result(
        output,
        "trace.equals.result",
        TextUtils::equals_sequences(false, Some(&left), Some(&right)),
    );
    emit(output, "trace.equals.left", &left.trace());
    emit(output, "trace.equals.right", &right.trace());

    let hash = ProbeSequence::new("Ab");
    emit_result(
        output,
        "trace.hash.result",
        TextUtils::hash_sequence(Some(&hash)),
    );
    emit(output, "trace.hash.calls", &hash.trace());

    let suffix_text = ProbeSequence::new("xxAb");
    let suffix = ProbeSequence::new("ab");
    emit_result(
        output,
        "trace.ends.result",
        TextUtils::ends_with_sequences(false, Some(&suffix_text), Some(&suffix)),
    );
    emit(output, "trace.ends.text", &suffix_text.trace());
    emit(output, "trace.ends.suffix", &suffix.trace());
}

fn emit_case_fold_digest(output: &mut String) {
    let mut digest = 0xcbf29ce484222325_u64;
    for source in u16::MIN..=u16::MAX {
        let upper = java_case_map(source, true);
        let lower = java_case_map(source, false);
        digest = mix(
            digest,
            bool_i32(TextUtils::equals_chars(false, Some(&[source]), Some(&[upper])).unwrap()),
        );
        digest = mix(
            digest,
            bool_i32(TextUtils::equals_chars(false, Some(&[source]), Some(&[lower])).unwrap()),
        );
        digest = mix(
            digest,
            TextUtils::compare_chars(false, Some(&[source]), Some(&[upper])).unwrap(),
        );
        digest = mix(
            digest,
            TextUtils::compare_chars(false, Some(&[source]), Some(&[lower])).unwrap(),
        );
    }
    emit(output, "digest.case_fold", &format!("{digest:x}"));
}

fn emit_contains_corpus(output: &mut String) {
    let texts = [
        java(""),
        java("a"),
        java("aa"),
        java("aab"),
        java("ababab"),
        java("mississippi"),
        java("\u{0131}I\u{03C2}\u{03A3}"),
        JavaString::from_utf16(vec![0xD800, 'x' as u16, 0xDC00]),
        java("0123456789"),
    ];
    let fragments = [
        java(""),
        java("a"),
        java("ab"),
        java("aba"),
        java("issi"),
        java("ssip"),
        java("I\u{03A3}"),
        JavaString::from_utf16(vec![0xD800, 'x' as u16]),
        JavaString::from_utf16(vec![0xDC00]),
        java("xyz"),
    ];
    let mut digest = 0xcbf29ce484222325_u64;
    let mut cases = 0_i32;
    for case_sensitive in [false, true] {
        for text in &texts {
            for fragment in &fragments {
                let expected =
                    TextUtils::contains_sequences(case_sensitive, Some(text), Some(fragment))
                        .unwrap();
                let mut padded_text = vec!['#' as u16];
                padded_text.extend_from_slice(text.as_utf16());
                padded_text.push('!' as u16);
                let mut padded_fragment = vec!['#' as u16];
                padded_fragment.extend_from_slice(fragment.as_utf16());
                padded_fragment.push('!' as u16);
                let ranged = TextUtils::contains_chars_range(
                    case_sensitive,
                    Some(&padded_text),
                    1,
                    text.len() as i32,
                    Some(&padded_fragment),
                    1,
                    fragment.len() as i32,
                )
                .unwrap();
                digest = mix(digest, bool_i32(expected));
                digest = mix(digest, bool_i32(ranged));
                cases += 2;
            }
        }
    }
    emit(output, "digest.contains", &format!("{digest:x}"));
    emit(output, "digest.contains_cases", &cases.to_string());
}

fn java(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn java_case_map(source: u16, upper: bool) -> u16 {
    // Golden 的完整 BMP 输入本身同时验证生产实现；这里用已固定的 JDK 21 数据
    // 构造 Java 侧传入的 upper/lower 参数，不调用 Rust Unicode 大小写 API。
    const DATA: &[u8] = include_bytes!("../src/util/text_utils_case_map.bin");
    let upper_count = usize::from(u16::from_be_bytes([DATA[6], DATA[7]]));
    let upper_start = 8;
    let lower_count_offset = upper_start + upper_count * 4;
    let (start, count) = if upper {
        (upper_start, upper_count)
    } else {
        (
            lower_count_offset + 2,
            usize::from(u16::from_be_bytes([
                DATA[lower_count_offset],
                DATA[lower_count_offset + 1],
            ])),
        )
    };
    let mut low = 0;
    let mut high = count;
    while low < high {
        let mid = low + (high - low) / 2;
        let offset = start + mid * 4;
        let candidate = u16::from_be_bytes([DATA[offset], DATA[offset + 1]]);
        if candidate < source {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    if low < count {
        let offset = start + low * 4;
        if u16::from_be_bytes([DATA[offset], DATA[offset + 1]]) == source {
            return u16::from_be_bytes([DATA[offset + 2], DATA[offset + 3]]);
        }
    }
    source
}

fn mix(hash: u64, value: i32) -> u64 {
    (hash ^ u64::from(value as u32)).wrapping_mul(0x100000001b3)
}

fn bool_i32(value: bool) -> i32 {
    i32::from(value)
}

fn emit_result<T: Display>(output: &mut String, key: &str, result: Result<T, TextUtilsError>) {
    match result {
        Ok(value) => emit(output, key, &value.to_string()),
        Err(error) => {
            let message = if error.java_class_name() == "java.lang.NullPointerException" {
                "<ignored>".to_owned()
            } else {
                error
                    .message()
                    .map_or_else(|| "<null>".to_owned(), |message| encode(message.as_utf16()))
            };
            emit(
                output,
                key,
                &format!("{}|{message}", error.java_class_name()),
            );
        }
    }
}

fn encode(value: &[u16]) -> String {
    value
        .iter()
        .map(|unit| format!("{unit:04X}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn emit(output: &mut String, key: &str, value: &str) {
    writeln!(output, "{key}={value}").unwrap();
}

fn exercise_prefix_failures(
    good: &JavaString,
    chars: &[u16],
    length_failure: &LengthFailure,
    char_failure: &CharFailure,
) {
    assert!(TextUtils::starts_with_sequences(false, None, Some(good)).is_err());
    assert!(TextUtils::starts_with_sequences(false, Some(good), None).is_err());
    assert!(TextUtils::starts_with_sequences(false, Some(length_failure), Some(good)).is_err());
    assert!(TextUtils::starts_with_sequences(false, Some(good), Some(length_failure)).is_err());
    assert!(TextUtils::starts_with_sequence_and_chars(false, None, Some(chars)).is_err());
    assert!(TextUtils::starts_with_sequence_and_chars(false, Some(good), None).is_err());
    assert!(TextUtils::starts_with_chars(false, None, Some(chars)).is_err());
    assert!(TextUtils::starts_with_chars(false, Some(chars), None).is_err());
    assert!(TextUtils::starts_with_chars_range(false, None, 0, 1, Some(chars), 0, 1).is_err());
    assert!(TextUtils::starts_with_chars_range(false, Some(chars), 0, 1, None, 0, 1).is_err());
    assert!(
        TextUtils::starts_with_sequence_and_chars_range(
            false,
            Some(char_failure),
            0,
            1,
            Some(chars),
            0,
            1
        )
        .is_err()
    );
    assert!(
        TextUtils::starts_with_chars_and_sequence_range(
            false,
            Some(chars),
            0,
            1,
            Some(char_failure),
            0,
            1
        )
        .is_err()
    );
    assert!(TextUtils::starts_with_sequences_range(false, None, 0, 1, Some(good), 0, 1).is_err());
    assert!(TextUtils::starts_with_sequences_range(false, Some(good), 0, 1, None, 0, 1).is_err());
    assert_eq!(
        TextUtils::starts_with_chars_range(false, Some(chars), 0, 0, Some(chars), 0, 1),
        Ok(false)
    );
}

fn exercise_suffix_failures(
    good: &JavaString,
    chars: &[u16],
    length_failure: &LengthFailure,
    char_failure: &CharFailure,
) {
    assert!(TextUtils::ends_with_sequences(false, None, Some(good)).is_err());
    assert!(TextUtils::ends_with_sequences(false, Some(good), None).is_err());
    assert!(TextUtils::ends_with_sequences(false, Some(length_failure), Some(good)).is_err());
    assert!(TextUtils::ends_with_sequences(false, Some(good), Some(length_failure)).is_err());
    assert!(TextUtils::ends_with_sequence_and_chars(false, None, Some(chars)).is_err());
    assert!(TextUtils::ends_with_sequence_and_chars(false, Some(good), None).is_err());
    assert!(TextUtils::ends_with_chars(false, None, Some(chars)).is_err());
    assert!(TextUtils::ends_with_chars(false, Some(chars), None).is_err());
    assert!(TextUtils::ends_with_chars_range(false, None, 0, 1, Some(chars), 0, 1).is_err());
    assert!(TextUtils::ends_with_chars_range(false, Some(chars), 0, 1, None, 0, 1).is_err());
    assert!(
        TextUtils::ends_with_sequence_and_chars_range(
            false,
            Some(char_failure),
            0,
            1,
            Some(chars),
            0,
            1
        )
        .is_err()
    );
    assert!(
        TextUtils::ends_with_chars_and_sequence_range(
            false,
            Some(chars),
            0,
            1,
            Some(char_failure),
            0,
            1
        )
        .is_err()
    );
    assert!(TextUtils::ends_with_sequences_range(false, None, 0, 1, Some(good), 0, 1).is_err());
    assert!(TextUtils::ends_with_sequences_range(false, Some(good), 0, 1, None, 0, 1).is_err());
    assert_eq!(
        TextUtils::ends_with_chars_range(false, Some(chars), 0, 0, Some(chars), 0, 1),
        Ok(false)
    );
}

fn exercise_contains_failures(
    good: &JavaString,
    chars: &[u16],
    length_failure: &LengthFailure,
    char_failure: &CharFailure,
) {
    assert!(TextUtils::contains_sequences(false, None, Some(good)).is_err());
    assert!(TextUtils::contains_sequences(false, Some(good), None).is_err());
    assert!(TextUtils::contains_sequences(false, Some(length_failure), Some(good)).is_err());
    assert!(TextUtils::contains_sequences(false, Some(good), Some(length_failure)).is_err());
    assert!(TextUtils::contains_sequence_and_chars(false, None, Some(chars)).is_err());
    assert!(TextUtils::contains_sequence_and_chars(false, Some(good), None).is_err());
    assert!(TextUtils::contains_chars(false, None, Some(chars)).is_err());
    assert!(TextUtils::contains_chars(false, Some(chars), None).is_err());
    assert!(TextUtils::contains_chars_range(false, None, 0, 1, Some(chars), 0, 1).is_err());
    assert!(TextUtils::contains_chars_range(false, Some(chars), 0, 1, None, 0, 1).is_err());
    assert!(
        TextUtils::contains_sequence_and_chars_range(
            false,
            Some(char_failure),
            0,
            1,
            Some(chars),
            0,
            1
        )
        .is_err()
    );
    assert!(
        TextUtils::contains_chars_and_sequence_range(
            false,
            Some(chars),
            0,
            1,
            Some(char_failure),
            0,
            1
        )
        .is_err()
    );
    assert!(TextUtils::contains_sequences_range(false, None, 0, 1, Some(good), 0, 1).is_err());
    assert!(TextUtils::contains_sequences_range(false, Some(good), 0, 1, None, 0, 1).is_err());
}

fn exercise_compare_failures(
    good: &JavaString,
    chars: &[u16],
    length_failure: &LengthFailure,
    char_failure: &CharFailure,
) {
    assert!(TextUtils::compare_sequences(false, None, Some(good)).is_err());
    assert!(TextUtils::compare_sequences(false, Some(good), None).is_err());
    assert!(TextUtils::compare_sequences(false, Some(length_failure), Some(good)).is_err());
    assert!(TextUtils::compare_sequences(false, Some(good), Some(length_failure)).is_err());
    assert!(TextUtils::compare_sequence_and_chars(false, None, Some(chars)).is_err());
    assert!(TextUtils::compare_sequence_and_chars(false, Some(good), None).is_err());
    assert!(TextUtils::compare_chars(false, None, Some(chars)).is_err());
    assert!(TextUtils::compare_chars(false, Some(chars), None).is_err());
    assert!(TextUtils::compare_chars_range(false, None, 0, 1, Some(chars), 0, 1).is_err());
    assert!(TextUtils::compare_chars_range(false, Some(chars), 0, 1, None, 0, 1).is_err());
    assert!(
        TextUtils::compare_sequence_and_chars_range(
            false,
            Some(char_failure),
            0,
            1,
            Some(chars),
            0,
            1
        )
        .is_err()
    );
    assert!(TextUtils::compare_sequences_range(false, None, 0, 1, Some(good), 0, 1).is_err());
    assert!(TextUtils::compare_sequences_range(false, Some(good), 0, 1, None, 0, 1).is_err());
    assert!(
        TextUtils::compare_sequences_range(false, Some(good), 0, 1, Some(char_failure), 0, 1)
            .is_err()
    );
}

fn exercise_binary_failures(good: &JavaString, chars: &[u16], length_failure: &LengthFailure) {
    let char_values = [Some(chars)];
    let sequence_values: [Option<&dyn JavaCharSequence>; 1] = [Some(good)];
    assert!(
        TextUtils::binary_search_chars_values_and_chars_range(false, None, 0, 1, Some(chars), 0, 1)
            .is_err()
    );
    assert!(
        TextUtils::binary_search_chars_values_and_sequence_range(
            false,
            None,
            0,
            1,
            Some(good),
            0,
            1
        )
        .is_err()
    );
    assert!(
        TextUtils::binary_search_sequence_values_and_chars_range(
            false,
            None,
            0,
            1,
            Some(chars),
            0,
            1
        )
        .is_err()
    );
    assert!(
        TextUtils::binary_search_sequence_values_and_sequence_range(
            false,
            None,
            0,
            1,
            Some(good),
            0,
            1
        )
        .is_err()
    );
    let empty_chars: [Option<&[u16]>; 0] = [];
    assert!(
        TextUtils::binary_search_chars_values_and_sequence_range(
            false,
            Some(&empty_chars),
            1,
            1,
            Some(good),
            0,
            1
        )
        .is_err()
    );
    let null_chars = [None];
    assert!(
        TextUtils::binary_search_chars_values_and_sequence_range(
            false,
            Some(&null_chars),
            0,
            1,
            Some(good),
            0,
            1
        )
        .is_err()
    );
    let empty_sequences: [Option<&dyn JavaCharSequence>; 0] = [];
    assert!(
        TextUtils::binary_search_sequence_values_and_chars_range(
            false,
            Some(&empty_sequences),
            1,
            1,
            Some(chars),
            0,
            1
        )
        .is_err()
    );
    let null_sequences = [None];
    assert!(
        TextUtils::binary_search_sequence_values_and_sequence_range(
            false,
            Some(&null_sequences),
            0,
            1,
            Some(good),
            0,
            1
        )
        .is_err()
    );
    let failing: [Option<&dyn JavaCharSequence>; 1] = [Some(length_failure)];
    assert!(
        TextUtils::binary_search_sequence_values_and_chars(
            false,
            Some(&failing),
            Some(chars),
            0,
            1
        )
        .is_err()
    );
    assert!(
        TextUtils::binary_search_sequence_values_and_sequence(
            false,
            Some(&failing),
            Some(good),
            0,
            1
        )
        .is_err()
    );
    assert!(
        TextUtils::binary_search_chars_values_and_chars_range(
            false,
            Some(&char_values),
            0,
            1,
            Some(&[]),
            1,
            1
        )
        .is_err()
    );
    assert!(
        TextUtils::binary_search_chars_values_and_sequence_range(
            false,
            Some(&char_values),
            0,
            1,
            Some(good),
            1,
            1
        )
        .is_err()
    );
    assert!(
        TextUtils::binary_search_sequence_values_and_chars_range(
            false,
            Some(&sequence_values),
            0,
            1,
            Some(&[]),
            1,
            1
        )
        .is_err()
    );
    assert!(
        TextUtils::binary_search_sequence_values_and_sequence_range(
            false,
            Some(&sequence_values),
            0,
            1,
            Some(good),
            1,
            1
        )
        .is_err()
    );
}

struct LengthFailure;

impl JavaCharSequence for LengthFailure {
    fn java_length(&self) -> Result<i32, TextUtilsError> {
        Err(TextUtilsError::SequenceAccess {
            class_name: "example.LengthFailure".to_owned(),
            message: None,
        })
    }

    fn java_char_at(&self, _index: i32) -> Result<u16, TextUtilsError> {
        Err(TextUtilsError::SequenceAccess {
            class_name: "example.LengthFailure".to_owned(),
            message: None,
        })
    }

    fn as_java_string(&self) -> Option<&JavaString> {
        None
    }
}

struct CharFailure;

impl JavaCharSequence for CharFailure {
    fn java_length(&self) -> Result<i32, TextUtilsError> {
        Ok(1)
    }

    fn java_char_at(&self, _index: i32) -> Result<u16, TextUtilsError> {
        Err(TextUtilsError::SequenceAccess {
            class_name: "example.CharFailure".to_owned(),
            message: None,
        })
    }

    fn as_java_string(&self) -> Option<&JavaString> {
        None
    }
}

struct ProbeSequence {
    value: JavaString,
    trace: std::sync::Mutex<String>,
}

impl ProbeSequence {
    fn new(value: &str) -> Self {
        Self {
            value: java(value),
            trace: std::sync::Mutex::new(String::new()),
        }
    }

    fn trace(&self) -> String {
        self.trace.lock().expect("trace lock").clone()
    }
}

impl JavaCharSequence for ProbeSequence {
    fn java_length(&self) -> Result<i32, TextUtilsError> {
        self.trace.lock().expect("trace lock").push_str("L;");
        Ok(self.value.len() as i32)
    }

    fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
        write!(self.trace.lock().expect("trace lock"), "C{index};").unwrap();
        self.value.java_char_at(index)
    }

    fn as_java_string(&self) -> Option<&JavaString> {
        None
    }
}
