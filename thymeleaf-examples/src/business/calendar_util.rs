//! 日历工具 —— 对应 Java `business/util/CalendarUtil.java`。
//!
//! Java `Calendar.getInstance()` 使用 JVM 默认时区；Rust 引擎默认时区为 UTC，
//! 因此示例统一使用 UTC 构造 Calendar 等价值（引擎内部格式化为该时区的显示名）。

use jiff::civil::DateTime;
use jiff::tz::TimeZone;
use thymeleaf::util::DateValue;

/// Java `CalendarUtil#calendarFor(year, month, day, hour, minute)`。
///
/// month 按 Java `Calendar.MONTH` 语义从 1 起（内部减 1）；秒与毫秒清零。
#[must_use]
pub fn calendar_for(year: i32, month: i32, day: i32, hour: i32, minute: i32) -> DateValue {
    let date_time = DateTime::new(
        year.try_into().expect("year"),
        month.try_into().expect("month"),
        day.try_into().expect("day"),
        hour.try_into().expect("hour"),
        minute.try_into().expect("minute"),
        0,
        0,
    )
    .expect("valid calendar date");
    let instant = TimeZone::UTC.to_timestamp(date_time).expect("UTC date");
    DateValue::calendar(instant, TimeZone::UTC)
}
