use std::any::Any;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use chrono_tz::Tz;

use crate::temporal::{
    TemporalCreationUtils, TemporalFormattingError, TemporalFormattingUtils, TemporalObjects,
    TemporalValue,
};
use crate::util::{Locale, NumberValue, Utf16String, template_integer};

use super::{TemplateObject, TemplateObjectMethodError, TemplateValue};

/// Standard Expression 中的 Java 8 Time 工具。
///
/// 对应 Java: `org.thymeleaf.expression.Temporals`。
pub struct Temporals {
    creation: TemporalCreationUtils,
    formatting: TemporalFormattingUtils,
}

impl Temporals {
    /// 使用 Locale 与系统默认 ZoneId 创建 `#temporals`。
    /// 对应 Java 语义：`Temporals` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(locale: Locale) -> Result<Self, TemporalsError> {
        Self::with_default_zone_id(locale, default_zone())
    }

    /// 使用 Locale 与显式默认 ZoneId 创建 `#temporals`。
    /// 对应 Java 语义：`Temporals` 的 `with_default_zone_id` 行为（Rust 侧辅助/私有路径）。
    pub fn with_default_zone_id(
        locale: Locale,
        default_zone_id: Tz,
    ) -> Result<Self, TemporalsError> {
        Ok(Self {
            creation: TemporalCreationUtils::new(),
            formatting: TemporalFormattingUtils::new(locale, default_zone_id).map_err(error)?,
        })
    }

    fn invoke(
        &self,
        method_name: &str,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Result<Option<Arc<TemplateValue>>, TemporalsError> {
        if let Some((set_semantics, scalar_method)) = collection_method(method_name) {
            return self.invoke_collection(&scalar_method, set_semantics, arguments);
        }
        match method_name {
            "create" if matches!(arguments.len(), 3 | 5 | 6 | 7) => {
                let fields = arguments
                    .iter()
                    .map(|value| {
                        template_integer(value)
                            .map_err(error)?
                            .ok_or_else(|| TemporalsError::new("Argument cannot be null"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.created(self.creation.create(&fields))
            }
            "createDate" if matches!(arguments.len(), 1 | 2) => {
                let text = required_string(&arguments[0], "ISO date cannot be null")?;
                let pattern = optional_pattern(arguments.get(1))?;
                self.created(self.creation.create_date(&text, pattern.as_deref()))
            }
            "createDateTime" if matches!(arguments.len(), 1 | 2) => {
                let text = required_string(&arguments[0], "ISO date-time cannot be null")?;
                let pattern = optional_pattern(arguments.get(1))?;
                self.created(self.creation.create_date_time(&text, pattern.as_deref()))
            }
            "createNow" if arguments.is_empty() => {
                Ok(Some(temporal_value(self.creation.create_now())))
            }
            "createNowForTimeZone" if arguments.len() == 1 => {
                let zone = required_string(&arguments[0], "ZoneId cannot be null")?;
                self.created(self.creation.create_now_for_time_zone(&zone))
            }
            "createToday" if arguments.is_empty() => {
                Ok(Some(temporal_value(self.creation.create_today())))
            }
            "createTodayForTimeZone" if arguments.len() == 1 => {
                let zone = required_string(&arguments[0], "ZoneId cannot be null")?;
                self.created(self.creation.create_today_for_time_zone(&zone))
            }
            "format" if (1..=3).contains(&arguments.len()) => self.format(arguments),
            "day" => self.integer(arguments, |utils, target| utils.day(target)),
            "month" => self.integer(arguments, |utils, target| utils.month(target)),
            "monthName" => self.named(arguments, "MMMM"),
            "monthNameShort" => self.named(arguments, "MMM"),
            "year" => self.integer(arguments, |utils, target| utils.year(target)),
            "dayOfWeek" => self.integer(arguments, |utils, target| utils.day_of_week(target)),
            "dayOfWeekName" => self.named(arguments, "EEEE"),
            "dayOfWeekNameShort" => self.named(arguments, "EEE"),
            "hour" => self.integer(arguments, |utils, target| utils.hour(target)),
            "minute" => self.integer(arguments, |utils, target| utils.minute(target)),
            "second" => self.integer(arguments, |utils, target| utils.second(target)),
            "nanosecond" => self.integer(arguments, |utils, target| utils.nanosecond(target)),
            "formatISO" if arguments.len() == 1 => Ok(string_value(
                self.formatting
                    .format_iso(temporal_argument(arguments)?)
                    .map_err(error)?,
            )),
            _ => Err(TemporalsError::new(format!(
                "Method {method_name} with {} arguments is not available on #temporals",
                arguments.len()
            ))),
        }
    }

    fn format(
        &self,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Result<Option<Arc<TemplateValue>>, TemporalsError> {
        let target = temporal(&arguments[0])?;
        let mut pattern = None;
        let mut locale = None;
        let mut zone = None;
        if let Some(second) = arguments.get(1) {
            if let Some(value) = locale_argument(second) {
                locale = Some(value);
            } else {
                pattern = Some(required_string(second, "Pattern cannot be null")?);
            }
        }
        if let Some(third) = arguments.get(2) {
            if let Some(value) = locale_argument(third) {
                locale = Some(value);
            } else {
                let zone_id = required_string(third, "ZoneId cannot be null")?;
                zone = Some(zone_id.parse().map_err(|_| {
                    TemporalsError::new(format!("Unknown time-zone ID: {zone_id}"))
                })?);
            }
        }
        Ok(string_value(
            self.formatting
                .format(target, pattern.as_deref(), locale, zone)
                .map_err(error)?,
        ))
    }

    fn integer(
        &self,
        arguments: &[Option<Arc<TemplateValue>>],
        method: impl Fn(
            &TemporalFormattingUtils,
            Option<&TemporalValue>,
        ) -> Result<Option<i32>, TemporalFormattingError>,
    ) -> Result<Option<Arc<TemplateValue>>, TemporalsError> {
        Ok(method(&self.formatting, temporal_argument(arguments)?)
            .map_err(error)?
            .map(|value| Arc::new(TemplateValue::Number(NumberValue::Integer(value)))))
    }

    fn named(
        &self,
        arguments: &[Option<Arc<TemplateValue>>],
        pattern: &str,
    ) -> Result<Option<Arc<TemplateValue>>, TemporalsError> {
        Ok(string_value(
            self.formatting
                .format(temporal_argument(arguments)?, Some(pattern), None, None)
                .map_err(error)?,
        ))
    }

    fn created(
        &self,
        result: Result<TemporalValue, impl Display>,
    ) -> Result<Option<Arc<TemplateValue>>, TemporalsError> {
        result
            .map(|value| Some(temporal_value(value)))
            .map_err(|cause| TemporalsError::new(cause.to_string()))
    }

    fn invoke_collection(
        &self,
        scalar_method: &str,
        set_semantics: bool,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Result<Option<Arc<TemplateValue>>, TemporalsError> {
        let Some((target, remaining)) = arguments.split_first() else {
            return Err(TemporalsError::new("Collection method requires a target"));
        };
        let Some(target) = target else {
            return Ok(None);
        };
        let TemplateValue::List(values) = target.as_ref() else {
            return Err(TemporalsError::new("Target is not an array, List or Set"));
        };
        let mut output = Vec::with_capacity(values.len());
        for value in values.iter() {
            let mut item_arguments = vec![Some(Arc::clone(value))];
            item_arguments.extend(remaining.iter().cloned());
            let item = self
                .invoke(scalar_method, &item_arguments)?
                .unwrap_or_else(|| Arc::new(TemplateValue::Null));
            if !set_semantics
                || !output.iter().any(|value: &Arc<TemplateValue>| {
                    value.to_utf16_string() == item.to_utf16_string()
                })
            {
                output.push(item);
            }
        }
        Ok(Some(Arc::new(TemplateValue::List(Arc::new(output)))))
    }
}

impl TemplateObject for Temporals {
    fn class_name(&self) -> &str {
        "org.thymeleaf.expression.Temporals"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str("org.thymeleaf.expression.Temporals")
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
                .map_err(|cause| Box::new(cause) as TemplateObjectMethodError),
        )
    }
}

/// `#temporals` 创建、格式化或字段访问错误。
#[derive(Debug)]
/// 对应 Java 语义：`Temporals` 的 Rust 侧类型 `TemporalsError`。
pub struct TemporalsError {
    message: String,
}

impl TemporalsError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for TemporalsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for TemporalsError {}

fn error(cause: impl Display) -> TemporalsError {
    TemporalsError::new(cause.to_string())
}

fn temporal(value: &Option<Arc<TemplateValue>>) -> Result<Option<&TemporalValue>, TemporalsError> {
    TemporalObjects::temporal(value.as_deref()).map_err(error)
}

fn temporal_argument(
    arguments: &[Option<Arc<TemplateValue>>],
) -> Result<Option<&TemporalValue>, TemporalsError> {
    match arguments {
        [target] => temporal(target),
        _ => Err(TemporalsError::new(
            "Temporal field method requires one argument",
        )),
    }
}

fn temporal_value(value: TemporalValue) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Object(Arc::new(value)))
}

fn string_value(value: Option<Utf16String>) -> Option<Arc<TemplateValue>> {
    value.map(|value| Arc::new(TemplateValue::string(value)))
}

fn required_string(
    value: &Option<Arc<TemplateValue>>,
    message: &str,
) -> Result<String, TemporalsError> {
    value
        .as_deref()
        .and_then(TemplateValue::to_utf16_string)
        .map(|value| value.to_string_lossy())
        .ok_or_else(|| TemporalsError::new(message))
}

fn optional_pattern(
    value: Option<&Option<Arc<TemplateValue>>>,
) -> Result<Option<String>, TemporalsError> {
    value
        .map(|value| required_string(value, "Pattern cannot be null"))
        .transpose()
}

fn locale_argument(value: &Option<Arc<TemplateValue>>) -> Option<&Locale> {
    match value.as_deref() {
        Some(TemplateValue::Object(object)) => object.as_any().downcast_ref(),
        _ => None,
    }
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

fn default_zone() -> Tz {
    std::env::var("TZ")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(chrono_tz::UTC)
}
