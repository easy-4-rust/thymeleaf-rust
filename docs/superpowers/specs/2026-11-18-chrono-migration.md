# 日期与时间依赖迁移（Issue #57）

核心和测试使用 Jiff 的 civil 日期、UTC 瞬时及命名时区，启用内置 IANA 数据。保持 `DateValue`、`TemporalValue`、`#dates`、`#temporals` 的名称、枚举分支与模板调用。原先公开的 Chrono 类型改为 Jiff 等价值。发布最小版本为 0.2.0，按公开类型身份变化处理，其他 API 差异需单独审查。

| 旧载荷 | 新载荷 |
|---|---|
| `DateTime<Utc>` | `jiff::Timestamp` |
| `NaiveDate` / `NaiveDateTime` / `NaiveTime` | `jiff::civil::{Date, DateTime, Time}` |
| `Tz` / `FixedOffset` | `jiff::tz::{TimeZone, Offset}` |
| 带固定偏移或命名时区的日期时间 | `jiff::ZonedDateTime`，固定偏移使用 `Offset::to_time_zone()` 构造 |

Java 3.1.5 差分语料和时间格式输出保持原有断言。保留现有夏令时重叠、跳时处理以及旧 Token RFC 3339 时间戳的可读取性。Jiff civil 日期支持 −9999 到 9999 年，超出范围的日期创建返回错误。Vernal 表达式和 Sa-Token 副本同步移除 Chrono。

验收以完整 Workspace 依赖图无 `chrono`/`chrono-tz`、Ubuntu/macOS CI、API 差分、Java 语义回归及 RustSec 审计为准。两条例外在验证候选后删除，再验证最终提交。
