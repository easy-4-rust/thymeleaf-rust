//! `MapUtils` 与 `ObjectUtils` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::collections::HashMap;
use std::fmt::{Display, Write};
use std::rc::Rc;

use thymeleaf::util::{MapUtils, ObjectUtils, ValidateError};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/map_object_utils_golden.txt");

#[test]
fn map_and_object_utils_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    let map = HashMap::from([
        (Some("one".to_owned()), Some("value".to_owned())),
        (Some("two".to_owned()), Some("other".to_owned())),
        (None, None),
    ]);
    let empty: HashMap<Option<String>, Option<String>> = HashMap::new();

    emit_outcome(&mut output, "map.size.value", MapUtils::size(Some(&map)));
    emit_outcome(&mut output, "map.size.empty", MapUtils::size(Some(&empty)));
    emit_outcome(
        &mut output,
        "map.size.null",
        MapUtils::size(None::<&HashMap<Option<String>, Option<String>>>),
    );
    emit(
        &mut output,
        "map.empty.value",
        MapUtils::is_empty(Some(&map)),
    );
    emit(
        &mut output,
        "map.empty.empty",
        MapUtils::is_empty(Some(&empty)),
    );
    emit(
        &mut output,
        "map.empty.null",
        MapUtils::is_empty(None::<&HashMap<String, String>>),
    );

    emit_outcome(
        &mut output,
        "map.key.present",
        MapUtils::contains_key(Some(&map), &Some("one".to_owned())),
    );
    emit_outcome(
        &mut output,
        "map.key.missing",
        MapUtils::contains_key(Some(&map), &Some("missing".to_owned())),
    );
    emit_outcome(
        &mut output,
        "map.key.null",
        MapUtils::contains_key(Some(&map), &None),
    );
    emit_outcome(
        &mut output,
        "map.key.null_target",
        MapUtils::contains_key(
            None::<&HashMap<Option<String>, Option<String>>>,
            &Some("one".to_owned()),
        ),
    );

    let present_keys = [Some("one".to_owned()), None];
    let missing_keys = [Some("one".to_owned()), Some("missing".to_owned())];
    let duplicate_keys = [Some("one".to_owned()), Some("one".to_owned())];
    emit_outcome(
        &mut output,
        "map.keys_array.present",
        MapUtils::contains_all_keys_array(Some(&map), Some(&present_keys)),
    );
    emit_outcome(
        &mut output,
        "map.keys_array.missing",
        MapUtils::contains_all_keys_array(Some(&map), Some(&missing_keys)),
    );
    emit_outcome(
        &mut output,
        "map.keys_array.empty",
        MapUtils::contains_all_keys_array(Some(&map), Some(&[] as &[Option<String>])),
    );
    emit_outcome(
        &mut output,
        "map.keys_array.duplicate",
        MapUtils::contains_all_keys_array(Some(&map), Some(&duplicate_keys)),
    );
    emit_outcome(
        &mut output,
        "map.keys_array.null_target",
        MapUtils::contains_all_keys_array(
            None::<&HashMap<Option<String>, Option<String>>>,
            None::<&[Option<String>]>,
        ),
    );
    emit_outcome(
        &mut output,
        "map.keys_array.null_keys",
        MapUtils::contains_all_keys_array(Some(&map), None::<&[Option<String>]>),
    );

    emit_outcome(
        &mut output,
        "map.keys_collection.present",
        MapUtils::contains_all_keys_collection(Some(&map), Some(present_keys.iter())),
    );
    emit_outcome(
        &mut output,
        "map.keys_collection.missing",
        MapUtils::contains_all_keys_collection(Some(&map), Some(missing_keys.iter())),
    );
    emit_outcome(
        &mut output,
        "map.keys_collection.empty",
        MapUtils::contains_all_keys_collection(
            Some(&map),
            Some([].iter() as std::slice::Iter<'_, Option<String>>),
        ),
    );
    emit_outcome(
        &mut output,
        "map.keys_collection.null_target",
        MapUtils::contains_all_keys_collection(
            None::<&HashMap<Option<String>, Option<String>>>,
            None::<std::slice::Iter<'_, Option<String>>>,
        ),
    );
    emit_outcome(
        &mut output,
        "map.keys_collection.null_keys",
        MapUtils::contains_all_keys_collection(
            Some(&map),
            None::<std::slice::Iter<'_, Option<String>>>,
        ),
    );

    emit_outcome(
        &mut output,
        "map.value.present",
        MapUtils::contains_value(Some(&map), &Some("value".to_owned())),
    );
    emit_outcome(
        &mut output,
        "map.value.missing",
        MapUtils::contains_value(Some(&map), &Some("missing".to_owned())),
    );
    emit_outcome(
        &mut output,
        "map.value.null",
        MapUtils::contains_value(Some(&map), &None),
    );
    emit_outcome(
        &mut output,
        "map.value.null_target",
        MapUtils::contains_value(
            None::<&HashMap<Option<String>, Option<String>>>,
            &Some("value".to_owned()),
        ),
    );

    let present_values = [Some("value".to_owned()), None];
    let missing_values = [Some("value".to_owned()), Some("missing".to_owned())];
    let duplicate_values = [Some("value".to_owned()), Some("value".to_owned())];
    emit_outcome(
        &mut output,
        "map.values_array.present",
        MapUtils::contains_all_values_array(Some(&map), Some(&present_values)),
    );
    emit_outcome(
        &mut output,
        "map.values_array.missing",
        MapUtils::contains_all_values_array(Some(&map), Some(&missing_values)),
    );
    emit_outcome(
        &mut output,
        "map.values_array.empty",
        MapUtils::contains_all_values_array(Some(&map), Some(&[] as &[Option<String>])),
    );
    emit_outcome(
        &mut output,
        "map.values_array.duplicate",
        MapUtils::contains_all_values_array(Some(&map), Some(&duplicate_values)),
    );
    emit_outcome(
        &mut output,
        "map.values_array.null_target",
        MapUtils::contains_all_values_array(
            None::<&HashMap<Option<String>, Option<String>>>,
            None::<&[Option<String>]>,
        ),
    );
    emit_outcome(
        &mut output,
        "map.values_array.null_values",
        MapUtils::contains_all_values_array(Some(&map), None::<&[Option<String>]>),
    );

    emit_outcome(
        &mut output,
        "map.values_collection.present",
        MapUtils::contains_all_values_collection(Some(&map), Some(present_values.iter())),
    );
    emit_outcome(
        &mut output,
        "map.values_collection.missing",
        MapUtils::contains_all_values_collection(Some(&map), Some(missing_values.iter())),
    );
    emit_outcome(
        &mut output,
        "map.values_collection.empty",
        MapUtils::contains_all_values_collection(
            Some(&map),
            Some([].iter() as std::slice::Iter<'_, Option<String>>),
        ),
    );
    emit_outcome(
        &mut output,
        "map.values_collection.null_target",
        MapUtils::contains_all_values_collection(
            None::<&HashMap<Option<String>, Option<String>>>,
            None::<std::slice::Iter<'_, Option<String>>>,
        ),
    );
    emit_outcome(
        &mut output,
        "map.values_collection.null_values",
        MapUtils::contains_all_values_collection(
            Some(&map),
            None::<std::slice::Iter<'_, Option<String>>>,
        ),
    );

    let target = Rc::new("target".to_owned());
    let default_value = Rc::new("default".to_owned());
    let selected =
        ObjectUtils::null_safe(Some(Rc::clone(&target)), Some(Rc::clone(&default_value)));
    emit(
        &mut output,
        "object.target",
        selected
            .as_ref()
            .is_some_and(|value| Rc::ptr_eq(value, &target)),
    );
    let selected_default = ObjectUtils::null_safe(None, Some(Rc::clone(&default_value)));
    emit(
        &mut output,
        "object.default",
        selected_default
            .as_ref()
            .is_some_and(|value| Rc::ptr_eq(value, &default_value)),
    );
    emit(
        &mut output,
        "object.both_null",
        ObjectUtils::null_safe::<String>(None, None).is_none(),
    );

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_outcome<T: Display>(output: &mut String, key: &str, result: Result<T, ValidateError>) {
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

fn emit(output: &mut String, key: &str, value: impl Display) {
    writeln!(output, "{key}={value}").expect("string output");
}
