use std::sync::Arc;

use thymeleaf::expression::{OgnlRuntime, OgnlRuntimeError, TemplateValue};
use thymeleaf::util::{NumberValue, Utf16String};

use super::corpus_byte_array_input_stream::CorpusByteArrayInputStream;
use super::corpus_optional::CorpusOptional;
use super::corpus_proxy::CorpusProxy;
use super::corpus_simple_date_format::CorpusSimpleDateFormat;
use super::corpus_string_array::CorpusStringArray;
use super::exception_lazy_context_variable::ExceptionLazyContextVariable;
use super::exception_throwing_bean::ExceptionThrowingBean;
use super::lazy_expression_returner::LazyExpressionReturner;
use super::list_lazy_context_variable::ListLazyContextVariable;
use super::value_lazy_context_variable::ValueLazyContextVariable;

/// 仅注册上游 `.thtest` 所需宿主构造器的 OGNL 测试运行时。
///
/// 该对象不扩大 `thymeleaf` 默认运行时权限。
pub struct CorpusOgnlRuntime;

impl OgnlRuntime for CorpusOgnlRuntime {
    fn read_static_field(
        &self,
        type_name: &Utf16String,
        member_name: &Utf16String,
    ) -> Option<Result<Option<Arc<TemplateValue>>, OgnlRuntimeError>> {
        let type_name = type_name.to_string_lossy();
        let member_name = member_name.to_string_lossy();
        match (type_name.as_str(), member_name.as_str()) {
            ("java.util.concurrent.TimeUnit", "class") => {
                Some(Ok(Some(Arc::new(TemplateValue::string(
                    Utf16String::from_rust_str("class java.util.concurrent.TimeUnit"),
                )))))
            }
            ("java.util.concurrent.TimeUnit", "MILLISECONDS" | "SECONDS") => {
                Some(Ok(Some(Arc::new(TemplateValue::string(
                    Utf16String::from_rust_str(&member_name),
                )))))
            }
            _ => None,
        }
    }

    fn invoke_static_method(
        &self,
        type_name: &Utf16String,
        method_name: &Utf16String,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, OgnlRuntimeError>> {
        let type_name = type_name.to_string_lossy();
        let method_name = method_name.to_string_lossy();
        match (type_name.as_str(), method_name.as_str(), arguments) {
            ("java.util.TimeZone", "getTimeZone", [Some(value)]) => {
                Some(Ok(Some(Arc::new(TemplateValue::string(
                    value
                        .to_utf16_string()
                        .unwrap_or_else(|| Utf16String::from_rust_str("GMT")),
                )))))
            }
            ("java.util.Optional", "of", [Some(value)]) => {
                Some(optional("java.util.Optional", value))
            }
            ("java.util.OptionalInt", "of", [Some(value)]) => {
                Some(optional("java.util.OptionalInt", value))
            }
            ("java.util.OptionalLong", "of", [Some(value)]) => {
                Some(optional("java.util.OptionalLong", value))
            }
            ("java.util.OptionalDouble", "of", [Some(value)]) => {
                Some(optional("java.util.OptionalDouble", value))
            }
            ("java.util.Optional", "empty", []) => Some(Ok(Some(Arc::new(TemplateValue::Object(
                Arc::new(CorpusOptional::empty("java.util.Optional")),
            ))))),
            ("java.util.UUID", "fromString", [Some(value)]) => {
                Some(Ok(Some(Arc::new(TemplateValue::string(
                    value
                        .to_utf16_string()
                        .unwrap_or_else(|| Utf16String::from_rust_str("")),
                )))))
            }
            ("org.thymeleaf.templateengine.features.TestProxyFactory", "createTestProxy", []) => {
                Some(Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
                    CorpusProxy,
                ))))))
            }
            _ => None,
        }
    }

    fn construct(
        &self,
        type_name: &Utf16String,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, OgnlRuntimeError>> {
        let type_name = type_name.to_string_lossy();
        match (type_name.as_str(), arguments) {
            ("java.lang.String[]", values) => Some(Ok(Some(Arc::new(TemplateValue::Object(
                Arc::new(CorpusStringArray::new(
                    values
                        .iter()
                        .map(|value| value.clone().unwrap_or_else(null_value))
                        .collect(),
                )),
            ))))),
            ("java.text.SimpleDateFormat", [Some(pattern)]) => {
                let pattern = pattern
                    .to_utf16_string()
                    .unwrap_or_else(|| Utf16String::from_rust_str(""));
                Some(Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
                    CorpusSimpleDateFormat::new(pattern),
                ))))))
            }
            ("org.thymeleaf.templateengine.features.lazy.ValueLazyContextVariable", [value]) => {
                Some(Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
                    ValueLazyContextVariable::new(value.clone()),
                ))))))
            }
            ("org.thymeleaf.templateengine.features.lazy.ListLazyContextVariable", []) => {
                Some(Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
                    ListLazyContextVariable::new(),
                ))))))
            }
            ("org.thymeleaf.templateengine.features.lazy.ExceptionLazyContextVariable", []) => {
                Some(Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
                    ExceptionLazyContextVariable,
                ))))))
            }
            ("org.thymeleaf.templateengine.features.lazy.LazyExpressionReturner", []) => {
                Some(Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
                    LazyExpressionReturner,
                ))))))
            }
            (
                "org.thymeleaf.templateengine.attrprocessors.model.Bean01"
                | "org.thymeleaf.templateengine.attrprocessors.model.Bean02",
                [],
            ) => Some(Ok(Some(empty_map()))),
            ("org.thymeleaf.templateengine.attrprocessors.model.ExceptionThrowingBean", []) => {
                Some(Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
                    ExceptionThrowingBean,
                ))))))
            }
            ("org.thymeleaf.templateengine.features.expression.ExpContainer1", []) => {
                Some(Ok(Some(map_value(vec![(
                    "value",
                    Arc::new(TemplateValue::string(Utf16String::from_rust_str(
                        "Container1 - value",
                    ))),
                )]))))
            }
            ("org.thymeleaf.templateengine.features.expression.ExpContainer2", []) => {
                Some(Ok(Some(map_value(vec![(
                    "value",
                    Arc::new(TemplateValue::Number(NumberValue::Integer(1979))),
                )]))))
            }
            ("org.thymeleaf.templateengine.features.expression.ExpBean1", []) => {
                Some(Ok(Some(map_value(vec![
                    (
                        "value",
                        Arc::new(TemplateValue::string(Utf16String::from_rust_str("a value"))),
                    ),
                    (
                        "code",
                        Arc::new(TemplateValue::string(Utf16String::from_rust_str("a code"))),
                    ),
                ]))))
            }
            ("org.thymeleaf.templateengine.features.User", [age, first, last, nationality]) => {
                let first_name = first.clone().unwrap_or_else(null_value);
                let last_name = last.clone().unwrap_or_else(null_value);
                let name = format!(
                    "{} {}",
                    first_name
                        .to_utf16_string()
                        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy()),
                    last_name
                        .to_utf16_string()
                        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy())
                );
                Some(Ok(Some(map_value(vec![
                    ("age", age.clone().unwrap_or_else(null_value)),
                    ("firstName", first_name),
                    ("lastName", last_name),
                    (
                        "nationality",
                        nationality.clone().unwrap_or_else(null_value),
                    ),
                    (
                        "name",
                        Arc::new(TemplateValue::string(Utf16String::from_rust_str(&name))),
                    ),
                ]))))
            }
            ("java.util.EnumMap", [_]) => Some(Ok(Some(empty_map()))),
            ("java.io.ByteArrayInputStream", [Some(value)]) => {
                Some(Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
                    CorpusByteArrayInputStream::new(value.as_ref().clone()),
                ))))))
            }
            ("java.sql.Date" | "java.sql.Time" | "java.sql.Timestamp", [Some(value)]) => {
                Some(Ok(Some(map_value(vec![("time", Arc::clone(value))]))))
            }
            _ => None,
        }
    }
}

fn optional(
    class_name: &'static str,
    value: &Arc<TemplateValue>,
) -> Result<Option<Arc<TemplateValue>>, OgnlRuntimeError> {
    Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
        CorpusOptional::new(class_name, Arc::clone(value)),
    )))))
}

fn null_value() -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Null)
}

fn empty_map() -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Map(Arc::new(Vec::new())))
}

fn map_value(entries: Vec<(&str, Arc<TemplateValue>)>) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Map(Arc::new(
        entries
            .into_iter()
            .map(|(key, value)| {
                (
                    Arc::new(TemplateValue::string(Utf16String::from_rust_str(key))),
                    value,
                )
            })
            .collect(),
    )))
}
