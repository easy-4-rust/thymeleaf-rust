use std::any::Any;
use std::sync::Arc;

use chrono::{
    DateTime, Datelike, Duration, FixedOffset, LocalResult, NaiveDate, NaiveDateTime, Offset,
    TimeZone, Timelike, Utc, Weekday,
};
use chrono_tz::Tz;
use thiserror::Error;

use crate::expression::{TemplateObject, TemplateValue};

use super::{Locale, NumberValue, Utf16String};

/// Java `Date`/`Calendar` 的 Rust 时间值适配。
/// 对应 Java 语义：`DateUtils` 的 Rust 侧类型 `DateValue`。
#[derive(Clone, Debug)]
pub struct DateValue {
    instant: DateTime<Utc>,
    time_zone: Option<Tz>,
    fixed_offset: Option<FixedOffset>,
    zone_display_name: Option<String>,
    calendar: bool,
}

impl DateValue {
    /// 从 UTC 瞬时创建 Java `Date` 等价值。
    #[must_use]
    pub const fn date(instant: DateTime<Utc>) -> Self {
        Self {
            instant,
            time_zone: None,
            fixed_offset: None,
            zone_display_name: None,
            calendar: false,
        }
    }

    /// 从瞬时和时区创建 Java `Calendar` 等价值。
    #[must_use]
    pub const fn calendar(instant: DateTime<Utc>, time_zone: Tz) -> Self {
        Self {
            instant,
            time_zone: Some(time_zone),
            fixed_offset: None,
            zone_display_name: None,
            calendar: true,
        }
    }

    fn calendar_for_time_zone(instant: DateTime<Utc>, time_zone: TimeZoneValue) -> Self {
        match time_zone {
            TimeZoneValue::Named(time_zone, zone_display_name) => Self {
                instant,
                time_zone: Some(time_zone),
                fixed_offset: None,
                zone_display_name,
                calendar: true,
            },
            TimeZoneValue::Fixed(fixed_offset) => Self {
                instant,
                time_zone: None,
                fixed_offset: Some(fixed_offset),
                zone_display_name: None,
                calendar: true,
            },
        }
    }

    /// 返回自 Unix epoch 起毫秒数。
    /// 对应 Java 语义：`DateUtils` 的 `time_in_millis` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn time_in_millis(&self) -> i64 {
        self.instant.timestamp_millis()
    }

    /// 判断对象是否保留 Calendar 时区。
    #[must_use]
    pub const fn is_calendar(&self) -> bool {
        self.calendar
    }

    /// 把 Calendar 转成只保留瞬时的 Date。
    #[must_use]
    pub const fn to_date(&self) -> Self {
        Self::date(self.instant)
    }
}

impl TemplateObject for DateValue {
    fn class_name(&self) -> &str {
        if self.calendar {
            "java.util.GregorianCalendar"
        } else {
            "java.util.Date"
        }
    }

    fn to_utf16_string(&self) -> Utf16String {
        let local = self.local_date_time();
        Utf16String::from_rust_str(&format!(
            "{} {} {:02} {:02}:{:02}:{:02} {} {}",
            local.format("%a"),
            local.format("%b"),
            local.day(),
            local.hour(),
            local.minute(),
            local.second(),
            self.zone_display_name(),
            local.year()
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<Result<Option<Arc<TemplateValue>>, crate::expression::TemplateObjectPropertyError>>
    {
        (property_name.to_string_lossy() == "time").then(|| {
            Ok(Some(Arc::new(TemplateValue::Number(NumberValue::Long(
                self.time_in_millis(),
            )))))
        })
    }

    fn invoke_method(
        &self,
        method_name: &Utf16String,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, crate::expression::TemplateObjectMethodError>>
    {
        match (method_name.to_string_lossy().as_str(), arguments) {
            ("getTime", []) => Some(Ok(Some(Arc::new(TemplateValue::Number(
                NumberValue::Long(self.time_in_millis()),
            ))))),
            ("get", [Some(field)]) if self.calendar => {
                let field = template_number_as_i32(field.as_ref())?;
                let value = match field {
                    1 => DateUtils::year(Some(self)),
                    2 => DateUtils::month(Some(self)).map(|month| month - 1),
                    5 => DateUtils::day(Some(self)),
                    11 => DateUtils::hour(Some(self)),
                    12 => DateUtils::minute(Some(self)),
                    13 => DateUtils::second(Some(self)),
                    14 => DateUtils::millisecond(Some(self)),
                    _ => return None,
                }?;
                Some(Ok(Some(Arc::new(TemplateValue::Number(
                    NumberValue::Integer(value),
                )))))
            }
            _ => None,
        }
    }
}

fn template_number_as_i32(value: &TemplateValue) -> Option<i32> {
    match value {
        TemplateValue::Number(NumberValue::Byte(value)) => Some(i32::from(*value)),
        TemplateValue::Number(NumberValue::Short(value)) => Some(i32::from(*value)),
        TemplateValue::Number(NumberValue::Integer(value)) => Some(*value),
        TemplateValue::Number(NumberValue::Long(value)) => i32::try_from(*value).ok(),
        _ => None,
    }
}

impl DateValue {
    fn effective_time_zone(&self) -> TimeZoneValue {
        if let Some(fixed_offset) = self.fixed_offset {
            TimeZoneValue::Fixed(fixed_offset)
        } else {
            TimeZoneValue::Named(self.time_zone.unwrap_or_else(default_time_zone), None)
        }
    }

    fn local_date_time(&self) -> NaiveDateTime {
        match self.effective_time_zone() {
            TimeZoneValue::Named(time_zone, _) => {
                self.instant.with_timezone(&time_zone).naive_local()
            }
            TimeZoneValue::Fixed(fixed_offset) => {
                self.instant.with_timezone(&fixed_offset).naive_local()
            }
        }
    }

    fn offset_seconds(&self) -> i32 {
        match self.effective_time_zone() {
            TimeZoneValue::Named(time_zone, _) => self
                .instant
                .with_timezone(&time_zone)
                .offset()
                .fix()
                .local_minus_utc(),
            TimeZoneValue::Fixed(fixed_offset) => fixed_offset.local_minus_utc(),
        }
    }

    fn zone_display_name(&self) -> String {
        if let Some(zone_display_name) = &self.zone_display_name {
            return zone_display_name.clone();
        }
        match self.effective_time_zone() {
            TimeZoneValue::Named(time_zone, _) => self
                .instant
                .with_timezone(&time_zone)
                .format("%Z")
                .to_string(),
            TimeZoneValue::Fixed(fixed_offset) => format_gmt_offset(fixed_offset.local_minus_utc()),
        }
    }
}

#[derive(Clone, Debug)]
enum TimeZoneValue {
    Named(Tz, Option<String>),
    Fixed(FixedOffset),
}

/// 日期创建、归一化或格式化错误。
/// 对应 Java 语义：`DateUtils` 的 Rust 侧类型 `DateUtilsError`。
#[derive(Debug, Error)]
pub enum DateUtilsError {
    /// Java 参数或目标类型不符合约束。
    #[error("{message}")]
    InvalidArgument {
        /// Java 异常消息。
        message: String,
    },
}

/// Thymeleaf `Date`/`Calendar` 创建、字段读取和格式化工具。
///
/// 对应 Java: `org.thymeleaf.util.DateUtils`。
pub struct DateUtils;

impl DateUtils {
    /// 按 Calendar lenient 字段规则创建带时区日历。
    /// 对应 Java: `DateUtils#create()`。
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        year: Option<i32>,
        month: Option<i32>,
        day: Option<i32>,
        hour: Option<i32>,
        minute: Option<i32>,
        second: Option<i32>,
        millisecond: Option<i32>,
        time_zone: Option<&str>,
        _locale: Option<&Locale>,
    ) -> Result<DateValue, DateUtilsError> {
        let (Some(year), Some(month), Some(day)) = (year, month, day) else {
            return Err(invalid(format!(
                "Cannot create Calendar/Date object with null year ({}), month ({}) or day ({})",
                nullable_i32(year),
                nullable_i32(month),
                nullable_i32(day)
            )));
        };
        match (hour, minute, second, millisecond) {
            (Some(_), None, _, _) | (None, Some(_), _, _) => {
                return Err(invalid(format!(
                    "Calendar/Date object can only be correctly created if hour ({}) and minute ({}) are either both null or non-null.",
                    nullable_i32(hour),
                    nullable_i32(minute)
                )));
            }
            (None, None, Some(_), _) | (None, None, _, Some(_)) => {
                return Err(invalid(
                    "Calendar/Date object cannot be correctly created from a null hour and minute but non-null second and/or millisecond.",
                ));
            }
            (Some(_), Some(_), None, Some(_)) => {
                return Err(invalid(
                    "Calendar/Date object cannot be correctly created from a null second but non-null millisecond.",
                ));
            }
            _ => {}
        }

        let total_months = i64::from(year) * 12 + i64::from(month) - 1;
        let normalized_year = total_months.div_euclid(12);
        let normalized_month = total_months.rem_euclid(12) + 1;
        let normalized_year = i32::try_from(normalized_year)
            .map_err(|_| invalid("Calendar year is outside supported range"))?;
        let date = NaiveDate::from_ymd_opt(normalized_year, normalized_month as u32, 1)
            .ok_or_else(|| invalid("Calendar date is outside supported range"))?
            + Duration::days(i64::from(day) - 1);
        let naive = date
            .and_hms_milli_opt(0, 0, 0, 0)
            .expect("midnight is valid")
            + Duration::hours(i64::from(hour.unwrap_or(0)))
            + Duration::minutes(i64::from(minute.unwrap_or(0)))
            + Duration::seconds(i64::from(second.unwrap_or(0)))
            + Duration::milliseconds(i64::from(millisecond.unwrap_or(0)));
        let time_zone = parse_time_zone(time_zone);
        Ok(DateValue::calendar_for_time_zone(
            resolve_local_datetime(&time_zone, naive),
            time_zone,
        ))
    }

    /// 返回指定时区的当前 Calendar。
    /// 对应 Java: `DateUtils#createNow()`。
    #[must_use]
    pub fn create_now(time_zone: Option<&str>, _locale: Option<&Locale>) -> DateValue {
        DateValue::calendar_for_time_zone(Utc::now(), parse_time_zone(time_zone))
    }

    /// 返回指定时区当天零点 Calendar。
    /// 对应 Java: `DateUtils#createToday()`。
    #[must_use]
    pub fn create_today(time_zone: Option<&str>, locale: Option<&Locale>) -> DateValue {
        let now = Self::create_now(time_zone, locale);
        let midnight = now
            .local_date_time()
            .date()
            .and_hms_opt(0, 0, 0)
            .expect("midnight");
        let time_zone = now.effective_time_zone();
        DateValue::calendar_for_time_zone(resolve_local_datetime(&time_zone, midnight), time_zone)
    }

    /// 使用 Locale 默认长日期时间格式或指定 SimpleDateFormat pattern 格式化。
    /// 对应 Java: `DateUtils#format()`。
    pub fn format(
        target: Option<&DateValue>,
        pattern: Option<&Utf16String>,
        locale: Option<&Locale>,
    ) -> Result<Option<Utf16String>, DateUtilsError> {
        let Some(target) = target else {
            return Ok(None);
        };
        let locale = locale.ok_or_else(|| invalid("Locale cannot be null"))?;
        let key = DateFormatKey::new(target, pattern, locale);
        if key.format.trim().is_empty() {
            return Err(invalid("Pattern cannot be null or empty"));
        }
        Ok(Some(format_java_pattern(
            &target.local_date_time(),
            target.offset_seconds(),
            &target.zone_display_name(),
            &key.format,
            &key.locale,
        )?))
    }

    /// 返回一个月中的日期。
    /// 对应 Java: `DateUtils#day()`。
    #[must_use]
    pub fn day(target: Option<&DateValue>) -> Option<i32> {
        target.map(|target| target.local_date_time().day() as i32)
    }

    /// 返回一月为 1 的月份。
    /// 对应 Java: `DateUtils#month()`。
    #[must_use]
    pub fn month(target: Option<&DateValue>) -> Option<i32> {
        target.map(|target| target.local_date_time().month() as i32)
    }

    /// 返回 Locale 对应的完整月份名称。
    ///
    /// 对应 Java: `DateUtils#monthName(Object,Locale)`。
    pub fn month_name(
        target: Option<&DateValue>,
        locale: Option<&Locale>,
    ) -> Result<Option<Utf16String>, DateUtilsError> {
        Self::format(target, Some(&Utf16String::from_rust_str("MMMM")), locale)
    }

    /// 返回 Locale 对应的月份缩写。
    ///
    /// 对应 Java: `DateUtils#monthNameShort(Object,Locale)`。
    pub fn month_name_short(
        target: Option<&DateValue>,
        locale: Option<&Locale>,
    ) -> Result<Option<Utf16String>, DateUtilsError> {
        Self::format(target, Some(&Utf16String::from_rust_str("MMM")), locale)
    }

    /// 返回年份。
    /// 对应 Java: `DateUtils#year()`。
    #[must_use]
    pub fn year(target: Option<&DateValue>) -> Option<i32> {
        target.map(|target| target.local_date_time().year())
    }

    /// 返回 Java Calendar 周日为 1 的星期编号。
    /// 对应 Java: `DateUtils#dayOfWeek()`。
    #[must_use]
    pub fn day_of_week(target: Option<&DateValue>) -> Option<i32> {
        target.map(|target| match target.local_date_time().weekday() {
            Weekday::Sun => 1,
            Weekday::Mon => 2,
            Weekday::Tue => 3,
            Weekday::Wed => 4,
            Weekday::Thu => 5,
            Weekday::Fri => 6,
            Weekday::Sat => 7,
        })
    }

    /// 返回 Locale 对应的完整星期名称。
    ///
    /// 对应 Java: `DateUtils#dayOfWeekName(Object,Locale)`。
    pub fn day_of_week_name(
        target: Option<&DateValue>,
        locale: Option<&Locale>,
    ) -> Result<Option<Utf16String>, DateUtilsError> {
        Self::format(target, Some(&Utf16String::from_rust_str("EEEE")), locale)
    }

    /// 返回 Locale 对应的星期缩写。
    ///
    /// 对应 Java: `DateUtils#dayOfWeekNameShort(Object,Locale)`。
    pub fn day_of_week_name_short(
        target: Option<&DateValue>,
        locale: Option<&Locale>,
    ) -> Result<Option<Utf16String>, DateUtilsError> {
        Self::format(target, Some(&Utf16String::from_rust_str("EEE")), locale)
    }

    /// 返回 24 小时制小时。
    /// 对应 Java: `DateUtils#hour()`。
    #[must_use]
    pub fn hour(target: Option<&DateValue>) -> Option<i32> {
        target.map(|target| target.local_date_time().hour() as i32)
    }

    /// 返回分钟。
    /// 对应 Java: `DateUtils#minute()`。
    #[must_use]
    pub fn minute(target: Option<&DateValue>) -> Option<i32> {
        target.map(|target| target.local_date_time().minute() as i32)
    }

    /// 返回秒。
    /// 对应 Java: `DateUtils#second()`。
    #[must_use]
    pub fn second(target: Option<&DateValue>) -> Option<i32> {
        target.map(|target| target.local_date_time().second() as i32)
    }

    /// 返回毫秒。
    /// 对应 Java: `DateUtils#millisecond()`。
    #[must_use]
    pub fn millisecond(target: Option<&DateValue>) -> Option<i32> {
        target.map(|target| target.local_date_time().and_utc().timestamp_subsec_millis() as i32)
    }

    /// 输出 `yyyy-MM-dd'T'HH:mm:ss.SSS+HH:MM` 形式（Java `formatISO` 的
    /// `ZZZ` + `insert(26, ':')`，零偏移为 `+00:00` 而非 "Z"）。
    /// 对应 Java 语义：`DateUtils` 的 `format_iso` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn format_iso(target: Option<&DateValue>) -> Option<Utf16String> {
        target.map(|target| {
            let local = target.local_date_time();
            Utf16String::from_rust_str(&format!(
                "{}{}",
                local.format("%Y-%m-%dT%H:%M:%S%.3f"),
                iso_offset_colon(target.offset_seconds())
            ))
        })
    }

    /// 从动态模板值读取 Java Date/Calendar。
    /// 对应 Java 语义：`DateUtils` 的 `from_template_value` 行为（Rust 侧辅助/私有路径）。
    pub fn from_template_value(
        value: Option<&TemplateValue>,
    ) -> Result<Option<&DateValue>, DateUtilsError> {
        match value {
            None | Some(TemplateValue::Null) => Ok(None),
            Some(TemplateValue::Object(object)) => object
                .as_any()
                .downcast_ref::<DateValue>()
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

    /// 包装为模板动态对象。
    /// 对应 Java 语义：`DateUtils` 的 `into_template_value` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn into_template_value(value: DateValue) -> Arc<TemplateValue> {
        Arc::new(TemplateValue::Object(Arc::new(value)))
    }
}

/// 日期格式化器缓存键的不可变 Rust 等价值。
///
/// 对应 Java: `DateUtils.DateFormatKey`。
///
/// Rust 格式化器本身是线程安全的，无须缓存可变 `DateFormat`；该键仍完整保留
/// Java 按 pattern、Calendar 时区与 Locale 区分格式化语义的方式。
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct DateFormatKey {
    format: String,
    time_zone: Option<String>,
    locale: Locale,
}

impl DateFormatKey {
    fn new(target: &DateValue, format: Option<&Utf16String>, locale: &Locale) -> Self {
        Self {
            format: format
                .map(Utf16String::to_string_lossy)
                .unwrap_or_else(|| default_long_pattern(locale).to_owned()),
            time_zone: target.is_calendar().then(|| target.zone_display_name()),
            locale: locale.clone(),
        }
    }
}

fn invalid(message: impl Into<String>) -> DateUtilsError {
    DateUtilsError::InvalidArgument {
        message: message.into(),
    }
}

fn nullable_i32(value: Option<i32>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn parse_time_zone(value: Option<&str>) -> TimeZoneValue {
    let Some(value) = value else {
        return TimeZoneValue::Named(default_time_zone(), None);
    };
    if let Some(fixed_offset) = fixed_time_zone(value) {
        return TimeZoneValue::Fixed(fixed_offset);
    }
    if let Some((time_zone, display_name)) = short_time_zone(value) {
        return TimeZoneValue::Named(time_zone, Some(display_name.to_owned()));
    }
    let time_zone = value.parse::<Tz>().unwrap_or(chrono_tz::UTC);
    TimeZoneValue::Named(
        // TimeZone#getTimeZone 对未知 ID 返回 GMT，而不是 JVM 默认时区。
        time_zone,
        (time_zone != chrono_tz::UTC
            && value.len() <= 4
            && value
                .chars()
                .all(|character| character.is_ascii_uppercase()))
        .then(|| value.to_owned()),
    )
}

fn fixed_time_zone(value: &str) -> Option<FixedOffset> {
    let suffix = value
        .strip_prefix("GMT")
        .or_else(|| value.strip_prefix("UTC"))?;
    if suffix.is_empty() {
        return FixedOffset::east_opt(0);
    }
    let (sign, digits) = match suffix.as_bytes().first() {
        Some(b'+') => (1, &suffix[1..]),
        Some(b'-') => (-1, &suffix[1..]),
        _ => return None,
    };
    let (hours, minutes) = if let Some((hours, minutes)) = digits.split_once(':') {
        (hours, minutes)
    } else if digits.len() <= 2 {
        (digits, "0")
    } else if digits.len() == 4 {
        (&digits[..2], &digits[2..])
    } else {
        return None;
    };
    let hours = hours.parse::<i32>().ok()?;
    let minutes = minutes.parse::<i32>().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    FixedOffset::east_opt(sign * (hours * 3_600 + minutes * 60))
}

fn short_time_zone(value: &str) -> Option<(Tz, &'static str)> {
    let canonical = match value {
        "GMT" | "UTC" | "UT" | "GMT+00:00" | "GMT-00:00" => "UTC",
        "ACT" => "Australia/Darwin",
        "AET" => "Australia/Sydney",
        "AGT" => "America/Argentina/Buenos_Aires",
        "ART" => "Africa/Cairo",
        "AST" => "America/Anchorage",
        "BET" => "America/Sao_Paulo",
        "BST" => "Asia/Dhaka",
        "CAT" => "Africa/Harare",
        "CNT" => "America/St_Johns",
        "CST" => "America/Chicago",
        "CTT" => "Asia/Shanghai",
        "EAT" => "Africa/Addis_Ababa",
        "ECT" => "Europe/Paris",
        "IET" => "America/Indiana/Indianapolis",
        "IST" => "Asia/Kolkata",
        "JST" => "Asia/Tokyo",
        "MIT" => "Pacific/Apia",
        "NET" => "Asia/Yerevan",
        "NST" => "Pacific/Auckland",
        "PLT" => "Asia/Karachi",
        "PNT" => "America/Phoenix",
        "PRT" => "America/Puerto_Rico",
        "PST" => "America/Los_Angeles",
        "SST" => "Pacific/Guadalcanal",
        "VST" => "Asia/Ho_Chi_Minh",
        _ => return None,
    };
    canonical
        .parse()
        .ok()
        .map(|time_zone| (time_zone, value_id(value)))
}

fn value_id(value: &str) -> &'static str {
    match value {
        "GMT" => "GMT",
        "UTC" => "UTC",
        "UT" => "UT",
        "GMT+00:00" => "GMT+00:00",
        "GMT-00:00" => "GMT-00:00",
        "ACT" => "ACT",
        "AET" => "AET",
        "AGT" => "AGT",
        "ART" => "ART",
        "AST" => "AST",
        "BET" => "BET",
        "BST" => "BST",
        "CAT" => "CAT",
        "CNT" => "CNT",
        "CST" => "CST",
        "CTT" => "CTT",
        "EAT" => "EAT",
        "ECT" => "ECT",
        "IET" => "IET",
        "IST" => "IST",
        "JST" => "JST",
        "MIT" => "MIT",
        "NET" => "NET",
        "NST" => "NST",
        "PLT" => "PLT",
        "PNT" => "PNT",
        "PRT" => "PRT",
        "PST" => "PST",
        "SST" => "SST",
        "VST" => "VST",
        _ => unreachable!("value_id is called only for a recognized Java short time-zone ID"),
    }
}

fn default_time_zone() -> Tz {
    std::env::var("TZ")
        .ok()
        .and_then(|value| value.parse::<Tz>().ok())
        .unwrap_or(chrono_tz::UTC)
}

fn resolve_local_datetime(time_zone: &TimeZoneValue, naive: NaiveDateTime) -> DateTime<Utc> {
    match time_zone {
        TimeZoneValue::Named(time_zone, _) => match time_zone.from_local_datetime(&naive) {
            LocalResult::Single(value) | LocalResult::Ambiguous(value, _) => {
                value.with_timezone(&Utc)
            }
            LocalResult::None => {
                let mut candidate = naive;
                loop {
                    candidate += Duration::minutes(1);
                    if let LocalResult::Single(value) | LocalResult::Ambiguous(value, _) =
                        time_zone.from_local_datetime(&candidate)
                    {
                        break value.with_timezone(&Utc);
                    }
                }
            }
        },
        TimeZoneValue::Fixed(fixed_offset) => fixed_offset
            .from_local_datetime(&naive)
            .single()
            .expect("固定时区的本地时间映射唯一")
            .with_timezone(&Utc),
    }
}

fn default_long_pattern(locale: &Locale) -> &'static str {
    match locale.get_language().to_string_lossy().as_str() {
        "zh" | "ja" | "ko" => "yyyy年M月d日 HH:mm:ss z",
        _ => "MMMM d, yyyy 'at' h:mm:ss a z",
    }
}

fn format_java_pattern(
    date_time: &NaiveDateTime,
    offset_seconds: i32,
    zone_display_name: &str,
    pattern: &str,
    locale: &Locale,
) -> Result<Utf16String, DateUtilsError> {
    let mut output = String::new();
    let characters = pattern.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut quoted = false;
    while index < characters.len() {
        if characters[index] == '\'' {
            if characters.get(index + 1) == Some(&'\'') {
                output.push('\'');
                index += 2;
                continue;
            }
            quoted = !quoted;
            index += 1;
            continue;
        }
        let character = characters[index];
        if quoted || !character.is_ascii_alphabetic() {
            output.push(character);
            index += 1;
            continue;
        }
        let mut end = index + 1;
        while characters.get(end) == Some(&character) {
            end += 1;
        }
        let count = end - index;
        append_pattern_field(
            &mut output,
            date_time,
            offset_seconds,
            zone_display_name,
            character,
            count,
            locale,
        )?;
        index = end;
    }
    if quoted {
        return Err(invalid("Unterminated quote in date pattern"));
    }
    Ok(Utf16String::from_rust_str(&output))
}

fn append_pattern_field(
    output: &mut String,
    date_time: &NaiveDateTime,
    offset_seconds: i32,
    zone_display_name: &str,
    field: char,
    count: usize,
    locale: &Locale,
) -> Result<(), DateUtilsError> {
    match field {
        'G' => output.push_str(if date_time.year() > 0 { "AD" } else { "BC" }),
        'y' => {
            let year = date_time.year();
            if count == 2 {
                output.push_str(&format!("{:02}", year.rem_euclid(100)));
            } else {
                append_signed_number(output, year, count);
            }
        }
        'Y' => append_signed_number(output, date_time.iso_week().year(), count),
        'M' | 'L' if count >= 4 => {
            output.push_str(localized_month_name(date_time.month0(), locale, false));
        }
        'M' | 'L' if count == 3 => {
            output.push_str(localized_month_name(date_time.month0(), locale, true));
        }
        'M' | 'L' => append_number(output, date_time.month(), count),
        'w' => append_number(output, date_time.iso_week().week(), count),
        'W' => append_number(output, date_time.day0() / 7 + 1, count),
        'D' => append_number(output, date_time.ordinal(), count),
        'd' => append_number(output, date_time.day(), count),
        'F' => append_number(output, date_time.day0() / 7 + 1, count),
        'E' => output.push_str(weekday_name(date_time.weekday(), locale, count < 4)),
        'u' => append_number(output, date_time.weekday().number_from_monday(), count),
        'a' => output.push_str(day_period(date_time.hour(), locale)),
        'H' => append_number(output, date_time.hour(), count),
        'k' => append_number(
            output,
            if date_time.hour() == 0 {
                24
            } else {
                date_time.hour()
            },
            count,
        ),
        'K' => append_number(output, date_time.hour() % 12, count),
        'h' => append_number(output, (date_time.hour() + 11) % 12 + 1, count),
        'm' => append_number(output, date_time.minute(), count),
        's' => append_number(output, date_time.second(), count),
        'S' => output.push_str(&format!(
            "{:0width$}",
            date_time.and_utc().timestamp_subsec_millis(),
            width = count
        )),
        'z' => output.push_str(localized_zone_display_name(
            zone_display_name,
            locale,
            count >= 4,
        )),
        // Java `DateTimeFormatter` 的 'Z'（零偏移 → "+0000"，非 "Z"；'X' 才用 "Z"）。
        'Z' => output.push_str(&iso_offset(offset_seconds, 2)),
        'X' if count <= 3 => output.push_str(&iso_offset(offset_seconds, count)),
        'X' => {
            return Err(invalid(format!("invalid ISO 8601 format: length={count}")));
        }
        _ => return Err(invalid(format!("Illegal pattern character '{field}'"))),
    }
    Ok(())
}

fn append_number(output: &mut String, value: u32, count: usize) {
    if count >= 2 {
        output.push_str(&format!("{value:0width$}", width = count));
    } else {
        output.push_str(&value.to_string());
    }
}

fn append_signed_number(output: &mut String, value: i32, count: usize) {
    if value < 0 {
        output.push('-');
    }
    output.push_str(&format!(
        "{:0width$}",
        value.unsigned_abs(),
        width = count.max(1)
    ));
}

fn iso_offset(seconds: i32, count: usize) -> String {
    if seconds == 0 {
        // Java 'X'（count=1/3）零偏移输出 "Z"；'Z'（count=2）输出 "+0000"。
        return if count == 2 {
            "+0000".to_owned()
        } else {
            "Z".to_owned()
        };
    }
    let sign = if seconds < 0 { '-' } else { '+' };
    let absolute = seconds.unsigned_abs();
    let hours = absolute / 3600;
    let minutes = absolute % 3600 / 60;
    match count {
        1 => format!("{sign}{hours:02}"),
        2 => format!("{sign}{hours:02}{minutes:02}"),
        _ => format!("{sign}{hours:02}:{minutes:02}"),
    }
}

/// Java `SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss.SSSZZZ")` + `insert(26, ':')`
/// （DateUtils.formatISO 与 JS 序列化器日期）：偏移恒为 "+HH:MM" 形态，零偏移 →
/// "+00:00"（非 "Z"）。
fn iso_offset_colon(seconds: i32) -> String {
    let sign = if seconds < 0 { '-' } else { '+' };
    let absolute = seconds.unsigned_abs();
    format!("{sign}{:02}:{:02}", absolute / 3600, absolute % 3600 / 60)
}

fn format_gmt_offset(seconds: i32) -> String {
    if seconds == 0 {
        return "GMT".to_owned();
    }
    let sign = if seconds < 0 { '-' } else { '+' };
    let absolute = seconds.unsigned_abs();
    format!(
        "GMT{sign}{:02}:{:02}",
        absolute / 3_600,
        absolute % 3_600 / 60
    )
}

fn day_period(hour: u32, locale: &Locale) -> &'static str {
    match locale.get_language().to_string_lossy().as_str() {
        "zh" => {
            if hour < 12 {
                "上午"
            } else {
                "下午"
            }
        }
        _ => {
            if hour < 12 {
                "AM"
            } else {
                "PM"
            }
        }
    }
}

fn localized_month_name(month: u32, locale: &Locale, short: bool) -> &'static str {
    const EN_LONG: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    const EN_SHORT: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    const ZH: [&str; 12] = [
        "一月",
        "二月",
        "三月",
        "四月",
        "五月",
        "六月",
        "七月",
        "八月",
        "九月",
        "十月",
        "十一月",
        "十二月",
    ];
    const ES_LONG: [&str; 12] = [
        "enero",
        "febrero",
        "marzo",
        "abril",
        "mayo",
        "junio",
        "julio",
        "agosto",
        "septiembre",
        "octubre",
        "noviembre",
        "diciembre",
    ];
    const ES_SHORT: [&str; 12] = [
        "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sept", "oct", "nov", "dic",
    ];
    const DE_LONG: [&str; 12] = [
        "Januar",
        "Februar",
        "März",
        "April",
        "Mai",
        "Juni",
        "Juli",
        "August",
        "September",
        "Oktober",
        "November",
        "Dezember",
    ];
    const DE_SHORT: [&str; 12] = [
        "Jan.", "Feb.", "März", "Apr.", "Mai", "Juni", "Juli", "Aug.", "Sept.", "Okt.", "Nov.",
        "Dez.",
    ];
    const FR_LONG: [&str; 12] = [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ];
    const FR_SHORT: [&str; 12] = [
        "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
        "déc.",
    ];
    match (locale.get_language().to_string_lossy().as_str(), short) {
        ("zh" | "ja", _) => ZH[month as usize],
        ("de", true) => DE_SHORT[month as usize],
        ("de", false) => DE_LONG[month as usize],
        ("fr", true) => FR_SHORT[month as usize],
        ("fr", false) => FR_LONG[month as usize],
        ("es", true) => ES_SHORT[month as usize],
        ("es", false) => ES_LONG[month as usize],
        (_, true) => EN_SHORT[month as usize],
        (_, false) => EN_LONG[month as usize],
    }
}

fn weekday_name(weekday: Weekday, locale: &Locale, short: bool) -> &'static str {
    let index = weekday.num_days_from_sunday() as usize;
    const EN_LONG: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    const EN_SHORT: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const ZH_LONG: [&str; 7] = [
        "星期日",
        "星期一",
        "星期二",
        "星期三",
        "星期四",
        "星期五",
        "星期六",
    ];
    const ZH_SHORT: [&str; 7] = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
    const DE_LONG: [&str; 7] = [
        "Sonntag",
        "Montag",
        "Dienstag",
        "Mittwoch",
        "Donnerstag",
        "Freitag",
        "Samstag",
    ];
    const DE_SHORT: [&str; 7] = ["So.", "Mo.", "Di.", "Mi.", "Do.", "Fr.", "Sa."];
    const FR_LONG: [&str; 7] = [
        "dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi",
    ];
    const FR_SHORT: [&str; 7] = ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."];
    match (locale.get_language().to_string_lossy().as_str(), short) {
        ("zh" | "ja", true) => ZH_SHORT[index],
        ("zh" | "ja", false) => ZH_LONG[index],
        ("de", true) => DE_SHORT[index],
        ("de", false) => DE_LONG[index],
        ("fr", true) => FR_SHORT[index],
        ("fr", false) => FR_LONG[index],
        (_, true) => EN_SHORT[index],
        (_, false) => EN_LONG[index],
    }
}

fn localized_zone_display_name<'a>(
    zone_display_name: &'a str,
    locale: &Locale,
    long: bool,
) -> &'a str {
    if !long || !matches!(zone_display_name, "UTC" | "GMT") {
        return zone_display_name;
    }
    match locale.get_language().to_string_lossy().as_str() {
        "de" => "Koordinierte Weltzeit",
        "fr" => "temps universel coordonné",
        "ja" => "協定世界時",
        _ => "Coordinated Universal Time",
    }
}

/// 从模板动态数字提取 Java intValue。
/// 对应 Java 语义：`DateUtils` 的 `template_integer` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn template_integer(
    value: &Option<Arc<TemplateValue>>,
) -> Result<Option<i32>, DateUtilsError> {
    match value.as_deref() {
        None | Some(TemplateValue::Null) => Ok(None),
        Some(TemplateValue::Number(NumberValue::Byte(value))) => Ok(Some(i32::from(*value))),
        Some(TemplateValue::Number(NumberValue::Short(value))) => Ok(Some(i32::from(*value))),
        Some(TemplateValue::Number(NumberValue::Integer(value))) => Ok(Some(*value)),
        Some(TemplateValue::Number(NumberValue::Long(value))) => Ok(Some(*value as i32)),
        Some(TemplateValue::Number(NumberValue::Float(value))) => Ok(Some(*value as i32)),
        Some(TemplateValue::Number(NumberValue::Double(value))) => Ok(Some(*value as i32)),
        Some(value) => value
            .to_utf16_string()
            .and_then(|value| value.to_string_lossy().parse::<i32>().ok())
            .map(Some)
            .ok_or_else(|| invalid("Value cannot be evaluated as a number")),
    }
}
