//! `org.thymeleaf.util.temporal` 族 Java 1:1 差分测试。
//!
//! 断言值逐字取自上游 `thymeleaf-tests-core` 的
//! `org.thymeleaf.standard.expression` 包 TemporalsArrayTest /
//! TemporalsListTest / TemporalsSetTest / TemporalsCreationTest /
//! TemporalsFormattingTest（Locale.US / ZoneOffset.UTC 基准）：
//! - `TemporalCreationUtils`（471）：create 字段组合与 TemporalValue 判别；
//! - `TemporalFormattingUtils`（472）：默认/SHORT/MEDIUM/LONG/FULL 本地化
//!   格式、自定义 pattern、时区换算、null 语义与字段读取；
//! - `TemporalArrayUtils`（470）/ `TemporalListUtils`（473）/
//!   `TemporalSetUtils`（475）：批量格式化与字段读取（Set 按 LinkedHashSet
//!   语义去重保序）；
//! - `TemporalObjects`（474）：date/time 字段分解与类型判别（被全部
//!   utils 委托的内部对象）；
//! - `Temporals`（167）：表达式对象（本批 utils 的委托层，既有
//!   `expression_invoker_methods_java_parity.rs` 的 `#temporals` 用例强化）。

use chrono::{NaiveDate, NaiveDateTime};
use chrono_tz::Tz;

use thymeleaf::temporal::{
    TemporalArrayUtils, TemporalCreationUtils, TemporalFormattingUtils, TemporalKind,
    TemporalListUtils, TemporalObjects, TemporalSetUtils, TemporalValue,
};
use thymeleaf::util::{Locale, Utf16String};

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn us() -> Locale {
    Locale::new(js("en"), js("US"))
}

fn germany() -> Locale {
    Locale::new(js("de"), js("DE"))
}

fn date(year: i32, month: u32, day: u32) -> TemporalValue {
    TemporalValue::LocalDate(NaiveDate::from_ymd_opt(year, month, day).expect("valid date"))
}

fn date_time(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> TemporalValue {
    TemporalValue::LocalDateTime(
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("valid date")
            .and_hms_opt(hour, minute, second)
            .expect("valid time"),
    )
}

fn text(value: Option<Utf16String>) -> String {
    value.map(|v| v.to_string_lossy()).unwrap_or_default()
}

// ===========================================================================
// 1. TemporalCreationUtils（471）
// ===========================================================================

#[test]
fn temporal_creation_utils_matches_java() {
    let creation = TemporalCreationUtils::new();

    // create(y,mo,d) -> LocalDate（Java: LocalDate.of）
    let created = creation.create(&[2024, 5, 17]).expect("create date");
    assert_eq!(created.kind(), TemporalKind::LocalDate);
    // Java ChronoField.DAY_OF_WEEK（ISO 1=Mon..7=Sun）：2024-05-17 是周五
    assert_eq!(
        TemporalObjects::date_fields(&created).expect("date fields"),
        (2024, 5, 17, 5)
    );

    // create(y,mo,d,h,mi) -> LocalDateTime（Java: LocalDateTime.of）
    let created = creation
        .create(&[2024, 5, 17, 12, 30])
        .expect("create date time");
    assert_eq!(created.kind(), TemporalKind::LocalDateTime);
    assert_eq!(
        TemporalObjects::time_fields(&created).expect("time fields"),
        (12, 30, 0, 0)
    );

    // create(y,mo,d,h,mi,s) 与秒+纳秒
    let created = creation
        .create(&[2024, 5, 17, 12, 30, 45])
        .expect("create with seconds");
    assert_eq!(
        TemporalObjects::time_fields(&created).expect("time fields"),
        (12, 30, 45, 0)
    );
    let created = creation
        .create(&[2024, 5, 17, 12, 30, 45, 500_000_000])
        .expect("create with nanos");
    assert_eq!(
        TemporalObjects::time_fields(&created).expect("time fields"),
        (12, 30, 45, 500_000_000)
    );

    // createDate / createDateTime 解析文本重载（Java Temporals API：
    // createDate(String, pattern)，默认 ISO yyyy-MM-dd）
    let created = creation
        .create_date("2015-01-01", None)
        .expect("create date");
    assert_eq!(created.kind(), TemporalKind::LocalDate);
    // 2015-01-01 是周四（ISO dayOfWeek 4）
    assert_eq!(
        TemporalObjects::date_fields(&created).expect("date fields"),
        (2015, 1, 1, 4)
    );
    let created = creation
        .create_date_time("2015-12-31T23:59:00", None)
        .expect("create date time");
    assert_eq!(
        TemporalObjects::time_fields(&created).expect("time fields"),
        (23, 59, 0, 0)
    );
    // 自定义 pattern 解析（Java: createDate(text, pattern)）
    let created = creation
        .create_date("31-12-2015", Some("dd-MM-yyyy"))
        .expect("create date with pattern");
    assert_eq!(
        TemporalObjects::date_fields(&created).expect("date fields"),
        (2015, 12, 31, 4)
    );

    // createNow（Java: LocalDateTime.now() 语义，仅断言类型与字段形状）
    let now = creation.create_now();
    assert_eq!(now.kind(), TemporalKind::LocalDateTime);

    // createToday（Java: LocalDate.now()）
    let today = creation.create_today();
    assert_eq!(today.kind(), TemporalKind::LocalDate);

    // 非法字段组合 -> 错误（Java 抛出 DateTimeException 族）
    assert!(creation.create(&[2024]).is_err(), "too few fields rejected");
    assert!(
        creation.create(&[2024, 13, 1, 0, 0]).is_err(),
        "invalid month rejected"
    );
}

// ===========================================================================
// 2. TemporalFormattingUtils（472）
// ===========================================================================

#[test]
fn temporal_formatting_utils_matches_java() {
    let formatting = TemporalFormattingUtils::new(us(), Tz::UTC).expect("formatting utils");

    // 默认本地化格式（Java: ofLocalizedDate(LONG).withLocale(US)）
    let day = date(2015, 1, 1);
    assert_eq!(
        text(
            formatting
                .format(Some(&day), None, None, None)
                .expect("format")
        ),
        "January 1, 2015"
    );
    // GERMANY 默认日期格式
    assert_eq!(
        text(
            formatting
                .format(Some(&day), None, Some(&germany()), None)
                .expect("format")
        ),
        "1. Januar 2015"
    );

    // 自定义 pattern
    assert_eq!(
        text(
            formatting
                .format(Some(&day), Some("yyyy-MM-dd"), None, None)
                .expect("format")
        ),
        "2015-01-01"
    );
    // pattern + locale（Java: "Donnerstag, 1 Januar, 2015"）
    assert_eq!(
        text(
            formatting
                .format(
                    Some(&day),
                    Some("EEEE, d MMMM, yyyy"),
                    Some(&germany()),
                    None
                )
                .expect("format")
        ),
        "Donnerstag, 1 Januar, 2015"
    );

    // 本地化样式 SHORT/MEDIUM/LONG/FULL（Java DateTimeFormatter 值）
    let year_end = date(2015, 12, 31);
    assert_eq!(
        text(
            formatting
                .format(Some(&year_end), Some("SHORT"), None, None)
                .expect("format")
        ),
        "12/31/15"
    );
    assert_eq!(
        text(
            formatting
                .format(Some(&year_end), Some("MEDIUM"), None, None)
                .expect("format")
        ),
        "Dec 31, 2015"
    );
    assert_eq!(
        text(
            formatting
                .format(Some(&year_end), Some("LONG"), None, None)
                .expect("format")
        ),
        "December 31, 2015"
    );
    assert_eq!(
        text(
            formatting
                .format(Some(&year_end), Some("FULL"), None, None)
                .expect("format")
        ),
        "Thursday, December 31, 2015"
    );

    // 日期时间 pattern 与时区换算（Java: 23:59 GMT+5 -> 18:59）
    let moment = date_time(2015, 12, 31, 23, 59, 0);
    assert_eq!(
        text(
            formatting
                .format(Some(&moment), Some("yyyy-MM-dd HH:mm:ss"), None, None)
                .expect("format")
        ),
        "2015-12-31 23:59:00"
    );
    assert_eq!(
        text(
            formatting
                .format(
                    Some(&moment),
                    Some("yyyy-MM-dd HH:mm:ss"),
                    None,
                    Some(Tz::Etc__GMTPlus5),
                )
                .expect("format")
        ),
        "2015-12-31 18:59:00"
    );

    // null 输入 -> null（Java format(null, "y") == null）
    assert!(
        formatting
            .format(None, Some("y"), None, None)
            .expect("null format")
            .is_none()
    );

    // 字段读取（Java day/month/year/hour/minute/second/nanosecond/dayOfWeek）
    assert_eq!(formatting.day(Some(&year_end)).expect("day"), Some(31));
    assert_eq!(formatting.month(Some(&year_end)).expect("month"), Some(12));
    assert_eq!(formatting.year(Some(&year_end)).expect("year"), Some(2015));
    assert_eq!(
        formatting
            .day_of_week(Some(&year_end))
            .expect("day of week"),
        Some(4),
        "2015-12-31 is Thursday (Java dayOfWeek 4)"
    );
    assert_eq!(
        text(formatting.month_name(Some(&year_end)).expect("month name")),
        "December"
    );
    assert_eq!(
        text(
            formatting
                .month_name_short(Some(&year_end))
                .expect("month name short")
        ),
        "Dec"
    );
    assert_eq!(
        text(
            formatting
                .day_of_week_name(Some(&year_end))
                .expect("day of week name")
        ),
        "Thursday"
    );
    assert_eq!(
        text(
            formatting
                .day_of_week_name_short(Some(&year_end))
                .expect("day of week name short")
        ),
        "Thu"
    );

    // 时间字段（Java: LocalDateTime 23:59:45）
    let moment = date_time(2015, 12, 31, 23, 59, 45);
    assert_eq!(formatting.hour(Some(&moment)).expect("hour"), Some(23));
    assert_eq!(formatting.minute(Some(&moment)).expect("minute"), Some(59));
    assert_eq!(formatting.second(Some(&moment)).expect("second"), Some(45));
    assert_eq!(
        text(
            formatting
                .format(Some(&moment), Some("HH:mm:ss"), None, None)
                .expect("format")
        ),
        "23:59:45"
    );

    // null 字段读取 -> None（Java null 传播）
    assert_eq!(formatting.day(None).expect("null day"), None);
}

// ===========================================================================
// 3. TemporalArrayUtils（470）
// ===========================================================================

#[test]
fn temporal_array_utils_matches_java() {
    let array_utils = TemporalArrayUtils::new(us(), Tz::UTC).expect("array utils");
    let array = [Some(date(2015, 1, 1)), Some(date(2015, 12, 31))];

    // arrayFormat 默认 US
    let formatted = array_utils
        .array_format(&array, None, None)
        .expect("array format");
    assert_eq!(
        formatted
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        ["January 1, 2015", "December 31, 2015"]
    );
    // arrayFormat GERMANY 默认
    let formatted = array_utils
        .array_format(&array, None, Some(&germany()))
        .expect("array format");
    assert_eq!(
        formatted
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        ["1. Januar 2015", "31. Dezember 2015"]
    );
    // arrayFormat pattern
    let formatted = array_utils
        .array_format(&array, Some("yyyy-MM-dd"), None)
        .expect("array format");
    assert_eq!(
        formatted
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        ["2015-01-01", "2015-12-31"]
    );
    // arrayFormat pattern + locale（Java: "Donnerstag, 1 Januar, 2015"）
    let formatted = array_utils
        .array_format(&array, Some("EEEE, d MMMM, yyyy"), Some(&germany()))
        .expect("array format");
    assert_eq!(
        formatted
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        [
            "Donnerstag, 1 Januar, 2015",
            "Donnerstag, 31 Dezember, 2015"
        ]
    );

    // 字段批量读取
    assert_eq!(
        array_utils.array_day(&array).expect("array day"),
        [Some(1), Some(31)]
    );
    assert_eq!(
        array_utils.array_month(&array).expect("array month"),
        [Some(1), Some(12)]
    );
    assert_eq!(
        array_utils.array_year(&array).expect("array year"),
        [Some(2015), Some(2015)]
    );
    assert_eq!(
        array_utils
            .array_day_of_week(&array)
            .expect("array day of week"),
        [Some(4), Some(4)],
        "both 2015-01-01 and 2015-12-31 are Thursday"
    );
    assert_eq!(
        array_utils
            .array_month_name(&array)
            .expect("array month name")
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        ["January", "December"]
    );
    assert_eq!(
        array_utils
            .array_month_name_short(&array)
            .expect("array month name short")
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        ["Jan", "Dec"]
    );
    assert_eq!(
        array_utils
            .array_day_of_week_name(&array)
            .expect("array day of week name")
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        ["Thursday", "Thursday"]
    );
    assert_eq!(
        array_utils
            .array_day_of_week_name_short(&array)
            .expect("array day of week name short")
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        ["Thu", "Thu"]
    );

    // null 元素 -> null 输出（Java 数组内 null 传播）
    let with_null = [Some(date(2015, 1, 1)), None];
    let formatted = array_utils
        .array_format(&with_null, Some("yyyy"), None)
        .expect("array format with null");
    assert_eq!(formatted[0].as_ref().unwrap().to_string_lossy(), "2015");
    assert!(formatted[1].is_none());
}

// ===========================================================================
// 4. TemporalListUtils（473）
// ===========================================================================

#[test]
fn temporal_list_utils_matches_java() {
    let list_utils = TemporalListUtils::new(us(), Tz::UTC).expect("list utils");
    let list = vec![Some(date(2015, 1, 1)), Some(date(2015, 12, 31))];

    let formatted = list_utils
        .list_format(&list, None, None)
        .expect("list format");
    assert_eq!(
        formatted
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        ["January 1, 2015", "December 31, 2015"]
    );
    assert_eq!(
        list_utils.list_day(&list).expect("list day"),
        [Some(1), Some(31)]
    );
    assert_eq!(
        list_utils.list_month(&list).expect("list month"),
        [Some(1), Some(12)]
    );
    assert_eq!(
        list_utils.list_year(&list).expect("list year"),
        [Some(2015), Some(2015)]
    );
    assert_eq!(
        list_utils
            .list_month_name(&list)
            .expect("list month name")
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        ["January", "December"]
    );
}

// ===========================================================================
// 5. TemporalSetUtils（475）
// ===========================================================================

#[test]
fn temporal_set_utils_matches_java() {
    let set_utils = TemporalSetUtils::new(us(), Tz::UTC).expect("set utils");
    let set = vec![Some(date(2015, 1, 1)), Some(date(2015, 12, 31))];

    // Java LinkedHashSet 语义：去重保序
    let formatted = set_utils.set_format(&set, None, None).expect("set format");
    assert_eq!(
        formatted
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        ["January 1, 2015", "December 31, 2015"]
    );
    let duplicate = vec![Some(date(2015, 1, 1)), Some(date(2015, 1, 1))];
    let formatted = set_utils
        .set_format(&duplicate, Some("yyyy"), None)
        .expect("set format dedupe");
    assert_eq!(
        formatted
            .iter()
            .map(|v| v.as_ref().unwrap().to_string_lossy())
            .collect::<Vec<_>>(),
        ["2015"]
    );
    assert_eq!(
        set_utils.set_day(&set).expect("set day"),
        [Some(1), Some(31)]
    );
    assert_eq!(
        set_utils.set_month(&set).expect("set month"),
        [Some(1), Some(12)]
    );
}

// ===========================================================================
// 6. TemporalObjects（474）：字段分解与类型判别
// ===========================================================================

#[test]
fn temporal_objects_dispatch_matches_java() {
    // Java LocalDate：dateFields 分解（year/month/day/dayOfWeek）
    let day = date(2015, 12, 31);
    assert_eq!(
        TemporalObjects::date_fields(&day).expect("date fields"),
        (2015, 12, 31, 4),
        "Thursday = dayOfWeek 4"
    );

    // Java LocalDateTime：timeFields 分解（hour/minute/second/nano）
    let moment = date_time(2015, 12, 31, 23, 59, 45);
    assert_eq!(
        TemporalObjects::time_fields(&moment).expect("time fields"),
        (23, 59, 45, 0)
    );

    // 类型判别（Java Temporal 具体类型）
    assert_eq!(day.kind(), TemporalKind::LocalDate);
    assert_eq!(moment.kind(), TemporalKind::LocalDateTime);
    assert_eq!(
        TemporalValue::LocalTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2015, 1, 1).expect("date"),
                chrono::NaiveTime::from_hms_opt(23, 59, 45).expect("time"),
            )
            .time(),
        )
        .kind(),
        TemporalKind::LocalTime
    );

    // 偏移秒（Java ZoneOffset.getTotalSeconds；UTC = 0）
    assert_eq!(TemporalObjects::offset_seconds(&day, Tz::UTC), 0);
}
