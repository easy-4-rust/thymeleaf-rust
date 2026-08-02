use std::any::Any;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::util::{
    DateUtils, DateUtilsError, JavaDate, JavaLocale, JavaNumber, JavaString, template_integer,
};

use super::{TemplateObject, TemplateObjectMethodError, TemplateValue};

/// Standard Expression 中的 `java.util.Date` 工具。
///
/// 对应 Java: `org.thymeleaf.expression.Dates`。
pub struct Dates {
    locale: JavaLocale,
}

impl Dates {
    /// 使用表达式上下文 Locale 创建 `#dates`。
    #[must_use]
    pub const fn new(locale: JavaLocale) -> Self {
        Self { locale }
    }

    /// 创建 Date；可选时间字段必须遵守 Calendar 成组规则。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java: `Dates#create()`。
    pub fn create(
        &self,
        year: Option<i32>,
        month: Option<i32>,
        day: Option<i32>,
        hour: Option<i32>,
        minute: Option<i32>,
        second: Option<i32>,
        millisecond: Option<i32>,
    ) -> Result<JavaDate, DatesError> {
        Ok(DateUtils::create(
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            None,
            Some(&self.locale),
        )?
        .to_date())
    }

    /// 返回当前瞬时 Date。
    #[must_use]
    /// 对应 Java: `Dates#createNow()`。
    pub fn create_now(&self, time_zone: Option<&str>) -> JavaDate {
        DateUtils::create_now(time_zone, Some(&self.locale)).to_date()
    }

    /// 返回指定时区当天零点对应的 Date 瞬时。
    #[must_use]
    /// 对应 Java: `Dates#createToday()`。
    pub fn create_today(&self, time_zone: Option<&str>) -> JavaDate {
        DateUtils::create_today(time_zone, Some(&self.locale)).to_date()
    }

    /// 使用默认长格式或指定 pattern 格式化 Date。
    /// 对应 Java: `Dates#format()`。
    pub fn format(
        &self,
        target: Option<&JavaDate>,
        pattern: Option<&JavaString>,
    ) -> Result<Option<JavaString>, DatesError> {
        Ok(DateUtils::format(target, pattern, Some(&self.locale))?)
    }

    fn invoke(
        &self,
        method_name: &str,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Result<Option<Arc<TemplateValue>>, DatesError> {
        if let Some((set_semantics, scalar_method)) = collection_method(method_name) {
            let Some((target, remaining)) = arguments.split_first() else {
                return Err(DatesError::new("Collection method requires a target"));
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

        match method_name {
            "create" if matches!(arguments.len(), 3 | 5 | 6 | 7) => {
                let fields = date_fields(arguments)?;
                Ok(Some(DateUtils::into_template_value(self.create(
                    fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], fields[6],
                )?)))
            }
            "createNow" if arguments.is_empty() => {
                Ok(Some(DateUtils::into_template_value(self.create_now(None))))
            }
            "createNowForTimeZone" if arguments.len() == 1 => {
                Ok(Some(DateUtils::into_template_value(
                    self.create_now(time_zone(&arguments[0]).as_deref()),
                )))
            }
            "createToday" if arguments.is_empty() => Ok(Some(DateUtils::into_template_value(
                self.create_today(None),
            ))),
            "createTodayForTimeZone" if arguments.len() == 1 => {
                Ok(Some(DateUtils::into_template_value(
                    self.create_today(time_zone(&arguments[0]).as_deref()),
                )))
            }
            "format" if matches!(arguments.len(), 1 | 2) => Ok(string_value(self.format(
                date(arguments.first().expect("length"))?,
                arguments.get(1).and_then(string_argument).as_ref(),
            )?)),
            "day" => Ok(integer_option(DateUtils::day(date_argument(arguments)?))),
            "month" => Ok(integer_option(DateUtils::month(date_argument(arguments)?))),
            "monthName" => self.named_date(arguments, "MMMM"),
            "monthNameShort" => self.named_date(arguments, "MMM"),
            "year" => Ok(integer_option(DateUtils::year(date_argument(arguments)?))),
            "dayOfWeek" => Ok(integer_option(DateUtils::day_of_week(date_argument(
                arguments,
            )?))),
            "dayOfWeekName" => self.named_date(arguments, "EEEE"),
            "dayOfWeekNameShort" => self.named_date(arguments, "EEE"),
            "hour" => Ok(integer_option(DateUtils::hour(date_argument(arguments)?))),
            "minute" => Ok(integer_option(DateUtils::minute(date_argument(arguments)?))),
            "second" => Ok(integer_option(DateUtils::second(date_argument(arguments)?))),
            "millisecond" => Ok(integer_option(DateUtils::millisecond(date_argument(
                arguments,
            )?))),
            "formatISO" => Ok(string_value(DateUtils::format_iso(date_argument(
                arguments,
            )?))),
            _ => Err(DatesError::new(format!(
                "Method {method_name} with {} arguments is not available on #dates",
                arguments.len()
            ))),
        }
    }

    fn named_date(
        &self,
        arguments: &[Option<Arc<TemplateValue>>],
        pattern: &str,
    ) -> Result<Option<Arc<TemplateValue>>, DatesError> {
        let pattern = JavaString::from_rust_str(pattern);
        Ok(string_value(
            self.format(date_argument(arguments)?, Some(&pattern))?,
        ))
    }
}

impl TemplateObject for Dates {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.expression.Dates"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str("org.thymeleaf.expression.Dates")
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

/// `#dates` 创建、类型转换和格式化错误。
#[derive(Debug)]
/// 对应 Java 语义：`Dates` 的 Rust 侧类型 `DatesError`。
pub struct DatesError {
    message: String,
}

impl DatesError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for DatesError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for DatesError {}

impl From<DateUtilsError> for DatesError {
    fn from(error: DateUtilsError) -> Self {
        Self::new(error.to_string())
    }
}

fn date(value: &Option<Arc<TemplateValue>>) -> Result<Option<&JavaDate>, DatesError> {
    let date = DateUtils::from_template_value(value.as_deref())?;
    if date.is_some_and(JavaDate::is_calendar) {
        return Err(DatesError::new(
            "java.util.GregorianCalendar cannot be cast to java.util.Date",
        ));
    }
    Ok(date)
}

fn date_argument(
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<&JavaDate>, DatesError> {
    match arguments {
        [target] => date(target),
        _ => Err(DatesError::new("Date field method requires one argument")),
    }
}

fn date_fields(arguments: &[Option<Arc<TemplateValue>>]) -> Result<[Option<i32>; 7], DatesError> {
    let mut fields = [None; 7];
    for (index, argument) in arguments.iter().enumerate() {
        fields[index] = template_integer(argument)?;
    }
    Ok(fields)
}

fn string_argument(value: &Option<Arc<TemplateValue>>) -> Option<JavaString> {
    value.as_deref().and_then(TemplateValue::to_java_string)
}

fn time_zone(value: &Option<Arc<TemplateValue>>) -> Option<String> {
    string_argument(value).map(|value| value.to_string_lossy())
}

fn list(value: &Option<Arc<TemplateValue>>) -> Result<&[Arc<TemplateValue>], DatesError> {
    match value.as_deref() {
        Some(TemplateValue::List(values)) => Ok(values),
        _ => Err(DatesError::new("Target is not an array, List or Set")),
    }
}

fn string_value(value: Option<JavaString>) -> Option<Arc<TemplateValue>> {
    value.map(|value| Arc::new(TemplateValue::string(value)))
}

fn integer_option(value: Option<i32>) -> Option<Arc<TemplateValue>> {
    value.map(|value| Arc::new(TemplateValue::Number(JavaNumber::Integer(value))))
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
