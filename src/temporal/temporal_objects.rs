use chrono::{
    DateTime, Datelike, LocalResult, NaiveDate, NaiveDateTime, Offset, TimeZone, Timelike, Utc,
};
use chrono_tz::Tz;
use thiserror::Error;

use crate::expression::TemplateValue;
use crate::util::JavaLocale;

use super::{JavaTemporal, JavaTemporalKind};

/// Java Time 对象规范化与默认格式选择工具。
///
/// 对应 Java: `org.thymeleaf.util.temporal.TemporalObjects`。
pub struct TemporalObjects;

impl TemporalObjects {
    /// 从模板动态值读取 Java `Temporal`。
    pub fn temporal(value: Option<&TemplateValue>) -> Result<Option<&JavaTemporal>, TemporalError> {
        match value {
            None | Some(TemplateValue::Null) => Ok(None),
            Some(TemplateValue::Object(object)) => object
                .as_any()
                .downcast_ref::<JavaTemporal>()
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
    pub fn formatter_for(
        target: &JavaTemporal,
        locale: &JavaLocale,
    ) -> Result<String, TemporalError> {
        let language = locale.get_language().to_string_lossy();
        Ok(match target.kind() {
            JavaTemporalKind::Instant => "%Y-%m-%dT%H:%M:%S%.fZ",
            JavaTemporalKind::LocalDate => {
                if language == "zh" {
                    "%Y年%m月%d日"
                } else {
                    "%B %-d, %Y"
                }
            }
            JavaTemporalKind::LocalDateTime | JavaTemporalKind::ZonedDateTime => {
                if language == "zh" {
                    "%Y年%m月%d日 %H:%M:%S"
                } else {
                    "%B %-d, %Y, %-I:%M:%S %p"
                }
            }
            JavaTemporalKind::LocalTime => "%-I:%M:%S %p",
            JavaTemporalKind::OffsetDateTime => "%B %-d, %Y, %-I:%M:%S %p %:z",
            JavaTemporalKind::OffsetTime => "%-H:%M:%S%:z",
            JavaTemporalKind::Year => "%Y",
            JavaTemporalKind::YearMonth => {
                if Self::should_display_year_before_month(locale) {
                    "%Y %B"
                } else {
                    "%B %Y"
                }
            }
        }
        .to_owned())
    }

    /// 将缺失字段按 Java `zonedTime` 规则补齐并换算到可格式化时间。
    pub fn zoned_time(
        target: &JavaTemporal,
        default_zone_id: Tz,
    ) -> Result<DateTime<Tz>, TemporalError> {
        let today = Utc::now().with_timezone(&default_zone_id).date_naive();
        let local = match target {
            JavaTemporal::Instant(value) => return Ok(value.with_timezone(&default_zone_id)),
            JavaTemporal::LocalDate(value) => value.and_hms_opt(0, 0, 0).expect("midnight"),
            JavaTemporal::LocalDateTime(value) => *value,
            JavaTemporal::LocalTime(value) => NaiveDateTime::new(today, *value),
            JavaTemporal::OffsetDateTime(value) => {
                return Ok(value.with_timezone(&Utc).with_timezone(&default_zone_id));
            }
            JavaTemporal::OffsetTime(value, offset) => {
                let fixed = today
                    .and_time(*value)
                    .and_local_timezone(*offset)
                    .single()
                    .ok_or_else(|| invalid("Cannot resolve OffsetTime"))?;
                return Ok(fixed.with_timezone(&Utc).with_timezone(&default_zone_id));
            }
            JavaTemporal::Year(year) => NaiveDate::from_ymd_opt(*year, 1, 1)
                .ok_or_else(|| invalid("Invalid Year"))?
                .and_hms_opt(0, 0, 0)
                .expect("midnight"),
            JavaTemporal::YearMonth(year, month) => NaiveDate::from_ymd_opt(*year, *month, 1)
                .ok_or_else(|| invalid("Invalid YearMonth"))?
                .and_hms_opt(0, 0, 0)
                .expect("midnight"),
            JavaTemporal::ZonedDateTime(value) => return Ok(value.with_timezone(&default_zone_id)),
        };
        match default_zone_id.from_local_datetime(&local) {
            LocalResult::Single(value) | LocalResult::Ambiguous(value, _) => Ok(value),
            LocalResult::None => Err(invalid("Local temporal is in a ZoneId gap")),
        }
    }

    /// 读取日期字段；目标不支持该字段时返回 Java 式错误。
    pub fn date_fields(target: &JavaTemporal) -> Result<(i32, u32, u32, u32), TemporalError> {
        let date = match target {
            JavaTemporal::LocalDate(value) => *value,
            JavaTemporal::LocalDateTime(value) => value.date(),
            JavaTemporal::OffsetDateTime(value) => value.date_naive(),
            JavaTemporal::ZonedDateTime(value) => value.date_naive(),
            JavaTemporal::Year(year) => NaiveDate::from_ymd_opt(*year, 1, 1).expect("valid year"),
            JavaTemporal::YearMonth(year, month) => {
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
    pub fn time_fields(target: &JavaTemporal) -> Result<(u32, u32, u32, u32), TemporalError> {
        let time = match target {
            JavaTemporal::LocalTime(value) | JavaTemporal::OffsetTime(value, _) => *value,
            JavaTemporal::LocalDateTime(value) => value.time(),
            JavaTemporal::OffsetDateTime(value) => value.time(),
            JavaTemporal::ZonedDateTime(value) => value.time(),
            _ => return Err(invalid("Unsupported field: HourOfDay")),
        };
        Ok((time.hour(), time.minute(), time.second(), time.nanosecond()))
    }

    /// 返回 temporal 自身固定偏移秒数；无偏移类型返回默认时区当前偏移。
    #[must_use]
    pub fn offset_seconds(target: &JavaTemporal, default_zone_id: Tz) -> i32 {
        match target {
            JavaTemporal::OffsetDateTime(value) => value.offset().local_minus_utc(),
            JavaTemporal::OffsetTime(_, offset) => offset.local_minus_utc(),
            JavaTemporal::ZonedDateTime(value) => value.offset().fix().local_minus_utc(),
            _ => Utc::now()
                .with_timezone(&default_zone_id)
                .offset()
                .fix()
                .local_minus_utc(),
        }
    }

    fn should_display_year_before_month(locale: &JavaLocale) -> bool {
        matches!(
            locale.get_country().to_string_lossy().as_str(),
            "BT" | "CA" | "CN" | "KP" | "KR" | "TW" | "HU" | "IR" | "JP" | "LT" | "MN"
        )
    }
}

/// Java temporal 规范化或字段访问错误。
#[derive(Debug, Error)]
#[error("{message}")]
pub struct TemporalError {
    message: String,
}

fn invalid(message: impl Into<String>) -> TemporalError {
    TemporalError {
        message: message.into(),
    }
}
