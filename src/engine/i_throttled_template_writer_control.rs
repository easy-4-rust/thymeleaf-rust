use std::io;

use super::ISSEThrottledTemplateWriterControl;

/// 节流模板输出 Writer 的状态监控合同。
///
/// 对应 Java: `org.thymeleaf.engine.IThrottledTemplateWriterControl`。
///
/// 该内部接口允许处理循环观察溢出/停止状态以及累计写入和溢出扩容指标。
/// `isOverflown` 与 `isStopped` 可能触发底层 Writer I/O，因此保留可失败返回。
pub trait IThrottledTemplateWriterControl {
    /// 若控制器支持 SSE 事件边界，则返回对应扩展接口。
    fn as_sse_control(&mut self) -> Option<&mut dyn ISSEThrottledTemplateWriterControl> {
        None
    }

    /// 判断 Writer 当前是否已超过本轮允许写入量。
    ///
    /// 对应 Java: `IThrottledTemplateWriterControl#isOverflown()`。
    ///
    /// # 返回
    ///
    /// 已溢出时返回 `true`。
    ///
    /// # 错误
    ///
    /// 查询底层 Writer 状态发生 I/O 错误时返回 [`io::Error`]。
    fn is_overflown(&mut self) -> io::Result<bool>;

    /// 判断 Writer 是否已被停止。
    ///
    /// 对应 Java: `IThrottledTemplateWriterControl#isStopped()`。
    ///
    /// # 返回
    ///
    /// 已停止时返回 `true`。
    ///
    /// # 错误
    ///
    /// 查询底层 Writer 状态发生 I/O 错误时返回 [`io::Error`]。
    fn is_stopped(&mut self) -> io::Result<bool>;

    /// 返回当前已写入的 Java 字符数。对应 Java `getWrittenCount()`。
    fn get_written_count(&self) -> i32;

    /// 返回允许的最大溢出字符数。对应 Java `getMaxOverflowSize()`。
    fn get_max_overflow_size(&self) -> i32;

    /// 返回溢出缓冲区扩容次数。对应 Java `getOverflowGrowCount()`。
    fn get_overflow_grow_count(&self) -> i32;
}
