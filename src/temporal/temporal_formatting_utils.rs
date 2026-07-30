use chrono_tz::Tz;
use thiserror::Error;

use crate::util::{JavaLocale, JavaString};

use super::temporal_creation_utils::java_pattern;
use super::{JavaTemporal, TemporalObjects};

/// Java 8 Time 对象格式化及字段读取工具。
///
/// 对应 Java: `org.thymeleaf.util.temporal.TemporalFormattingUtils`。
pub struct TemporalFormattingUtils {
    locale: JavaLocale,
    default_zone_id: Tz,
}

impl TemporalFormattingUtils {
    /// 使用 Locale 与默认 ZoneId 创建格式化工具。
    pub fn new(locale: JavaLocale, default_zone_id: Tz) -> Result<Self, TemporalFormattingError> {
        Ok(Self {
            locale,
            default_zone_id,
        })
    }

    /// 使用默认格式或指定 Java DateTimeFormatter pattern 格式化 temporal。
    pub fn format(
        &self,
        target: Option<&JavaTemporal>,
        pattern: Option<&str>,
        locale: Option<&JavaLocale>,
        zone_id: Option<Tz>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        let Some(target) = target else {
            return Ok(None);
        };
        if pattern.is_some_and(|pattern| pattern.trim().is_empty()) {
            return Err(invalid("Pattern cannot be null or empty"));
        }
        let locale = locale.unwrap_or(&self.locale);
        let pattern = match pattern {
            None => TemporalObjects::formatter_for(target, locale)?,
            Some("SHORT") => localized_pattern(target, locale, "SHORT"),
            Some("MEDIUM") => localized_pattern(target, locale, "MEDIUM"),
            Some("LONG") => localized_pattern(target, locale, "LONG"),
            Some("FULL") => localized_pattern(target, locale, "FULL"),
            Some(pattern) => java_pattern(pattern),
        };

        let formatted = match target {
            JavaTemporal::Instant(value) if pattern.contains('Z') => {
                value.format(&pattern).to_string()
            }
            JavaTemporal::LocalDate(value) if zone_id.is_none() => {
                value.format(&pattern).to_string()
            }
            JavaTemporal::LocalDateTime(value) if zone_id.is_none() => {
                value.format(&pattern).to_string()
            }
            JavaTemporal::LocalTime(value) if zone_id.is_none() => {
                value.format(&pattern).to_string()
            }
            JavaTemporal::OffsetDateTime(value) if zone_id.is_none() => {
                value.format(&pattern).to_string()
            }
            JavaTemporal::OffsetTime(value, offset) if zone_id.is_none() => format_offset(
                value.format(&pattern).to_string(),
                offset.local_minus_utc(),
                &pattern,
            ),
            JavaTemporal::Year(value) if zone_id.is_none() => format_year(*value, &pattern),
            JavaTemporal::YearMonth(year, month) if zone_id.is_none() => {
                let date = chrono::NaiveDate::from_ymd_opt(*year, *month, 1)
                    .ok_or_else(|| invalid("Invalid YearMonth"))?;
                date.format(&pattern).to_string()
            }
            JavaTemporal::ZonedDateTime(value) if zone_id.is_none() => {
                value.format(&pattern).to_string()
            }
            _ => TemporalObjects::zoned_time(target, zone_id.unwrap_or(self.default_zone_id))?
                .format(&pattern)
                .to_string(),
        };
        Ok(Some(JavaString::from_rust_str(&formatted)))
    }

    /// 返回一个月中的日期。
    pub fn day(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        target
            .map(|target| TemporalObjects::date_fields(target).map(|(_, _, day, _)| day as i32))
            .transpose()
            .map_err(Into::into)
    }

    /// 返回一月为 1 的月份。
    pub fn month(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        target
            .map(|target| TemporalObjects::date_fields(target).map(|(_, month, _, _)| month as i32))
            .transpose()
            .map_err(Into::into)
    }

    /// 返回当前 Locale 下的完整月份名称。
    ///
    /// 对应 Java: `TemporalFormattingUtils#monthName(Object)`。
    pub fn month_name(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        self.format(target, Some("MMMM"), None, None)
    }

    /// 返回当前 Locale 下的短月份名称。
    ///
    /// 对应 Java: `TemporalFormattingUtils#monthNameShort(Object)`。
    pub fn month_name_short(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        self.format(target, Some("MMM"), None, None)
    }

    /// 返回年份。
    pub fn year(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        target
            .map(|target| TemporalObjects::date_fields(target).map(|(year, _, _, _)| year))
            .transpose()
            .map_err(Into::into)
    }

    /// 返回 ISO 周一为 1 的星期编号。
    pub fn day_of_week(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        target
            .map(|target| {
                TemporalObjects::date_fields(target).map(|(_, _, _, weekday)| weekday as i32)
            })
            .transpose()
            .map_err(Into::into)
    }

    /// 返回当前 Locale 下的完整星期名称。
    ///
    /// 对应 Java: `TemporalFormattingUtils#dayOfWeekName(Object)`。
    pub fn day_of_week_name(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        self.format(target, Some("EEEE"), None, None)
    }

    /// 返回当前 Locale 下的短星期名称。
    ///
    /// 对应 Java: `TemporalFormattingUtils#dayOfWeekNameShort(Object)`。
    pub fn day_of_week_name_short(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        self.format(target, Some("EEE"), None, None)
    }

    /// 返回小时。
    pub fn hour(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        self.time_field(target, 0)
    }

    /// 返回分钟。
    pub fn minute(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        self.time_field(target, 1)
    }

    /// 返回秒。
    pub fn second(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        self.time_field(target, 2)
    }

    /// 返回纳秒。
    pub fn nanosecond(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        self.time_field(target, 3)
    }

    /// 按 Thymeleaf 固定 ISO pattern 格式化并补齐缺失字段。
    pub fn format_iso(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        let Some(target) = target else {
            return Ok(None);
        };
        let value = TemporalObjects::zoned_time(target, self.default_zone_id)?;
        Ok(Some(JavaString::from_rust_str(
            &value.format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string(),
        )))
    }

    fn time_field(
        &self,
        target: Option<&JavaTemporal>,
        index: usize,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        target
            .map(|target| {
                let fields = TemporalObjects::time_fields(target)?;
                Ok([fields.0, fields.1, fields.2, fields.3][index] as i32)
            })
            .transpose()
    }
}

/// Temporal 格式化或字段读取错误。
#[derive(Debug, Error)]
#[error("{message}")]
pub struct TemporalFormattingError {
    message: String,
}

impl From<super::temporal_objects::TemporalError> for TemporalFormattingError {
    fn from(error: super::temporal_objects::TemporalError) -> Self {
        invalid(error.to_string())
    }
}

fn invalid(message: impl Into<String>) -> TemporalFormattingError {
    TemporalFormattingError {
        message: message.into(),
    }
}

fn localized_pattern(target: &JavaTemporal, locale: &JavaLocale, style: &str) -> String {
    let date_only = matches!(target, JavaTemporal::LocalDate(_));
    let time_only = matches!(
        target,
        JavaTemporal::LocalTime(_) | JavaTemporal::OffsetTime(_, _)
    );
    let zh = locale.get_language().to_string_lossy() == "zh";
    match (date_only, time_only, zh, style) {
        (true, _, true, "FULL") => "%Y年%m月%d日 %A".to_owned(),
        (true, _, true, _) => "%Y年%m月%d日".to_owned(),
        (true, _, false, "SHORT") => "%-m/%-d/%y".to_owned(),
        (true, _, false, "FULL") => "%A, %B %-d, %Y".to_owned(),
        (true, _, false, _) => "%B %-d, %Y".to_owned(),
        (_, true, _, "SHORT") => "%-I:%M %p".to_owned(),
        (_, true, _, _) => "%-I:%M:%S %p".to_owned(),
        (_, _, true, _) => "%Y年%m月%d日 %H:%M:%S".to_owned(),
        (_, _, false, "SHORT") => "%-m/%-d/%y, %-I:%M %p".to_owned(),
        _ => "%B %-d, %Y, %-I:%M:%S %p".to_owned(),
    }
}

fn format_year(year: i32, pattern: &str) -> String {
    pattern
        .replace("%Y", &format!("{year:04}"))
        .replace("%y", &format!("{:02}", year.rem_euclid(100)))
}

fn format_offset(mut value: String, seconds: i32, pattern: &str) -> String {
    if pattern.contains("%:z") {
        let sign = if seconds < 0 { '-' } else { '+' };
        let seconds = seconds.unsigned_abs();
        value = value.replace(
            "%:z",
            &format!("{sign}{:02}:{:02}", seconds / 3600, seconds % 3600 / 60),
        );
    }
    value
}
