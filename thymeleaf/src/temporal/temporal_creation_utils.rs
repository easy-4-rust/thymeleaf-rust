use chrono::{LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use thiserror::Error;

use super::JavaTemporal;

/// Java 8 Time 对象创建工具。
///
/// 对应 Java: `org.thymeleaf.util.temporal.TemporalCreationUtils`。
pub struct TemporalCreationUtils;

impl TemporalCreationUtils {
    /// 创建无状态工具。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 按 Java `LocalDate.of` / `LocalDateTime.of` 规则创建 temporal。
    /// 对应 Java: `TemporalCreationUtils#create()`。
    pub fn create(&self, fields: &[i32]) -> Result<JavaTemporal, TemporalCreationError> {
        if !matches!(fields.len(), 3 | 5 | 6 | 7) {
            return Err(invalid("Temporal create requires 3, 5, 6 or 7 fields"));
        }
        let date = NaiveDate::from_ymd_opt(fields[0], to_u32(fields[1])?, to_u32(fields[2])?)
            .ok_or_else(|| invalid("Invalid value for Year/MonthOfYear/DayOfMonth"))?;
        if fields.len() == 3 {
            return Ok(JavaTemporal::LocalDate(date));
        }
        let time = NaiveTime::from_hms_nano_opt(
            to_u32(fields[3])?,
            to_u32(fields[4])?,
            fields.get(5).copied().map_or(Ok(0), to_u32)?,
            fields.get(6).copied().map_or(Ok(0), to_u32)?,
        )
        .ok_or_else(|| invalid("Invalid value for Hour/Minute/Second/NanoOfSecond"))?;
        Ok(JavaTemporal::LocalDateTime(NaiveDateTime::new(date, time)))
    }

    /// 解析 ISO 或自定义 Java pattern 的 `LocalDate`。
    /// 对应 Java: `TemporalCreationUtils#createDate()`。
    pub fn create_date(
        &self,
        text: &str,
        pattern: Option<&str>,
    ) -> Result<JavaTemporal, TemporalCreationError> {
        let pattern = pattern.map_or("%Y-%m-%d".to_owned(), java_pattern);
        NaiveDate::parse_from_str(text, &pattern)
            .map(JavaTemporal::LocalDate)
            .map_err(|error| invalid(error.to_string()))
    }

    /// 解析 ISO 或自定义 Java pattern 的 `LocalDateTime`。
    /// 对应 Java: `TemporalCreationUtils#createDateTime()`。
    pub fn create_date_time(
        &self,
        text: &str,
        pattern: Option<&str>,
    ) -> Result<JavaTemporal, TemporalCreationError> {
        let pattern = pattern.map_or("%Y-%m-%dT%H:%M:%S".to_owned(), java_pattern);
        NaiveDateTime::parse_from_str(text, &pattern)
            .map(JavaTemporal::LocalDateTime)
            .map_err(|error| invalid(error.to_string()))
    }

    /// 返回系统默认时区当前 `LocalDateTime`。
    #[must_use]
    /// 对应 Java: `TemporalCreationUtils#createNow()`。
    pub fn create_now(&self) -> JavaTemporal {
        JavaTemporal::LocalDateTime(Utc::now().with_timezone(&default_zone()).naive_local())
    }

    /// 返回指定 ZoneId 当前 `ZonedDateTime`。
    /// 对应 Java: `TemporalCreationUtils#createNowForTimeZone()`。
    pub fn create_now_for_time_zone(
        &self,
        zone_id: &str,
    ) -> Result<JavaTemporal, TemporalCreationError> {
        let zone = parse_zone(zone_id)?;
        Ok(JavaTemporal::ZonedDateTime(Utc::now().with_timezone(&zone)))
    }

    /// 返回系统默认时区当前 `LocalDate`。
    #[must_use]
    /// 对应 Java: `TemporalCreationUtils#createToday()`。
    pub fn create_today(&self) -> JavaTemporal {
        JavaTemporal::LocalDate(Utc::now().with_timezone(&default_zone()).date_naive())
    }

    /// 返回指定 ZoneId 当天零点 `ZonedDateTime`。
    /// 对应 Java: `TemporalCreationUtils#createTodayForTimeZone()`。
    pub fn create_today_for_time_zone(
        &self,
        zone_id: &str,
    ) -> Result<JavaTemporal, TemporalCreationError> {
        let zone = parse_zone(zone_id)?;
        let now = Utc::now().with_timezone(&zone);
        let midnight = now
            .date_naive()
            .and_hms_nano_opt(0, 0, 0, 0)
            .expect("midnight is valid");
        let value = match zone.from_local_datetime(&midnight) {
            LocalResult::Single(value) | LocalResult::Ambiguous(value, _) => value,
            LocalResult::None => zone
                .from_local_datetime(&(midnight + chrono::Duration::hours(1)))
                .earliest()
                .ok_or_else(|| invalid("Cannot resolve local midnight in ZoneId"))?,
        };
        Ok(JavaTemporal::ZonedDateTime(
            value
                .with_hour(0)
                .and_then(|value| value.with_minute(0))
                .and_then(|value| value.with_second(0))
                .and_then(|value| value.with_nanosecond(0))
                .unwrap_or(value),
        ))
    }
}

impl Default for TemporalCreationUtils {
    fn default() -> Self {
        Self::new()
    }
}

/// Temporal 创建或解析错误。
#[derive(Debug, Error)]
#[error("{message}")]
/// 对应 Java 语义：`TemporalCreationUtils` 的 Rust 侧类型 `TemporalCreationError`。
pub struct TemporalCreationError {
    message: String,
}

fn invalid(message: impl Into<String>) -> TemporalCreationError {
    TemporalCreationError {
        message: message.into(),
    }
}

fn to_u32(value: i32) -> Result<u32, TemporalCreationError> {
    u32::try_from(value).map_err(|_| invalid(format!("Invalid negative temporal field: {value}")))
}

fn parse_zone(value: &str) -> Result<Tz, TemporalCreationError> {
    value
        .parse()
        .map_err(|_| invalid(format!("Unknown time-zone ID: {value}")))
}

fn default_zone() -> Tz {
    std::env::var("TZ")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(chrono_tz::UTC)
}
/// 对应 Java 语义：`TemporalCreationUtils` 的 `java_pattern` 行为（Rust 侧辅助/私有路径）。

pub(crate) fn java_pattern(pattern: &str) -> String {
    let mut output = String::new();
    let mut chars = pattern.chars().peekable();
    let mut quoted = false;
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            if chars.peek() == Some(&'\'') {
                chars.next();
                output.push('\'');
            } else {
                quoted = !quoted;
            }
            continue;
        }
        if quoted {
            output.push(ch);
            continue;
        }
        let mut count = 1;
        while chars.peek() == Some(&ch) {
            chars.next();
            count += 1;
        }
        let directive = match ch {
            'y' | 'u' if count == 2 => "%y",
            'y' | 'u' => "%Y",
            'M' if count >= 4 => "%B",
            'M' if count == 3 => "%b",
            'M' => "%m",
            // Java: "d" 不补零、"dd" 补零
            'd' if count == 2 => "%d",
            'd' => "%-d",
            'E' if count >= 4 => "%A",
            'E' => "%a",
            'H' => "%H",
            'h' => "%I",
            'm' => "%M",
            's' => "%S",
            'S' => {
                output.push_str("__THYMELEAF_FRACTION_");
                output.push_str(&count.to_string());
                output.push_str("__");
                continue;
            }
            'n' => "%f",
            'a' => "%p",
            'X' | 'x' | 'Z' => "%:z",
            'z' => "%Z",
            _ => {
                for _ in 0..count {
                    output.push(ch);
                }
                continue;
            }
        };
        output.push_str(directive);
    }
    output
}
