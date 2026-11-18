use jiff::civil::{Date, DateTime};
use jiff::tz::{AmbiguousOffset, Offset, TimeZone};
use jiff::{Timestamp, ZonedDateTime};
use thiserror::Error;

use crate::expression::TemplateValue;
use crate::util::Locale;

use super::{TemporalKind, TemporalValue};

/// Java Time 对象规范化与默认格式选择工具。
///
/// 对应 Java: `org.thymeleaf.util.temporal.TemporalObjects`。
pub struct TemporalObjects;

/// Java `z` pattern 的通用时区名：带偏移类型保留偏移（UTC → "Z"、否则
/// "+HH:MM"）；其余在默认 ZoneId 下取 UTC → "Z"、命名时区取缩写。
/// 对应 Java 语义：`TemporalObjects` 的 `java_short_zone` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn java_short_zone(target: &TemporalValue, default_zone: &TimeZone) -> String {
    match target {
        TemporalValue::OffsetDateTime(value) => fixed_offset_zone(&value.offset()),
        TemporalValue::OffsetTime(_, offset) => fixed_offset_zone(offset),
        TemporalValue::ZonedDateTime(value) => tz_zone(value.time_zone()),
        _ => tz_zone(default_zone),
    }
}

/// 对应 Java 语义：`TemporalObjects` 的 `fixed_offset_zone` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn fixed_offset_zone(offset: &Offset) -> String {
    if offset.seconds() == 0 {
        "Z".to_owned()
    } else {
        let seconds = offset.seconds().unsigned_abs();
        let sign = if offset.seconds() < 0 { '-' } else { '+' };
        format!("{sign}{:02}:{:02}", seconds / 3600, seconds / 60 % 60)
    }
}

/// 对应 Java 语义：`TemporalObjects` 的 `tz_zone` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn tz_zone(zone: &TimeZone) -> String {
    if zone == &TimeZone::UTC || zone.iana_name() == Some("UTC") {
        "Z".to_owned()
    } else {
        zone.to_offset_info(Timestamp::now())
            .abbreviation()
            .to_owned()
    }
}

impl TemporalObjects {
    /// 从模板动态值读取 Java `Temporal`。
    /// 对应 Java: `TemporalObjects#temporal()`。
    pub fn temporal(
        value: Option<&TemplateValue>,
    ) -> Result<Option<&TemporalValue>, TemporalError> {
        match value {
            None | Some(TemplateValue::Null) => Ok(None),
            Some(TemplateValue::Object(object)) => object
                .as_any()
                .downcast_ref::<TemporalValue>()
                .map(Some)
                .ok_or_else(|| {
                    invalid(format!(
                        "Cannot normalize class \"{}\" as a date",
                        object.class_name()
                    ))
                }),
            Some(value) => Err(invalid(format!(
                "Cannot normalize class \"{}\" as a date",
                value.class_name()
            ))),
        }
    }

    /// 返回与 Java `formatterFor` 类型分派一致的默认 strftime pattern。
    ///
    /// Java（JDK 9+ CLDR）基准：LocalDateTime → `ofLocalizedDateTime(LONG, MEDIUM)`
    /// （无时区名）；ZonedDateTime → `ofLocalizedDateTime(LONG)`（含 `z` 时区名，
    /// UTC → "Z"）；OffsetDateTime → `appendLocalized(LONG, MEDIUM)` +
    /// `appendLocalizedOffset(FULL)`（"GMT"/"GMT+HH:MM"，在 format 时追加）；
    /// OffsetTime → `HH:mm:ss` + 同样的偏移段。
    /// 对应 Java: `TemporalObjects#formatterFor()`。
    pub fn formatter_for(target: &TemporalValue, locale: &Locale) -> Result<String, TemporalError> {
        let language = locale.get_language().to_string_lossy();
        Ok(match target.kind() {
            TemporalKind::Instant => "%Y-%m-%dT%H:%M:%S%.fZ".to_owned(),
            TemporalKind::LocalDate => localized_date_pattern(&language),
            TemporalKind::LocalDateTime => localized_datetime_pattern(&language),
            TemporalKind::ZonedDateTime => {
                let zone = match target {
                    TemporalValue::ZonedDateTime(value) => tz_zone(value.time_zone()),
                    _ => unreachable!("ZonedDateTime kind implies ZonedDateTime value"),
                };
                if language == "zh" {
                    // Java zh_CLDR LONG datetime：`y年M月d日 z HH:mm:ss`（时区名在日期与时间之间）。
                    format!("%Y年%m月%d日 {zone} %H:%M:%S")
                } else if language == "de" {
                    // Java de_CLDR LONG datetime：`d. MMMM y, HH:mm:ss z`。
                    format!("%-d. %B %Y, %H:%M:%S {zone}")
                } else {
                    // Java en_US LONG datetime：`MMMM d, y, h:mm:ss a z`。
                    format!("%B %-d, %Y, %-I:%M:%S %p {zone}")
                }
            }
            TemporalKind::LocalTime => {
                if language == "zh" || language == "de" {
                    "%H:%M:%S".to_owned()
                } else {
                    "%-I:%M:%S %p".to_owned()
                }
            }
            // 偏移段（"GMT"/"GMT+HH:MM"）由 TemporalFormattingUtils::format 追加。
            TemporalKind::OffsetDateTime => localized_datetime_pattern(&language),
            TemporalKind::OffsetTime => "%H:%M:%S".to_owned(),
            TemporalKind::Year => "%Y".to_owned(),
            TemporalKind::YearMonth => {
                if Self::should_display_year_before_month(locale) {
                    "%Y %B".to_owned()
                } else {
                    "%B %Y".to_owned()
                }
            }
        })
    }

    /// 将缺失字段按 Java `zonedTime` 规则补齐并换算到可格式化时间。
    /// 对应 Java: `TemporalObjects#zonedTime()`。
    pub fn zoned_time(
        target: &TemporalValue,
        default_zone_id: TimeZone,
    ) -> Result<ZonedDateTime, TemporalError> {
        let today = default_zone_id.to_datetime(Timestamp::now()).date();
        let local = match target {
            TemporalValue::Instant(value) => return Ok(value.to_zoned_datetime(default_zone_id)),
            TemporalValue::LocalDate(value) => value.at(0, 0, 0, 0),
            TemporalValue::LocalDateTime(value) => *value,
            TemporalValue::LocalTime(value) => DateTime::from_parts(today, *value),
            TemporalValue::OffsetDateTime(value) => {
                return Ok(value.timestamp().to_zoned_datetime(default_zone_id));
            }
            TemporalValue::OffsetTime(value, offset) => {
                let fixed = offset
                    .to_timestamp(DateTime::from_parts(today, *value))
                    .map_err(|_| invalid("Cannot resolve OffsetTime"))?;
                return Ok(fixed.to_zoned_datetime(default_zone_id));
            }
            TemporalValue::Year(year) => year_month_date(*year, 1)?.at(0, 0, 0, 0),
            TemporalValue::YearMonth(year, month) => year_month_date(*year, *month)?.at(0, 0, 0, 0),
            TemporalValue::ZonedDateTime(value) => return Ok(value.clone()),
        };
        let candidate = default_zone_id.to_ambiguous_zoned_datetime(local);
        if matches!(candidate.offset(), AmbiguousOffset::Gap { .. }) {
            return Err(invalid("Local temporal is in a ZoneId gap"));
        }
        candidate
            .earlier()
            .map_err(|error| invalid(error.to_string()))
    }

    /// 读取日期字段；目标不支持该字段时返回 Java 式错误。
    /// 对应 Java 语义：`TemporalObjects` 的 `date_fields` 行为（Rust 侧辅助/私有路径）。
    pub fn date_fields(target: &TemporalValue) -> Result<(i32, u32, u32, u32), TemporalError> {
        let date = match target {
            TemporalValue::LocalDate(value) => *value,
            TemporalValue::LocalDateTime(value) => value.date(),
            TemporalValue::OffsetDateTime(value) => value.date(),
            TemporalValue::ZonedDateTime(value) => value.date(),
            TemporalValue::Year(year) => year_month_date(*year, 1)?,
            TemporalValue::YearMonth(year, month) => year_month_date(*year, *month)?,
            _ => return Err(invalid("Unsupported field: DayOfMonth")),
        };
        Ok((
            i32::from(date.year()),
            date.month() as u32,
            date.day() as u32,
            date.weekday() as u32,
        ))
    }

    /// 读取时间字段；目标不支持该字段时返回 Java 式错误。
    /// 对应 Java 语义：`TemporalObjects` 的 `time_fields` 行为（Rust 侧辅助/私有路径）。
    pub fn time_fields(target: &TemporalValue) -> Result<(u32, u32, u32, u32), TemporalError> {
        let time = match target {
            TemporalValue::LocalTime(value) | TemporalValue::OffsetTime(value, _) => *value,
            TemporalValue::LocalDateTime(value) => value.time(),
            TemporalValue::OffsetDateTime(value) => value.time(),
            TemporalValue::ZonedDateTime(value) => value.time(),
            _ => return Err(invalid("Unsupported field: HourOfDay")),
        };
        Ok((
            time.hour() as u32,
            time.minute() as u32,
            time.second() as u32,
            time.subsec_nanosecond() as u32,
        ))
    }

    /// 返回 temporal 自身固定偏移秒数；无偏移类型返回默认时区当前偏移。
    #[must_use]
    /// 对应 Java 语义：`TemporalObjects` 的 `offset_seconds` 行为（Rust 侧辅助/私有路径）。
    pub fn offset_seconds(target: &TemporalValue, default_zone_id: TimeZone) -> i32 {
        match target {
            TemporalValue::OffsetDateTime(value) => value.offset().seconds(),
            TemporalValue::OffsetTime(_, offset) => offset.seconds(),
            TemporalValue::ZonedDateTime(value) => value.offset().seconds(),
            _ => default_zone_id.to_offset(Timestamp::now()).seconds(),
        }
    }

    fn should_display_year_before_month(locale: &Locale) -> bool {
        matches!(
            locale.get_country().to_string_lossy().as_str(),
            "BT" | "CA" | "CN" | "KP" | "KR" | "TW" | "HU" | "IR" | "JP" | "LT" | "MN"
        )
    }
}

/// 将公开的年份与月份字段校验后构造该月第一天。
pub(crate) fn year_month_date(year: i32, month: u32) -> Result<Date, TemporalError> {
    let year = i16::try_from(year).map_err(|_| invalid("Invalid Year"))?;
    let month = i8::try_from(month).map_err(|_| invalid("Invalid YearMonth"))?;
    Date::new(year, month, 1).map_err(|error| invalid(error.to_string()))
}

/// Java `DateTimeFormatter.ofLocalizedDate(LONG)` 的 strftime 等价。
fn localized_date_pattern(language: &str) -> String {
    if language == "zh" {
        "%Y年%m月%d日".to_owned()
    } else if language == "de" {
        "%-d. %B %Y".to_owned()
    } else {
        "%B %-d, %Y".to_owned()
    }
}

/// Java `DateTimeFormatter.ofLocalizedDateTime(LONG, MEDIUM)` 的 strftime 等价（不含时区名）。
fn localized_datetime_pattern(language: &str) -> String {
    if language == "zh" {
        "%Y年%m月%d日 %H:%M:%S".to_owned()
    } else if language == "de" {
        "%-d. %B %Y, %H:%M:%S".to_owned()
    } else {
        "%B %-d, %Y, %-I:%M:%S %p".to_owned()
    }
}

/// Java temporal 规范化或字段访问错误。
#[derive(Debug, Error)]
#[error("{message}")]
/// 对应 Java 语义：`TemporalObjects` 的 Rust 侧类型 `TemporalError`。
pub struct TemporalError {
    message: String,
}

fn invalid(message: impl Into<String>) -> TemporalError {
    TemporalError {
        message: message.into(),
    }
}
