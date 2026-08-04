//! 日历工具 —— 对应 Java `business/util/CalendarUtil.java`。
//!
//! Java `Calendar.getInstance()` 使用 JVM 默认时区；Rust 引擎默认时区为 UTC，
//! 因此示例统一使用 UTC 构造 Calendar 等价值（引擎内部格式化为该时区的显示名）。

use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use thymeleaf::util::DateValue;

/// Java `CalendarUtil#calendarFor(year, month, day, hour, minute)`。
///
/// month 按 Java `Calendar.MONTH` 语义从 1 起（内部减 1）；秒与毫秒清零。
#[must_use]
pub fn calendar_for(year: i32, month: i32, day: i32, hour: i32, minute: i32) -> DateValue {
    let instant = Utc
        .with_ymd_and_hms(
            year,
            month as u32,
            day as u32,
            hour as u32,
            minute as u32,
            0,
        )
        .single()
        .expect("valid calendar date");
    DateValue::calendar(instant, Tz::UTC)
}
