use chrono::Timelike;
use chrono_tz::Tz;
use thiserror::Error;

use crate::util::{JavaLocale, JavaString};

use super::temporal_creation_utils::java_pattern;
use super::temporal_objects::java_short_zone;
use super::{JavaTemporal, TemporalObjects};

/// Java 8 Time 对象格式化及字段读取工具。
///
/// 对应 Java: `org.thymeleaf.util.temporal.TemporalFormattingUtils`。
pub struct TemporalFormattingUtils {
    locale: JavaLocale,
    default_zone_id: Tz,
}

impl TemporalFormattingUtils {
    /// 使用 Locale 与默认 ZoneId 创建格式化工具。
    pub fn new(locale: JavaLocale, default_zone_id: Tz) -> Result<Self, TemporalFormattingError> {
        Ok(Self {
            locale,
            default_zone_id,
        })
    }

    /// 使用默认格式或指定 Java DateTimeFormatter pattern 格式化 temporal。
    pub fn format(
        &self,
        target: Option<&JavaTemporal>,
        pattern: Option<&str>,
        locale: Option<&JavaLocale>,
        zone_id: Option<Tz>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        let Some(target) = target else {
            return Ok(None);
        };
        if pattern.is_some_and(|pattern| pattern.trim().is_empty()) {
            return Err(invalid("Pattern cannot be null or empty"));
        }
        let locale = locale.unwrap_or(&self.locale);
        let has_explicit_pattern = pattern.is_some();
        let pattern = match pattern {
            None => TemporalObjects::formatter_for(target, locale)?,
            Some("SHORT") => localized_pattern(target, locale, "SHORT", &self.default_zone_id),
            Some("MEDIUM") => localized_pattern(target, locale, "MEDIUM", &self.default_zone_id),
            Some("LONG") => localized_pattern(target, locale, "LONG", &self.default_zone_id),
            Some("FULL") => localized_pattern(target, locale, "FULL", &self.default_zone_id),
            Some(pattern) => java_pattern(pattern),
        };

        let formatted = match target {
            JavaTemporal::Instant(value) if !has_explicit_pattern && pattern.contains('Z') => {
                value.format(&pattern).to_string()
            }
            JavaTemporal::LocalDate(value) if !has_explicit_pattern && zone_id.is_none() => {
                value.format(&pattern).to_string()
            }
            JavaTemporal::LocalDateTime(value) if !has_explicit_pattern && zone_id.is_none() => {
                value.format(&pattern).to_string()
            }
            JavaTemporal::LocalTime(value) if !has_explicit_pattern && zone_id.is_none() => {
                value.format(&pattern).to_string()
            }
            JavaTemporal::OffsetDateTime(value) if !has_explicit_pattern && zone_id.is_none() => {
                // Java formatterFor(OffsetDateTime) = appendLocalized(LONG, MEDIUM)
                // + appendLocalizedOffset(FULL)：偏移段为 "GMT"（零偏移）或
                // "GMT+HH:MM"，由 format() 按目标自身偏移追加。
                format!(
                    "{}{}",
                    value.format(&pattern),
                    java_gmt_offset(value.offset().local_minus_utc())
                )
            }
            JavaTemporal::OffsetTime(value, offset)
                if !has_explicit_pattern && zone_id.is_none() =>
            {
                // Java formatterFor(OffsetTime) = `HH:mm:ss` + appendLocalizedOffset(FULL)。
                format!(
                    "{}{}",
                    value.format(&pattern),
                    java_gmt_offset(offset.local_minus_utc())
                )
            }
            JavaTemporal::Year(value) if !has_explicit_pattern && zone_id.is_none() => {
                format_year(*value, &pattern)
            }
            JavaTemporal::YearMonth(year, month) if !has_explicit_pattern && zone_id.is_none() => {
                let date = chrono::NaiveDate::from_ymd_opt(*year, *month, 1)
                    .ok_or_else(|| invalid("Invalid YearMonth"))?;
                date.format(&pattern).to_string()
            }
            JavaTemporal::ZonedDateTime(value) if !has_explicit_pattern && zone_id.is_none() => {
                value.format(&pattern).to_string()
            }
            // Java: `DateTimeFormatter.ofPattern(pattern).withZone(zoneId)`
            // 对 `zonedTime(target, defaultZoneId)` 按同一瞬时重定区；
            // 显式 zone 时先以默认区建立带区时间，再保持瞬时换算。
            _ => {
                let zoned = TemporalObjects::zoned_time(target, self.default_zone_id)?;
                let converted = match zone_id {
                    Some(zone) => zoned.with_timezone(&zone),
                    None => zoned,
                };
                converted.format(&pattern).to_string()
            }
        };
        let formatted = replace_java_fraction_markers(formatted, target)?;
        // Java CLDR 按 locale 本地化月份/星期名；chrono 只输出英文，德语
        // locale 下替换为 Java `Locale.GERMANY` 的名称。
        let formatted = if locale.get_language().to_string_lossy() == "de" {
            localize_german_names(formatted)
        } else {
            formatted
        };
        Ok(Some(JavaString::from_rust_str(&formatted)))
    }

    /// 返回一个月中的日期。
    pub fn day(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        target
            .map(|target| TemporalObjects::date_fields(target).map(|(_, _, day, _)| day as i32))
            .transpose()
            .map_err(Into::into)
    }

    /// 返回一月为 1 的月份。
    pub fn month(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        target
            .map(|target| TemporalObjects::date_fields(target).map(|(_, month, _, _)| month as i32))
            .transpose()
            .map_err(Into::into)
    }

    /// 返回当前 Locale 下的完整月份名称。
    ///
    /// 对应 Java: `TemporalFormattingUtils#monthName(Object)`。
    pub fn month_name(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        self.format(target, Some("MMMM"), None, None)
    }

    /// 返回当前 Locale 下的短月份名称。
    ///
    /// 对应 Java: `TemporalFormattingUtils#monthNameShort(Object)`。
    pub fn month_name_short(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        self.format(target, Some("MMM"), None, None)
    }

    /// 返回年份。
    pub fn year(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        target
            .map(|target| TemporalObjects::date_fields(target).map(|(year, _, _, _)| year))
            .transpose()
            .map_err(Into::into)
    }

    /// 返回 ISO 周一为 1 的星期编号。
    pub fn day_of_week(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        target
            .map(|target| {
                TemporalObjects::date_fields(target).map(|(_, _, _, weekday)| weekday as i32)
            })
            .transpose()
            .map_err(Into::into)
    }

    /// 返回当前 Locale 下的完整星期名称。
    ///
    /// 对应 Java: `TemporalFormattingUtils#dayOfWeekName(Object)`。
    pub fn day_of_week_name(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        self.format(target, Some("EEEE"), None, None)
    }

    /// 返回当前 Locale 下的短星期名称。
    ///
    /// 对应 Java: `TemporalFormattingUtils#dayOfWeekNameShort(Object)`。
    pub fn day_of_week_name_short(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        self.format(target, Some("EEE"), None, None)
    }

    /// 返回小时。
    pub fn hour(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        self.time_field(target, 0)
    }

    /// 返回分钟。
    pub fn minute(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        self.time_field(target, 1)
    }

    /// 返回秒。
    pub fn second(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        self.time_field(target, 2)
    }

    /// 返回纳秒。
    pub fn nanosecond(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        self.time_field(target, 3)
    }

    /// 按 Thymeleaf 固定 ISO pattern 格式化并补齐缺失字段。
    ///
    /// 对应 Java `TemporalFormattingUtils#formatISO`：
    /// `DateTimeFormatter.ofPattern("yyyy-MM-dd'T'HH:mm:ss.SSSZZZ")` —— 偏移使用
    /// `ZZZ` 的 `+HHmm` 无冒号形态；带偏移的类型（OffsetDateTime/OffsetTime/
    /// ZonedDateTime）保留原始偏移，其余在默认 ZoneId 下组装。
    pub fn format_iso(
        &self,
        target: Option<&JavaTemporal>,
    ) -> Result<Option<JavaString>, TemporalFormattingError> {
        let Some(target) = target else {
            return Ok(None);
        };
        let formatted = match target {
            JavaTemporal::OffsetDateTime(value) => {
                value.format("%Y-%m-%dT%H:%M:%S%.3f%z").to_string()
            }
            // OffsetTime 与其余类型一致走 `zonedTime`：Java 在默认 ZoneId 下组装
            // （`ZonedDateTime.of(LocalDate.now(), localTime, defaultZoneId)`），
            // 偏移本身不参与。
            JavaTemporal::ZonedDateTime(value) => {
                value.format("%Y-%m-%dT%H:%M:%S%.3f%z").to_string()
            }
            _ => TemporalObjects::zoned_time(target, self.default_zone_id)?
                .format("%Y-%m-%dT%H:%M:%S%.3f%z")
                .to_string(),
        };
        Ok(Some(JavaString::from_rust_str(&formatted)))
    }

    fn time_field(
        &self,
        target: Option<&JavaTemporal>,
        index: usize,
    ) -> Result<Option<i32>, TemporalFormattingError> {
        target
            .map(|target| {
                let fields = TemporalObjects::time_fields(target)?;
                Ok([fields.0, fields.1, fields.2, fields.3][index] as i32)
            })
            .transpose()
    }
}

fn replace_java_fraction_markers(
    mut formatted: String,
    target: &JavaTemporal,
) -> Result<String, TemporalFormattingError> {
    let nanosecond = match target {
        JavaTemporal::Instant(value) => value.nanosecond(),
        JavaTemporal::LocalDate(_) | JavaTemporal::Year(_) | JavaTemporal::YearMonth(_, _) => 0,
        JavaTemporal::LocalDateTime(value) => value.nanosecond(),
        JavaTemporal::LocalTime(value) | JavaTemporal::OffsetTime(value, _) => value.nanosecond(),
        JavaTemporal::OffsetDateTime(value) => value.nanosecond(),
        JavaTemporal::ZonedDateTime(value) => value.nanosecond(),
    };
    const PREFIX: &str = "__THYMELEAF_FRACTION_";
    const SUFFIX: &str = "__";
    while let Some(start) = formatted.find(PREFIX) {
        let marker_end = formatted[start + PREFIX.len()..]
            .find(SUFFIX)
            .map(|offset| start + PREFIX.len() + offset)
            .ok_or_else(|| invalid("Invalid Java fraction marker"))?;
        let count = formatted[start + PREFIX.len()..marker_end]
            .parse::<usize>()
            .map_err(|_| invalid("Invalid Java fraction width"))?;
        if !(1..=9).contains(&count) {
            return Err(invalid("Fraction width must be between 1 and 9"));
        }
        let digits = format!("{nanosecond:09}");
        formatted.replace_range(start..marker_end + SUFFIX.len(), &digits[..count]);
    }
    Ok(formatted)
}

/// Temporal 格式化或字段读取错误。
#[derive(Debug, Error)]
#[error("{message}")]
pub struct TemporalFormattingError {
    message: String,
}

impl From<super::temporal_objects::TemporalError> for TemporalFormattingError {
    fn from(error: super::temporal_objects::TemporalError) -> Self {
        invalid(error.to_string())
    }
}

fn invalid(message: impl Into<String>) -> TemporalFormattingError {
    TemporalFormattingError {
        message: message.into(),
    }
}

fn localized_pattern(
    target: &JavaTemporal,
    locale: &JavaLocale,
    style: &str,
    default_zone: &Tz,
) -> String {
    let date_only = matches!(target, JavaTemporal::LocalDate(_));
    let time_only = matches!(
        target,
        JavaTemporal::LocalTime(_) | JavaTemporal::OffsetTime(_, _)
    );
    let language = locale.get_language().to_string_lossy();
    let zh = language == "zh";
    // Java CLDR（DateTimeFormatter.ofLocalizedDate/ofLocalizedDateTime）：
    // 德语日期样式为 `dd.MM.yy` / `dd.MM.y` / `d. MMMM y` /
    // `EEEE, d. MMMM y`（chrono 等价 %d.%m.%y 等）。
    let de = language == "de";
    // Java `z`（LONG/FULL 的通用时区名）：ZoneOffset.UTC → "Z"；固定偏移 →
    // "+18:00"；命名时区 → 缩写（EST/PST 等）。
    let zone = java_short_zone(target, default_zone);
    match (date_only, time_only, zh, de, style) {
        // ---- 日期（Java DateTimeFormatter.ofLocalizedDate）----
        (true, _, true, _, "FULL") => "%Y年%m月%d日 %A".to_owned(),
        (true, _, true, _, _) => "%Y年%m月%d日".to_owned(),
        (true, _, false, true, "SHORT") => "%d.%m.%y".to_owned(),
        (true, _, false, true, "MEDIUM") => "%d.%m.%Y".to_owned(),
        (true, _, false, true, "FULL") => "%A, %-d. %B %Y".to_owned(),
        (true, _, false, true, _) => "%-d. %B %Y".to_owned(),
        (true, _, false, false, "SHORT") => "%-m/%-d/%y".to_owned(),
        (true, _, false, false, "MEDIUM") => "%b %-d, %Y".to_owned(),
        (true, _, false, false, "FULL") => "%A, %B %-d, %Y".to_owned(),
        (true, _, false, false, _) => "%B %-d, %Y".to_owned(),
        // ---- 时间（Java DateTimeFormatter.ofLocalizedTime）----
        // zh/de 为 24 小时制；zh 的 LONG/FULL 时区名在时间之前（`z HH:mm:ss`）。
        (_, true, true, _, "SHORT") | (_, true, true, _, "MEDIUM") => "%H:%M:%S".to_owned(),
        (_, true, true, _, _) => format!("{zone} %H:%M:%S"),
        (_, true, false, true, "SHORT") => "%H:%M".to_owned(),
        (_, true, false, true, "MEDIUM") => "%H:%M:%S".to_owned(),
        (_, true, false, true, _) => format!("%H:%M:%S {zone}"),
        (_, true, false, false, "SHORT") => "%-I:%M %p".to_owned(),
        (_, true, false, false, "MEDIUM") => "%-I:%M:%S %p".to_owned(),
        (_, true, false, false, _) => format!("%-I:%M:%S %p {zone}"),
        // ---- 日期时间（Java DateTimeFormatter.ofLocalizedDateTime）----
        // zh SHORT 为斜杠分隔，LONG/FULL 时区名位于日期与时间之间。
        (_, _, true, _, "SHORT") => "%Y/%m/%d %H:%M".to_owned(),
        (_, _, true, _, "MEDIUM") => "%Y年%m月%d日 %H:%M:%S".to_owned(),
        (_, _, true, _, _) => format!("%Y年%m月%d日 {zone} %H:%M:%S"),
        (_, _, false, true, "SHORT") => "%d.%m.%y, %H:%M".to_owned(),
        (_, _, false, true, "MEDIUM") => "%d.%m.%Y, %H:%M:%S".to_owned(),
        (_, _, false, true, "FULL") => format!("%A, %-d. %B %Y, %H:%M:%S {zone}"),
        (_, _, false, true, _) => format!("%-d. %B %Y, %H:%M:%S {zone}"),
        (_, _, false, false, "SHORT") => "%-m/%-d/%y, %-I:%M %p".to_owned(),
        (_, _, false, false, "MEDIUM") => "%b %-d, %Y, %-I:%M:%S %p".to_owned(),
        // Java en_US CLDR（JDK 9+）LONG/FULL datetime：`MMMM d, y, h:mm:ss a z`。
        (_, _, false, false, "LONG") => format!("%B %-d, %Y, %-I:%M:%S %p {zone}"),
        (_, _, false, false, "FULL") => format!("%A, %B %-d, %Y, %-I:%M:%S %p {zone}"),
        _ => "%B %-d, %Y, %-I:%M:%S %p".to_owned(),
    }
}

/// 把英文月份/星期名替换为德语名（Java CLDR `Locale.GERMANY`）。
fn localize_german_names(formatted: String) -> String {
    const MONTHS: [(&str, &str); 12] = [
        ("January", "Januar"),
        ("February", "Februar"),
        ("March", "März"),
        ("April", "April"),
        ("May", "Mai"),
        ("June", "Juni"),
        ("July", "Juli"),
        ("August", "August"),
        ("September", "September"),
        ("October", "Oktober"),
        ("November", "November"),
        ("December", "Dezember"),
    ];
    const DAYS: [(&str, &str); 7] = [
        ("Monday", "Montag"),
        ("Tuesday", "Dienstag"),
        ("Wednesday", "Mittwoch"),
        ("Thursday", "Donnerstag"),
        ("Friday", "Freitag"),
        ("Saturday", "Samstag"),
        ("Sunday", "Sonntag"),
    ];
    let mut out = formatted;
    for (english, german) in MONTHS {
        out = out.replace(english, german);
    }
    for (english, german) in DAYS {
        out = out.replace(english, german);
    }
    out
}

fn format_year(year: i32, pattern: &str) -> String {
    pattern
        .replace("%Y", &format!("{year:04}"))
        .replace("%y", &format!("{:02}", year.rem_euclid(100)))
}

/// Java `appendLocalizedOffset(TextStyle.FULL)`：零偏移 → "GMT"，否则 "GMT+HH:MM"。
fn java_gmt_offset(seconds: i32) -> String {
    if seconds == 0 {
        "GMT".to_owned()
    } else {
        let sign = if seconds < 0 { '-' } else { '+' };
        let seconds = seconds.unsigned_abs();
        format!("GMT{sign}{:02}:{:02}", seconds / 3600, seconds % 3600 / 60)
    }
}
