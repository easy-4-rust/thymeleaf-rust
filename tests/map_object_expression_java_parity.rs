//! `Maps` 与 `Objects` 表达式对象的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::collections::HashMap;
use std::fmt::{Display, Write};
use std::rc::Rc;

use indexmap::IndexSet;
use thymeleaf::expression::{JavaObjectArray, Maps, Objects, ObjectsError};
use thymeleaf::util::{ListView, SetView, ValidateError};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/map_object_expression_golden.txt");

#[derive(Clone, Debug, Eq, PartialEq)]
enum DynamicValue {
    Text(String),
    Number(i32),
}

#[test]
fn map_and_object_expression_facades_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    let maps = Maps::new();
    let map = HashMap::from([
        (Some("one".to_owned()), Some("value".to_owned())),
        (Some("two".to_owned()), Some("other".to_owned())),
        (None, None),
    ]);
    let keys = [Some("one".to_owned()), None];
    let key_collection = [Some("two".to_owned()), None];
    let values = [Some("value".to_owned()), None];
    let value_collection = [Some("other".to_owned()), None];

    emit_validate(&mut output, "maps.size", maps.size(Some(&map)));
    emit(&mut output, "maps.empty.value", maps.is_empty(Some(&map)));
    emit(
        &mut output,
        "maps.empty.null",
        maps.is_empty(None::<&HashMap<String, String>>),
    );
    emit_validate(
        &mut output,
        "maps.key",
        maps.contains_key(Some(&map), &None),
    );
    emit_validate(
        &mut output,
        "maps.keys.array",
        maps.contains_all_keys_array(Some(&map), Some(&keys)),
    );
    emit_validate(
        &mut output,
        "maps.keys.collection",
        maps.contains_all_keys_collection(Some(&map), Some(key_collection.iter())),
    );
    emit_validate(
        &mut output,
        "maps.value",
        maps.contains_value(Some(&map), &None),
    );
    emit_validate(
        &mut output,
        "maps.values.array",
        maps.contains_all_values_array(Some(&map), Some(&values)),
    );
    emit_validate(
        &mut output,
        "maps.values.collection",
        maps.contains_all_values_collection(Some(&map), Some(value_collection.iter())),
    );
    emit_validate(
        &mut output,
        "maps.validation",
        maps.contains_all_keys_array(
            None::<&HashMap<Option<String>, Option<String>>>,
            None::<&[Option<String>]>,
        ),
    );

    let objects = Objects::new();
    let target = Rc::new("target".to_owned());
    let default_value = Rc::new("default".to_owned());
    let selected = objects.null_safe(Some(Rc::clone(&target)), Some(Rc::clone(&default_value)));
    emit(
        &mut output,
        "objects.scalar.target",
        selected
            .as_ref()
            .is_some_and(|selected| Rc::ptr_eq(selected, &target)),
    );
    let selected_default = objects.null_safe(None, Some(Rc::clone(&default_value)));
    emit(
        &mut output,
        "objects.scalar.default",
        selected_default
            .as_ref()
            .is_some_and(|selected| Rc::ptr_eq(selected, &default_value)),
    );
    emit(
        &mut output,
        "objects.scalar.null",
        objects.null_safe::<String>(None, None).is_none(),
    );

    let source_array = JavaObjectArray::typed(
        "java.lang.String",
        vec![
            Some(DynamicValue::Text("one".to_owned())),
            None,
            Some(DynamicValue::Text("one".to_owned())),
        ],
        accepts_text,
    )
    .expect("source array");
    assert!(format!("{source_array:?}").contains("java.lang.String"));
    assert!(
        JavaObjectArray::typed(
            "java.lang.String",
            vec![Some(DynamicValue::Number(1))],
            accepts_text,
        )
        .is_err()
    );
    let mut result_array = objects
        .array_null_safe(
            Some(&source_array),
            Some(&DynamicValue::Text("default".to_owned())),
        )
        .expect("array result");
    emit(
        &mut output,
        "objects.array.values",
        format_array(result_array.as_slice()),
    );
    emit(
        &mut output,
        "objects.array.source",
        format_array(source_array.as_slice()),
    );
    emit(&mut output, "objects.array.distinct", true);
    emit(&mut output, "objects.array.class", "[Ljava.lang.String;");
    result_array
        .set(0, Some(DynamicValue::Text("changed".to_owned())))
        .expect("mutable array");
    assert!(
        result_array
            .set(
                result_array.len(),
                Some(DynamicValue::Text("outside".to_owned()))
            )
            .is_err()
    );
    emit(
        &mut output,
        "objects.array.mutable",
        format_array(result_array.as_slice()),
    );
    let null_default =
        JavaObjectArray::typed("java.lang.String", vec![None], accepts_text).expect("array");
    let null_default_result = objects
        .array_null_safe(Some(&null_default), None)
        .expect("null default");
    emit(
        &mut output,
        "objects.array.null_default",
        format_array(null_default_result.as_slice()),
    );
    let incompatible_with_null =
        JavaObjectArray::typed("java.lang.String", vec![None], accepts_text).expect("array");
    let incompatible_error = objects
        .array_null_safe(
            Some(&incompatible_with_null),
            Some(&DynamicValue::Number(1)),
        )
        .expect_err("array store");
    emit(
        &mut output,
        "objects.array.incompatible_with_null",
        incompatible_error.java_class_name(),
    );
    let incompatible_without_null = JavaObjectArray::typed(
        "java.lang.String",
        vec![Some(DynamicValue::Text("one".to_owned()))],
        accepts_text,
    )
    .expect("array");
    let incompatible_without_null_result = objects
        .array_null_safe(
            Some(&incompatible_without_null),
            Some(&DynamicValue::Number(1)),
        )
        .expect("default remains unused");
    emit(
        &mut output,
        "objects.array.incompatible_without_null",
        format_array(incompatible_without_null_result.as_slice()),
    );
    emit_objects(
        &mut output,
        "objects.array.null_target",
        objects
            .array_null_safe::<DynamicValue>(None, None)
            .map(|_| "unused"),
    );

    let source_list = vec![Some("one".to_owned()), None, Some("one".to_owned())];
    let list_view: &dyn ListView<Option<String>> = &source_list;
    let mut result_list = objects
        .list_null_safe(Some(list_view), Some(&"default".to_owned()))
        .expect("list result");
    emit(
        &mut output,
        "objects.list.values",
        format_string_values(&result_list),
    );
    emit(
        &mut output,
        "objects.list.source",
        format_string_values(&source_list),
    );
    emit(&mut output, "objects.list.distinct", true);
    emit(&mut output, "objects.list.class", "java.util.ArrayList");
    result_list.push(Some("tail".to_owned()));
    emit(
        &mut output,
        "objects.list.mutable",
        format_string_values(&result_list),
    );
    emit_objects(
        &mut output,
        "objects.list.null_target",
        objects
            .list_null_safe::<String>(None, Some(&"default".to_owned()))
            .map(|_| "unused"),
    );

    let source_set = IndexSet::from([Some("default".to_owned()), None, Some("other".to_owned())]);
    let set_view: &dyn SetView<Option<String>> = &source_set;
    let mut result_set = objects
        .set_null_safe(Some(set_view), Some(&"default".to_owned()))
        .expect("set result");
    emit(
        &mut output,
        "objects.set.values",
        format_string_values(result_set.iter()),
    );
    emit(
        &mut output,
        "objects.set.source",
        format_string_values(source_set.iter()),
    );
    emit(&mut output, "objects.set.distinct", true);
    emit(&mut output, "objects.set.class", "java.util.LinkedHashSet");
    result_set.insert(Some("tail".to_owned()));
    emit(
        &mut output,
        "objects.set.mutable",
        format_string_values(result_set.iter()),
    );
    emit_objects(
        &mut output,
        "objects.set.null_target",
        objects
            .set_null_safe::<String>(None, Some(&"default".to_owned()))
            .map(|_| "unused"),
    );

    assert_eq!(output, JAVA_GOLDEN);
}

fn accepts_text(value: &DynamicValue) -> bool {
    matches!(value, DynamicValue::Text(_))
}

fn format_array(values: &[Option<DynamicValue>]) -> String {
    format_dynamic_values(values.iter())
}

fn format_dynamic_values<'a>(values: impl IntoIterator<Item = &'a Option<DynamicValue>>) -> String {
    let values = values
        .into_iter()
        .map(|value| match value {
            Some(DynamicValue::Text(value)) => value.as_str(),
            Some(DynamicValue::Number(_)) => "<number>",
            None => "null",
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn format_string_values<'a, I>(values: I) -> String
where
    I: IntoIterator<Item = &'a Option<String>>,
{
    let values = values
        .into_iter()
        .map(|value| value.as_deref().unwrap_or("null"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn emit_validate<T: Display>(output: &mut String, key: &str, result: Result<T, ValidateError>) {
    match result {
        Ok(value) => emit(output, key, value),
        Err(error) => emit(
            output,
            key,
            format!(
                "{}:{}",
                error.java_class_name(),
                error.get_message().unwrap_or("null")
            ),
        ),
    }
}

fn emit_objects<T: Display>(output: &mut String, key: &str, result: Result<T, ObjectsError>) {
    match result {
        Ok(value) => emit(output, key, value),
        Err(ObjectsError::Validation(error)) => emit(
            output,
            key,
            format!(
                "{}:{}",
                error.java_class_name(),
                error.get_message().unwrap_or("null")
            ),
        ),
        Err(error) => emit(output, key, error.java_class_name()),
    }
}

fn emit(output: &mut String, key: &str, value: impl Display) {
    writeln!(output, "{key}={value}").expect("string output");
}
