//! `AggregateUtils`/`Aggregates` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::cell::Cell;
use std::fmt::Write;

use num_bigint::BigInt;
use thymeleaf::expression::Aggregates;
use thymeleaf::util::{
    AggregateError, AggregateObjectValue, AggregateUtils, BigDecimalValue, NumberIterableValue,
    NumberListValue, NumberValue,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/aggregate_utils_golden.txt");

#[test]
fn aggregate_utils_and_facade_match_java_golden() {
    // 在非 test cfg 的核心库实例中覆盖 Java BigDecimal 的无整数部分语法边界。
    assert!(BigDecimalValue::parse(".").is_err());
    assert!(BigDecimalValue::parse("1E+").is_err());
    assert!(BigDecimalValue::parse("0.0E-2147483648").is_err());
    assert_eq!(
        BigDecimalValue::parse(".1")
            .expect("Java accepts a decimal without an integer part")
            .to_plain_string(),
        "0.1"
    );
    assert_eq!(
        BigDecimalValue::parse("1E+7")
            .expect("positive scientific exponent")
            .to_string(),
        "1E+7"
    );
    assert_eq!(
        BigDecimalValue::parse("12E+7")
            .expect("multi-digit scientific mantissa")
            .to_string(),
        "1.2E+8"
    );
    let changing_iterable = ChangesAfterValidation {
        iterator_calls: Cell::new(0),
        number: NumberValue::Integer(1),
        second_number: None,
    };
    assert!(AggregateUtils::sum_iterable(Some(&changing_iterable)).is_err());
    let changing_to_nan = ChangesAfterValidation {
        iterator_calls: Cell::new(0),
        number: NumberValue::Integer(1),
        second_number: Some(NumberValue::Double(f64::NAN)),
    };
    assert!(AggregateUtils::sum_iterable(Some(&changing_to_nan)).is_err());
    assert!(AggregateUtils::sum_bytes(None).is_err());
    assert!(
        AggregateUtils::sum_objects(Some(&[AggregateObjectValue::Number(NumberValue::Double(
            f64::NAN
        ))]))
        .is_err()
    );
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_utility_cases(&mut output);
    emit_facade_cases(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

struct ChangesAfterValidation {
    iterator_calls: Cell<usize>,
    number: NumberValue,
    second_number: Option<NumberValue>,
}

impl NumberIterableValue for ChangesAfterValidation {
    fn iter_java_numbers(&self) -> Box<dyn Iterator<Item = Option<&NumberValue>> + '_> {
        let iterator_call = self.iterator_calls.get();
        self.iterator_calls.set(iterator_call + 1);
        if iterator_call == 0 {
            Box::new(std::iter::once(Some(&self.number)))
        } else {
            Box::new(std::iter::once(self.second_number.as_ref()))
        }
    }
}

fn emit_utility_cases(output: &mut String) {
    emit_outcome(
        output,
        "util.sum.iterable.null",
        AggregateUtils::sum_iterable(None),
    );
    let empty_numbers = NumberListValue::new(Vec::new());
    emit_outcome(
        output,
        "util.sum.iterable.empty",
        AggregateUtils::sum_iterable(Some(&empty_numbers)),
    );
    let null_numbers = NumberListValue::new(vec![Some(NumberValue::Integer(1)), None]);
    emit_outcome(
        output,
        "util.sum.iterable.null_element",
        AggregateUtils::sum_iterable(Some(&null_numbers)),
    );
    emit_outcome(
        output,
        "util.avg.iterable.null_element",
        AggregateUtils::avg_iterable(Some(&null_numbers)),
    );

    let counting = CountingNumbers::new(vec![
        Some(NumberValue::Integer(1)),
        Some(NumberValue::Integer(2)),
        Some(NumberValue::Integer(3)),
    ]);
    emit_outcome(
        output,
        "util.sum.iterable.counting",
        AggregateUtils::sum_iterable(Some(&counting)),
    );
    emit(
        output,
        "util.sum.iterable.iterator_calls",
        counting.iterator_calls.get(),
    );

    emit_outcome(
        output,
        "util.sum.objects.null",
        AggregateUtils::sum_objects(None),
    );
    emit_outcome(
        output,
        "util.sum.objects.empty",
        AggregateUtils::sum_objects(Some(&[])),
    );
    emit_outcome(
        output,
        "util.sum.objects.null_element",
        AggregateUtils::sum_objects(Some(&[
            AggregateObjectValue::Number(NumberValue::Integer(1)),
            AggregateObjectValue::Null,
        ])),
    );
    emit_outcome(
        output,
        "util.sum.objects.null_priority",
        AggregateUtils::sum_objects(Some(&[
            AggregateObjectValue::Other("java.lang.String".to_owned()),
            AggregateObjectValue::Null,
        ])),
    );
    emit_outcome(
        output,
        "util.sum.objects.class_cast",
        AggregateUtils::sum_objects(Some(&[AggregateObjectValue::Other(
            "java.lang.String".to_owned(),
        )])),
    );

    let mixed = [
        AggregateObjectValue::Number(NumberValue::BigDecimal(
            BigDecimalValue::parse("1.20").expect("decimal"),
        )),
        AggregateObjectValue::Number(NumberValue::BigInteger(BigInt::from(2))),
        AggregateObjectValue::Number(NumberValue::Byte(3)),
        AggregateObjectValue::Number(NumberValue::Short(4)),
        AggregateObjectValue::Number(NumberValue::Integer(5)),
        AggregateObjectValue::Number(NumberValue::Long(6)),
        AggregateObjectValue::Number(NumberValue::Float(0.5)),
        AggregateObjectValue::Number(NumberValue::Double(0.25)),
        AggregateObjectValue::Number(NumberValue::Other {
            class_name: "AggregateUtilsGolden$1".to_owned(),
            double_value: 0.05,
        }),
    ];
    emit_outcome(
        output,
        "util.sum.objects.mixed",
        AggregateUtils::sum_objects(Some(&mixed)),
    );
    emit_outcome(
        output,
        "util.avg.objects.mixed",
        AggregateUtils::avg_objects(Some(&mixed)),
    );

    emit_bytes(output, "bytes", &[-128, 1, 127]);
    emit_shorts(output, "shorts", &[-32768, 1, 32767]);
    emit_ints(output, "ints", &[i32::MIN, 1, i32::MAX]);
    emit_longs(output, "longs", &[i64::MIN, 1, i64::MAX]);
    emit_floats(output, "floats", &[0.1, -0.0, 1.25]);
    emit_doubles(output, "doubles", &[0.1, -0.0, 1.25]);

    emit_outcome(
        output,
        "util.sum.floats.min",
        AggregateUtils::sum_floats(Some(&[f32::MIN_POSITIVE * f32::EPSILON])),
    );
    emit_outcome(
        output,
        "util.sum.floats.max",
        AggregateUtils::sum_floats(Some(&[f32::MAX])),
    );
    emit_outcome(
        output,
        "util.sum.floats.nan",
        AggregateUtils::sum_floats(Some(&[f32::NAN])),
    );
    emit_outcome(
        output,
        "util.sum.floats.infinity",
        AggregateUtils::sum_floats(Some(&[f32::INFINITY])),
    );
    emit_outcome(
        output,
        "util.sum.doubles.min",
        AggregateUtils::sum_doubles(Some(&[f64::from_bits(1)])),
    );
    emit_outcome(
        output,
        "util.sum.doubles.max",
        AggregateUtils::sum_doubles(Some(&[f64::MAX])),
    );
    emit_outcome(
        output,
        "util.sum.doubles.threshold_plain",
        AggregateUtils::sum_doubles(Some(&[9_999_999.0])),
    );
    emit_outcome(
        output,
        "util.sum.doubles.threshold_scientific",
        AggregateUtils::sum_doubles(Some(&[10_000_000.0])),
    );
    emit_outcome(
        output,
        "util.sum.doubles.small_plain",
        AggregateUtils::sum_doubles(Some(&[0.001])),
    );
    emit_outcome(
        output,
        "util.sum.doubles.small_scientific",
        AggregateUtils::sum_doubles(Some(&[0.0001])),
    );
    emit_outcome(
        output,
        "util.sum.doubles.negative_zero",
        AggregateUtils::sum_doubles(Some(&[-0.0])),
    );
    emit_outcome(
        output,
        "util.sum.doubles.nan",
        AggregateUtils::sum_doubles(Some(&[f64::NAN])),
    );
    emit_outcome(
        output,
        "util.sum.doubles.infinity",
        AggregateUtils::sum_doubles(Some(&[f64::NEG_INFINITY])),
    );

    emit_outcome(
        output,
        "util.avg.exact",
        AggregateUtils::avg_ints(Some(&[1, 2])),
    );
    emit_outcome(
        output,
        "util.avg.repeating",
        AggregateUtils::avg_ints(Some(&[1, 1, 2])),
    );
    emit_outcome(
        output,
        "util.avg.repeating_negative",
        AggregateUtils::avg_ints(Some(&[-1, -1, -2])),
    );
    emit_outcome(
        output,
        "util.avg.scale_12",
        AggregateUtils::avg_objects(Some(&[
            AggregateObjectValue::Number(NumberValue::BigDecimal(
                BigDecimalValue::parse("1.000000000000").expect("decimal"),
            )),
            AggregateObjectValue::Number(NumberValue::Integer(2)),
            AggregateObjectValue::Number(NumberValue::Integer(2)),
        ])),
    );
    emit_outcome(
        output,
        "util.sum.long_no_overflow",
        AggregateUtils::sum_longs(Some(&[i64::MAX, i64::MAX])),
    );
    emit_double_matrix(output);
}

fn emit_facade_cases(output: &mut String) {
    let aggregates = Aggregates::new();
    let numbers = [Some(NumberValue::Integer(1)), Some(NumberValue::Integer(2))];
    let iterable_numbers = NumberListValue::new(numbers.to_vec());
    assert_eq!(iterable_numbers.as_slice(), numbers);
    emit_outcome(
        output,
        "facade.sum.iterable",
        aggregates.sum_iterable(Some(&iterable_numbers)),
    );
    emit_outcome(
        output,
        "facade.sum.numbers",
        aggregates.sum_numbers(Some(&numbers)),
    );
    emit_outcome(
        output,
        "facade.sum.bytes",
        aggregates.sum_bytes(Some(&[1, 2])),
    );
    emit_outcome(
        output,
        "facade.sum.shorts",
        aggregates.sum_shorts(Some(&[1, 2])),
    );
    emit_outcome(
        output,
        "facade.sum.ints",
        aggregates.sum_ints(Some(&[1, 2])),
    );
    emit_outcome(
        output,
        "facade.sum.longs",
        aggregates.sum_longs(Some(&[1, 2])),
    );
    emit_outcome(
        output,
        "facade.sum.floats",
        aggregates.sum_floats(Some(&[0.5, 0.25])),
    );
    emit_outcome(
        output,
        "facade.sum.doubles",
        aggregates.sum_doubles(Some(&[0.5, 0.25])),
    );
    emit_outcome(
        output,
        "facade.avg.iterable",
        aggregates.avg_iterable(Some(&iterable_numbers)),
    );
    emit_outcome(
        output,
        "facade.avg.numbers",
        aggregates.avg_numbers(Some(&numbers)),
    );
    emit_outcome(
        output,
        "facade.avg.bytes",
        aggregates.avg_bytes(Some(&[1, 2])),
    );
    emit_outcome(
        output,
        "facade.avg.shorts",
        aggregates.avg_shorts(Some(&[1, 2])),
    );
    emit_outcome(
        output,
        "facade.avg.ints",
        aggregates.avg_ints(Some(&[1, 2])),
    );
    emit_outcome(
        output,
        "facade.avg.longs",
        aggregates.avg_longs(Some(&[1, 2])),
    );
    emit_outcome(
        output,
        "facade.avg.floats",
        aggregates.avg_floats(Some(&[0.5, 0.25])),
    );
    emit_outcome(
        output,
        "facade.avg.doubles",
        aggregates.avg_doubles(Some(&[0.5, 0.25])),
    );
}

fn emit_bytes(output: &mut String, key: &str, values: &[i8]) {
    emit_outcome(
        output,
        &format!("util.sum.{key}"),
        AggregateUtils::sum_bytes(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.avg.{key}"),
        AggregateUtils::avg_bytes(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.sum.{key}.empty"),
        AggregateUtils::sum_bytes(Some(&[])),
    );
}

fn emit_shorts(output: &mut String, key: &str, values: &[i16]) {
    emit_outcome(
        output,
        &format!("util.sum.{key}"),
        AggregateUtils::sum_shorts(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.avg.{key}"),
        AggregateUtils::avg_shorts(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.sum.{key}.empty"),
        AggregateUtils::sum_shorts(Some(&[])),
    );
}

fn emit_ints(output: &mut String, key: &str, values: &[i32]) {
    emit_outcome(
        output,
        &format!("util.sum.{key}"),
        AggregateUtils::sum_ints(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.avg.{key}"),
        AggregateUtils::avg_ints(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.sum.{key}.empty"),
        AggregateUtils::sum_ints(Some(&[])),
    );
}

fn emit_longs(output: &mut String, key: &str, values: &[i64]) {
    emit_outcome(
        output,
        &format!("util.sum.{key}"),
        AggregateUtils::sum_longs(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.avg.{key}"),
        AggregateUtils::avg_longs(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.sum.{key}.empty"),
        AggregateUtils::sum_longs(Some(&[])),
    );
}

fn emit_floats(output: &mut String, key: &str, values: &[f32]) {
    emit_outcome(
        output,
        &format!("util.sum.{key}"),
        AggregateUtils::sum_floats(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.avg.{key}"),
        AggregateUtils::avg_floats(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.sum.{key}.empty"),
        AggregateUtils::sum_floats(Some(&[])),
    );
}

fn emit_doubles(output: &mut String, key: &str, values: &[f64]) {
    emit_outcome(
        output,
        &format!("util.sum.{key}"),
        AggregateUtils::sum_doubles(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.avg.{key}"),
        AggregateUtils::avg_doubles(Some(values)),
    );
    emit_outcome(
        output,
        &format!("util.sum.{key}.empty"),
        AggregateUtils::sum_doubles(Some(&[])),
    );
}

fn emit_outcome(
    output: &mut String,
    key: &str,
    result: Result<Option<BigDecimalValue>, AggregateError>,
) {
    emit(output, key, describe_outcome(result));
}

fn describe_outcome(result: Result<Option<BigDecimalValue>, AggregateError>) -> String {
    match result {
        Ok(None) => "OK:null".to_owned(),
        Ok(Some(decimal)) => format!(
            "OK:{decimal}|scale={}|unscaled={}|plain={}",
            decimal.scale(),
            decimal.unscaled_value(),
            decimal.to_plain_string()
        ),
        Err(AggregateError::IllegalArgument { message }) => {
            format!("ERR:java.lang.IllegalArgumentException:{message}")
        }
        Err(AggregateError::ClassCast { .. }) => "ERR:java.lang.ClassCastException".to_owned(),
        Err(AggregateError::NumberFormat { .. }) => {
            "ERR:java.lang.NumberFormatException".to_owned()
        }
        Err(AggregateError::Arithmetic { .. }) => "ERR:java.lang.ArithmeticException".to_owned(),
    }
}

fn emit_double_matrix(output: &mut String) {
    let edges = [
        0_u64,
        1_u64 << 63,
        1,
        (1_u64 << 63) | 1,
        f64::MIN_POSITIVE.to_bits(),
        f64::MAX.to_bits(),
        0.001_f64.to_bits(),
        10_000_000.0_f64.to_bits(),
        f64::NAN.to_bits(),
        f64::INFINITY.to_bits(),
    ];
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut count = 0_usize;
    for bits in edges {
        hash = hash_double_outcome(hash, bits);
        count += 1;
    }
    let mut bits = 0x6a09_e667_f3bc_c909_u64;
    for _ in 0..20_000 {
        bits = bits
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        hash = hash_double_outcome(hash, bits);
        count += 1;
    }
    emit(output, "util.double_matrix.count", count);
    emit(output, "util.double_matrix.fnv64", format!("{hash:x}"));
}

fn hash_double_outcome(mut hash: u64, bits: u64) -> u64 {
    let value = f64::from_bits(bits);
    let text = format!(
        "{bits:x}:{}",
        describe_outcome(AggregateUtils::sum_doubles(Some(&[value])))
    );
    for unit in text.encode_utf16() {
        hash ^= u64::from(unit);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("string output");
}

struct CountingNumbers {
    values: Vec<Option<NumberValue>>,
    iterator_calls: Cell<usize>,
}

impl CountingNumbers {
    fn new(values: Vec<Option<NumberValue>>) -> Self {
        Self {
            values,
            iterator_calls: Cell::new(0),
        }
    }
}

impl NumberIterableValue for CountingNumbers {
    fn iter_java_numbers(&self) -> Box<dyn Iterator<Item = Option<&NumberValue>> + '_> {
        self.iterator_calls.set(self.iterator_calls.get() + 1);
        Box::new(self.values.iter().map(Option::as_ref))
    }
}
