use std::any::Any;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use chrono_tz::Tz;

use crate::expression::TemplateObject;
use crate::util::Utf16String;

/// Java Time API 中具体 `Temporal` 类型的判别值。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
pub enum JavaTemporalKind {
    /// `Instant`。
    Instant,
    /// `LocalDate`。
    LocalDate,
    /// `LocalDateTime`。
    LocalDateTime,
    /// `LocalTime`。
    LocalTime,
    /// `OffsetDateTime`。
    OffsetDateTime,
    /// `OffsetTime`。
    OffsetTime,
    /// `Year`。
    Year,
    /// `YearMonth`。
    YearMonth,
    /// `ZonedDateTime`。
    ZonedDateTime,
}

/// Java `java.time.temporal.Temporal` 的保真 Rust 值适配。
///
/// 这是迁移层对象，用于承载 `Temporals` 所接受的九种 Java Time 类型。
#[derive(Clone, Debug)]
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
pub enum JavaTemporal {
    /// `java.time.Instant`。
    Instant(DateTime<Utc>),
    /// `java.time.LocalDate`。
    LocalDate(NaiveDate),
    /// `java.time.LocalDateTime`。
    LocalDateTime(NaiveDateTime),
    /// `java.time.LocalTime`。
    LocalTime(NaiveTime),
    /// `java.time.OffsetDateTime`。
    OffsetDateTime(DateTime<FixedOffset>),
    /// `java.time.OffsetTime`。
    OffsetTime(NaiveTime, FixedOffset),
    /// `java.time.Year`。
    Year(i32),
    /// `java.time.YearMonth`。
    YearMonth(i32, u32),
    /// `java.time.ZonedDateTime`。
    ZonedDateTime(DateTime<Tz>),
}

impl JavaTemporal {
    /// 返回 Java 具体 temporal 类型。
    #[must_use]
    pub const fn kind(&self) -> JavaTemporalKind {
        match self {
            Self::Instant(_) => JavaTemporalKind::Instant,
            Self::LocalDate(_) => JavaTemporalKind::LocalDate,
            Self::LocalDateTime(_) => JavaTemporalKind::LocalDateTime,
            Self::LocalTime(_) => JavaTemporalKind::LocalTime,
            Self::OffsetDateTime(_) => JavaTemporalKind::OffsetDateTime,
            Self::OffsetTime(_, _) => JavaTemporalKind::OffsetTime,
            Self::Year(_) => JavaTemporalKind::Year,
            Self::YearMonth(_, _) => JavaTemporalKind::YearMonth,
            Self::ZonedDateTime(_) => JavaTemporalKind::ZonedDateTime,
        }
    }

    /// 返回 Jackson Java Time 模块使用的 ISO 文本。
    ///
    /// `LocalDateTime#toString()` 会省略全零秒字段，但 Jackson 的序列化格式保留秒，
    /// 因此该表示与普通 Java `toString()` 分开维护。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn to_javascript_iso_string(&self) -> Utf16String {
        Utf16String::from_rust_str(&match self {
            Self::LocalDateTime(value) => value.format("%Y-%m-%dT%H:%M:%S%.f").to_string(),
            _ => self.to_utf16_string().to_string_lossy(),
        })
    }
}

impl TemplateObject for JavaTemporal {
    fn java_class_name(&self) -> &str {
        match self {
            Self::Instant(_) => "java.time.Instant",
            Self::LocalDate(_) => "java.time.LocalDate",
            Self::LocalDateTime(_) => "java.time.LocalDateTime",
            Self::LocalTime(_) => "java.time.LocalTime",
            Self::OffsetDateTime(_) => "java.time.OffsetDateTime",
            Self::OffsetTime(_, _) => "java.time.OffsetTime",
            Self::Year(_) => "java.time.Year",
            Self::YearMonth(_, _) => "java.time.YearMonth",
            Self::ZonedDateTime(_) => "java.time.ZonedDateTime",
        }
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(&match self {
            Self::Instant(value) => value.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
            Self::LocalDate(value) => value.format("%Y-%m-%d").to_string(),
            Self::LocalDateTime(value) if value.second() == 0 && value.nanosecond() == 0 => {
                value.format("%Y-%m-%dT%H:%M").to_string()
            }
            Self::LocalDateTime(value) => value.format("%Y-%m-%dT%H:%M:%S%.f").to_string(),
            Self::LocalTime(value) => value.format("%H:%M:%S%.f").to_string(),
            Self::OffsetDateTime(value) => {
                value.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, false)
            }
            Self::OffsetTime(value, offset) => {
                format!("{}{}", value.format("%H:%M:%S%.f"), offset)
            }
            Self::Year(value) => value.to_string(),
            Self::YearMonth(year, month) => format!("{year:04}-{month:02}"),
            Self::ZonedDateTime(value) => {
                format!(
                    "{}[{}]",
                    value.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, false),
                    value.timezone()
                )
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
