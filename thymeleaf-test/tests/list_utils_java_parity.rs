//! `ListUtils` 与 `Lists` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::cmp::Ordering;
use std::collections::LinkedList;
use std::fmt::{Display, Write};

use thymeleaf::expression::Lists;
use thymeleaf::util::{
    ComparableValue, ListTarget, ListTypeValue, ListUtils, ListUtilsError, ListValue, ListView,
    ValidateError,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/list_utils_golden.txt");

struct CustomList<T> {
    values: Vec<T>,
    list_type: ListTypeValue,
}

impl<T> ListView<T> for CustomList<T> {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.values.get(index)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.values.iter())
    }

    fn list_type(&self) -> ListTypeValue {
        self.list_type.clone()
    }
}

struct AddFailingList<T> {
    values: Vec<T>,
}

impl<T> ListView<T> for AddFailingList<T> {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.values.get(index)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.values.iter())
    }

    fn list_type(&self) -> ListTypeValue {
        ListTypeValue::custom("ListUtilsGolden$AddFailingList", true)
    }

    fn fill_sorted(&self, _elements: Vec<T>) -> Result<ListValue<'static, T>, ListUtilsError>
    where
        T: 'static,
    {
        Err(ListUtilsError::runtime(
            "java.lang.UnsupportedOperationException",
            "add failed",
        ))
    }
}

#[derive(Clone)]
enum Mixed {
    Text(String),
    Integer(i32),
}

impl ComparableValue for Mixed {
    fn java_compare_to(&self, other: &Self) -> Result<Ordering, ListUtilsError> {
        match (self, other) {
            (Self::Text(left), Self::Text(right)) => left.java_compare_to(right),
            (Self::Integer(left), Self::Integer(right)) => left.java_compare_to(right),
            (Self::Text(_), Self::Integer(_)) => Err(ListUtilsError::NaturalOrderingClassCast {
                left_class: "java.lang.String".to_owned(),
                right_class: "java.lang.Integer".to_owned(),
            }),
            (Self::Integer(_), Self::Text(_)) => Err(ListUtilsError::NaturalOrderingClassCast {
                left_class: "java.lang.Integer".to_owned(),
                right_class: "java.lang.String".to_owned(),
            }),
        }
    }
}

#[test]
fn list_utils_and_expression_facade_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    let source = LinkedList::from([
        Some("two".to_owned()),
        Some("one".to_owned()),
        Some("two".to_owned()),
        None,
    ]);
    let source_view: &dyn ListView<Option<String>> = &source;
    let empty = Vec::<Option<String>>::new();
    let empty_view: &dyn ListView<Option<String>> = &empty;

    emit_list_result(
        &mut output,
        "convert.list.value",
        ListUtils::to_list(Some(ListTarget::List(source_view))),
        render_nullable_string,
    );
    emit_list_identity(
        &mut output,
        "convert.list.identity",
        ListUtils::to_list(Some(ListTarget::List(source_view))),
        source_view,
    );
    emit_list_type(
        &mut output,
        "convert.list.type",
        ListUtils::to_list(Some(ListTarget::List(source_view))),
    );

    let array = [
        Some("two".to_owned()),
        Some("one".to_owned()),
        Some("two".to_owned()),
        None,
    ];
    emit_list_result(
        &mut output,
        "convert.array.value",
        ListUtils::to_list(Some(ListTarget::Array(&array))),
        render_nullable_string,
    );
    emit_list_type(
        &mut output,
        "convert.array.type",
        ListUtils::to_list(Some(ListTarget::Array(&[Some("one".to_owned())]))),
    );
    emit_list_result(
        &mut output,
        "convert.array.empty",
        ListUtils::to_list(Some(ListTarget::Array(&[] as &[Option<String>]))),
        render_nullable_string,
    );
    emit_list_result(
        &mut output,
        "convert.iterable.value",
        ListUtils::to_list(Some(ListTarget::Iterable(Box::new(
            vec![
                Some("b".to_owned()),
                Some("a".to_owned()),
                Some("b".to_owned()),
                None,
            ]
            .into_iter(),
        )))),
        render_nullable_string,
    );
    emit_list_type(
        &mut output,
        "convert.iterable.type",
        ListUtils::to_list(Some(ListTarget::Iterable(Box::new(
            Vec::<Option<String>>::new().into_iter(),
        )))),
    );
    emit_list_result(
        &mut output,
        "convert.iterable.empty",
        ListUtils::to_list(Some(ListTarget::Iterable(Box::new(
            Vec::<Option<String>>::new().into_iter(),
        )))),
        render_nullable_string,
    );
    emit_list_result(
        &mut output,
        "convert.null",
        ListUtils::to_list(None::<ListTarget<'_, Option<String>>>),
        render_nullable_string,
    );
    emit_list_result(
        &mut output,
        "convert.unsupported",
        ListUtils::to_list(Some(ListTarget::<Option<String>>::Unsupported(
            "java.lang.Integer",
        ))),
        render_nullable_string,
    );
    emit_list_result(
        &mut output,
        "convert.iterator_not_iterable",
        ListUtils::to_list(Some(ListTarget::<Option<String>>::Unsupported(
            "java.util.Arrays$ArrayItr",
        ))),
        render_nullable_string,
    );
    emit_list_result(
        &mut output,
        "convert.primitive_array",
        ListUtils::to_list(Some(ListTarget::<Option<String>>::PrimitiveArray("[I"))),
        render_nullable_string,
    );

    emit_validate(
        &mut output,
        "size.value",
        ListUtils::size(Some(source_view)),
    );
    emit_validate(&mut output, "size.empty", ListUtils::size(Some(empty_view)));
    emit_validate(
        &mut output,
        "size.null",
        ListUtils::size(None::<&dyn ListView<Option<String>>>),
    );
    emit(
        &mut output,
        "empty.value",
        ListUtils::is_empty(Some(source_view)),
    );
    emit(
        &mut output,
        "empty.empty",
        ListUtils::is_empty(Some(empty_view)),
    );
    emit(
        &mut output,
        "empty.null",
        ListUtils::is_empty(None::<&dyn ListView<Option<String>>>),
    );
    emit_validate(
        &mut output,
        "contains.present",
        ListUtils::contains(Some(source_view), &Some("two".to_owned())),
    );
    emit_validate(
        &mut output,
        "contains.missing",
        ListUtils::contains(Some(source_view), &Some("missing".to_owned())),
    );
    emit_validate(
        &mut output,
        "contains.null",
        ListUtils::contains(Some(source_view), &None),
    );
    emit_validate(
        &mut output,
        "contains.null_target",
        ListUtils::contains(
            None::<&dyn ListView<Option<String>>>,
            &Some("two".to_owned()),
        ),
    );

    let present = [Some("two".to_owned()), None];
    let missing = [Some("two".to_owned()), Some("missing".to_owned())];
    let duplicate = [Some("two".to_owned()), Some("two".to_owned())];
    emit_validate(
        &mut output,
        "all.array.present",
        ListUtils::contains_all_array(Some(source_view), Some(&present)),
    );
    emit_validate(
        &mut output,
        "all.array.missing",
        ListUtils::contains_all_array(Some(source_view), Some(&missing)),
    );
    emit_validate(
        &mut output,
        "all.array.empty",
        ListUtils::contains_all_array(Some(source_view), Some(&[] as &[Option<String>])),
    );
    emit_validate(
        &mut output,
        "all.array.duplicate",
        ListUtils::contains_all_array(Some(source_view), Some(&duplicate)),
    );
    emit_validate(
        &mut output,
        "all.array.null_target",
        ListUtils::contains_all_array(
            None::<&dyn ListView<Option<String>>>,
            None::<&[Option<String>]>,
        ),
    );
    emit_validate(
        &mut output,
        "all.array.null_elements",
        ListUtils::contains_all_array(Some(source_view), None::<&[Option<String>]>),
    );
    emit_validate(
        &mut output,
        "all.collection.present",
        ListUtils::contains_all_collection(Some(source_view), Some(present.iter())),
    );
    emit_validate(
        &mut output,
        "all.collection.missing",
        ListUtils::contains_all_collection(Some(source_view), Some(missing.iter())),
    );
    emit_validate(
        &mut output,
        "all.collection.empty",
        ListUtils::contains_all_collection(
            Some(source_view),
            Some([].iter() as std::slice::Iter<'_, Option<String>>),
        ),
    );
    emit_validate(
        &mut output,
        "all.collection.duplicate",
        ListUtils::contains_all_collection(Some(source_view), Some(duplicate.iter())),
    );
    emit_validate(
        &mut output,
        "all.collection.null_target",
        ListUtils::contains_all_collection(
            None::<&dyn ListView<Option<String>>>,
            None::<std::slice::Iter<'_, Option<String>>>,
        ),
    );
    emit_validate(
        &mut output,
        "all.collection.null_elements",
        ListUtils::contains_all_collection(
            Some(source_view),
            None::<std::slice::Iter<'_, Option<String>>>,
        ),
    );

    let linked = LinkedList::from(["c".to_owned(), "a".to_owned(), "b".to_owned()]);
    let linked_view: &dyn ListView<String> = &linked;
    emit_list_result(
        &mut output,
        "sort.linked.value",
        ListUtils::sort(Some(linked_view)),
        Clone::clone,
    );
    emit_list_type(
        &mut output,
        "sort.linked.type",
        ListUtils::sort(Some(linked_view)),
    );
    emit(
        &mut output,
        "sort.linked.original",
        render_view(linked_view, Clone::clone),
    );
    emit(
        &mut output,
        "sort.linked.identity",
        ListUtils::sort(Some(linked_view))
            .map(|list| list.is_borrowed_from(linked_view))
            .unwrap(),
    );

    let fixed = CustomList {
        values: vec!["c".to_owned(), "a".to_owned(), "b".to_owned()],
        list_type: ListTypeValue::custom("java.util.Arrays$ArrayList", false),
    };
    emit_list_result(
        &mut output,
        "sort.fixed.value",
        ListUtils::sort(Some(&fixed)),
        Clone::clone,
    );
    emit_list_type(
        &mut output,
        "sort.fixed.type",
        ListUtils::sort(Some(&fixed)),
    );
    let public_list = CustomList {
        values: vec!["c".to_owned(), "a".to_owned(), "b".to_owned()],
        list_type: ListTypeValue::custom("ListUtilsGolden$PublicList", true),
    };
    emit_list_result(
        &mut output,
        "sort.public.value",
        ListUtils::sort(Some(&public_list)),
        Clone::clone,
    );
    emit_list_type(
        &mut output,
        "sort.public.type",
        ListUtils::sort(Some(&public_list)),
    );
    let private_list = CustomList {
        values: vec!["c".to_owned(), "a".to_owned(), "b".to_owned()],
        list_type: ListTypeValue::custom("ListUtilsGolden$PrivateList", false),
    };
    emit_list_result(
        &mut output,
        "sort.private.value",
        ListUtils::sort(Some(&private_list)),
        Clone::clone,
    );
    emit_list_type(
        &mut output,
        "sort.private.type",
        ListUtils::sort(Some(&private_list)),
    );
    let add_failing_list = AddFailingList {
        values: vec!["c".to_owned(), "a".to_owned(), "b".to_owned()],
    };
    emit_list_result(
        &mut output,
        "sort.add_failure",
        ListUtils::sort(Some(&add_failing_list)),
        Clone::clone,
    );
    emit_list_result(
        &mut output,
        "sort.null_list",
        ListUtils::sort(None::<&dyn ListView<String>>),
        Clone::clone,
    );
    let null_element = vec![Some("a".to_owned()), None];
    emit_list_result(
        &mut output,
        "sort.null_element",
        ListUtils::sort(Some(&null_element)),
        render_nullable_string,
    );
    let heterogeneous = vec![Mixed::Text("a".to_owned()), Mixed::Integer(1)];
    emit_list_result(
        &mut output,
        "sort.heterogeneous",
        ListUtils::sort(Some(&heterogeneous)),
        |_| "unused".to_owned(),
    );
    let utf16 = vec!["\u{e000}".to_owned(), "\u{1f600}".to_owned()];
    emit_list_result(
        &mut output,
        "sort.utf16",
        ListUtils::sort(Some(&utf16)),
        Clone::clone,
    );
    let doubles = vec![f64::NAN, 0.0, -0.0, f64::INFINITY, -1.0];
    emit_list_result(
        &mut output,
        "sort.double",
        ListUtils::sort(Some(&doubles)),
        render_java_double,
    );

    let mut descending = |left: &String, right: &String| right.java_compare_to(left);
    emit_list_result(
        &mut output,
        "sort.comparator.descending",
        ListUtils::sort_with_comparator(Some(linked_view), Some(&mut descending)),
        Clone::clone,
    );
    emit_list_result(
        &mut output,
        "sort.comparator.null",
        ListUtils::sort_with_comparator(Some(linked_view), None),
        Clone::clone,
    );
    let stable = vec!["b1".to_owned(), "a".to_owned(), "b2".to_owned()];
    let mut by_length = |left: &String, right: &String| Ok(left.len().cmp(&right.len()));
    emit_list_result(
        &mut output,
        "sort.comparator.stable",
        ListUtils::sort_with_comparator(Some(&stable), Some(&mut by_length)),
        Clone::clone,
    );
    let mut failing_comparator = |_left: &String, _right: &String| {
        Err(ListUtilsError::runtime(
            "java.lang.IllegalStateException",
            "compare failed",
        ))
    };
    emit_list_result(
        &mut output,
        "sort.comparator.failure",
        ListUtils::sort_with_comparator(Some(linked_view), Some(&mut failing_comparator)),
        Clone::clone,
    );

    let lists = Lists::new();
    emit_list_result(
        &mut output,
        "facade.convert.value",
        lists.to_list(Some(ListTarget::Array(&array))),
        render_nullable_string,
    );
    emit_list_identity(
        &mut output,
        "facade.convert.identity",
        lists.to_list(Some(ListTarget::List(source_view))),
        source_view,
    );
    emit_list_result(
        &mut output,
        "facade.convert.null",
        lists.to_list(None::<ListTarget<'_, Option<String>>>),
        render_nullable_string,
    );
    emit_validate(&mut output, "facade.size", lists.size(Some(source_view)));
    emit(
        &mut output,
        "facade.empty.null",
        lists.is_empty(None::<&dyn ListView<Option<String>>>),
    );
    emit_validate(
        &mut output,
        "facade.contains",
        lists.contains(Some(source_view), &None),
    );
    emit_validate(
        &mut output,
        "facade.all.array",
        lists.contains_all_array(Some(source_view), Some(&present)),
    );
    emit_validate(
        &mut output,
        "facade.all.collection",
        lists.contains_all_collection(Some(source_view), Some(present.iter())),
    );
    emit_list_result(
        &mut output,
        "facade.sort",
        lists.sort(Some(linked_view)),
        Clone::clone,
    );
    let mut descending = |left: &String, right: &String| right.java_compare_to(left);
    emit_list_result(
        &mut output,
        "facade.sort.comparator",
        lists.sort_with_comparator(Some(linked_view), Some(&mut descending)),
        Clone::clone,
    );

    // 覆盖四元素排序左半区 Comparator 异常的递归传播；该路径不增加 Golden 字段。
    let four_strings = vec![
        "a".to_owned(),
        "b".to_owned(),
        "c".to_owned(),
        "d".to_owned(),
    ];
    let mut fail_in_left_half = |_left: &String, _right: &String| {
        Err(ListUtilsError::runtime(
            "java.lang.IllegalStateException",
            "left comparison failed",
        ))
    };
    assert!(
        ListUtils::sort_with_comparator(Some(&four_strings), Some(&mut fail_in_left_half)).is_err()
    );
    let owned_array = ListUtils::to_list(Some(ListTarget::Array(&array))).unwrap();
    assert!(!owned_array.is_borrowed_from(source_view));
    let mut failing_double_comparator = |_left: &f64, _right: &f64| {
        Err(ListUtilsError::runtime(
            "java.lang.IllegalStateException",
            "double comparison failed",
        ))
    };
    assert!(
        ListUtils::sort_with_comparator(Some(&doubles), Some(&mut failing_double_comparator))
            .is_err()
    );

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_list_result<T, F>(
    output: &mut String,
    key: &str,
    result: Result<ListValue<'_, T>, ListUtilsError>,
    render_element: F,
) where
    F: Fn(&T) -> String,
{
    match result {
        Ok(value) => emit(output, key, render_view(&value, render_element)),
        Err(error) => emit(output, key, format_list_error(&error)),
    }
}

fn emit_list_identity<T>(
    output: &mut String,
    key: &str,
    result: Result<ListValue<'_, T>, ListUtilsError>,
    source: &dyn ListView<T>,
) {
    match result {
        Ok(value) => emit(output, key, value.is_borrowed_from(source)),
        Err(error) => emit(output, key, format_list_error(&error)),
    }
}

fn emit_list_type<T>(
    output: &mut String,
    key: &str,
    result: Result<ListValue<'_, T>, ListUtilsError>,
) {
    match result {
        Ok(value) => emit(output, key, value.list_type().class_name()),
        Err(error) => emit(output, key, format_list_error(&error)),
    }
}

fn render_view<T, F>(values: &dyn ListView<T>, render_element: F) -> String
where
    F: Fn(&T) -> String,
{
    let rendered = values.iter().map(render_element).collect::<Vec<_>>();
    format!("[{}]", rendered.join(","))
}

fn render_nullable_string(value: &Option<String>) -> String {
    value.clone().unwrap_or_else(|| "<null>".to_owned())
}

fn render_java_double(value: &f64) -> String {
    if value.is_nan() {
        "NaN".to_owned()
    } else if value.is_infinite() {
        if value.is_sign_negative() {
            "-Infinity".to_owned()
        } else {
            "Infinity".to_owned()
        }
    } else if *value == 0.0 {
        if value.is_sign_negative() {
            "-0.0".to_owned()
        } else {
            "0.0".to_owned()
        }
    } else if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
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

fn format_list_error(error: &ListUtilsError) -> String {
    match error {
        ListUtilsError::Validation(error) => format!(
            "java.lang.IllegalArgumentException:{}",
            error.get_message().unwrap_or("null")
        ),
        ListUtilsError::CannotConvert { .. } => {
            format!("java.lang.IllegalArgumentException:{error}")
        }
        ListUtilsError::ClassCast { .. } | ListUtilsError::NaturalOrderingClassCast { .. } => {
            "java.lang.ClassCastException".to_owned()
        }
        ListUtilsError::NaturalOrderingNull => "java.lang.NullPointerException".to_owned(),
        ListUtilsError::Runtime {
            class_name,
            message,
        } => format!("{class_name}:{message}"),
    }
}

fn emit(output: &mut String, key: &str, value: impl Display) {
    writeln!(output, "{key}={value}").expect("write output");
}
