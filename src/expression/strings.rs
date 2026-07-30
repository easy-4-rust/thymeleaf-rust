use std::any::Any;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::util::{JavaLocale, JavaNumber, JavaString, StringUtils};

use super::{TemplateObject, TemplateObjectMethodError, TemplateValue};

/// Standard Expression 中的字符串工具对象。
///
/// 对应 Java: `org.thymeleaf.expression.Strings`。
pub struct Strings {
    locale: JavaLocale,
}

impl Strings {
    /// 使用当前表达式上下文 Locale 创建 `#strings`。
    #[must_use]
    pub const fn new(locale: JavaLocale) -> Self {
        Self { locale }
    }

    /// 执行 null-safe 文本转换。
    #[must_use]
    pub fn to_string(&self, target: Option<&TemplateValue>) -> Option<JavaString> {
        value_as_string(target)
    }

    /// 将文本缩略到最大 UTF-16 长度。
    pub fn abbreviate(
        &self,
        target: Option<&TemplateValue>,
        max_size: i32,
    ) -> Result<Option<JavaString>, StringsError> {
        Ok(StringUtils::abbreviate(
            value_as_string(target).as_ref(),
            max_size,
        )?)
    }

    /// 比较两个对象的 `toString()` 文本。
    #[must_use]
    pub fn equals(&self, first: Option<&TemplateValue>, second: Option<&TemplateValue>) -> bool {
        StringUtils::equals(
            value_as_string(first).as_ref(),
            value_as_string(second).as_ref(),
        )
    }

    /// 忽略大小写比较两个对象文本。
    #[must_use]
    pub fn equals_ignore_case(
        &self,
        first: Option<&TemplateValue>,
        second: Option<&TemplateValue>,
    ) -> bool {
        StringUtils::equals_ignore_case(
            value_as_string(first).as_ref(),
            value_as_string(second).as_ref(),
        )
    }

    /// 判断目标是否包含片段。
    pub fn contains(
        &self,
        target: Option<&TemplateValue>,
        fragment: Option<&JavaString>,
    ) -> Result<bool, StringsError> {
        Ok(StringUtils::contains(
            value_as_string(target).as_ref(),
            fragment,
        )?)
    }

    /// 按当前 Locale 忽略大小写判断包含关系。
    pub fn contains_ignore_case(
        &self,
        target: Option<&TemplateValue>,
        fragment: Option<&JavaString>,
    ) -> Result<bool, StringsError> {
        Ok(StringUtils::contains_ignore_case(
            value_as_string(target).as_ref(),
            fragment,
            Some(&self.locale),
        )?)
    }

    /// 判断目标文本是否以前缀开始。
    pub fn starts_with(
        &self,
        target: Option<&TemplateValue>,
        prefix: Option<&JavaString>,
    ) -> Result<bool, StringsError> {
        Ok(StringUtils::starts_with(
            value_as_string(target).as_ref(),
            prefix,
        )?)
    }

    /// 判断目标文本是否以后缀结束。
    pub fn ends_with(
        &self,
        target: Option<&TemplateValue>,
        suffix: Option<&JavaString>,
    ) -> Result<bool, StringsError> {
        Ok(StringUtils::ends_with(
            value_as_string(target).as_ref(),
            suffix,
        )?)
    }

    /// 返回 `[start,end)` 子串。
    pub fn substring(
        &self,
        target: Option<&TemplateValue>,
        start: i32,
        end: i32,
    ) -> Result<Option<JavaString>, StringsError> {
        Ok(StringUtils::substring(
            value_as_string(target).as_ref(),
            start,
            end,
        )?)
    }

    /// 返回从 `start` 到结尾的子串。
    pub fn substring_from(
        &self,
        target: Option<&TemplateValue>,
        start: i32,
    ) -> Result<Option<JavaString>, StringsError> {
        Ok(StringUtils::substring_from(
            value_as_string(target).as_ref(),
            start,
        )?)
    }

    /// 判断目标为 null、空或全 whitespace。
    #[must_use]
    pub fn is_empty(&self, target: Option<&TemplateValue>) -> bool {
        StringUtils::is_empty_or_whitespace(value_as_string(target).as_ref())
    }

    /// 使用当前 Locale 转为大写。
    pub fn to_upper_case(
        &self,
        target: Option<&TemplateValue>,
    ) -> Result<Option<JavaString>, StringsError> {
        Ok(StringUtils::to_upper_case(
            value_as_string(target).as_ref(),
            Some(&self.locale),
        )?)
    }

    /// 使用当前 Locale 转为小写。
    pub fn to_lower_case(
        &self,
        target: Option<&TemplateValue>,
    ) -> Result<Option<JavaString>, StringsError> {
        Ok(StringUtils::to_lower_case(
            value_as_string(target).as_ref(),
            Some(&self.locale),
        )?)
    }

    fn invoke(
        &self,
        method_name: &str,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Result<Option<Arc<TemplateValue>>, StringsError> {
        if let Some((collection_kind, scalar_method)) = collection_method(method_name)
            && !matches!(
                method_name,
                "arrayJoin" | "listJoin" | "setJoin" | "arraySplit" | "listSplit" | "setSplit"
            )
        {
            return self.invoke_collection(collection_kind, &scalar_method, arguments);
        }

        match (method_name, arguments) {
            ("toString", [target]) => Ok(string_value(value_as_string(value_ref(target)))),
            ("abbreviate", [target, max_size]) => Ok(string_value(
                self.abbreviate(value_ref(target), integer(max_size)?)?,
            )),
            ("equals", [first, second]) => Ok(boolean_value(
                self.equals(value_ref(first), value_ref(second)),
            )),
            ("equalsIgnoreCase", [first, second]) => Ok(boolean_value(
                self.equals_ignore_case(value_ref(first), value_ref(second)),
            )),
            ("contains", [target, fragment]) => Ok(boolean_value(
                self.contains(value_ref(target), string_argument(fragment).as_ref())?,
            )),
            ("containsIgnoreCase", [target, fragment]) => Ok(boolean_value(
                self.contains_ignore_case(value_ref(target), string_argument(fragment).as_ref())?,
            )),
            ("startsWith", [target, prefix]) => Ok(boolean_value(
                self.starts_with(value_ref(target), string_argument(prefix).as_ref())?,
            )),
            ("endsWith", [target, suffix]) => Ok(boolean_value(
                self.ends_with(value_ref(target), string_argument(suffix).as_ref())?,
            )),
            ("substring", [target, start, end]) => Ok(string_value(self.substring(
                value_ref(target),
                integer(start)?,
                integer(end)?,
            )?)),
            ("substring", [target, start]) => Ok(string_value(
                self.substring_from(value_ref(target), integer(start)?)?,
            )),
            ("substringAfter", [target, substring]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::substring_after(
                    target.as_ref(),
                    string_argument(substring).as_ref(),
                )?))
            }
            ("substringBefore", [target, substring]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::substring_before(
                    target.as_ref(),
                    string_argument(substring).as_ref(),
                )?))
            }
            ("prepend", [target, prefix]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::prepend(
                    target.as_ref(),
                    string_argument(prefix).as_ref(),
                )?))
            }
            ("repeat", [target, times]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::repeat(
                    target.as_ref(),
                    integer(times)?,
                )))
            }
            ("append", [target, suffix]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::append(
                    target.as_ref(),
                    string_argument(suffix).as_ref(),
                )?))
            }
            ("concat", values) => {
                let values = values.iter().map(string_argument).collect::<Vec<_>>();
                Ok(Some(Arc::new(TemplateValue::string(StringUtils::concat(
                    Some(&values),
                )))))
            }
            ("concatReplaceNulls", [null_value, values @ ..]) => {
                let null_value = string_argument(null_value);
                let values = values.iter().map(string_argument).collect::<Vec<_>>();
                Ok(Some(Arc::new(TemplateValue::string(
                    StringUtils::concat_replace_nulls(null_value.as_ref(), Some(&values)),
                ))))
            }
            ("indexOf", [target, fragment]) => {
                let target = value_as_string(value_ref(target));
                Ok(integer_value(StringUtils::index_of(
                    target.as_ref(),
                    string_argument(fragment).as_ref(),
                )?))
            }
            ("isEmpty", [target]) => Ok(boolean_value(self.is_empty(value_ref(target)))),
            ("arrayJoin" | "listJoin" | "setJoin", [target, separator]) => {
                let values = list_argument(target)?
                    .iter()
                    .map(|value| value_as_string(Some(value.as_ref())))
                    .collect::<Vec<_>>();
                Ok(string_value(StringUtils::join(
                    Some(&values),
                    string_argument(separator).as_ref(),
                )?))
            }
            ("arraySplit" | "listSplit" | "setSplit", [target, separator]) => {
                let target = value_as_string(value_ref(target));
                let values =
                    StringUtils::split(target.as_ref(), string_argument(separator).as_ref())?;
                Ok(values.map(|values| {
                    Arc::new(TemplateValue::List(Arc::new(
                        values
                            .into_iter()
                            .map(|value| Arc::new(TemplateValue::string(value)))
                            .collect(),
                    )))
                }))
            }
            ("length", [target]) => {
                let target = value_as_string(value_ref(target));
                Ok(integer_value(StringUtils::length(target.as_ref())?))
            }
            ("replace", [target, before, after]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::replace(
                    target.as_ref(),
                    string_argument(before).as_ref(),
                    string_argument(after).as_ref(),
                )?))
            }
            ("multipleReplace", [target, before, after]) => {
                let mut result = value_as_string(value_ref(target));
                let before = string_list(before)?;
                let after = string_list(after)?;
                if before.len() != after.len() {
                    return Err(StringsError::new(
                        "Arrays of 'before' and 'after' values must have the same length",
                    ));
                }
                for (before, after) in before.iter().zip(&after) {
                    result =
                        StringUtils::replace(result.as_ref(), before.as_ref(), after.as_ref())?;
                }
                Ok(string_value(result))
            }
            ("toUpperCase", [target]) => Ok(string_value(self.to_upper_case(value_ref(target))?)),
            ("toLowerCase", [target]) => Ok(string_value(self.to_lower_case(value_ref(target))?)),
            ("trim", [target]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::trim(target.as_ref())))
            }
            ("capitalize", [target]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::capitalize(target.as_ref())))
            }
            ("unCapitalize", [target]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::un_capitalize(target.as_ref())))
            }
            ("capitalizeWords", [target]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::capitalize_words(
                    target.as_ref(),
                    None,
                )))
            }
            ("capitalizeWords", [target, delimiters]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::capitalize_words(
                    target.as_ref(),
                    string_argument(delimiters).as_ref(),
                )))
            }
            ("escapeXml", [target]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::escape_xml(target.as_ref())))
            }
            ("escapeJavaScript", [target]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::escape_java_script(
                    target.as_ref(),
                )))
            }
            ("unescapeJavaScript", [target]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::unescape_java_script(
                    target.as_ref(),
                )))
            }
            ("escapeJava", [target]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::escape_java(target.as_ref())))
            }
            ("unescapeJava", [target]) => {
                let target = value_as_string(value_ref(target));
                Ok(string_value(StringUtils::unescape_java(target.as_ref())))
            }
            ("randomAlphanumeric", [count]) => Ok(Some(Arc::new(TemplateValue::string(
                StringUtils::random_alphanumeric(integer(count)?),
            )))),
            ("defaultString", [target, default]) => {
                let target = value_as_string(value_ref(target));
                let value = if StringUtils::is_empty_or_whitespace(target.as_ref()) {
                    string_argument(default).or_else(|| Some(JavaString::from_rust_str("null")))
                } else {
                    target
                };
                Ok(string_value(value))
            }
            _ => Err(StringsError::new(format!(
                "Method {method_name} with {} arguments is not available on #strings",
                arguments.len()
            ))),
        }
    }

    fn invoke_collection(
        &self,
        kind: CollectionKind,
        scalar_method: &str,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Result<Option<Arc<TemplateValue>>, StringsError> {
        let Some((target, remaining)) = arguments.split_first() else {
            return Err(StringsError::new("Collection method requires a target"));
        };
        if target.is_none() || matches!(value_ref(target), Some(TemplateValue::Null)) {
            return Ok(None);
        }
        let values = list_argument(target)?;
        let mut results = Vec::with_capacity(values.len());
        for value in values.iter() {
            let mut item_arguments = Vec::with_capacity(arguments.len());
            item_arguments.push(Some(Arc::clone(value)));
            item_arguments.extend(remaining.iter().cloned());
            let result = self
                .invoke(scalar_method, &item_arguments)?
                .unwrap_or_else(|| Arc::new(TemplateValue::Null));
            if kind != CollectionKind::Set || !contains_java_value(&results, &result) {
                results.push(result);
            }
        }
        Ok(Some(Arc::new(TemplateValue::List(Arc::new(results)))))
    }
}

impl TemplateObject for Strings {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.expression.Strings"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str("org.thymeleaf.expression.Strings")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_invoke_method(
        &self,
        method_name: &JavaString,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        Some(
            self.invoke(&method_name.to_string_lossy(), arguments)
                .map_err(|error| Box::new(error) as TemplateObjectMethodError),
        )
    }
}

/// `#strings` 动态方法调用错误。
#[derive(Debug)]
pub struct StringsError {
    message: String,
}

impl StringsError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for StringsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for StringsError {}

impl From<crate::util::StringUtilsError> for StringsError {
    fn from(error: crate::util::StringUtilsError) -> Self {
        Self::new(error.to_string())
    }
}

fn value_ref(value: &Option<Arc<TemplateValue>>) -> Option<&TemplateValue> {
    value
        .as_deref()
        .filter(|value| !matches!(value, TemplateValue::Null))
}

fn value_as_string(value: Option<&TemplateValue>) -> Option<JavaString> {
    value
        .filter(|value| !matches!(value, TemplateValue::Null))
        .and_then(TemplateValue::to_java_string)
}

fn string_argument(value: &Option<Arc<TemplateValue>>) -> Option<JavaString> {
    value_as_string(value_ref(value))
}

fn integer(value: &Option<Arc<TemplateValue>>) -> Result<i32, StringsError> {
    match value_ref(value) {
        Some(TemplateValue::Number(JavaNumber::Byte(value))) => Ok(i32::from(*value)),
        Some(TemplateValue::Number(JavaNumber::Short(value))) => Ok(i32::from(*value)),
        Some(TemplateValue::Number(JavaNumber::Integer(value))) => Ok(*value),
        Some(TemplateValue::Number(JavaNumber::Long(value))) => i32::try_from(*value)
            .map_err(|_| StringsError::new("Numeric argument is outside Java int range")),
        _ => Err(StringsError::new("Argument is not a Java integer")),
    }
}

fn list_argument(
    value: &Option<Arc<TemplateValue>>,
) -> Result<&[Arc<TemplateValue>], StringsError> {
    match value_ref(value) {
        Some(TemplateValue::List(values)) => Ok(values),
        _ => Err(StringsError::new("Argument is not an array, List or Set")),
    }
}

fn string_list(
    value: &Option<Arc<TemplateValue>>,
) -> Result<Vec<Option<JavaString>>, StringsError> {
    Ok(list_argument(value)?
        .iter()
        .map(|value| value_as_string(Some(value.as_ref())))
        .collect())
}

fn string_value(value: Option<JavaString>) -> Option<Arc<TemplateValue>> {
    value.map(|value| Arc::new(TemplateValue::string(value)))
}

fn boolean_value(value: bool) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::Boolean(value)))
}

fn integer_value(value: i32) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(value))))
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CollectionKind {
    Array,
    List,
    Set,
}

fn collection_method(method_name: &str) -> Option<(CollectionKind, String)> {
    for (prefix, kind) in [
        ("array", CollectionKind::Array),
        ("list", CollectionKind::List),
        ("set", CollectionKind::Set),
    ] {
        if let Some(suffix) = method_name.strip_prefix(prefix)
            && let Some(first) = suffix.chars().next()
        {
            let mut scalar = first.to_lowercase().collect::<String>();
            scalar.push_str(&suffix[first.len_utf8()..]);
            return Some((kind, scalar));
        }
    }
    None
}

fn contains_java_value(values: &[Arc<TemplateValue>], candidate: &Arc<TemplateValue>) -> bool {
    let candidate = candidate.to_java_string();
    values
        .iter()
        .any(|value| value.to_java_string() == candidate)
}
