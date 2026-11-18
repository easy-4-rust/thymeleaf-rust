use jiff::civil::{Date, DateTime, Time};
use jiff::tz::AmbiguousOffset;
use jiff::tz::TimeZone;
use jiff::{SignedDuration, Timestamp};
use thiserror::Error;

use super::TemporalValue;

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
    pub fn create(&self, fields: &[i32]) -> Result<TemporalValue, TemporalCreationError> {
        if !matches!(fields.len(), 3 | 5 | 6 | 7) {
            return Err(invalid("Temporal create requires 3, 5, 6 or 7 fields"));
        }
        let year = i16::try_from(fields[0]).map_err(|_| invalid("Invalid Year"))?;
        let date = Date::new(year, field(fields[1])?, field(fields[2])?)
            .map_err(|_| invalid("Invalid value for Year/MonthOfYear/DayOfMonth"))?;
        if fields.len() == 3 {
            return Ok(TemporalValue::LocalDate(date));
        }
        let time = Time::new(
            field(fields[3])?,
            field(fields[4])?,
            field(fields.get(5).copied().unwrap_or(0))?,
            fields.get(6).copied().unwrap_or(0),
        )
        .map_err(|_| invalid("Invalid value for Hour/Minute/Second/NanoOfSecond"))?;
        Ok(TemporalValue::LocalDateTime(DateTime::from_parts(
            date, time,
        )))
    }

    /// 解析 ISO 或自定义 Java pattern 的 `LocalDate`。
    /// 对应 Java: `TemporalCreationUtils#createDate()`。
    pub fn create_date(
        &self,
        text: &str,
        pattern: Option<&str>,
    ) -> Result<TemporalValue, TemporalCreationError> {
        let pattern = pattern.map_or("%Y-%m-%d".to_owned(), process_pattern);
        Date::strptime(&pattern, text)
            .map(TemporalValue::LocalDate)
            .map_err(|error| invalid(error.to_string()))
    }

    /// 解析 ISO 或自定义 Java pattern 的 `LocalDateTime`。
    /// 对应 Java: `TemporalCreationUtils#createDateTime()`。
    pub fn create_date_time(
        &self,
        text: &str,
        pattern: Option<&str>,
    ) -> Result<TemporalValue, TemporalCreationError> {
        let pattern = pattern.map_or("%Y-%m-%dT%H:%M:%S".to_owned(), process_pattern);
        DateTime::strptime(&pattern, text)
            .map(TemporalValue::LocalDateTime)
            .map_err(|error| invalid(error.to_string()))
    }

    /// 返回系统默认时区当前 `LocalDateTime`。
    #[must_use]
    /// 对应 Java: `TemporalCreationUtils#createNow()`。
    pub fn create_now(&self) -> TemporalValue {
        TemporalValue::LocalDateTime(default_zone().to_datetime(Timestamp::now()))
    }

    /// 返回指定 ZoneId 当前 `ZonedDateTime`。
    /// 对应 Java: `TemporalCreationUtils#createNowForTimeZone()`。
    pub fn create_now_for_time_zone(
        &self,
        zone_id: &str,
    ) -> Result<TemporalValue, TemporalCreationError> {
        let zone = parse_zone(zone_id)?;
        Ok(TemporalValue::ZonedDateTime(
            Timestamp::now().to_zoned_datetime(zone),
        ))
    }

    /// 返回系统默认时区当前 `LocalDate`。
    #[must_use]
    /// 对应 Java: `TemporalCreationUtils#createToday()`。
    pub fn create_today(&self) -> TemporalValue {
        TemporalValue::LocalDate(default_zone().to_datetime(Timestamp::now()).date())
    }

    /// 返回指定 ZoneId 当天零点 `ZonedDateTime`。
    /// 对应 Java: `TemporalCreationUtils#createTodayForTimeZone()`。
    pub fn create_today_for_time_zone(
        &self,
        zone_id: &str,
    ) -> Result<TemporalValue, TemporalCreationError> {
        let zone = parse_zone(zone_id)?;
        let midnight = zone.to_datetime(Timestamp::now()).date().at(0, 0, 0, 0);
        let candidate = zone.to_ambiguous_zoned_datetime(midnight);
        let value = match candidate.offset() {
            AmbiguousOffset::Gap { .. } => {
                let next_hour = midnight
                    .checked_add(SignedDuration::from_hours(1))
                    .map_err(|_| invalid("Cannot resolve local midnight in ZoneId"))?;
                let next = zone.to_ambiguous_zoned_datetime(next_hour);
                if matches!(next.offset(), AmbiguousOffset::Gap { .. }) {
                    return Err(invalid("Cannot resolve local midnight in ZoneId"));
                }
                next.earlier()
            }
            _ => candidate.earlier(),
        }
        .map_err(|_| invalid("Cannot resolve local midnight in ZoneId"))?;
        Ok(TemporalValue::ZonedDateTime(value))
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

fn field(value: i32) -> Result<i8, TemporalCreationError> {
    i8::try_from(value).map_err(|_| invalid(format!("Invalid temporal field: {value}")))
}

fn parse_zone(value: &str) -> Result<TimeZone, TemporalCreationError> {
    TimeZone::get(value).map_err(|_| invalid(format!("Unknown time-zone ID: {value}")))
}

fn default_zone() -> TimeZone {
    std::env::var("TZ")
        .ok()
        .and_then(|value| TimeZone::get(&value).ok())
        .unwrap_or(TimeZone::UTC)
}

/// 对应 Java 语义：`TemporalCreationUtils` 的 `process_pattern` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn process_pattern(pattern: &str) -> String {
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
