//! `#temporals` 表达式对象差分 —— 1:1 移植 Java
//! `TemporalsFormattingTest`（44 个方法：format 系列 / 标准 pattern / 字段方法
//! / 名称方法 / formatISO / issue17，各含 null 变体）。
//!
//! 与现有 `temporal_utils_java_parity.rs` 的区别：本文件直接断言
//! `Temporals.java_invoke_method` 的**表达式对象分派路径**（参数校验、null
//! 折叠、返回类型），而非底层 `TemporalFormattingUtils`。
//!
//! Java 夹具：`new Temporals(Locale.US, ZoneOffset.UTC)`；期望值以 Java 21
//! （`assertEqualsNormalized` 归一化 U+202F 后）实测为准 —— JDK 9+ CLDR en_US
//! 的 LONG/FULL 日期时间为 `MMMM d, y, h:mm:ss a z`（无 "at"，UTC 时区名为 "Z"）。

use std::sync::Arc;

use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use thymeleaf::expression::{TemplateObject, TemplateValue, Temporals};
use thymeleaf::temporal::JavaTemporal;
use thymeleaf::util::{JavaLocale, JavaString};

fn js(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn locale_us() -> JavaLocale {
    JavaLocale::new(
        JavaString::from_rust_str("en"),
        JavaString::from_rust_str("US"),
    )
}

fn locale_de() -> JavaLocale {
    JavaLocale::new(
        JavaString::from_rust_str("de"),
        JavaString::from_rust_str("DE"),
    )
}

fn temporals() -> Temporals {
    Temporals::with_default_zone_id(locale_us(), Tz::UTC).expect("temporals")
}

/// 包装时间值为模板参数（Java 方法实参）。
fn arg(value: JavaTemporal) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::Object(Arc::new(value))))
}

/// 空参数（Java null 实参）。
fn null_arg() -> Option<Arc<TemplateValue>> {
    None
}

/// 调用 `temporals.<method>(args...)` 并返回 Option 值。
fn call(
    temporals: &Temporals,
    method: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> Option<Arc<TemplateValue>> {
    temporals
        .java_invoke_method(&js(method), arguments)
        .expect("invoke succeeds")
        .expect("invoke returns value")
}

/// 断言整数值结果（Java `temporals.xxx(time).intValue()`）。
fn assert_integer(temporals: &Temporals, method: &str, value: JavaTemporal, expected: i64) {
    let result = call(temporals, method, &[arg(value)]);
    let actual = result
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    assert_eq!(
        actual.as_deref(),
        Some(expected.to_string().as_str()),
        "{method} 返回值"
    );
}

/// 断言字符串结果（Java `temporals.xxx(time)`）。
fn assert_text(temporals: &Temporals, method: &str, value: JavaTemporal, expected: &str) {
    let result = call(temporals, method, &[arg(value)]);
    let actual = result
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    assert_eq!(actual.as_deref(), Some(expected), "{method} 返回值");
}

/// 断言 null 输入返回 null（Java `assertNull(temporals.xxx(null))`）。
fn assert_null(temporals: &Temporals, method: &str) {
    let result = call(temporals, method, &[null_arg()]);
    assert!(
        result.is_none(),
        "{method}(null) 应返回 null，得到 {result:?}"
    );
}

// ===========================================================================
// format 系列
// ===========================================================================

#[test]
fn temporals_format_matches_java() {
    let temporals = temporals();
    let moment = JavaTemporal::ZonedDateTime(
        chrono_tz::Tz::UTC.from_utc_datetime(
            &NaiveDate::from_ymd_opt(2015, 12, 31)
                .expect("date")
                .and_hms_opt(23, 59, 45)
                .expect("time"),
        ),
    );
    // Java: DateTimeFormatter.ofLocalizedDateTime(LONG).withLocale(US)
    // -> "December 31, 2015, 11:59:45 PM Z"（JDK 9+ CLDR en_US，含 `z` 时区名，
    // UTC → "Z"；由 Java 21 实测确认）
    assert_text(
        &temporals,
        "format",
        moment,
        "December 31, 2015, 11:59:45 PM Z",
    );
}

#[test]
fn temporals_format_with_locale_matches_java() {
    // Java testFormatWithLocale：GERMANY 无 pattern -> ofLocalizedDateTime(LONG)
    // -> "31. Dezember 2015, 23:59:45 Z"（Java 21 实测）
    let temporals_de = Temporals::with_default_zone_id(locale_de(), Tz::UTC).expect("temporals");
    let moment = JavaTemporal::ZonedDateTime(
        chrono_tz::Tz::UTC.from_utc_datetime(
            &NaiveDate::from_ymd_opt(2015, 12, 31)
                .expect("date")
                .and_hms_opt(23, 59, 45)
                .expect("time"),
        ),
    );
    assert_text(
        &temporals_de,
        "format",
        moment,
        "31. Dezember 2015, 23:59:45 Z",
    );
}

#[test]
fn temporals_format_null_and_locale_matches_java() {
    let temporals = temporals();
    assert_null(&temporals, "format");

    // Java testFormatWithLocaleAndNullTemporal：format(null, Locale.GERMANY) == null。
    // java_invoke_method 无法直接传 Locale 对象（该路径由语料 dateformat-* 覆盖），
    // 此处用 de 配置实例验证 null 折叠。
    let temporals_de = Temporals::with_default_zone_id(locale_de(), Tz::UTC).expect("temporals");
    assert_null(&temporals_de, "format");
}

#[test]
fn temporals_format_with_pattern_matches_java() {
    let temporals = temporals();
    // testFormatWithPattern：LocalDateTime 2015-12-31T23:59 -> "2015-12-31 23:59:00"
    let moment = JavaTemporal::LocalDateTime(
        NaiveDate::from_ymd_opt(2015, 12, 31)
            .expect("date")
            .and_hms_opt(23, 59, 0)
            .expect("time"),
    );
    let result = call(
        &temporals,
        "format",
        &[
            arg(moment.clone()),
            Some(Arc::new(TemplateValue::string(js("yyyy-MM-dd HH:mm:ss")))),
        ],
    );
    let actual = result
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    assert_eq!(actual.as_deref(), Some("2015-12-31 23:59:00"));

    // testFormatWithPatternAndLocale：GERMANY "EEEE, d MMMM, yyyy"
    // -> "Donnerstag, 31 Dezember, 2015"
    let temporals_de = Temporals::with_default_zone_id(locale_de(), Tz::UTC).expect("temporals");
    let result = call(
        &temporals_de,
        "format",
        &[
            arg(moment),
            Some(Arc::new(TemplateValue::string(js("EEEE, d MMMM, yyyy")))),
        ],
    );
    let actual = result
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    assert_eq!(actual.as_deref(), Some("Donnerstag, 31 Dezember, 2015"));

    // null 变体
    assert_null(&temporals, "format");
    let result = call(
        &temporals,
        "format",
        &[null_arg(), Some(Arc::new(TemplateValue::string(js("y"))))],
    );
    assert!(result.is_none(), "format(null, pattern) 应返回 null");
}

#[test]
fn temporals_standard_patterns_matches_java() {
    let temporals = temporals();
    // testFormatStandardPatternDate：LocalDate 2015-12-31
    let day = JavaTemporal::LocalDate(NaiveDate::from_ymd_opt(2015, 12, 31).expect("date"));
    for (pattern, expected) in [
        ("SHORT", "12/31/15"),
        ("MEDIUM", "Dec 31, 2015"),
        ("LONG", "December 31, 2015"),
        ("FULL", "Thursday, December 31, 2015"),
    ] {
        let result = call(
            &temporals,
            "format",
            &[
                arg(day.clone()),
                Some(Arc::new(TemplateValue::string(js(pattern)))),
            ],
        );
        let actual = result
            .as_deref()
            .and_then(TemplateValue::to_java_string)
            .map(|value| value.to_string_lossy());
        assert_eq!(actual.as_deref(), Some(expected), "pattern {pattern}");
    }

    // testFormatStandardPatternTime：LocalTime 23:59（Java 期望带 PM/Z）
    let time = JavaTemporal::LocalTime(NaiveTime::from_hms_opt(23, 59, 0).expect("time"));
    for (pattern, expected) in [
        ("SHORT", "11:59 PM"),
        ("MEDIUM", "11:59:00 PM"),
        ("LONG", "11:59:00 PM Z"),
        ("FULL", "11:59:00 PM Z"),
    ] {
        let result = call(
            &temporals,
            "format",
            &[
                arg(time.clone()),
                Some(Arc::new(TemplateValue::string(js(pattern)))),
            ],
        );
        let actual = result
            .as_deref()
            .and_then(TemplateValue::to_java_string)
            .map(|value| value.to_string_lossy());
        assert_eq!(
            actual
                .as_deref()
                .map(|value| value.replace('\u{202f}', " ")),
            Some(expected.to_owned()),
            "time pattern {pattern}"
        );
    }
}

#[test]
fn temporals_standard_pattern_datetime_matches_java() {
    // Java testFormatStandardPatternDateTime：LocalDateTime 2015-12-31 23:59，
    // computeFormatter 走 ofLocalizedDateTime(style)（zonedTime 到 UTC）。
    // 期望值由 Java 21 实测：JDK 9+ CLDR en_US 无 "at"，LONG/FULL 含 "Z"。
    let temporals = temporals();
    let moment = JavaTemporal::LocalDateTime(
        NaiveDate::from_ymd_opt(2015, 12, 31)
            .expect("date")
            .and_hms_opt(23, 59, 0)
            .expect("time"),
    );
    for (pattern, expected) in [
        ("SHORT", "12/31/15, 11:59 PM"),
        ("MEDIUM", "Dec 31, 2015, 11:59:00 PM"),
        ("LONG", "December 31, 2015, 11:59:00 PM Z"),
        ("FULL", "Thursday, December 31, 2015, 11:59:00 PM Z"),
    ] {
        let result = call(
            &temporals,
            "format",
            &[
                arg(moment.clone()),
                Some(Arc::new(TemplateValue::string(js(pattern)))),
            ],
        );
        let actual = result
            .as_deref()
            .and_then(TemplateValue::to_java_string)
            .map(|value| value.to_string_lossy());
        assert_eq!(
            actual
                .as_deref()
                .map(|value| value.replace('\u{202f}', " ")),
            Some(expected.to_owned()),
            "datetime pattern {pattern}"
        );
    }
}

#[test]
fn temporals_specific_types_with_pattern_matches_java() {
    let temporals = temporals();
    // localTimeWithPattern
    let time = JavaTemporal::LocalTime(NaiveTime::from_hms_opt(23, 59, 45).expect("time"));
    let result = call(
        &temporals,
        "format",
        &[
            arg(time),
            Some(Arc::new(TemplateValue::string(js("HH:mm:ss")))),
        ],
    );
    let actual = result
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    assert_eq!(actual.as_deref(), Some("23:59:45"));

    // offsetDateTimeWithPattern
    let offset_date_time = JavaTemporal::OffsetDateTime(
        NaiveDate::from_ymd_opt(2015, 12, 31)
            .expect("date")
            .and_hms_opt(23, 59, 45)
            .expect("time")
            .and_local_timezone(FixedOffset::east_opt(0).expect("offset"))
            .single()
            .expect("local time in UTC"),
    );
    let result = call(
        &temporals,
        "format",
        &[
            arg(offset_date_time),
            Some(Arc::new(TemplateValue::string(js("MM/dd/yyyy HH:mm:ss")))),
        ],
    );
    let actual = result
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    assert_eq!(actual.as_deref(), Some("12/31/2015 23:59:45"));

    // offsetTimeWithPattern
    let offset_time = JavaTemporal::OffsetTime(
        NaiveTime::from_hms_opt(23, 59, 45).expect("time"),
        FixedOffset::east_opt(0).expect("offset"),
    );
    let result = call(
        &temporals,
        "format",
        &[
            arg(offset_time),
            Some(Arc::new(TemplateValue::string(js("HH:mm:ss")))),
        ],
    );
    let actual = result
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    assert_eq!(actual.as_deref(), Some("23:59:45"));

    // yearWithPattern
    let year = JavaTemporal::Year(2015);
    let result = call(
        &temporals,
        "format",
        &[arg(year), Some(Arc::new(TemplateValue::string(js("yyyy"))))],
    );
    let actual = result
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    assert_eq!(actual.as_deref(), Some("2015"));

    // yearMonthWithPattern
    let year_month = JavaTemporal::YearMonth(2015, 12);
    let result = call(
        &temporals,
        "format",
        &[
            arg(year_month),
            Some(Arc::new(TemplateValue::string(js("MM/yyyy")))),
        ],
    );
    let actual = result
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    assert_eq!(actual.as_deref(), Some("12/2015"));
}

// ===========================================================================
// 字段方法（day/month/year/dayOfWeek/hour/minute/second/nanosecond）
// ===========================================================================

#[test]
fn temporals_date_fields_matches_java() {
    let temporals = temporals();
    let day = JavaTemporal::LocalDate(NaiveDate::from_ymd_opt(2015, 12, 31).expect("date"));
    assert_integer(&temporals, "day", day.clone(), 31);
    assert_integer(&temporals, "month", day.clone(), 12);
    assert_integer(&temporals, "year", day.clone(), 2015);
    assert_integer(&temporals, "dayOfWeek", day.clone(), 4);
    // null 变体
    assert_null(&temporals, "day");
    assert_null(&temporals, "month");
    assert_null(&temporals, "year");
    assert_null(&temporals, "dayOfWeek");
}

#[test]
fn temporals_time_fields_matches_java() {
    let temporals = temporals();
    let moment = JavaTemporal::LocalDateTime(
        NaiveDate::from_ymd_opt(2015, 12, 31)
            .expect("date")
            .and_hms_nano_opt(23, 59, 45, 1)
            .expect("time"),
    );
    assert_integer(&temporals, "hour", moment.clone(), 23);
    assert_integer(&temporals, "minute", moment.clone(), 59);
    assert_integer(&temporals, "second", moment.clone(), 45);
    assert_integer(&temporals, "nanosecond", moment.clone(), 1);
    // null 变体
    assert_null(&temporals, "hour");
    assert_null(&temporals, "minute");
    assert_null(&temporals, "second");
    assert_null(&temporals, "nanosecond");
}

#[test]
fn temporals_name_fields_matches_java() {
    let temporals = temporals();
    let day = JavaTemporal::LocalDate(NaiveDate::from_ymd_opt(2015, 12, 31).expect("date"));
    assert_text(&temporals, "monthName", day.clone(), "December");
    assert_text(&temporals, "monthNameShort", day.clone(), "Dec");
    assert_text(&temporals, "dayOfWeekName", day.clone(), "Thursday");
    assert_text(&temporals, "dayOfWeekNameShort", day.clone(), "Thu");
    // null 变体
    assert_null(&temporals, "monthName");
    assert_null(&temporals, "monthNameShort");
    assert_null(&temporals, "dayOfWeekName");
    assert_null(&temporals, "dayOfWeekNameShort");
}

// ===========================================================================
// formatISO / issue17
// ===========================================================================

#[test]
fn temporals_format_iso_and_issue17_matches_java() {
    let temporals = temporals();
    // testFormatISO：LocalDateTime.of(2015,12,31,23,59,45,1).atZone(ZoneOffset.MAX)
    // -> "2015-12-31T23:59:45.000+1800"。23:59:45 是 +18:00 下的**本地**时间，
    // 必须用 and_local_timezone（DateTime::from_utc 会把 naive 当 UTC 瞬时）。
    let offset_max = JavaTemporal::OffsetDateTime(
        NaiveDate::from_ymd_opt(2015, 12, 31)
            .expect("date")
            .and_hms_nano_opt(23, 59, 45, 1)
            .expect("time")
            .and_local_timezone(FixedOffset::east_opt(18 * 3600).expect("offset max"))
            .single()
            .expect("local time in +18:00"),
    );
    let result = call(&temporals, "formatISO", &[arg(offset_max)]);
    let actual = result
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    assert_eq!(actual.as_deref(), Some("2015-12-31T23:59:45.000+1800"));
    assert_null(&temporals, "formatISO");

    // testIssue17：Instant(1s) format "yyyy-MM-dd" -> "1970-01-01"
    let instant = JavaTemporal::Instant(Utc.timestamp_opt(1, 0).single().expect("instant"));
    let result = call(
        &temporals,
        "format",
        &[
            arg(instant),
            Some(Arc::new(TemplateValue::string(js("yyyy-MM-dd")))),
        ],
    );
    let actual = result
        .as_deref()
        .and_then(TemplateValue::to_java_string)
        .map(|value| value.to_string_lossy());
    assert_eq!(actual.as_deref(), Some("1970-01-01"));
}
