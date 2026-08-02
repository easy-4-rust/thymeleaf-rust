use chrono_tz::Tz;

use crate::util::{JavaLocale, JavaString};

use super::temporal_formatting_utils::TemporalFormattingError;
use super::{JavaTemporal, TemporalArrayUtils};

/// Java temporal `List` 批量格式化工具。
///
/// 对应 Java: `org.thymeleaf.util.temporal.TemporalListUtils`。
pub struct TemporalListUtils {
    array_utils: TemporalArrayUtils,
}

impl TemporalListUtils {
    /// 使用 Locale 与默认 ZoneId 创建 List 工具。
    pub fn new(locale: JavaLocale, default_zone_id: Tz) -> Result<Self, TemporalFormattingError> {
        Ok(Self {
            array_utils: TemporalArrayUtils::new(locale, default_zone_id)?,
        })
    }

    /// 批量格式化，保持输入顺序和空值。
    pub fn list_format(
        &self,
        target: &[Option<JavaTemporal>],
        pattern: Option<&str>,
        locale: Option<&JavaLocale>,
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        self.array_utils.array_format(target, pattern, locale)
    }

    /// 批量读取日。
    pub fn list_day(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.array_utils.array_day(target)
    }
    /// 批量读取月。
    pub fn list_month(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.array_utils.array_month(target)
    }
    /// 批量读取年份。
    pub fn list_year(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.array_utils.array_year(target)
    }
    /// 批量读取星期。
    pub fn list_day_of_week(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.array_utils.array_day_of_week(target)
    }
    /// 批量读取小时。
    pub fn list_hour(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.array_utils.array_hour(target)
    }
    /// 批量读取分钟。
    pub fn list_minute(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.array_utils.array_minute(target)
    }
    /// 批量读取秒。
    pub fn list_second(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.array_utils.array_second(target)
    }
    /// 批量读取纳秒。
    pub fn list_nanosecond(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        self.array_utils.array_nanosecond(target)
    }
    /// 批量读取完整月份名。
    pub fn list_month_name(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        self.array_utils.array_month_name(target)
    }
    /// 批量读取短月份名。
    pub fn list_month_name_short(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        self.array_utils.array_month_name_short(target)
    }
    /// 批量读取完整星期名。
    pub fn list_day_of_week_name(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        self.array_utils.array_day_of_week_name(target)
    }
    /// 批量读取短星期名。
    pub fn list_day_of_week_name_short(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        self.array_utils.array_day_of_week_name_short(target)
    }
    /// 批量输出 ISO 格式。
    pub fn list_format_iso(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        self.array_utils.array_format_iso(target)
    }
}
