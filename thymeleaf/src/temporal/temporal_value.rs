use std::any::Any;

use jiff::civil::{Date, DateTime, Time};
use jiff::tz::Offset;
use jiff::{Timestamp, ZonedDateTime};

use crate::expression::TemplateObject;
use crate::util::Utf16String;

/// Java Time API 中具体 `Temporal` 类型的判别值。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
pub enum TemporalKind {
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
pub enum TemporalValue {
    /// `java.time.Instant`。
    Instant(Timestamp),
    /// `java.time.LocalDate`。
    LocalDate(Date),
    /// `java.time.LocalDateTime`。
    LocalDateTime(DateTime),
    /// `java.time.LocalTime`。
    LocalTime(Time),
    /// `java.time.OffsetDateTime`。
    OffsetDateTime(ZonedDateTime),
    /// `java.time.OffsetTime`。
    OffsetTime(Time, Offset),
    /// `java.time.Year`。
    Year(i32),
    /// `java.time.YearMonth`。
    YearMonth(i32, u32),
    /// `java.time.ZonedDateTime`。
    ZonedDateTime(ZonedDateTime),
}

impl TemporalValue {
    /// 返回 Java 具体 temporal 类型。
    #[must_use]
    pub const fn kind(&self) -> TemporalKind {
        match self {
            Self::Instant(_) => TemporalKind::Instant,
            Self::LocalDate(_) => TemporalKind::LocalDate,
            Self::LocalDateTime(_) => TemporalKind::LocalDateTime,
            Self::LocalTime(_) => TemporalKind::LocalTime,
            Self::OffsetDateTime(_) => TemporalKind::OffsetDateTime,
            Self::OffsetTime(_, _) => TemporalKind::OffsetTime,
            Self::Year(_) => TemporalKind::Year,
            Self::YearMonth(_, _) => TemporalKind::YearMonth,
            Self::ZonedDateTime(_) => TemporalKind::ZonedDateTime,
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
            Self::LocalDateTime(value) => value
                .strftime(&iso_pattern(value.subsec_nanosecond(), ""))
                .to_string(),
            _ => self.to_utf16_string().to_string_lossy(),
        })
    }
}

impl TemplateObject for TemporalValue {
    fn class_name(&self) -> &str {
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
            Self::Instant(value) => value
                .strftime(&iso_pattern(value.subsec_nanosecond(), "Z"))
                .to_string(),
            Self::LocalDate(value) => value.strftime("%Y-%m-%d").to_string(),
            Self::LocalDateTime(value) if value.second() == 0 && value.subsec_nanosecond() == 0 => {
                value.strftime("%Y-%m-%dT%H:%M").to_string()
            }
            Self::LocalDateTime(value) => value
                .strftime(&iso_pattern(value.subsec_nanosecond(), ""))
                .to_string(),
            Self::LocalTime(value) => value
                .strftime(time_pattern(value.subsec_nanosecond()))
                .to_string(),
            Self::OffsetDateTime(value) => value
                .strftime(&iso_pattern(value.subsec_nanosecond(), "%:z"))
                .to_string(),
            Self::OffsetTime(value, offset) => {
                let time = value.strftime(time_pattern(value.subsec_nanosecond()));
                if offset.seconds() % 3_600 == 0 {
                    format!("{time}{offset}:00")
                } else {
                    format!("{time}{offset}")
                }
            }
            Self::Year(value) => value.to_string(),
            Self::YearMonth(year, month) => format!("{year:04}-{month:02}"),
            Self::ZonedDateTime(value) => {
                format!(
                    "{}[{}]",
                    value.strftime(&iso_pattern(value.subsec_nanosecond(), "%:z")),
                    value.time_zone().iana_name().unwrap_or("UTC")
                )
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 保留 Java 兼容的 ISO 秒与 3、6、9 位小数输出。
fn iso_pattern(nanos: i32, suffix: &str) -> String {
    format!("%Y-%m-%dT{}{suffix}", time_pattern(nanos))
}

fn time_pattern(nanos: i32) -> &'static str {
    match nanos {
        0 => "%H:%M:%S",
        value if value % 1_000_000 == 0 => "%H:%M:%S%.3f",
        value if value % 1_000 == 0 => "%H:%M:%S%.6f",
        _ => "%H:%M:%S%.9f",
    }
}
