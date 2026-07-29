//! `EvaluationUtils`/`Bools` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::Write;
use std::sync::Arc;

use num_bigint::BigInt;
use thymeleaf::expression::{Bools, JavaObjectArray};
use thymeleaf::util::{
    EvaluationError, EvaluationUtils, JavaBigDecimal, JavaBigDecimalResult, JavaEvaluationArray,
    JavaEvaluationElement, JavaEvaluationList, JavaEvaluationListType, JavaEvaluationTarget,
    JavaEvaluationValue, JavaHashCode, JavaMapEntry, JavaNumber, JavaString,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/evaluation_utils_golden.txt");

#[test]
fn evaluation_utils_and_bools_match_java_golden() {
    cover_public_adapter_contracts();
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_boolean_cases(&mut output);
    emit_number_cases(&mut output);
    emit_collection_cases(&mut output);
    emit_bools_cases(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn cover_public_adapter_contracts() {
    let decimal_value = decimal("1.0");
    let borrowed_source =
        JavaEvaluationValue::Number(JavaNumber::BigDecimal(decimal_value.clone()));
    let borrowed = EvaluationUtils::evaluate_as_number(&borrowed_source)
        .expect("decimal conversion")
        .expect("decimal result");
    assert!(!borrowed.is_borrowed_from(&decimal_value));
    let JavaEvaluationValue::Number(JavaNumber::BigDecimal(source_decimal)) = &borrowed_source
    else {
        panic!("decimal source")
    };
    assert!(borrowed.is_borrowed_from(source_decimal));
    let owned_source = number(JavaNumber::Integer(1));
    let owned = EvaluationUtils::evaluate_as_number(&owned_source)
        .expect("integer conversion")
        .expect("integer result");
    assert!(!owned.is_borrowed_from(source_decimal));

    assert_eq!(7_i32.java_hash_code(), 7);
    assert_eq!("Aa".to_owned().java_hash_code(), 2_112);
    assert_eq!(JavaString::from_rust_str("Aa").java_hash_code(), 2_112);

    let mut null_entry = JavaMapEntry::<String>::new(None, None);
    assert_eq!(null_entry.get_key(), None);
    assert_eq!(null_entry.get_value(), None);
    assert_eq!(null_entry.to_string(), "null=null");
    assert_eq!(null_entry.java_hash_code(), 0);
    let unsupported = null_entry
        .set_value(Some("ignored".to_owned()))
        .expect_err("entry is immutable");
    assert_eq!(
        unsupported.java_class_name(),
        "java.lang.UnsupportedOperationException"
    );
    let first = JavaMapEntry::new(Some("a".to_owned()), Some("1".to_owned()));
    let same = JavaMapEntry::raw(
        "different.Entry",
        Some("a".to_owned()),
        Some("1".to_owned()),
    );
    let different_key = JavaMapEntry::new(Some("b".to_owned()), Some("1".to_owned()));
    let different_value = JavaMapEntry::new(Some("a".to_owned()), Some("2".to_owned()));
    assert_eq!(first, same);
    assert_ne!(first, different_key);
    assert_ne!(first, different_value);
    assert_display_failure(&first, 1);
    assert_display_failure(&first, 2);
    assert_display_failure(&null_entry, 1);

    let empty = EvaluationUtils::evaluate_as_list::<String>(None);
    assert!(empty.is_empty());
    let reference = JavaObjectArray::object(vec![Some("x".to_owned())]);
    let borrowed_array =
        EvaluationUtils::evaluate_as_array(Some(JavaEvaluationTarget::ReferenceArray(&reference)))
            .expect("reference array");
    assert!(borrowed_array.as_owned_array().is_none());

    for value in [
        JavaEvaluationValue::Number(JavaNumber::Byte(1)),
        JavaEvaluationValue::Number(JavaNumber::Short(1)),
        JavaEvaluationValue::Number(JavaNumber::Long(1)),
        JavaEvaluationValue::Number(JavaNumber::Float(1.0)),
        JavaEvaluationValue::Number(JavaNumber::Other {
            class_name: "CustomNumber".to_owned(),
            double_value: 1.0,
        }),
    ] {
        assert!(EvaluationUtils::evaluate_as_boolean(&value).expect("boolean number"));
    }
    for value in [
        JavaEvaluationValue::Number(JavaNumber::Double(f64::from_bits(1))),
        JavaEvaluationValue::Number(JavaNumber::Double(-0.1)),
        JavaEvaluationValue::Number(JavaNumber::Double(2_f64.powi(60))),
        JavaEvaluationValue::Number(JavaNumber::Double(-2_f64.powi(60))),
    ] {
        assert!(
            EvaluationUtils::evaluate_as_number(&value)
                .expect("finite double")
                .is_some()
        );
    }
    assert!(
        EvaluationUtils::evaluate_as_number(&string("+12"))
            .expect("signed number")
            .is_some()
    );
    let malformed =
        JavaEvaluationValue::String(JavaString::from_utf16(vec![u16::from(b'1'), 0xD800]));
    assert!(
        EvaluationUtils::evaluate_as_number(&malformed)
            .expect("invalid string becomes null")
            .is_none()
    );
    assert!(EvaluationUtils::evaluate_as_number(&number(JavaNumber::Float(f32::NAN))).is_err());
    assert!(EvaluationUtils::evaluate_as_boolean(&string("     ")).expect("blank is true"));
    assert!(
        EvaluationUtils::evaluate_as_boolean(&string("fałse")).expect("non-ASCII token is true")
    );

    for result in [
        EvaluationUtils::evaluate_as_array::<String>(Some(JavaEvaluationTarget::Shorts(&[1]))),
        EvaluationUtils::evaluate_as_array::<String>(Some(JavaEvaluationTarget::Longs(&[1]))),
        EvaluationUtils::evaluate_as_array::<String>(Some(JavaEvaluationTarget::Floats(&[1.0]))),
        EvaluationUtils::evaluate_as_array::<String>(Some(JavaEvaluationTarget::Doubles(&[1.0]))),
        EvaluationUtils::evaluate_as_array::<String>(Some(JavaEvaluationTarget::Characters(&[
            u16::from(b'x'),
        ]))),
    ] {
        assert!(result.is_err());
    }

    let all_true = [
        JavaEvaluationValue::Boolean(true),
        JavaEvaluationValue::Other("java.lang.Object".to_owned()),
    ];
    assert!(Bools::new().array_and(Some(&all_true)).expect("all true"));
    let bools = Bools::new();
    assert!(bools.array_is_true(None).is_err());
    assert!(bools.list_is_true(None).is_err());
    assert!(bools.set_is_true(None).is_err());
    assert!(
        bools
            .is_false(&JavaEvaluationValue::LiteralValue(None))
            .is_err()
    );
    assert!(bools.array_is_false(None).is_err());
    assert!(bools.list_is_false(None).is_err());
    assert!(bools.set_is_false(None).is_err());
    assert!(bools.list_and(None).is_err());
    assert!(bools.set_and(None).is_err());
    assert!(bools.array_or(None).is_err());
    assert!(bools.list_or(None).is_err());
    assert!(bools.set_or(None).is_err());
    let invalid = [JavaEvaluationValue::LiteralValue(None)];
    assert!(bools.array_and(Some(&invalid)).is_err());
    assert!(bools.array_or(Some(&invalid)).is_err());
}

fn assert_display_failure(entry: &JavaMapEntry<String>, fail_on: usize) {
    let mut writer = FailingWriter { calls: 0, fail_on };
    assert!(write!(&mut writer, "{entry}").is_err());
}

struct FailingWriter {
    calls: usize,
    fail_on: usize,
}

impl std::fmt::Write for FailingWriter {
    fn write_str(&mut self, _value: &str) -> std::fmt::Result {
        self.calls += 1;
        if self.calls == self.fail_on {
            Err(std::fmt::Error)
        } else {
            Ok(())
        }
    }
}

fn emit_boolean_cases(output: &mut String) {
    emit_boolean(output, "bool.null", JavaEvaluationValue::Null);
    emit_boolean(output, "bool.false", JavaEvaluationValue::Boolean(false));
    emit_boolean(output, "bool.true", JavaEvaluationValue::Boolean(true));
    emit_boolean(
        output,
        "bool.big_decimal.zero_scale",
        number(JavaNumber::BigDecimal(decimal("0.000"))),
    );
    emit_boolean(
        output,
        "bool.big_decimal.nonzero",
        number(JavaNumber::BigDecimal(decimal("-0.01"))),
    );
    emit_boolean(
        output,
        "bool.big_integer.zero",
        number(JavaNumber::BigInteger(BigInt::from(0))),
    );
    emit_boolean(output, "bool.integer.zero", number(JavaNumber::Integer(0)));
    emit_boolean(
        output,
        "bool.double.negative_zero",
        number(JavaNumber::Double(-0.0)),
    );
    emit_boolean(
        output,
        "bool.double.nan",
        number(JavaNumber::Double(f64::NAN)),
    );
    emit_boolean(
        output,
        "bool.character.zero",
        JavaEvaluationValue::Character(0),
    );
    emit_boolean(
        output,
        "bool.character.value",
        JavaEvaluationValue::Character(u16::from(b'x')),
    );
    emit_boolean(output, "bool.string.false", string(" \tFALSE\r\n"));
    emit_boolean(output, "bool.string.off", string("OfF"));
    emit_boolean(output, "bool.string.no", string("NO"));
    emit_boolean(output, "bool.string.empty", string(""));
    emit_boolean(output, "bool.string.nbsp", string("\u{a0}false\u{a0}"));
    emit_boolean(
        output,
        "bool.literal.false",
        JavaEvaluationValue::LiteralValue(Some(JavaString::from_rust_str(" false "))),
    );
    emit_boolean(
        output,
        "bool.literal.null",
        JavaEvaluationValue::LiteralValue(None),
    );
    emit_boolean(
        output,
        "bool.empty_list",
        JavaEvaluationValue::Other("java.util.ArrayList".to_owned()),
    );
    emit_boolean(
        output,
        "bool.empty_array",
        JavaEvaluationValue::Other("[Ljava.lang.Object;".to_owned()),
    );
    emit_boolean(
        output,
        "bool.other",
        JavaEvaluationValue::Other("java.lang.Class".to_owned()),
    );
}

fn emit_number_cases(output: &mut String) {
    emit_number(output, "number.null", JavaEvaluationValue::Null);
    emit_number(
        output,
        "number.decimal",
        number(JavaNumber::BigDecimal(decimal("1.20"))),
    );
    emit_number(
        output,
        "number.big_integer",
        number(JavaNumber::BigInteger(BigInt::from(123))),
    );
    emit_number(output, "number.byte", number(JavaNumber::Byte(-2)));
    emit_number(output, "number.short", number(JavaNumber::Short(-3)));
    emit_number(output, "number.integer", number(JavaNumber::Integer(-4)));
    emit_number(output, "number.long", number(JavaNumber::Long(i64::MIN)));
    emit_number(output, "number.float", number(JavaNumber::Float(0.1)));
    emit_number(output, "number.double", number(JavaNumber::Double(0.1)));
    emit_number(
        output,
        "number.negative_zero",
        number(JavaNumber::Double(-0.0)),
    );
    emit_number(output, "number.nan", number(JavaNumber::Double(f64::NAN)));
    emit_number(
        output,
        "number.infinity",
        number(JavaNumber::Double(f64::INFINITY)),
    );
    emit_number(
        output,
        "number.custom",
        number(JavaNumber::Other {
            class_name: "EvaluationUtilsGolden$1".to_owned(),
            double_value: 7.0,
        }),
    );
    emit_number(output, "number.string.integer", string("123"));
    emit_number(output, "number.string.scale", string("-1.20E+3"));
    emit_number(
        output,
        "number.string.unicode_digits",
        string("1\u{0662}.\u{0663}"),
    );
    emit_number(output, "number.string.leading_space", string(" 123 "));
    emit_number(output, "number.string.trailing_space", string("123 "));
    emit_number(output, "number.string.dot_prefix", string(".5"));
    emit_number(output, "number.string.invalid", string("+ 1"));
    emit_number(
        output,
        "number.literal",
        JavaEvaluationValue::LiteralValue(Some(JavaString::from_rust_str("12"))),
    );
    emit_number(
        output,
        "number.other",
        JavaEvaluationValue::Other("java.lang.Class".to_owned()),
    );
    emit_number_matrix(output);
}

fn emit_collection_cases(output: &mut String) {
    let null_list = EvaluationUtils::evaluate_as_list::<String>(None);
    emit(output, "list.null", describe_list(&null_list));
    emit(
        output,
        "list.empty_iterable",
        describe_list(&EvaluationUtils::evaluate_as_list(Some(
            JavaEvaluationTarget::Iterable(&[]),
        ))),
    );
    let iterable = [Some("a".to_owned()), None, Some("b".to_owned())];
    emit(
        output,
        "list.iterable",
        describe_list(&EvaluationUtils::evaluate_as_list(Some(
            JavaEvaluationTarget::Iterable(&iterable),
        ))),
    );

    let raw_a = Arc::new(JavaMapEntry::raw(
        "java.util.LinkedHashMap$Entry",
        Some("a".to_owned()),
        Some("1".to_owned()),
    ));
    let raw_b = Arc::new(JavaMapEntry::raw(
        "java.util.LinkedHashMap$Entry",
        Some("b".to_owned()),
        None,
    ));
    let entries = [Arc::clone(&raw_a), Arc::clone(&raw_b)];
    let map_list = EvaluationUtils::evaluate_as_list(Some(JavaEvaluationTarget::Map(&entries)));
    emit(output, "list.map", describe_list(&map_list));
    let fresh_entry = matches!(
        map_list.as_slice().first(),
        Some(Some(JavaEvaluationElement::MapEntry(entry))) if !Arc::ptr_eq(entry, &raw_a)
    );
    emit(output, "list.map.fresh_entry", fresh_entry);
    let entry_hash = match map_list.as_slice().first() {
        Some(Some(JavaEvaluationElement::MapEntry(entry))) => entry.java_hash_code(),
        _ => panic!("map list must contain entry"),
    };
    emit(output, "list.map.entry_hash", entry_hash);

    emit_primitive_lists(output);
    let reference =
        JavaObjectArray::typed("java.lang.String", vec![Some("a".to_owned()), None], |_| {
            true
        })
        .expect("valid reference array");
    emit(
        output,
        "list.reference",
        describe_list(&EvaluationUtils::evaluate_as_list(Some(
            JavaEvaluationTarget::ReferenceArray(&reference),
        ))),
    );
    let scalar = "a".to_owned();
    emit(
        output,
        "list.scalar",
        describe_list(&EvaluationUtils::evaluate_as_list(Some(
            JavaEvaluationTarget::Other(&scalar),
        ))),
    );

    let null_array = EvaluationUtils::evaluate_as_array::<String>(None).expect("null array");
    emit(output, "array.null", describe_array(&null_array));
    let array_iterable = [Some("a".to_owned()), None];
    let iterable_array =
        EvaluationUtils::evaluate_as_array(Some(JavaEvaluationTarget::Iterable(&array_iterable)))
            .expect("iterable array");
    emit(output, "array.iterable", describe_array(&iterable_array));
    let map_array = EvaluationUtils::evaluate_as_array(Some(JavaEvaluationTarget::Map(&entries)))
        .expect("map array");
    emit(output, "array.map", describe_array(&map_array));
    let raw_entry = match map_array
        .as_owned_array()
        .and_then(|array| array.as_slice().first())
    {
        Some(Some(JavaEvaluationElement::MapEntry(entry))) => Arc::ptr_eq(entry, &raw_a),
        _ => false,
    };
    emit(output, "array.map.raw_entry", raw_entry);
    let reference_array =
        EvaluationUtils::evaluate_as_array(Some(JavaEvaluationTarget::ReferenceArray(&reference)))
            .expect("reference array");
    emit(
        output,
        "array.reference",
        describe_borrowed_array(&reference_array),
    );
    let scalar_array =
        EvaluationUtils::evaluate_as_array(Some(JavaEvaluationTarget::Other(&scalar)))
            .expect("scalar array");
    emit(output, "array.scalar", describe_array(&scalar_array));
    emit_array_error(
        output,
        "array.primitive.bytes",
        EvaluationUtils::evaluate_as_array::<String>(Some(JavaEvaluationTarget::Bytes(&[1]))),
    );
    emit_array_error(
        output,
        "array.primitive.ints",
        EvaluationUtils::evaluate_as_array::<String>(Some(JavaEvaluationTarget::Integers(&[1]))),
    );
    emit_array_error(
        output,
        "array.primitive.booleans",
        EvaluationUtils::evaluate_as_array::<String>(Some(JavaEvaluationTarget::Booleans(&[true]))),
    );
}

fn emit_primitive_lists(output: &mut String) {
    emit(
        output,
        "list.bytes",
        describe_list(&EvaluationUtils::evaluate_as_list::<String>(Some(
            JavaEvaluationTarget::Bytes(&[-1, 2]),
        ))),
    );
    emit(
        output,
        "list.shorts",
        describe_list(&EvaluationUtils::evaluate_as_list::<String>(Some(
            JavaEvaluationTarget::Shorts(&[-2, 3]),
        ))),
    );
    emit(
        output,
        "list.ints",
        describe_list(&EvaluationUtils::evaluate_as_list::<String>(Some(
            JavaEvaluationTarget::Integers(&[-3, 4]),
        ))),
    );
    emit(
        output,
        "list.longs",
        describe_list(&EvaluationUtils::evaluate_as_list::<String>(Some(
            JavaEvaluationTarget::Longs(&[-4, 5]),
        ))),
    );
    emit(
        output,
        "list.floats",
        describe_list(&EvaluationUtils::evaluate_as_list::<String>(Some(
            JavaEvaluationTarget::Floats(&[-0.0, 0.5]),
        ))),
    );
    emit(
        output,
        "list.doubles",
        describe_list(&EvaluationUtils::evaluate_as_list::<String>(Some(
            JavaEvaluationTarget::Doubles(&[-0.0, 0.5]),
        ))),
    );
    emit(
        output,
        "list.booleans",
        describe_list(&EvaluationUtils::evaluate_as_list::<String>(Some(
            JavaEvaluationTarget::Booleans(&[false, true]),
        ))),
    );
    emit(
        output,
        "list.characters",
        describe_list(&EvaluationUtils::evaluate_as_list::<String>(Some(
            JavaEvaluationTarget::Characters(&[0, u16::from(b'x')]),
        ))),
    );
}

fn emit_bools_cases(output: &mut String) {
    let bools = Bools::new();
    let values = [
        JavaEvaluationValue::Null,
        string("false"),
        number(JavaNumber::Integer(1)),
        string("no"),
    ];
    emit_result(output, "bools.is_true", bools.is_true(&string("yes")));
    emit_result(output, "bools.is_false", bools.is_false(&string("off")));
    emit_result(
        output,
        "bools.array_is_true",
        bools
            .array_is_true(Some(&values))
            .map(|value| format!("{value:?}")),
    );
    emit_result(
        output,
        "bools.list_is_true",
        bools
            .list_is_true(Some(&values))
            .map(|value| format!("{value:?}")),
    );
    emit_result(
        output,
        "bools.set_is_true",
        bools
            .set_is_true(Some(&values))
            .map(|value| format_index_set(&value)),
    );
    emit_result(
        output,
        "bools.array_is_false",
        bools
            .array_is_false(Some(&values))
            .map(|value| format!("{value:?}")),
    );
    emit_result(
        output,
        "bools.list_is_false",
        bools
            .list_is_false(Some(&values))
            .map(|value| format!("{value:?}")),
    );
    emit_result(
        output,
        "bools.set_is_false",
        bools
            .set_is_false(Some(&values))
            .map(|value| format_index_set(&value)),
    );
    emit_result(output, "bools.array_and", bools.array_and(Some(&values)));
    emit_result(output, "bools.list_and", bools.list_and(Some(&values)));
    emit_result(output, "bools.set_and", bools.set_and(Some(&values)));
    emit_result(output, "bools.array_or", bools.array_or(Some(&values)));
    emit_result(output, "bools.list_or", bools.list_or(Some(&values)));
    emit_result(output, "bools.set_or", bools.set_or(Some(&values)));
    emit_result(output, "bools.empty_and", bools.array_and(Some(&[])));
    emit_result(output, "bools.empty_or", bools.array_or(Some(&[])));
    emit_result(output, "bools.null_array", bools.array_and(None));
    let short_and = [
        JavaEvaluationValue::Boolean(false),
        JavaEvaluationValue::LiteralValue(None),
    ];
    emit_result(
        output,
        "bools.short_circuit_and",
        bools.array_and(Some(&short_and)),
    );
    let short_or = [
        JavaEvaluationValue::Boolean(true),
        JavaEvaluationValue::LiteralValue(None),
    ];
    emit_result(
        output,
        "bools.short_circuit_or",
        bools.array_or(Some(&short_or)),
    );
}

fn emit_boolean(output: &mut String, key: &str, value: JavaEvaluationValue) {
    emit_result(output, key, EvaluationUtils::evaluate_as_boolean(&value));
}

fn emit_number(output: &mut String, key: &str, value: JavaEvaluationValue) {
    emit(output, key, describe_number_outcome(&value));
}

fn emit_number_matrix(output: &mut String) {
    let edges = [
        0_u64,
        1_u64 << 63,
        1,
        (1_u64 << 63) | 1,
        f64::MIN_POSITIVE.to_bits(),
        f64::MAX.to_bits(),
        0.1_f64.to_bits(),
        (-0.1_f64).to_bits(),
        f64::NAN.to_bits(),
        f64::INFINITY.to_bits(),
    ];
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut count = 0;
    for bits in edges {
        hash = hash_number_outcome(hash, bits);
        count += 1;
    }
    let mut bits = 0x6a09_e667_f3bc_c909_u64;
    for _ in 0..20_000 {
        bits = bits
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        hash = hash_number_outcome(hash, bits);
        count += 1;
    }
    emit(output, "number.double_matrix.count", count);
    emit(output, "number.double_matrix.fnv64", format!("{hash:x}"));
}

fn hash_number_outcome(mut hash: u64, bits: u64) -> u64 {
    let value = number(JavaNumber::Double(f64::from_bits(bits)));
    let text = format!("{bits:x}:{}", describe_number_outcome(&value));
    for unit in text.encode_utf16() {
        hash ^= u64::from(unit);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

fn describe_number_outcome(value: &JavaEvaluationValue) -> String {
    match EvaluationUtils::evaluate_as_number(value) {
        Ok(result) => {
            let description = result.map_or_else(
                || "null".to_owned(),
                |result| {
                    let decimal = result.as_decimal();
                    let same = matches!(result, JavaBigDecimalResult::Borrowed(_));
                    format!(
                        "{}|scale={}|unscaled={}|same={same}",
                        decimal,
                        decimal.scale(),
                        decimal.unscaled_value()
                    )
                },
            );
            format!("OK:{description}")
        }
        Err(error) => format!("ERR:{}", error.java_class_name()),
    }
}

fn describe_list(values: &JavaEvaluationList<String>) -> String {
    let class_name = match values.list_type() {
        JavaEvaluationListType::EmptyList => "java.util.Collections$EmptyList",
        JavaEvaluationListType::UnmodifiableRandomAccessList => {
            "java.util.Collections$UnmodifiableRandomAccessList"
        }
    };
    let elements = values
        .as_slice()
        .iter()
        .map(describe_element)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{class_name}|{}|{elements}|java.lang.UnsupportedOperationException",
        values.len()
    )
}

fn describe_array(values: &JavaEvaluationArray<'_, String>) -> String {
    let array = values.as_owned_array().expect("owned Object[]");
    let elements = array
        .as_slice()
        .iter()
        .map(describe_element)
        .collect::<Vec<_>>()
        .join(",");
    format!("[Ljava.lang.Object;|{}|same=false|{elements}", array.len())
}

fn describe_borrowed_array(values: &JavaEvaluationArray<'_, String>) -> String {
    let JavaEvaluationArray::Borrowed(array) = values else {
        panic!("expected borrowed reference array");
    };
    let elements = array
        .as_slice()
        .iter()
        .map(|value| match value {
            Some(value) => format!("java.lang.String:{value}"),
            None => "null".to_owned(),
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[Ljava.lang.String;|{}|same={}|{elements}",
        array.len(),
        values.is_borrowed_from(array)
    )
}

fn describe_element(value: &Option<JavaEvaluationElement<String>>) -> String {
    match value {
        None => "null".to_owned(),
        Some(JavaEvaluationElement::Object(value)) => format!("java.lang.String:{value}"),
        Some(JavaEvaluationElement::Byte(value)) => format!("java.lang.Byte:{value}"),
        Some(JavaEvaluationElement::Short(value)) => format!("java.lang.Short:{value}"),
        Some(JavaEvaluationElement::Integer(value)) => format!("java.lang.Integer:{value}"),
        Some(JavaEvaluationElement::Long(value)) => format!("java.lang.Long:{value}"),
        Some(JavaEvaluationElement::Float(value)) => {
            format!("java.lang.Float:{}", java_float(*value))
        }
        Some(JavaEvaluationElement::Double(value)) => {
            format!("java.lang.Double:{}", java_double(*value))
        }
        Some(JavaEvaluationElement::Boolean(value)) => format!("java.lang.Boolean:{value}"),
        Some(JavaEvaluationElement::Character(value)) => {
            format!("java.lang.Character:{value:x}")
        }
        Some(JavaEvaluationElement::MapEntry(value)) => {
            format!("{}:{value}", value.java_class_name())
        }
    }
}

fn emit_array_error<T>(
    output: &mut String,
    key: &str,
    result: Result<JavaEvaluationArray<'_, T>, EvaluationError>,
) {
    match result {
        Ok(_) => emit(output, key, "OK:unexpected"),
        Err(error) => emit(output, key, format!("ERR:{}", error.java_class_name())),
    }
}

fn emit_result<T: std::fmt::Display>(
    output: &mut String,
    key: &str,
    result: Result<T, EvaluationError>,
) {
    match result {
        Ok(value) => emit(output, key, format!("OK:{value}")),
        Err(error) => {
            let message = match &error {
                EvaluationError::Validation(_) => format!(":{error}"),
                _ => String::new(),
            };
            emit(
                output,
                key,
                format!("ERR:{}{message}", error.java_class_name()),
            );
        }
    }
}

fn format_index_set(values: &indexmap::IndexSet<bool>) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(bool::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn java_float(value: f32) -> String {
    if value == 0.0 && value.is_sign_negative() {
        "-0.0".to_owned()
    } else {
        value.to_string()
    }
}

fn java_double(value: f64) -> String {
    if value == 0.0 && value.is_sign_negative() {
        "-0.0".to_owned()
    } else {
        value.to_string()
    }
}

fn decimal(value: &str) -> JavaBigDecimal {
    JavaBigDecimal::parse(value).expect("valid Java BigDecimal")
}

fn number(value: JavaNumber) -> JavaEvaluationValue {
    JavaEvaluationValue::Number(value)
}

fn string(value: &str) -> JavaEvaluationValue {
    JavaEvaluationValue::String(JavaString::from_rust_str(value))
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write to String");
}
