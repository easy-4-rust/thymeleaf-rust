use std::any::Any;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::util::{
    JavaLocale, JavaNumber, JavaString, NumberPointType, NumberUtils, NumberUtilsError,
};

use super::{TemplateObject, TemplateObjectMethodError, TemplateValue};

/// Standard Expression 中的数字格式化与序列工具。
///
/// 对应 Java: `org.thymeleaf.expression.Numbers`。
pub struct Numbers {
    locale: JavaLocale,
}

impl Numbers {
    /// 使用表达式上下文 Locale 创建 `#numbers`。
    #[must_use]
    pub const fn new(locale: JavaLocale) -> Self {
        Self { locale }
    }

    /// 格式化整数，可选择分组点类型。
    pub fn format_integer(
        &self,
        target: Option<&JavaNumber>,
        min_integer_digits: i32,
        thousands_point_type: Option<NumberPointType>,
    ) -> Result<Option<JavaString>, NumbersError> {
        Ok(NumberUtils::format(
            target,
            Some(min_integer_digits),
            Some(thousands_point_type.unwrap_or(NumberPointType::None)),
            Some(0),
            Some(NumberPointType::None),
            Some(&self.locale),
        )?)
    }

    /// 格式化定点小数。
    pub fn format_decimal(
        &self,
        target: Option<&JavaNumber>,
        min_integer_digits: i32,
        thousands_point_type: NumberPointType,
        decimal_digits: i32,
        decimal_point_type: NumberPointType,
    ) -> Result<Option<JavaString>, NumbersError> {
        Ok(NumberUtils::format(
            target,
            Some(min_integer_digits),
            Some(thousands_point_type),
            Some(decimal_digits),
            Some(decimal_point_type),
            Some(&self.locale),
        )?)
    }

    /// 按当前 Locale 格式化货币。
    pub fn format_currency(
        &self,
        target: Option<&JavaNumber>,
    ) -> Result<Option<JavaString>, NumbersError> {
        Ok(NumberUtils::format_currency(target, Some(&self.locale))?)
    }

    /// 按当前 Locale 格式化百分比。
    pub fn format_percent(
        &self,
        target: Option<&JavaNumber>,
        min_integer_digits: i32,
        decimal_digits: i32,
    ) -> Result<Option<JavaString>, NumbersError> {
        Ok(NumberUtils::format_percent(
            target,
            Some(min_integer_digits),
            Some(decimal_digits),
            Some(&self.locale),
        )?)
    }

    /// 创建包含终点的整数序列。
    pub fn sequence(
        &self,
        from: i32,
        to: i32,
        step: Option<i32>,
    ) -> Result<Vec<i32>, NumbersError> {
        Ok(match step {
            Some(step) => NumberUtils::sequence_with_step(Some(from), Some(to), Some(step))?,
            None => NumberUtils::sequence(Some(from), Some(to))?,
        })
    }

    fn invoke(
        &self,
        method_name: &str,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Result<Option<Arc<TemplateValue>>, NumbersError> {
        if let Some((set_semantics, scalar_method)) = collection_method(method_name) {
            let Some((target, remaining)) = arguments.split_first() else {
                return Err(NumbersError::new("Collection method requires a target"));
            };
            if target.is_none() {
                return Ok(None);
            }
            let values = list(target)?;
            let mut output = Vec::with_capacity(values.len());
            for value in values {
                let mut item_arguments = vec![Some(Arc::clone(value))];
                item_arguments.extend(remaining.iter().cloned());
                let item = self
                    .invoke(&scalar_method, &item_arguments)?
                    .unwrap_or_else(|| Arc::new(TemplateValue::Null));
                if !set_semantics || !contains_text(&output, &item) {
                    output.push(item);
                }
            }
            return Ok(Some(Arc::new(TemplateValue::List(Arc::new(output)))));
        }

        match (method_name, arguments) {
            ("formatInteger", [target, min]) => Ok(string_value(self.format_integer(
                number(target)?,
                integer(min)?,
                None,
            )?)),
            ("formatInteger", [target, min, thousands]) => Ok(string_value(self.format_integer(
                number(target)?,
                integer(min)?,
                Some(point_type(thousands)?),
            )?)),
            ("formatDecimal", [target, min, decimals]) => Ok(string_value(self.format_decimal(
                number(target)?,
                integer(min)?,
                NumberPointType::None,
                integer(decimals)?,
                NumberPointType::Default,
            )?)),
            ("formatDecimal", [target, min, decimals, decimal_point]) => {
                Ok(string_value(self.format_decimal(
                    number(target)?,
                    integer(min)?,
                    NumberPointType::None,
                    integer(decimals)?,
                    point_type(decimal_point)?,
                )?))
            }
            ("formatDecimal", [target, min, thousands, decimals, decimal_point]) => {
                Ok(string_value(self.format_decimal(
                    number(target)?,
                    integer(min)?,
                    point_type(thousands)?,
                    integer(decimals)?,
                    point_type(decimal_point)?,
                )?))
            }
            ("formatCurrency", [target]) => {
                Ok(string_value(self.format_currency(number(target)?)?))
            }
            ("formatPercent", [target, min, decimals]) => Ok(string_value(self.format_percent(
                number(target)?,
                integer(min)?,
                integer(decimals)?,
            )?)),
            ("sequence", [from, to]) => Ok(Some(sequence_value(self.sequence(
                integer(from)?,
                integer(to)?,
                None,
            )?))),
            ("sequence", [from, to, step]) => Ok(Some(sequence_value(self.sequence(
                integer(from)?,
                integer(to)?,
                Some(integer(step)?),
            )?))),
            _ => Err(NumbersError::new(format!(
                "Method {method_name} with {} arguments is not available on #numbers",
                arguments.len()
            ))),
        }
    }
}

impl TemplateObject for Numbers {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.expression.Numbers"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str("org.thymeleaf.expression.Numbers")
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

/// `#numbers` 动态调用和格式化错误。
#[derive(Debug)]
pub struct NumbersError {
    message: String,
}

impl NumbersError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for NumbersError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for NumbersError {}

impl From<NumberUtilsError> for NumbersError {
    fn from(error: NumberUtilsError) -> Self {
        Self::new(error.to_string())
    }
}

fn number(value: &Option<Arc<TemplateValue>>) -> Result<Option<&JavaNumber>, NumbersError> {
    match value.as_deref() {
        None | Some(TemplateValue::Null) => Ok(None),
        Some(TemplateValue::Number(number)) => Ok(Some(number)),
        Some(value) => Err(NumbersError::new(format!(
            "{} cannot be cast to java.lang.Number",
            value.java_class_name()
        ))),
    }
}

fn integer(value: &Option<Arc<TemplateValue>>) -> Result<i32, NumbersError> {
    match number(value)? {
        Some(JavaNumber::Byte(value)) => Ok(i32::from(*value)),
        Some(JavaNumber::Short(value)) => Ok(i32::from(*value)),
        Some(JavaNumber::Integer(value)) => Ok(*value),
        Some(JavaNumber::Long(value)) => i32::try_from(*value)
            .map_err(|_| NumbersError::new("Number is outside Java Integer range")),
        Some(_) => Err(NumbersError::new("Number is not an integer")),
        None => Err(NumbersError::new("Integer argument cannot be null")),
    }
}

fn point_type(value: &Option<Arc<TemplateValue>>) -> Result<NumberPointType, NumbersError> {
    let text = value
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    NumberPointType::match_name(text.as_deref()).ok_or_else(|| {
        NumbersError::new(format!(
            "Unrecognized point format \"{}\"",
            text.unwrap_or_else(|| "null".to_owned())
        ))
    })
}

fn list(value: &Option<Arc<TemplateValue>>) -> Result<&[Arc<TemplateValue>], NumbersError> {
    match value.as_deref() {
        Some(TemplateValue::List(values)) => Ok(values),
        _ => Err(NumbersError::new("Target is not an array, List or Set")),
    }
}

fn string_value(value: Option<JavaString>) -> Option<Arc<TemplateValue>> {
    value.map(|value| Arc::new(TemplateValue::string(value)))
}

fn sequence_value(values: Vec<i32>) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::List(Arc::new(
        values
            .into_iter()
            .map(|value| Arc::new(TemplateValue::Number(JavaNumber::Integer(value))))
            .collect(),
    )))
}

fn collection_method(method_name: &str) -> Option<(bool, String)> {
    for (prefix, set_semantics) in [("array", false), ("list", false), ("set", true)] {
        if let Some(suffix) = method_name.strip_prefix(prefix)
            && let Some(first) = suffix.chars().next()
        {
            let mut scalar = first.to_lowercase().collect::<String>();
            scalar.push_str(&suffix[first.len_utf8()..]);
            return Some((set_semantics, scalar));
        }
    }
    None
}

fn contains_text(values: &[Arc<TemplateValue>], candidate: &Arc<TemplateValue>) -> bool {
    let candidate = candidate.to_java_string();
    values
        .iter()
        .any(|value| value.to_java_string() == candidate)
}
