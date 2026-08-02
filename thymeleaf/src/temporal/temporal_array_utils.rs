use chrono_tz::Tz;

use crate::util::{JavaLocale, JavaString};

use super::temporal_formatting_utils::TemporalFormattingError;
use super::{JavaTemporal, TemporalFormattingUtils};

/// Java temporal 数组批量格式化工具。
///
/// 对应 Java: `org.thymeleaf.util.temporal.TemporalArrayUtils`。
pub struct TemporalArrayUtils {
    formatting: TemporalFormattingUtils,
}

impl TemporalArrayUtils {
    /// 使用 Locale 与默认 ZoneId 创建数组工具。
    /// 对应 Java 语义：`TemporalArrayUtils` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(locale: JavaLocale, default_zone_id: Tz) -> Result<Self, TemporalFormattingError> {
        Ok(Self {
            formatting: TemporalFormattingUtils::new(locale, default_zone_id)?,
        })
    }

    /// 批量格式化。
    /// 对应 Java: `TemporalArrayUtils#arrayFormat()`。
    pub fn array_format(
        &self,
        target: &[Option<JavaTemporal>],
        pattern: Option<&str>,
        locale: Option<&JavaLocale>,
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        target
            .iter()
            .map(|value| {
                self.formatting
                    .format(value.as_ref(), pattern, locale, None)
            })
            .collect()
    }

    /// 批量读取日。
    /// 对应 Java: `TemporalArrayUtils#arrayDay()`。
    pub fn array_day(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.map_integer(target, |utils, value| utils.day(value))
    }

    /// 批量读取月。
    /// 对应 Java: `TemporalArrayUtils#arrayMonth()`。
    pub fn array_month(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.map_integer(target, |utils, value| utils.month(value))
    }

    /// 批量读取年份。
    /// 对应 Java: `TemporalArrayUtils#arrayYear()`。
    pub fn array_year(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.map_integer(target, |utils, value| utils.year(value))
    }

    /// 批量读取星期。
    /// 对应 Java: `TemporalArrayUtils#arrayDayOfWeek()`。
    pub fn array_day_of_week(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.map_integer(target, |utils, value| utils.day_of_week(value))
    }

    /// 批量读取小时。
    /// 对应 Java: `TemporalArrayUtils#arrayHour()`。
    pub fn array_hour(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.map_integer(target, |utils, value| utils.hour(value))
    }

    /// 批量读取分钟。
    /// 对应 Java: `TemporalArrayUtils#arrayMinute()`。
    pub fn array_minute(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.map_integer(target, |utils, value| utils.minute(value))
    }

    /// 批量读取秒。
    /// 对应 Java: `TemporalArrayUtils#arraySecond()`。
    pub fn array_second(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.map_integer(target, |utils, value| utils.second(value))
    }

    /// 批量读取纳秒。
    /// 对应 Java: `TemporalArrayUtils#arrayNanosecond()`。
    pub fn array_nanosecond(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.map_integer(target, |utils, value| utils.nanosecond(value))
    }

    /// 批量读取完整月份名。
    /// 对应 Java: `TemporalArrayUtils#arrayMonthName()`。
    pub fn array_month_name(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        self.array_format(target, Some("MMMM"), None)
    }

    /// 批量读取短月份名。
    /// 对应 Java: `TemporalArrayUtils#arrayMonthNameShort()`。
    pub fn array_month_name_short(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        self.array_format(target, Some("MMM"), None)
    }

    /// 批量读取完整星期名。
    /// 对应 Java: `TemporalArrayUtils#arrayDayOfWeekName()`。
    pub fn array_day_of_week_name(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        self.array_format(target, Some("EEEE"), None)
    }

    /// 批量读取短星期名。
    /// 对应 Java: `TemporalArrayUtils#arrayDayOfWeekNameShort()`。
    pub fn array_day_of_week_name_short(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        self.array_format(target, Some("EEE"), None)
    }

    /// 批量输出 ISO 格式。
    /// 对应 Java 语义：`TemporalArrayUtils` 的 `array_format_iso` 行为（Rust 侧辅助/私有路径）。
    pub fn array_format_iso(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        target
            .iter()
            .map(|value| self.formatting.format_iso(value.as_ref()))
            .collect()
    }

    fn map_integer(
        &self,
        target: &[Option<JavaTemporal>],
        mapper: impl Fn(
            &TemporalFormattingUtils,
            Option<&JavaTemporal>,
        ) -> Result<Option<i32>, TemporalFormattingError>,
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        target
            .iter()
            .map(|value| mapper(&self.formatting, value.as_ref()))
            .collect()
    }
}
