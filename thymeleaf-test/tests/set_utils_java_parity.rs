//! `SetUtils` 与 `Sets` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::collections::BTreeSet;
use std::fmt::{Display, Write};

use indexmap::IndexSet;
use thymeleaf::expression::Sets;
use thymeleaf::util::{JavaSet, SetTarget, SetUtils, SetUtilsError, SetView, ValidateError};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/set_utils_golden.txt");

#[test]
fn set_utils_and_expression_facade_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    let source = IndexSet::from([Some("two".to_owned()), Some("one".to_owned()), None]);
    let source_view: &dyn SetView<Option<String>> = &source;
    let empty = IndexSet::<Option<String>>::new();
    let empty_view: &dyn SetView<Option<String>> = &empty;
    let sorted = BTreeSet::from([Some("two".to_owned()), Some("one".to_owned())]);
    let sorted_view: &dyn SetView<Option<String>> = &sorted;

    emit_set_result(
        &mut output,
        "convert.set.value",
        SetUtils::to_set(Some(SetTarget::Set(source_view))),
    );
    emit_set_identity(
        &mut output,
        "convert.set.identity",
        SetUtils::to_set(Some(SetTarget::Set(source_view))),
        source_view,
    );
    emit_set_result(
        &mut output,
        "convert.set.sorted",
        SetUtils::to_set(Some(SetTarget::Set(sorted_view))),
    );

    let array = [
        Some("two".to_owned()),
        Some("one".to_owned()),
        Some("two".to_owned()),
        None,
    ];
    emit_set_result(
        &mut output,
        "convert.array.value",
        SetUtils::to_set(Some(SetTarget::Array(&array))),
    );
    emit_set_result(
        &mut output,
        "convert.array.empty",
        SetUtils::to_set(Some(SetTarget::Array(&[] as &[Option<String>]))),
    );
    emit_set_result(
        &mut output,
        "convert.iterable.value",
        SetUtils::to_set(Some(SetTarget::Iterable(Box::new(
            vec![
                Some("b".to_owned()),
                Some("a".to_owned()),
                Some("b".to_owned()),
                None,
            ]
            .into_iter(),
        )))),
    );
    emit_set_result(
        &mut output,
        "convert.iterable.empty",
        SetUtils::to_set(Some(SetTarget::Iterable(Box::new(
            Vec::<Option<String>>::new().into_iter(),
        )))),
    );
    emit_set_result(
        &mut output,
        "convert.null",
        SetUtils::to_set(None::<SetTarget<'_, Option<String>>>),
    );
    emit_set_result(
        &mut output,
        "convert.unsupported",
        SetUtils::to_set(Some(SetTarget::<Option<String>>::Unsupported(
            "java.lang.Integer",
        ))),
    );
    emit_set_result(
        &mut output,
        "convert.iterator_not_iterable",
        SetUtils::to_set(Some(SetTarget::<Option<String>>::Unsupported(
            "java.util.Arrays$ArrayItr",
        ))),
    );
    emit_set_result(
        &mut output,
        "convert.primitive_array",
        SetUtils::to_set(Some(SetTarget::<Option<String>>::PrimitiveArray("[I"))),
    );

    emit_validate(&mut output, "size.value", SetUtils::size(Some(source_view)));
    emit_validate(&mut output, "size.empty", SetUtils::size(Some(empty_view)));
    emit_validate(
        &mut output,
        "size.null",
        SetUtils::size(None::<&dyn SetView<Option<String>>>),
    );
    emit(
        &mut output,
        "empty.value",
        SetUtils::is_empty(Some(source_view)),
    );
    emit(
        &mut output,
        "empty.empty",
        SetUtils::is_empty(Some(empty_view)),
    );
    emit(
        &mut output,
        "empty.null",
        SetUtils::is_empty(None::<&dyn SetView<Option<String>>>),
    );

    emit_validate(
        &mut output,
        "contains.present",
        SetUtils::contains(Some(source_view), &Some("two".to_owned())),
    );
    emit_validate(
        &mut output,
        "contains.missing",
        SetUtils::contains(Some(source_view), &Some("missing".to_owned())),
    );
    emit_validate(
        &mut output,
        "contains.null",
        SetUtils::contains(Some(source_view), &None),
    );
    emit_validate(
        &mut output,
        "contains.null_target",
        SetUtils::contains(
            None::<&dyn SetView<Option<String>>>,
            &Some("two".to_owned()),
        ),
    );

    let present = [Some("two".to_owned()), None];
    let missing = [Some("two".to_owned()), Some("missing".to_owned())];
    let duplicate = [Some("two".to_owned()), Some("two".to_owned())];
    emit_validate(
        &mut output,
        "all.array.present",
        SetUtils::contains_all_array(Some(source_view), Some(&present)),
    );
    emit_validate(
        &mut output,
        "all.array.missing",
        SetUtils::contains_all_array(Some(source_view), Some(&missing)),
    );
    emit_validate(
        &mut output,
        "all.array.empty",
        SetUtils::contains_all_array(Some(source_view), Some(&[] as &[Option<String>])),
    );
    emit_validate(
        &mut output,
        "all.array.duplicate",
        SetUtils::contains_all_array(Some(source_view), Some(&duplicate)),
    );
    emit_validate(
        &mut output,
        "all.array.null_target",
        SetUtils::contains_all_array(
            None::<&dyn SetView<Option<String>>>,
            None::<&[Option<String>]>,
        ),
    );
    emit_validate(
        &mut output,
        "all.array.null_elements",
        SetUtils::contains_all_array(Some(source_view), None::<&[Option<String>]>),
    );

    emit_validate(
        &mut output,
        "all.collection.present",
        SetUtils::contains_all_collection(Some(source_view), Some(present.iter())),
    );
    emit_validate(
        &mut output,
        "all.collection.missing",
        SetUtils::contains_all_collection(Some(source_view), Some(missing.iter())),
    );
    emit_validate(
        &mut output,
        "all.collection.empty",
        SetUtils::contains_all_collection(
            Some(source_view),
            Some([].iter() as std::slice::Iter<'_, Option<String>>),
        ),
    );
    emit_validate(
        &mut output,
        "all.collection.duplicate",
        SetUtils::contains_all_collection(Some(source_view), Some(duplicate.iter())),
    );
    emit_validate(
        &mut output,
        "all.collection.null_target",
        SetUtils::contains_all_collection(
            None::<&dyn SetView<Option<String>>>,
            None::<std::slice::Iter<'_, Option<String>>>,
        ),
    );
    emit_validate(
        &mut output,
        "all.collection.null_elements",
        SetUtils::contains_all_collection(
            Some(source_view),
            None::<std::slice::Iter<'_, Option<String>>>,
        ),
    );

    let singleton = SetUtils::singleton_set(Some("one".to_owned()));
    let null_singleton = SetUtils::singleton_set(None::<String>);
    emit(&mut output, "singleton.value", render_set(&singleton));
    emit(&mut output, "singleton.null", render_set(&null_singleton));
    emit(
        &mut output,
        "singleton.unmodifiable",
        "java.lang.UnsupportedOperationException:null",
    );

    let sets = Sets::new();
    emit_set_result(
        &mut output,
        "facade.convert.value",
        sets.to_set(Some(SetTarget::Array(&array))),
    );
    emit_set_identity(
        &mut output,
        "facade.convert.identity",
        sets.to_set(Some(SetTarget::Set(source_view))),
        source_view,
    );
    emit_set_result(
        &mut output,
        "facade.convert.null",
        sets.to_set(None::<SetTarget<'_, Option<String>>>),
    );
    emit_validate(&mut output, "facade.size", sets.size(Some(source_view)));
    emit(
        &mut output,
        "facade.empty.null",
        sets.is_empty(None::<&dyn SetView<Option<String>>>),
    );
    emit_validate(
        &mut output,
        "facade.contains",
        sets.contains(Some(source_view), &None),
    );
    emit_validate(
        &mut output,
        "facade.all.array",
        sets.contains_all_array(Some(source_view), Some(&present)),
    );
    emit_validate(
        &mut output,
        "facade.all.collection",
        sets.contains_all_collection(Some(source_view), Some(present.iter())),
    );
    emit_validate(
        &mut output,
        "facade.all.array.null_target",
        sets.contains_all_array(
            None::<&dyn SetView<Option<String>>>,
            None::<&[Option<String>]>,
        ),
    );
    emit_validate(
        &mut output,
        "facade.all.collection.null_target",
        sets.contains_all_collection(
            None::<&dyn SetView<Option<String>>>,
            None::<std::slice::Iter<'_, Option<String>>>,
        ),
    );

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_set_result(
    output: &mut String,
    key: &str,
    result: Result<JavaSet<'_, Option<String>>, SetUtilsError>,
) {
    match result {
        Ok(value) => emit(output, key, render_set(&value)),
        Err(error) => emit(output, key, format_set_error(&error)),
    }
}

fn emit_set_identity(
    output: &mut String,
    key: &str,
    result: Result<JavaSet<'_, Option<String>>, SetUtilsError>,
    source: &dyn SetView<Option<String>>,
) {
    match result {
        Ok(value) => emit(output, key, value.is_borrowed_from(source)),
        Err(error) => emit(output, key, format_set_error(&error)),
    }
}

fn render_set<T>(values: &dyn SetView<Option<T>>) -> String
where
    T: Display,
{
    let rendered = values
        .iter()
        .map(|value| match value {
            Some(value) => value.to_string(),
            None => "<null>".to_owned(),
        })
        .collect::<Vec<_>>();
    format!("[{}]", rendered.join(","))
}

fn emit_validate<T>(output: &mut String, key: &str, result: Result<T, ValidateError>)
where
    T: Display,
{
    match result {
        Ok(value) => emit(output, key, value),
        Err(error) => emit(
            output,
            key,
            format!(
                "java.lang.IllegalArgumentException:{}",
                error.get_message().unwrap_or("null")
            ),
        ),
    }
}

fn format_set_error(error: &SetUtilsError) -> String {
    match error {
        SetUtilsError::Validation(error) => format!(
            "java.lang.IllegalArgumentException:{}",
            error.get_message().unwrap_or("null")
        ),
        SetUtilsError::CannotConvert { .. } => {
            format!("java.lang.IllegalArgumentException:{error}")
        }
        SetUtilsError::ClassCast { .. } => "java.lang.ClassCastException".to_owned(),
    }
}

fn emit(output: &mut String, key: &str, value: impl Display) {
    writeln!(output, "{key}={value}").expect("write output");
}
