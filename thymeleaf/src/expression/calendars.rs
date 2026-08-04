use std::any::Any;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::util::{
    DateUtils, DateUtilsError, DateValue, Locale, NumberValue, Utf16String, template_integer,
};

use super::{TemplateObject, TemplateObjectMethodError, TemplateValue};

/// Standard Expression 中的 `java.util.Calendar` 工具。
///
/// 对应 Java: `org.thymeleaf.expression.Calendars`。
pub struct Calendars {
    locale: Locale,
}

impl Calendars {
    /// 使用表达式上下文 Locale 创建 `#calendars`。
    #[must_use]
    pub const fn new(locale: Locale) -> Self {
        Self { locale }
    }

    /// 创建 Calendar；时间字段遵守 Java `DateUtils#create` 的成组规则。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java: `Calendars#create()`。
    pub fn create(
        &self,
        year: Option<i32>,
        month: Option<i32>,
        day: Option<i32>,
        hour: Option<i32>,
        minute: Option<i32>,
        second: Option<i32>,
        millisecond: Option<i32>,
        time_zone: Option<&str>,
    ) -> Result<DateValue, CalendarsError> {
        Ok(DateUtils::create(
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            time_zone,
            Some(&self.locale),
        )?)
    }

    /// 返回指定时区的当前 Calendar。
    #[must_use]
    /// 对应 Java: `Calendars#createNow()`。
    pub fn create_now(&self, time_zone: Option<&str>) -> DateValue {
        DateUtils::create_now(time_zone, Some(&self.locale))
    }

    /// 返回指定时区当天零点 Calendar。
    #[must_use]
    /// 对应 Java: `Calendars#createToday()`。
    pub fn create_today(&self, time_zone: Option<&str>) -> DateValue {
        DateUtils::create_today(time_zone, Some(&self.locale))
    }

    /// 使用默认长格式或指定 pattern 格式化 Calendar。
    /// 对应 Java: `Calendars#format()`。
    pub fn format(
        &self,
        target: Option<&DateValue>,
        pattern: Option<&Utf16String>,
    ) -> Result<Option<Utf16String>, CalendarsError> {
        Ok(DateUtils::format(target, pattern, Some(&self.locale))?)
    }

    fn invoke(
        &self,
        method_name: &str,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Result<Option<Arc<TemplateValue>>, CalendarsError> {
        if let Some((set_semantics, scalar_method)) = collection_method(method_name) {
            let Some((target, remaining)) = arguments.split_first() else {
                return Err(CalendarsError::new("Collection method requires a target"));
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
                self.create_value(arguments, false)
            }
            "createForTimeZone" if matches!(arguments.len(), 4 | 6 | 7 | 8) => {
                self.create_value(arguments, true)
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
                calendar(arguments.first().expect("length"))?,
                arguments.get(1).and_then(string_argument).as_ref(),
            )?)),
            "day" => Ok(integer_option(DateUtils::day(calendar_argument(
                arguments,
            )?))),
            "month" => Ok(integer_option(DateUtils::month(calendar_argument(
                arguments,
            )?))),
            "monthName" => self.named_calendar(arguments, "MMMM"),
            "monthNameShort" => self.named_calendar(arguments, "MMM"),
            "year" => Ok(integer_option(DateUtils::year(calendar_argument(
                arguments,
            )?))),
            "dayOfWeek" => Ok(integer_option(DateUtils::day_of_week(calendar_argument(
                arguments,
            )?))),
            "dayOfWeekName" => self.named_calendar(arguments, "EEEE"),
            "dayOfWeekNameShort" => self.named_calendar(arguments, "EEE"),
            "hour" => Ok(integer_option(DateUtils::hour(calendar_argument(
                arguments,
            )?))),
            "minute" => Ok(integer_option(DateUtils::minute(calendar_argument(
                arguments,
            )?))),
            "second" => Ok(integer_option(DateUtils::second(calendar_argument(
                arguments,
            )?))),
            "millisecond" => Ok(integer_option(DateUtils::millisecond(calendar_argument(
                arguments,
            )?))),
            "formatISO" => Ok(string_value(DateUtils::format_iso(calendar_argument(
                arguments,
            )?))),
            _ => Err(CalendarsError::new(format!(
                "Method {method_name} with {} arguments is not available on #calendars",
                arguments.len()
            ))),
        }
    }

    fn create_value(
        &self,
        arguments: &[Option<Arc<TemplateValue>>],
        with_time_zone: bool,
    ) -> Result<Option<Arc<TemplateValue>>, CalendarsError> {
        let field_count = arguments.len() - usize::from(with_time_zone);
        let mut fields = [None; 7];
        for (index, argument) in arguments[..field_count].iter().enumerate() {
            fields[index] = template_integer(argument)?;
        }
        let time_zone = with_time_zone
            .then(|| time_zone(arguments.last().expect("time zone")))
            .flatten();
        Ok(Some(DateUtils::into_template_value(self.create(
            fields[0],
            fields[1],
            fields[2],
            fields[3],
            fields[4],
            fields[5],
            fields[6],
            time_zone.as_deref(),
        )?)))
    }

    fn named_calendar(
        &self,
        arguments: &[Option<Arc<TemplateValue>>],
        pattern: &str,
    ) -> Result<Option<Arc<TemplateValue>>, CalendarsError> {
        let pattern = Utf16String::from_rust_str(pattern);
        Ok(string_value(
            self.format(calendar_argument(arguments)?, Some(&pattern))?,
        ))
    }
}

impl TemplateObject for Calendars {
    fn class_name(&self) -> &str {
        "org.thymeleaf.expression.Calendars"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str("org.thymeleaf.expression.Calendars")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn invoke_method(
        &self,
        method_name: &Utf16String,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        Some(
            self.invoke(&method_name.to_string_lossy(), arguments)
                .map_err(|error| Box::new(error) as TemplateObjectMethodError),
        )
    }
}

/// `#calendars` 创建、类型转换和格式化错误。
#[derive(Debug)]
/// 对应 Java 语义：`Calendars` 的 Rust 侧类型 `CalendarsError`。
pub struct CalendarsError {
    message: String,
}

impl CalendarsError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for CalendarsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for CalendarsError {}

impl From<DateUtilsError> for CalendarsError {
    fn from(error: DateUtilsError) -> Self {
        Self::new(error.to_string())
    }
}

fn calendar(value: &Option<Arc<TemplateValue>>) -> Result<Option<&DateValue>, CalendarsError> {
    let calendar = DateUtils::from_template_value(value.as_deref())?;
    if calendar.is_some_and(|value| !value.is_calendar()) {
        return Err(CalendarsError::new(
            "java.util.Date cannot be cast to java.util.Calendar",
        ));
    }
    Ok(calendar)
}

fn calendar_argument(
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<&DateValue>, CalendarsError> {
    match arguments {
        [target] => calendar(target),
        _ => Err(CalendarsError::new(
            "Calendar field method requires one argument",
        )),
    }
}

fn string_argument(value: &Option<Arc<TemplateValue>>) -> Option<Utf16String> {
    value.as_deref().and_then(TemplateValue::to_utf16_string)
}

fn time_zone(value: &Option<Arc<TemplateValue>>) -> Option<String> {
    string_argument(value).map(|value| value.to_string_lossy())
}

fn list(value: &Option<Arc<TemplateValue>>) -> Result<&[Arc<TemplateValue>], CalendarsError> {
    match value.as_deref() {
        Some(TemplateValue::List(values)) => Ok(values),
        _ => Err(CalendarsError::new("Target is not an array, List or Set")),
    }
}

fn string_value(value: Option<Utf16String>) -> Option<Arc<TemplateValue>> {
    value.map(|value| Arc::new(TemplateValue::string(value)))
}

fn integer_option(value: Option<i32>) -> Option<Arc<TemplateValue>> {
    value.map(|value| Arc::new(TemplateValue::Number(NumberValue::Integer(value))))
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
    let candidate = candidate.to_utf16_string();
    values
        .iter()
        .any(|value| value.to_utf16_string() == candidate)
}
