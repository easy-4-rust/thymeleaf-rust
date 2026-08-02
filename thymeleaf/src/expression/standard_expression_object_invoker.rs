use std::any::Any;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::util::{AggregateUtils, EvaluationUtils, JavaNumber, JavaString};

use super::binary_operation_expression::{compare_java_values, java_values_equal};
use super::{
    Conversions, ExecutionInfo, Ids, JavaConversionResult, Messages, TemplateObjectMethodError,
    TemplateValue, Uris,
};

/// 调用无状态 Standard Expression 对象的 Java 方法面。
pub(crate) fn invoke_stateless_expression_object(
    object: &dyn Any,
    class_name: &str,
    method_name: &JavaString,
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError> {
    let method_name = method_name.to_string_lossy();
    let result = if let Some(messages) = object.downcast_ref::<Messages>() {
        invoke_messages(messages, &method_name, arguments)
    } else if let Some(ids) = object.downcast_ref::<Ids>() {
        invoke_ids(ids, &method_name, arguments)
    } else if let Some(conversions) = object.downcast_ref::<Conversions>() {
        invoke_conversions(conversions, &method_name, arguments)
    } else if let Some(execution_info) = object.downcast_ref::<ExecutionInfo>() {
        invoke_execution_info(execution_info, &method_name, arguments)
    } else {
        match class_name {
            "org.thymeleaf.expression.Bools" => invoke_bools(&method_name, arguments),
            "org.thymeleaf.expression.Aggregates" => invoke_aggregates(&method_name, arguments),
            "org.thymeleaf.expression.Arrays" => invoke_sequence(&method_name, arguments, "array"),
            "org.thymeleaf.expression.Lists" => invoke_sequence(&method_name, arguments, "list"),
            "org.thymeleaf.expression.Sets" => invoke_sequence(&method_name, arguments, "set"),
            "org.thymeleaf.expression.Maps" => invoke_maps(&method_name, arguments),
            "org.thymeleaf.expression.Objects" => invoke_objects(&method_name, arguments),
            "org.thymeleaf.expression.Uris" => invoke_uris(&method_name, arguments),
            _ => Err(invocation_error(format!(
                "Method {method_name} is not available on {class_name}"
            ))),
        }
    };
    result.map_err(|error| Box::new(error) as TemplateObjectMethodError)
}

/// 读取 Standard Expression 对象的 JavaBean 属性。
pub(crate) fn get_standard_expression_object_property(
    object: &dyn Any,
    property_name: &JavaString,
) -> Option<Arc<TemplateValue>> {
    let execution_info = object.downcast_ref::<ExecutionInfo>()?;
    match property_name.to_string_lossy().as_str() {
        "templateName" => execution_info
            .get_template_name()
            .map(|value| Arc::new(TemplateValue::string(value))),
        "templateMode" => mode_value(execution_info.get_template_mode()),
        "processedTemplateName" => execution_info
            .get_processed_template_name()
            .map(|value| Arc::new(TemplateValue::string(value))),
        "processedTemplateMode" => mode_value(execution_info.get_processed_template_mode()),
        "templateNames" => Some(Arc::new(TemplateValue::List(Arc::new(
            execution_info
                .get_template_names()
                .into_iter()
                .map(string_or_null)
                .collect(),
        )))),
        "templateModes" => Some(Arc::new(TemplateValue::List(Arc::new(
            execution_info
                .get_template_modes()
                .into_iter()
                .map(|value| mode_value(value).unwrap_or_else(|| Arc::new(TemplateValue::Null)))
                .collect(),
        )))),
        "now" => Some(crate::util::DateUtils::into_template_value(
            execution_info.get_now().clone(),
        )),
        "templateStack" => Some(template_stack_value(execution_info)),
        _ => None,
    }
}

fn invoke_execution_info(
    execution_info: &ExecutionInfo,
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<Arc<TemplateValue>>, InvocationError> {
    if !arguments.is_empty() {
        return Err(method_error("#execInfo", method, arguments));
    }
    // Java 侧 OGNL 以 bean 属性规则解析 `#execInfo.templateName` ->
    // `getTemplateName()`；Rust invoker 同时接受 getter 名与属性名。
    let property = method.strip_prefix("get").map(str::to_owned);
    let getter = property.as_deref().unwrap_or(method);
    Ok(match getter {
        "TemplateName" => execution_info
            .get_template_name()
            .map(|value| Arc::new(TemplateValue::string(value))),
        "TemplateMode" => mode_value(execution_info.get_template_mode()),
        "ProcessedTemplateName" => execution_info
            .get_processed_template_name()
            .map(|value| Arc::new(TemplateValue::string(value))),
        "ProcessedTemplateMode" => mode_value(execution_info.get_processed_template_mode()),
        "TemplateNames" => list_value(
            execution_info
                .get_template_names()
                .into_iter()
                .map(string_or_null)
                .collect(),
        ),
        "TemplateModes" => list_value(
            execution_info
                .get_template_modes()
                .into_iter()
                .map(|value| mode_value(value).unwrap_or_else(|| Arc::new(TemplateValue::Null)))
                .collect(),
        ),
        "TemplateStack" => Some(template_stack_value(execution_info)),
        "Now" => Some(crate::util::DateUtils::into_template_value(
            execution_info.get_now().clone(),
        )),
        _ => return Err(method_error("#execInfo", method, arguments)),
    })
}

fn template_stack_value(execution_info: &ExecutionInfo) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::List(Arc::new(
        execution_info
            .get_template_stack()
            .into_iter()
            .map(|value| Arc::new(TemplateValue::Object(value)))
            .collect(),
    )))
}

fn invoke_messages(
    messages: &Messages,
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<Arc<TemplateValue>>, InvocationError> {
    let collection =
        method.starts_with("array") || method.starts_with("list") || method.starts_with("set");
    let use_absent = !method.ends_with("OrNull");
    if collection {
        let keys = required_list(
            arguments
                .first()
                .ok_or_else(|| invocation_error("Message keys cannot be null"))?,
        )?;
        let parameters = &arguments[1..];
        let mut output = Vec::with_capacity(keys.len());
        for key in keys {
            let key = key
                .to_java_string()
                .ok_or_else(|| invocation_error("Message key cannot be null"))?;
            let value = if use_absent {
                messages.msg_with_params(&key, parameters)
            } else {
                messages.msg_or_null_with_params(&key, parameters)
            }
            .map_err(|error| invocation_error(error.to_string()))?;
            let value = string_or_null(value);
            if !method.starts_with("set") || !contains_value(&output, &value)? {
                output.push(value);
            }
        }
        return Ok(list_value(output));
    }
    if !matches!(method, "msg" | "msgOrNull") || arguments.is_empty() {
        return Err(method_error("#messages", method, arguments));
    }
    let key = required_string(&arguments[0], "Message key cannot be null")?;
    let result = if method == "msg" {
        messages.msg_with_params(&key, &arguments[1..])
    } else {
        messages.msg_or_null_with_params(&key, &arguments[1..])
    }
    .map_err(|error| invocation_error(error.to_string()))?;
    Ok(result.map(|value| Arc::new(TemplateValue::string(value))))
}

fn invoke_ids(
    ids: &Ids,
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<Arc<TemplateValue>>, InvocationError> {
    if arguments.len() != 1 {
        return Err(method_error("#ids", method, arguments));
    }
    let target = arguments[0].as_deref();
    let value = match method {
        // Java `Ids` 公开方法名为 seq/nextSeq/prevSeq；Rust 方法为
        // seq/next/prev，invoker 按 Java 名称注册并把别名一并接受。
        "seq" => ids.seq(target),
        "next" | "nextSeq" => ids.next(target),
        "prev" | "prevSeq" => ids.prev(target),
        _ => return Err(method_error("#ids", method, arguments)),
    }
    .map_err(|error| invocation_error(error.to_string()))?;
    Ok(Some(Arc::new(TemplateValue::string(value))))
}

fn invoke_conversions(
    conversions: &Conversions,
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<Arc<TemplateValue>>, InvocationError> {
    if method != "convert" || arguments.len() != 2 {
        return Err(method_error("#conversions", method, arguments));
    }
    let class_name = required_string(&arguments[1], "Class name cannot be null")?;
    let result = conversions
        .convert_by_class_name(arguments[0].as_deref(), Some(&class_name))
        .map_err(|error| invocation_error(error.to_string()))?;
    Ok(match result {
        JavaConversionResult::Null => None,
        JavaConversionResult::BorrowedString(value) => {
            Some(Arc::new(TemplateValue::string(value.clone())))
        }
        JavaConversionResult::OwnedString(value) => Some(Arc::new(TemplateValue::string(value))),
        JavaConversionResult::BorrowedObject(_) => arguments[0].clone(),
        JavaConversionResult::OwnedObject(value) => {
            if value.is::<TemplateValue>() {
                Some(Arc::new(
                    *value
                        .downcast::<TemplateValue>()
                        .expect("type checked before downcast"),
                ))
            } else if value.is::<JavaString>() {
                Some(Arc::new(TemplateValue::string(
                    *value
                        .downcast::<JavaString>()
                        .expect("type checked before downcast"),
                )))
            } else {
                return Err(invocation_error(
                    "Conversion service returned an unregistered dynamic object",
                ));
            }
        }
    })
}

fn invoke_bools(
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<Arc<TemplateValue>>, InvocationError> {
    match method {
        "isTrue" | "isFalse" if arguments.len() == 1 => {
            let value = arguments[0]
                .as_deref()
                .map(TemplateValue::to_evaluation_value)
                .unwrap_or(crate::util::JavaEvaluationValue::Null);
            let value = EvaluationUtils::evaluate_as_boolean(&value)
                .map_err(|error| invocation_error(error.to_string()))?;
            Ok(boolean_value(if method == "isFalse" {
                !value
            } else {
                value
            }))
        }
        "arrayIsTrue" | "listIsTrue" | "setIsTrue" | "arrayIsFalse" | "listIsFalse"
        | "setIsFalse"
            if arguments.len() == 1 =>
        {
            let values = required_list(&arguments[0])?;
            let negate = method.ends_with("False");
            let mut result = Vec::with_capacity(values.len());
            for value in values {
                let evaluated = EvaluationUtils::evaluate_as_boolean(&value.to_evaluation_value())
                    .map_err(|error| invocation_error(error.to_string()))?;
                let evaluated = if negate { !evaluated } else { evaluated };
                if method.starts_with("set") {
                    let candidate = Arc::new(TemplateValue::Boolean(evaluated));
                    if !contains_value(&result, &candidate)? {
                        result.push(candidate);
                    }
                } else {
                    result.push(Arc::new(TemplateValue::Boolean(evaluated)));
                }
            }
            Ok(list_value(result))
        }
        "arrayAnd" | "listAnd" | "setAnd" | "arrayOr" | "listOr" | "setOr"
            if arguments.len() == 1 =>
        {
            let values = required_list(&arguments[0])?;
            let use_and = method.ends_with("And");
            let mut answer = use_and;
            for value in values {
                let current = EvaluationUtils::evaluate_as_boolean(&value.to_evaluation_value())
                    .map_err(|error| invocation_error(error.to_string()))?;
                if use_and && !current {
                    answer = false;
                    break;
                }
                if !use_and && current {
                    answer = true;
                    break;
                }
            }
            Ok(boolean_value(answer))
        }
        _ => Err(method_error("#bools", method, arguments)),
    }
}

fn invoke_aggregates(
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<Arc<TemplateValue>>, InvocationError> {
    if !matches!(method, "sum" | "avg") || arguments.len() != 1 {
        return Err(method_error("#aggregates", method, arguments));
    }
    let values = required_list(&arguments[0])?
        .iter()
        .map(|value| match value.as_ref() {
            TemplateValue::Null => Ok(None),
            TemplateValue::Number(value) => Ok(Some(value.clone())),
            value => Err(invocation_error(format!(
                "Cannot aggregate non-number type {}",
                value.java_class_name()
            ))),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let result = if method == "sum" {
        AggregateUtils::sum_numbers(Some(&values))
    } else {
        AggregateUtils::avg_numbers(Some(&values))
    }
    .map_err(|error| invocation_error(error.to_string()))?;
    Ok(result.map(|value| Arc::new(TemplateValue::Number(JavaNumber::BigDecimal(value)))))
}

fn invoke_sequence(
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
    family: &str,
) -> Result<Option<Arc<TemplateValue>>, InvocationError> {
    match method {
        "toArray" | "toList" | "toSet" if arguments.len() == 1 => {
            let mut values = sequence_values(&arguments[0])?;
            if method == "toSet" {
                values = dedupe(values)?;
            }
            Ok(list_value(values))
        }
        "toStringArray" | "toIntegerArray" | "toLongArray" | "toDoubleArray" | "toFloatArray"
        | "toBooleanArray"
            if arguments.len() == 1 =>
        {
            let values = sequence_values(&arguments[0])?;
            values
                .iter()
                .map(|value| convert_sequence_value(method, value))
                .collect::<Result<Vec<_>, _>>()
                .map(list_value)
        }
        "length" | "size" if arguments.len() == 1 => {
            let values = required_list(&arguments[0])?;
            Ok(integer_value(
                i32::try_from(values.len()).unwrap_or(i32::MAX),
            ))
        }
        "isEmpty" if arguments.len() == 1 => {
            let empty = arguments[0]
                .as_deref()
                .is_none_or(|value| matches!(value, TemplateValue::Null))
                || required_list(&arguments[0])?.is_empty();
            Ok(boolean_value(empty))
        }
        "contains" if arguments.len() == 2 => {
            let values = required_list(&arguments[0])?;
            let candidate = java_null(&arguments[1]);
            Ok(boolean_value(contains_value(values, &candidate)?))
        }
        "containsAll" if arguments.len() == 2 => {
            let values = required_list(&arguments[0])?;
            let required = required_list(&arguments[1])?;
            for candidate in required {
                if !contains_value(values, candidate)? {
                    return Ok(boolean_value(false));
                }
            }
            Ok(boolean_value(true))
        }
        "sort" if family == "list" && arguments.len() == 1 => {
            let mut values = required_list(&arguments[0])?.to_vec();
            stable_sort(&mut values)?;
            Ok(list_value(values))
        }
        _ => Err(method_error(
            &format!("#{family}s").replace("arrays", "arrays"),
            method,
            arguments,
        )),
    }
}

fn invoke_maps(
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<Arc<TemplateValue>>, InvocationError> {
    let entries = match arguments.first().and_then(Option::as_deref) {
        Some(TemplateValue::Map(entries)) => entries.as_slice(),
        Some(TemplateValue::Null) | None => {
            if method == "isEmpty" {
                return Ok(boolean_value(true));
            }
            return Err(invocation_error("Target cannot be null"));
        }
        Some(value) => {
            return Err(invocation_error(format!(
                "Target of class {} is not a Map",
                value.java_class_name()
            )));
        }
    };
    match method {
        "size" if arguments.len() == 1 => Ok(integer_value(
            i32::try_from(entries.len()).unwrap_or(i32::MAX),
        )),
        "isEmpty" if arguments.len() == 1 => Ok(boolean_value(entries.is_empty())),
        "containsKey" | "containsValue" if arguments.len() == 2 => {
            let candidate = java_null(&arguments[1]);
            let mut found = false;
            for (key, value) in entries {
                let target = if method == "containsKey" { key } else { value };
                if java_values_equal(Some(target), Some(&candidate))
                    .map_err(|error| invocation_error(error.to_string()))?
                {
                    found = true;
                    break;
                }
            }
            Ok(boolean_value(found))
        }
        "containsAllKeys" | "containsAllValues" if arguments.len() == 2 => {
            let required = required_list(&arguments[1])?;
            let keys = method == "containsAllKeys";
            for candidate in required {
                let mut found = false;
                for (key, value) in entries {
                    let target = if keys { key } else { value };
                    if java_values_equal(Some(target), Some(candidate))
                        .map_err(|error| invocation_error(error.to_string()))?
                    {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Ok(boolean_value(false));
                }
            }
            Ok(boolean_value(true))
        }
        _ => Err(method_error("#maps", method, arguments)),
    }
}

fn invoke_objects(
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<Arc<TemplateValue>>, InvocationError> {
    if method != "nullSafe" || arguments.len() != 2 {
        return Err(method_error("#objects", method, arguments));
    }
    let default = java_null(&arguments[1]);
    match arguments[0].as_deref() {
        None | Some(TemplateValue::Null) => Ok(Some(default)),
        Some(TemplateValue::List(values)) => {
            let mut output = Vec::with_capacity(values.len());
            for value in values.iter() {
                if matches!(value.as_ref(), TemplateValue::Null) {
                    output.push(Arc::clone(&default));
                } else {
                    output.push(Arc::clone(value));
                }
            }
            Ok(list_value(output))
        }
        Some(_) => Ok(arguments[0].clone()),
    }
}

fn invoke_uris(
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<Arc<TemplateValue>>, InvocationError> {
    if !matches!(arguments.len(), 1 | 2) {
        return Err(method_error("#uris", method, arguments));
    }
    let text = optional_string(&arguments[0]);
    let encoding = arguments.get(1).and_then(optional_string);
    let uris = Uris::new();
    let result = match method {
        "escapePath" => uris.escape_path_with_encoding(text.as_ref(), encoding.as_ref()),
        "unescapePath" => uris.unescape_path_with_encoding(text.as_ref(), encoding.as_ref()),
        "escapePathSegment" => {
            uris.escape_path_segment_with_encoding(text.as_ref(), encoding.as_ref())
        }
        "unescapePathSegment" => {
            uris.unescape_path_segment_with_encoding(text.as_ref(), encoding.as_ref())
        }
        "escapeFragmentId" => {
            uris.escape_fragment_id_with_encoding(text.as_ref(), encoding.as_ref())
        }
        "unescapeFragmentId" => {
            uris.unescape_fragment_id_with_encoding(text.as_ref(), encoding.as_ref())
        }
        "escapeQueryParam" => {
            uris.escape_query_param_with_encoding(text.as_ref(), encoding.as_ref())
        }
        "unescapeQueryParam" => {
            uris.unescape_query_param_with_encoding(text.as_ref(), encoding.as_ref())
        }
        _ => return Err(method_error("#uris", method, arguments)),
    }
    .map_err(|error| invocation_error(error.to_string()))?;
    Ok(result.map(|value| Arc::new(TemplateValue::string(value))))
}

fn convert_sequence_value(
    method: &str,
    value: &Arc<TemplateValue>,
) -> Result<Arc<TemplateValue>, InvocationError> {
    if matches!(value.as_ref(), TemplateValue::Null) {
        return Ok(Arc::new(TemplateValue::Null));
    }
    let compatible = matches!(
        (method, value.as_ref()),
        (
            "toStringArray",
            TemplateValue::String(_) | TemplateValue::SafeHtml(_)
        ) | (
            "toIntegerArray",
            TemplateValue::Number(JavaNumber::Integer(_))
        ) | ("toLongArray", TemplateValue::Number(JavaNumber::Long(_)))
            | (
                "toDoubleArray",
                TemplateValue::Number(JavaNumber::Double(_))
            )
            | ("toFloatArray", TemplateValue::Number(JavaNumber::Float(_)))
            | ("toBooleanArray", TemplateValue::Boolean(_))
    );
    if compatible {
        Ok(Arc::clone(value))
    } else {
        let component = method
            .strip_prefix("to")
            .and_then(|value| value.strip_suffix("Array"))
            .unwrap_or("Object");
        Err(invocation_error(format!(
            "Cannot store object of class \"{}\" in {component}[]",
            value.java_class_name()
        )))
    }
}

fn sequence_values(
    value: &Option<Arc<TemplateValue>>,
) -> Result<Vec<Arc<TemplateValue>>, InvocationError> {
    match value.as_deref() {
        Some(TemplateValue::List(values)) => Ok(values.as_ref().clone()),
        // Java 的 Object[] 强制转换拒绝 primitive array；`Bytes` 对应 byte[]，
        // 不能为了方便而把它静默装箱成 Byte 列表。
        Some(TemplateValue::Bytes(_)) => Err(invocation_error(
            "Cannot convert primitive byte[] to Object[]",
        )),
        Some(TemplateValue::Object(object)) => object
            .java_iterable_values()
            .ok_or_else(|| invocation_error("Cannot convert target to array/List/Set")),
        Some(TemplateValue::Null) | None => Err(invocation_error("Target cannot be null")),
        Some(value) => Err(invocation_error(format!(
            "Cannot convert object of class \"{}\" to array/List/Set",
            value.java_class_name()
        ))),
    }
}

fn required_list(
    value: &Option<Arc<TemplateValue>>,
) -> Result<&[Arc<TemplateValue>], InvocationError> {
    match value.as_deref() {
        Some(TemplateValue::List(values)) => Ok(values),
        Some(TemplateValue::Null) | None => Err(invocation_error("Target cannot be null")),
        Some(value) => Err(invocation_error(format!(
            "Target of class {} is not an array, List or Set",
            value.java_class_name()
        ))),
    }
}

fn optional_string(value: &Option<Arc<TemplateValue>>) -> Option<JavaString> {
    match value.as_deref() {
        None | Some(TemplateValue::Null) => None,
        Some(value) => value.to_java_string(),
    }
}

fn required_string(
    value: &Option<Arc<TemplateValue>>,
    message: &str,
) -> Result<JavaString, InvocationError> {
    optional_string(value).ok_or_else(|| invocation_error(message))
}

fn string_or_null(value: Option<JavaString>) -> Arc<TemplateValue> {
    value.map_or_else(
        || Arc::new(TemplateValue::Null),
        |value| Arc::new(TemplateValue::string(value)),
    )
}

fn mode_value(value: Option<crate::TemplateMode>) -> Option<Arc<TemplateValue>> {
    value.map(|value| Arc::new(TemplateValue::Object(Arc::new(value))))
}

fn java_null(value: &Option<Arc<TemplateValue>>) -> Arc<TemplateValue> {
    value
        .clone()
        .unwrap_or_else(|| Arc::new(TemplateValue::Null))
}

fn contains_value(
    values: &[Arc<TemplateValue>],
    candidate: &Arc<TemplateValue>,
) -> Result<bool, InvocationError> {
    for value in values {
        if java_values_equal(Some(value), Some(candidate))
            .map_err(|error| invocation_error(error.to_string()))?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn dedupe(values: Vec<Arc<TemplateValue>>) -> Result<Vec<Arc<TemplateValue>>, InvocationError> {
    let mut output = Vec::with_capacity(values.len());
    for value in values {
        if !contains_value(&output, &value)? {
            output.push(value);
        }
    }
    Ok(output)
}

fn stable_sort(values: &mut [Arc<TemplateValue>]) -> Result<(), InvocationError> {
    for index in 1..values.len() {
        let mut current = index;
        while current > 0 {
            let ordering = compare_java_values(&values[current - 1], &values[current])
                .map_err(|error| invocation_error(error.to_string()))?
                .ok_or_else(|| invocation_error("Elements are not mutually Comparable"))?;
            if ordering != Ordering::Greater {
                break;
            }
            values.swap(current - 1, current);
            current -= 1;
        }
    }
    Ok(())
}

fn list_value(values: Vec<Arc<TemplateValue>>) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::List(Arc::new(values))))
}

fn boolean_value(value: bool) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::Boolean(value)))
}

fn integer_value(value: i32) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(value))))
}

fn method_error(
    object: &str,
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> InvocationError {
    invocation_error(format!(
        "Method {method} with {} arguments is not available on {object}",
        arguments.len()
    ))
}

fn invocation_error(message: impl Into<String>) -> InvocationError {
    InvocationError {
        message: message.into(),
    }
}

#[derive(Debug)]
struct InvocationError {
    message: String,
}

impl Display for InvocationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for InvocationError {}
