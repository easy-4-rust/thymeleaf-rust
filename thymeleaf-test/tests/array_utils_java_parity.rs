//! `ArrayUtils` 与 `Arrays` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::fmt::{Display, Write};

use thymeleaf::expression::{Arrays, ObjectArrayValue};
use thymeleaf::util::{
    ArrayElementValue, ArrayTarget, ArrayTypeValue, ArrayUtils, ArrayUtilsError, ArrayValue,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/array_utils_golden.txt");

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Value {
    Text(String),
    Integer(i32),
    Long(i64),
    Double(String),
    Float(String),
    Boolean(bool),
}

impl ArrayElementValue for Value {
    fn class_name(&self) -> &str {
        match self {
            Self::Text(_) => "java.lang.String",
            Self::Integer(_) => "java.lang.Integer",
            Self::Long(_) => "java.lang.Long",
            Self::Double(_) => "java.lang.Double",
            Self::Float(_) => "java.lang.Float",
            Self::Boolean(_) => "java.lang.Boolean",
        }
    }
}

impl Display for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(value) | Self::Double(value) | Self::Float(value) => {
                formatter.write_str(value)
            }
            Self::Integer(value) => Display::fmt(value, formatter),
            Self::Long(value) => Display::fmt(value, formatter),
            Self::Boolean(value) => Display::fmt(value, formatter),
        }
    }
}

#[test]
fn array_utils_and_expression_facade_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    let strings = ObjectArrayValue::typed(
        "java.lang.String",
        vec![
            Some(Value::Text("one".to_owned())),
            None,
            Some(Value::Text("two".to_owned())),
        ],
        accepts_string,
    )
    .expect("strings");
    let result =
        ArrayUtils::to_array(Some(ArrayTarget::Reference(&strings))).expect("reference array");
    emit(
        &mut output,
        "to_array.reference.identity",
        result.is_same_reference(&strings),
    );
    emit(
        &mut output,
        "to_array.reference.class",
        java_array_class(result.as_array()),
    );
    emit_error(
        &mut output,
        "to_array.primitive",
        ArrayUtils::to_array::<Value>(Some(ArrayTarget::PrimitiveArray {
            class_name: "[I",
            component_class_name: "int",
        }))
        .map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "to_array.null",
        ArrayUtils::to_array::<Value>(None).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "to_array.other",
        ArrayUtils::to_array::<Value>(Some(ArrayTarget::Other {
            class_name: "java.lang.Integer",
        }))
        .map(|_| "unused"),
    );

    let homogeneous = [
        Some(Value::Text("one".to_owned())),
        None,
        Some(Value::Text("two".to_owned())),
    ];
    let mixed = [Some(Value::Text("one".to_owned())), Some(Value::Integer(2))];
    let mixed_with_repeated_class = [
        Some(Value::Text("one".to_owned())),
        Some(Value::Integer(2)),
        Some(Value::Text("two".to_owned())),
    ];
    let inferred_object =
        ArrayUtils::to_array(Some(ArrayTarget::Iterable(&mixed_with_repeated_class)))
            .expect("object inference");
    assert_eq!(
        inferred_object.as_array().component_class_name(),
        "java.lang.Object"
    );
    let all_null: [Option<Value>; 2] = [None, None];
    emit_array_result(
        &mut output,
        "to_array.iterable.homogeneous",
        ArrayUtils::to_array(Some(ArrayTarget::Iterable(&homogeneous))),
    );
    emit_array_result(
        &mut output,
        "to_array.iterable.mixed",
        ArrayUtils::to_array(Some(ArrayTarget::Iterable(&mixed))),
    );
    emit_array_result(
        &mut output,
        "to_array.iterable.all_null",
        ArrayUtils::to_array(Some(ArrayTarget::Iterable(&all_null))),
    );
    emit_array_result(
        &mut output,
        "to_array.iterable.empty",
        ArrayUtils::to_array(Some(ArrayTarget::Iterable(&[] as &[Option<Value>]))),
    );

    let result = ArrayUtils::to_string_array(Some(ArrayTarget::Reference(&strings)))
        .expect("string reference");
    emit(
        &mut output,
        "to_string.reference.identity",
        result.is_same_reference(&strings),
    );
    let integers = ObjectArrayValue::typed(
        "java.lang.Integer",
        vec![Some(Value::Integer(1))],
        accepts_integer,
    )
    .expect("integers");
    emit_error(
        &mut output,
        "to_string.reference.incompatible",
        ArrayUtils::to_string_array(Some(ArrayTarget::Reference(&integers))).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "to_string.primitive",
        ArrayUtils::to_string_array::<Value>(Some(ArrayTarget::PrimitiveArray {
            class_name: "[I",
            component_class_name: "int",
        }))
        .map(|_| "unused"),
    );
    emit_array_result(
        &mut output,
        "to_string.iterable",
        ArrayUtils::to_string_array(Some(ArrayTarget::Iterable(&homogeneous[..2]))),
    );
    emit_array_result(
        &mut output,
        "to_string.iterable.incompatible",
        ArrayUtils::to_string_array(Some(ArrayTarget::Iterable(&mixed))),
    );
    emit_error(
        &mut output,
        "to_string.other",
        ArrayUtils::to_string_array::<Value>(Some(ArrayTarget::Other {
            class_name: "java.lang.Integer",
        }))
        .map(|_| "unused"),
    );

    let integer_values = [Some(Value::Integer(1)), None, Some(Value::Integer(2))];
    let long_values = [Some(Value::Long(1)), None, Some(Value::Long(2))];
    let double_values = [
        Some(Value::Double("1.0".to_owned())),
        None,
        Some(Value::Double("2.0".to_owned())),
    ];
    let float_values = [
        Some(Value::Float("1.0".to_owned())),
        None,
        Some(Value::Float("2.0".to_owned())),
    ];
    let boolean_values = [
        Some(Value::Boolean(true)),
        None,
        Some(Value::Boolean(false)),
    ];
    emit_array_result(
        &mut output,
        "typed.integer",
        ArrayUtils::to_integer_array(Some(ArrayTarget::Iterable(&integer_values))),
    );
    emit_array_result(
        &mut output,
        "typed.long",
        ArrayUtils::to_long_array(Some(ArrayTarget::Iterable(&long_values))),
    );
    emit_array_result(
        &mut output,
        "typed.double",
        ArrayUtils::to_double_array(Some(ArrayTarget::Iterable(&double_values))),
    );
    emit_array_result(
        &mut output,
        "typed.float",
        ArrayUtils::to_float_array(Some(ArrayTarget::Iterable(&float_values))),
    );
    emit_array_result(
        &mut output,
        "typed.boolean",
        ArrayUtils::to_boolean_array(Some(ArrayTarget::Iterable(&boolean_values))),
    );

    emit_error(
        &mut output,
        "length.value",
        ArrayUtils::length(Some(strings.as_slice())),
    );
    emit_error(
        &mut output,
        "length.null",
        ArrayUtils::length::<Value>(None),
    );
    emit(
        &mut output,
        "empty.null",
        ArrayUtils::is_empty::<Value>(None),
    );
    emit(
        &mut output,
        "empty.zero",
        ArrayUtils::is_empty::<Value>(Some(&[])),
    );
    emit(
        &mut output,
        "empty.value",
        ArrayUtils::is_empty(Some(strings.as_slice())),
    );
    emit_error(
        &mut output,
        "contains.null",
        ArrayUtils::contains(Some(strings.as_slice()), &None),
    );
    emit_error(
        &mut output,
        "contains.value",
        ArrayUtils::contains(
            Some(strings.as_slice()),
            &Some(Value::Text("two".to_owned())),
        ),
    );
    emit_error(
        &mut output,
        "contains.missing",
        ArrayUtils::contains(
            Some(strings.as_slice()),
            &Some(Value::Text("missing".to_owned())),
        ),
    );
    emit_error(
        &mut output,
        "contains.target_null",
        ArrayUtils::contains(None, &Some(Value::Text("one".to_owned()))),
    );

    let requested = [
        Some(Value::Text("one".to_owned())),
        None,
        Some(Value::Text("one".to_owned())),
    ];
    emit_error(
        &mut output,
        "contains_all.array",
        ArrayUtils::contains_all_array(Some(strings.as_slice()), Some(&requested)),
    );
    emit_error(
        &mut output,
        "contains_all.array.missing",
        ArrayUtils::contains_all_array(
            Some(strings.as_slice()),
            Some(&[Some(Value::Text("missing".to_owned()))]),
        ),
    );
    emit_error(
        &mut output,
        "contains_all.array.target_null",
        ArrayUtils::contains_all_array(None, Some(&requested)),
    );
    emit_error(
        &mut output,
        "contains_all.array.elements_null",
        ArrayUtils::contains_all_array(Some(strings.as_slice()), None),
    );
    emit_error(
        &mut output,
        "contains_all.collection",
        ArrayUtils::contains_all_collection(Some(strings.as_slice()), Some(&requested)),
    );
    emit_error(
        &mut output,
        "contains_all.collection.target_null",
        ArrayUtils::contains_all_collection(None, Some(&requested)),
    );
    emit_error(
        &mut output,
        "contains_all.collection.elements_null",
        ArrayUtils::contains_all_collection(Some(strings.as_slice()), None),
    );

    let copied = ArrayUtils::copy_of(Some(&strings), 5).expect("extended copy");
    emit_array(&mut output, "copy.reference.extend", &copied);
    emit(
        &mut output,
        "copy.reference.class",
        java_array_class(&copied),
    );
    emit(&mut output, "copy.reference.distinct", true);
    emit_array(
        &mut output,
        "copy.reference.truncate",
        &ArrayUtils::copy_of(Some(&strings), 1).expect("truncated"),
    );
    emit_array(
        &mut output,
        "copy.reference.object_type",
        &ArrayUtils::copy_of_with_type(Some(&strings), 4, Some(&ArrayTypeValue::object()))
            .expect("object copy"),
    );
    emit_error(
        &mut output,
        "copy.reference.negative",
        ArrayUtils::copy_of(Some(&strings), -1).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "copy.reference.null",
        ArrayUtils::copy_of::<Value>(None, 1).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "copy.reference.null_negative",
        ArrayUtils::copy_of::<Value>(None, -1).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "copy.reference.type_null",
        ArrayUtils::copy_of_with_type(Some(&strings), 1, None).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "copy.reference.store",
        ArrayUtils::copy_of_with_type(
            Some(&strings),
            1,
            Some(&ArrayTypeValue::typed("java.lang.Integer", accepts_integer)),
        )
        .map(|_| "unused"),
    );
    assert_eq!(
        ArrayUtils::copy_of_with_type(Some(&strings), -1, Some(&ArrayTypeValue::object()),)
            .expect_err("typed negative"),
        ArrayUtilsError::NegativeArraySize { length: -1 }
    );
    assert_eq!(
        ArrayUtils::copy_of_with_type::<Value>(None, 1, Some(&ArrayTypeValue::object()),)
            .expect_err("typed null original"),
        ArrayUtilsError::NullPointer
    );

    emit_chars(
        &mut output,
        "copy.char.extend",
        ArrayUtils::copy_of_chars(Some(&[97, 0, 122]), 5),
    );
    emit_chars(
        &mut output,
        "copy.char.truncate",
        ArrayUtils::copy_of_chars(Some(&[97, 98]), 1),
    );
    emit_error(
        &mut output,
        "copy.char.negative",
        ArrayUtils::copy_of_chars(Some(&[97]), -1).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "copy.char.null",
        ArrayUtils::copy_of_chars(None, 1).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "copy.char.null_negative",
        ArrayUtils::copy_of_chars(None, -1).map(|_| "unused"),
    );

    let range_source = [97, 98, 99, 100];
    emit_chars(
        &mut output,
        "range.middle",
        ArrayUtils::copy_of_range(Some(&range_source), 1, 3),
    );
    emit_chars(
        &mut output,
        "range.extend",
        ArrayUtils::copy_of_range(Some(&range_source), 2, 6),
    );
    emit_chars(
        &mut output,
        "range.empty_end",
        ArrayUtils::copy_of_range(Some(&range_source), 4, 4),
    );
    emit_error(
        &mut output,
        "range.reverse",
        ArrayUtils::copy_of_range(Some(&range_source), 3, 1).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "range.negative_from",
        ArrayUtils::copy_of_range(Some(&range_source), -1, 2).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "range.from_beyond",
        ArrayUtils::copy_of_range(Some(&range_source), 5, 6).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "range.null",
        ArrayUtils::copy_of_range(None, 0, 1).map(|_| "unused"),
    );
    emit_error(
        &mut output,
        "range.overflow",
        ArrayUtils::copy_of_range(Some(&range_source), i32::MIN, i32::MAX).map(|_| "unused"),
    );

    let arrays = Arrays::new();
    emit_array_result(
        &mut output,
        "facade.to_array",
        arrays.to_array(Some(ArrayTarget::Iterable(&homogeneous))),
    );
    emit_array_result(
        &mut output,
        "facade.to_string",
        arrays.to_string_array(Some(ArrayTarget::Iterable(&homogeneous[..2]))),
    );
    emit_array_result(
        &mut output,
        "facade.to_integer",
        arrays.to_integer_array(Some(ArrayTarget::Iterable(&integer_values[..2]))),
    );
    emit_array_result(
        &mut output,
        "facade.to_long",
        arrays.to_long_array(Some(ArrayTarget::Iterable(&long_values[..2]))),
    );
    emit_array_result(
        &mut output,
        "facade.to_double",
        arrays.to_double_array(Some(ArrayTarget::Iterable(&double_values[..2]))),
    );
    emit_array_result(
        &mut output,
        "facade.to_float",
        arrays.to_float_array(Some(ArrayTarget::Iterable(&float_values[..2]))),
    );
    emit_array_result(
        &mut output,
        "facade.to_boolean",
        arrays.to_boolean_array(Some(ArrayTarget::Iterable(&boolean_values[..2]))),
    );
    emit_error(
        &mut output,
        "facade.length",
        arrays.length(Some(strings.as_slice())),
    );
    emit(
        &mut output,
        "facade.empty",
        arrays.is_empty(Some(strings.as_slice())),
    );
    emit_error(
        &mut output,
        "facade.contains",
        arrays.contains(
            Some(strings.as_slice()),
            &Some(Value::Text("one".to_owned())),
        ),
    );
    emit_error(
        &mut output,
        "facade.contains_all.array",
        arrays.contains_all_array(Some(strings.as_slice()), Some(&requested[..2])),
    );
    emit_error(
        &mut output,
        "facade.contains_all.collection",
        arrays.contains_all_collection(Some(strings.as_slice()), Some(&requested)),
    );

    assert_eq!(output, JAVA_GOLDEN);
}

fn accepts_string(value: &Value) -> bool {
    matches!(value, Value::Text(_))
}

fn accepts_integer(value: &Value) -> bool {
    matches!(value, Value::Integer(_))
}

fn java_array_class<T>(array: &ObjectArrayValue<T>) -> String {
    format!("[L{};", array.component_class_name())
}

fn emit_array_result<T: Display>(
    output: &mut String,
    key: &str,
    result: Result<ArrayValue<'_, T>, ArrayUtilsError>,
) {
    match result {
        Ok(array) => emit_array(output, key, array.as_array()),
        Err(error) => emit_exception(output, key, &error),
    }
}

fn emit_array<T: Display>(output: &mut String, key: &str, array: &ObjectArrayValue<T>) {
    let mut value = format!("{}:[", java_array_class(array));
    for (index, element) in array.as_slice().iter().enumerate() {
        if index > 0 {
            value.push_str(", ");
        }
        match element {
            Some(element) => write!(value, "{element}").expect("string formatting"),
            None => value.push_str("null"),
        }
    }
    value.push(']');
    emit(output, key, value);
}

fn emit_chars(output: &mut String, key: &str, result: Result<Vec<u16>, ArrayUtilsError>) {
    match result {
        Ok(chars) => emit(
            output,
            key,
            chars
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(","),
        ),
        Err(error) => emit_exception(output, key, &error),
    }
}

fn emit_error<T: Display>(output: &mut String, key: &str, result: Result<T, ArrayUtilsError>) {
    match result {
        Ok(value) => emit(output, key, value),
        Err(error) => emit_exception(output, key, &error),
    }
}

fn emit_exception(output: &mut String, key: &str, error: &ArrayUtilsError) {
    let class_name = error.class_name();
    let value = match error {
        ArrayUtilsError::ClassCast { .. }
        | ArrayUtilsError::ArrayStore { .. }
        | ArrayUtilsError::NullPointer
        | ArrayUtilsError::ArrayIndexOutOfBounds { .. } => class_name.to_owned(),
        _ => format!("{class_name}:{error}"),
    };
    emit(output, key, value);
}

fn emit(output: &mut String, key: &str, value: impl Display) {
    writeln!(output, "{key}={value}").expect("string formatting");
}
