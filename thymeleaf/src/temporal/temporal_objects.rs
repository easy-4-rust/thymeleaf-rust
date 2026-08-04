use chrono::{
    DateTime, Datelike, LocalResult, NaiveDate, NaiveDateTime, Offset, TimeZone, Timelike, Utc,
};
use chrono_tz::Tz;
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
pub(crate) fn java_short_zone(target: &TemporalValue, default_zone: &Tz) -> String {
    match target {
        TemporalValue::OffsetDateTime(value) => fixed_offset_zone(value.offset()),
        TemporalValue::OffsetTime(_, offset) => fixed_offset_zone(offset),
        TemporalValue::ZonedDateTime(value) => tz_zone(value.timezone()),
        _ => tz_zone(*default_zone),
    }
}

/// 对应 Java 语义：`TemporalObjects` 的 `fixed_offset_zone` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn fixed_offset_zone(offset: &chrono::FixedOffset) -> String {
    if offset.local_minus_utc() == 0 {
        "Z".to_owned()
    } else {
        // FixedOffset 通过 TimeZone trait 构造参考时刻后按 `%:z` 渲染 "+HH:MM"。
        chrono::Utc::now()
            .with_timezone(offset)
            .format("%:z")
            .to_string()
    }
}

/// 对应 Java 语义：`TemporalObjects` 的 `tz_zone` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn tz_zone(zone: Tz) -> String {
    if zone == Tz::UTC {
        "Z".to_owned()
    } else {
        chrono::Utc::now()
            .with_timezone(&zone)
            .format("%Z")
            .to_string()
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
                        object.java_class_name()
                    ))
                }),
            Some(value) => Err(invalid(format!(
                "Cannot normalize class \"{}\" as a date",
                value.java_class_name()
            ))),
        }
    }

    /// 返回与 Java `formatterFor` 类型分派一致的默认 chrono pattern。
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
                    TemporalValue::ZonedDateTime(value) => tz_zone(value.timezone()),
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
        default_zone_id: Tz,
    ) -> Result<DateTime<Tz>, TemporalError> {
        let today = Utc::now().with_timezone(&default_zone_id).date_naive();
        let local = match target {
            TemporalValue::Instant(value) => return Ok(value.with_timezone(&default_zone_id)),
            TemporalValue::LocalDate(value) => value.and_hms_opt(0, 0, 0).expect("midnight"),
            TemporalValue::LocalDateTime(value) => *value,
            TemporalValue::LocalTime(value) => NaiveDateTime::new(today, *value),
            TemporalValue::OffsetDateTime(value) => {
                return Ok(value.with_timezone(&Utc).with_timezone(&default_zone_id));
            }
            TemporalValue::OffsetTime(value, offset) => {
                let fixed = today
                    .and_time(*value)
                    .and_local_timezone(*offset)
                    .single()
                    .ok_or_else(|| invalid("Cannot resolve OffsetTime"))?;
                return Ok(fixed.with_timezone(&Utc).with_timezone(&default_zone_id));
            }
            TemporalValue::Year(year) => NaiveDate::from_ymd_opt(*year, 1, 1)
                .ok_or_else(|| invalid("Invalid Year"))?
                .and_hms_opt(0, 0, 0)
                .expect("midnight"),
            TemporalValue::YearMonth(year, month) => NaiveDate::from_ymd_opt(*year, *month, 1)
                .ok_or_else(|| invalid("Invalid YearMonth"))?
                .and_hms_opt(0, 0, 0)
                .expect("midnight"),
            TemporalValue::ZonedDateTime(value) => return Ok(*value),
        };
        match default_zone_id.from_local_datetime(&local) {
            LocalResult::Single(value) | LocalResult::Ambiguous(value, _) => Ok(value),
            LocalResult::None => Err(invalid("Local temporal is in a ZoneId gap")),
        }
    }

    /// 读取日期字段；目标不支持该字段时返回 Java 式错误。
    /// 对应 Java 语义：`TemporalObjects` 的 `date_fields` 行为（Rust 侧辅助/私有路径）。
    pub fn date_fields(target: &TemporalValue) -> Result<(i32, u32, u32, u32), TemporalError> {
        let date = match target {
            TemporalValue::LocalDate(value) => *value,
            TemporalValue::LocalDateTime(value) => value.date(),
            TemporalValue::OffsetDateTime(value) => value.date_naive(),
            TemporalValue::ZonedDateTime(value) => value.date_naive(),
            TemporalValue::Year(year) => NaiveDate::from_ymd_opt(*year, 1, 1).expect("valid year"),
            TemporalValue::YearMonth(year, month) => {
                NaiveDate::from_ymd_opt(*year, *month, 1).expect("valid year-month")
            }
            _ => return Err(invalid("Unsupported field: DayOfMonth")),
        };
        Ok((
            date.year(),
            date.month(),
            date.day(),
            date.weekday().number_from_monday(),
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
        Ok((time.hour(), time.minute(), time.second(), time.nanosecond()))
    }

    /// 返回 temporal 自身固定偏移秒数；无偏移类型返回默认时区当前偏移。
    #[must_use]
    /// 对应 Java 语义：`TemporalObjects` 的 `offset_seconds` 行为（Rust 侧辅助/私有路径）。
    pub fn offset_seconds(target: &TemporalValue, default_zone_id: Tz) -> i32 {
        match target {
            TemporalValue::OffsetDateTime(value) => value.offset().local_minus_utc(),
            TemporalValue::OffsetTime(_, offset) => offset.local_minus_utc(),
            TemporalValue::ZonedDateTime(value) => value.offset().fix().local_minus_utc(),
            _ => Utc::now()
                .with_timezone(&default_zone_id)
                .offset()
                .fix()
                .local_minus_utc(),
        }
    }

    fn should_display_year_before_month(locale: &Locale) -> bool {
        matches!(
            locale.get_country().to_string_lossy().as_str(),
            "BT" | "CA" | "CN" | "KP" | "KR" | "TW" | "HU" | "IR" | "JP" | "LT" | "MN"
        )
    }
}

/// Java `DateTimeFormatter.ofLocalizedDate(LONG)` 的 chrono 等价。
fn localized_date_pattern(language: &str) -> String {
    if language == "zh" {
        "%Y年%m月%d日".to_owned()
    } else if language == "de" {
        "%-d. %B %Y".to_owned()
    } else {
        "%B %-d, %Y".to_owned()
    }
}

/// Java `DateTimeFormatter.ofLocalizedDateTime(LONG, MEDIUM)` 的 chrono 等价（不含时区名）。
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
