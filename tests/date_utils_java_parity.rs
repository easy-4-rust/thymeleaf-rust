//! `DateUtils` Java Golden 差分测试。
//!
//! 覆盖：create 参数校验、create_now/create_today、format、
//! day/month/year/hour/minute/second/millisecond 组件提取、
//! format_iso 和 TemplateValue 往返。

use std::sync::Arc;

use thymeleaf::expression::TemplateValue;
use thymeleaf::util::{DateUtils, JavaDate, JavaLocale, JavaString};

fn js(s: &str) -> JavaString {
    JavaString::from_rust_str(s)
}

fn locale_en() -> JavaLocale {
    JavaLocale::new(js("en"), js("US"))
}

fn date_of(year: i32, month: i32, day: i32) -> JavaDate {
    DateUtils::create(
        Some(year),
        Some(month),
        Some(day),
        None,
        None,
        None,
        None,
        None,
        Some(&locale_en()),
    )
    .expect("date creation must succeed")
}

// ===========================================================================
// 1. create 参数校验
// ===========================================================================

#[test]
fn create_with_all_fields() {
    let d = DateUtils::create(
        Some(2024),
        Some(5),
        Some(17),
        Some(10),
        Some(30),
        Some(45),
        Some(123),
        Some("UTC"),
        Some(&locale_en()),
    )
    .expect("create with all fields");
    assert_eq!(DateUtils::year(Some(&d)), Some(2024));
    assert_eq!(DateUtils::month(Some(&d)), Some(5));
    assert_eq!(DateUtils::day(Some(&d)), Some(17));
    assert_eq!(DateUtils::hour(Some(&d)), Some(10));
    assert_eq!(DateUtils::minute(Some(&d)), Some(30));
    assert_eq!(DateUtils::second(Some(&d)), Some(45));
    assert_eq!(DateUtils::millisecond(Some(&d)), Some(123));
}

#[test]
fn create_null_year_errors() {
    assert!(
        DateUtils::create(
            None,
            Some(1),
            Some(1),
            None,
            None,
            None,
            None,
            None,
            Some(&locale_en())
        )
        .is_err()
    );
}

#[test]
fn create_null_month_errors() {
    assert!(
        DateUtils::create(
            Some(2024),
            None,
            Some(1),
            None,
            None,
            None,
            None,
            None,
            Some(&locale_en())
        )
        .is_err()
    );
}

#[test]
fn create_null_day_errors() {
    assert!(
        DateUtils::create(
            Some(2024),
            Some(1),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&locale_en())
        )
        .is_err()
    );
}

#[test]
fn create_hour_without_minute_errors() {
    assert!(
        DateUtils::create(
            Some(2024),
            Some(1),
            Some(1),
            Some(10),
            None,
            None,
            None,
            None,
            Some(&locale_en())
        )
        .is_err()
    );
}

#[test]
fn create_minute_without_hour_errors() {
    assert!(
        DateUtils::create(
            Some(2024),
            Some(1),
            Some(1),
            None,
            Some(10),
            None,
            None,
            None,
            Some(&locale_en())
        )
        .is_err()
    );
}

#[test]
fn create_second_without_hour_minute_errors() {
    assert!(
        DateUtils::create(
            Some(2024),
            Some(1),
            Some(1),
            None,
            None,
            Some(30),
            None,
            None,
            Some(&locale_en())
        )
        .is_err()
    );
}

#[test]
fn create_millisecond_without_second_errors() {
    assert!(
        DateUtils::create(
            Some(2024),
            Some(1),
            Some(1),
            Some(10),
            Some(30),
            None,
            Some(500),
            None,
            Some(&locale_en())
        )
        .is_err()
    );
}

#[test]
fn create_month_rollover() {
    // Calendar lenient：month 13 滚动到下一年
    let d = DateUtils::create(
        Some(2024),
        Some(13),
        Some(1),
        None,
        None,
        None,
        None,
        None,
        Some(&locale_en()),
    )
    .unwrap();
    assert_eq!(DateUtils::year(Some(&d)), Some(2025));
    assert_eq!(DateUtils::month(Some(&d)), Some(1));
}

#[test]
fn create_month_zero_rollover() {
    // Calendar lenient：month 0 回退到上一年 12 月
    let d = DateUtils::create(
        Some(2024),
        Some(0),
        Some(1),
        None,
        None,
        None,
        None,
        None,
        Some(&locale_en()),
    )
    .unwrap();
    assert_eq!(DateUtils::year(Some(&d)), Some(2023));
    assert_eq!(DateUtils::month(Some(&d)), Some(12));
}

// ===========================================================================
// 2. create_now / create_today
// ===========================================================================

#[test]
fn create_now_returns_current_time() {
    let d = DateUtils::create_now(Some("UTC"), Some(&locale_en()));
    assert!(d.time_in_millis() > 0);
}

#[test]
fn create_today_has_midnight_time() {
    let d = DateUtils::create_today(Some("UTC"), Some(&locale_en()));
    assert_eq!(DateUtils::hour(Some(&d)), Some(0));
    assert_eq!(DateUtils::minute(Some(&d)), Some(0));
    assert_eq!(DateUtils::second(Some(&d)), Some(0));
}

// ===========================================================================
// 3. 组件提取
// ===========================================================================

#[test]
fn year_of_known_date() {
    let d = date_of(2024, 5, 17);
    assert_eq!(DateUtils::year(Some(&d)), Some(2024));
}

#[test]
fn month_of_known_date() {
    let d = date_of(2024, 5, 17);
    assert_eq!(DateUtils::month(Some(&d)), Some(5));
}

#[test]
fn day_of_known_date() {
    let d = date_of(2024, 5, 17);
    assert_eq!(DateUtils::day(Some(&d)), Some(17));
}

#[test]
fn hour_of_known_date() {
    let d = DateUtils::create(
        Some(2024),
        Some(5),
        Some(17),
        Some(8),
        Some(15),
        None,
        None,
        Some("UTC"),
        Some(&locale_en()),
    )
    .unwrap();
    assert_eq!(DateUtils::hour(Some(&d)), Some(8));
}

#[test]
fn minute_of_known_date() {
    let d = DateUtils::create(
        Some(2024),
        Some(5),
        Some(17),
        Some(8),
        Some(15),
        None,
        None,
        Some("UTC"),
        Some(&locale_en()),
    )
    .unwrap();
    assert_eq!(DateUtils::minute(Some(&d)), Some(15));
}

#[test]
fn second_of_known_date() {
    let d = DateUtils::create(
        Some(2024),
        Some(5),
        Some(17),
        Some(8),
        Some(15),
        Some(30),
        None,
        Some("UTC"),
        Some(&locale_en()),
    )
    .unwrap();
    assert_eq!(DateUtils::second(Some(&d)), Some(30));
}

#[test]
fn millisecond_of_known_date() {
    let d = DateUtils::create(
        Some(2024),
        Some(5),
        Some(17),
        Some(8),
        Some(15),
        Some(30),
        Some(250),
        Some("UTC"),
        Some(&locale_en()),
    )
    .unwrap();
    assert_eq!(DateUtils::millisecond(Some(&d)), Some(250));
}

#[test]
fn components_of_null_are_none() {
    assert_eq!(DateUtils::year(None), None);
    assert_eq!(DateUtils::month(None), None);
    assert_eq!(DateUtils::day(None), None);
    assert_eq!(DateUtils::hour(None), None);
    assert_eq!(DateUtils::minute(None), None);
    assert_eq!(DateUtils::second(None), None);
    assert_eq!(DateUtils::millisecond(None), None);
}

// ===========================================================================
// 4. day_of_week / 名称
// ===========================================================================

#[test]
fn day_of_week_of_known_date() {
    // 2024-05-17 是星期五
    let d = date_of(2024, 5, 17);
    assert_eq!(DateUtils::day_of_week(Some(&d)), Some(6)); // Calendar.FRIDAY = 6
}

#[test]
fn day_of_week_name_of_known_date() {
    let d = date_of(2024, 5, 17);
    let name = DateUtils::day_of_week_name(Some(&d), Some(&locale_en()))
        .unwrap()
        .unwrap();
    assert_eq!(name.to_string_lossy(), "Friday");
}

#[test]
fn day_of_week_name_short_of_known_date() {
    let d = date_of(2024, 5, 17);
    let name = DateUtils::day_of_week_name_short(Some(&d), Some(&locale_en()))
        .unwrap()
        .unwrap();
    assert_eq!(name.to_string_lossy(), "Fri");
}

#[test]
fn month_name_of_known_date() {
    let d = date_of(2024, 5, 17);
    let name = DateUtils::month_name(Some(&d), Some(&locale_en()))
        .unwrap()
        .unwrap();
    assert_eq!(name.to_string_lossy(), "May");
}

#[test]
fn month_name_short_of_known_date() {
    let d = date_of(2024, 5, 17);
    let name = DateUtils::month_name_short(Some(&d), Some(&locale_en()))
        .unwrap()
        .unwrap();
    assert_eq!(name.to_string_lossy(), "May");
}

// ===========================================================================
// 5. format
// ===========================================================================

#[test]
fn format_with_pattern() {
    let d = date_of(2024, 5, 17);
    let pattern = js("yyyy-MM-dd");
    let formatted = DateUtils::format(Some(&d), Some(&pattern), Some(&locale_en()))
        .unwrap()
        .unwrap();
    assert_eq!(formatted.to_string_lossy(), "2024-05-17");
}

#[test]
fn format_with_time_pattern() {
    let d = DateUtils::create(
        Some(2024),
        Some(5),
        Some(17),
        Some(8),
        Some(15),
        Some(30),
        Some(0),
        Some("UTC"),
        Some(&locale_en()),
    )
    .unwrap();
    let pattern = js("HH:mm:ss");
    let formatted = DateUtils::format(Some(&d), Some(&pattern), Some(&locale_en()))
        .unwrap()
        .unwrap();
    assert_eq!(formatted.to_string_lossy(), "08:15:30");
}

#[test]
fn format_null_target_returns_none() {
    let pattern = js("yyyy");
    assert!(
        DateUtils::format(None, Some(&pattern), Some(&locale_en()))
            .unwrap()
            .is_none()
    );
}

#[test]
fn format_empty_pattern_errors() {
    let d = date_of(2024, 5, 17);
    let pattern = js("");
    assert!(DateUtils::format(Some(&d), Some(&pattern), Some(&locale_en())).is_err());
}

#[test]
fn format_null_locale_errors() {
    let d = date_of(2024, 5, 17);
    let pattern = js("yyyy");
    assert!(DateUtils::format(Some(&d), Some(&pattern), None).is_err());
}

#[test]
fn format_iso_known_date() {
    let d = date_of(2024, 5, 17);
    let iso = DateUtils::format_iso(Some(&d)).unwrap();
    assert!(iso.to_string_lossy().starts_with("2024-05-17"));
}

// ===========================================================================
// 6. TemplateValue 往返
// ===========================================================================

#[test]
fn from_and_into_template_value_roundtrip() {
    let d = date_of(2024, 5, 17);
    let value = DateUtils::into_template_value(d.clone());
    let back = DateUtils::from_template_value(Some(&value))
        .expect("convert back")
        .expect("non-null");
    assert_eq!(DateUtils::year(Some(back)), Some(2024));
    assert_eq!(DateUtils::month(Some(back)), Some(5));
    assert_eq!(DateUtils::day(Some(back)), Some(17));
}

#[test]
fn from_template_value_null_returns_none() {
    let value = Arc::new(TemplateValue::Null);
    let result = DateUtils::from_template_value(Some(&value)).expect("no error for null");
    assert!(result.is_none());
}

#[test]
fn from_template_value_non_date_errors() {
    let value = Arc::new(TemplateValue::string(js("not a date")));
    assert!(DateUtils::from_template_value(Some(&value)).is_err());
}

#[test]
fn from_template_value_none_returns_none() {
    let result = DateUtils::from_template_value(None).expect("no error for none");
    assert!(result.is_none());
}
