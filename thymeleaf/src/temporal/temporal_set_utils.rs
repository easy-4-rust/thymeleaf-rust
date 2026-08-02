use chrono_tz::Tz;

use crate::util::{JavaLocale, JavaString};

use super::temporal_formatting_utils::TemporalFormattingError;
use super::{JavaTemporal, TemporalArrayUtils};

/// Java temporal `Set` 批量格式化工具。
///
/// 对应 Java: `org.thymeleaf.util.temporal.TemporalSetUtils`。
pub struct TemporalSetUtils {
    array_utils: TemporalArrayUtils,
}

impl TemporalSetUtils {
    /// 使用 Locale 与默认 ZoneId 创建 Set 工具。
    pub fn new(locale: JavaLocale, default_zone_id: Tz) -> Result<Self, TemporalFormattingError> {
        Ok(Self {
            array_utils: TemporalArrayUtils::new(locale, default_zone_id)?,
        })
    }

    /// 批量格式化，并按 Java `LinkedHashSet` 语义去重。
    pub fn set_format(
        &self,
        target: &[Option<JavaTemporal>],
        pattern: Option<&str>,
        locale: Option<&JavaLocale>,
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        Ok(dedupe(
            self.array_utils.array_format(target, pattern, locale)?,
        ))
    }

    /// 批量读取日并去重。
    pub fn set_day(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_day(target)?))
    }
    /// 批量读取月并去重。
    pub fn set_month(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_month(target)?))
    }
    /// 批量读取年份并去重。
    pub fn set_year(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_year(target)?))
    }
    /// 批量读取星期并去重。
    pub fn set_day_of_week(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_day_of_week(target)?))
    }
    /// 批量读取小时并去重。
    pub fn set_hour(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_hour(target)?))
    }
    /// 批量读取分钟并去重。
    pub fn set_minute(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_minute(target)?))
    }
    /// 批量读取秒并去重。
    pub fn set_second(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_second(target)?))
    }
    /// 批量读取纳秒并去重。
    pub fn set_nanosecond(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<i32>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_nanosecond(target)?))
    }
    /// 批量读取完整月份名并去重。
    pub fn set_month_name(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_month_name(target)?))
    }
    /// 批量读取短月份名并去重。
    pub fn set_month_name_short(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_month_name_short(target)?))
    }
    /// 批量读取完整星期名并去重。
    pub fn set_day_of_week_name(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_day_of_week_name(target)?))
    }
    /// 批量读取短星期名并去重。
    pub fn set_day_of_week_name_short(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        Ok(dedupe(
            self.array_utils.array_day_of_week_name_short(target)?,
        ))
    }
    /// 批量输出 ISO 格式并去重。
    pub fn set_format_iso(
        &self,
        target: &[Option<JavaTemporal>],
    ) -> Result<Vec<Option<JavaString>>, TemporalFormattingError> {
        Ok(dedupe(self.array_utils.array_format_iso(target)?))
    }
}

fn dedupe<T: PartialEq>(values: Vec<Option<T>>) -> Vec<Option<T>> {
    let mut output = Vec::with_capacity(values.len());
    for value in values {
        if !output.contains(&value) {
            output.push(value);
        }
    }
    output
}
