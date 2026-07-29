use std::io;

use super::IThrottledTemplateWriterControl;

/// SSE 节流输出 writer 的事件边界控制合同。
///
/// 对应 Java:
/// `org.thymeleaf.engine.ISSEThrottledTemplateWriterControl`。
///
/// 该接口供引擎内部在普通节流状态监控之上标记 Server-Sent Events 的开始和结束，
/// 通常不直接暴露给模板应用代码。
pub trait ISSEThrottledTemplateWriterControl: IThrottledTemplateWriterControl {
    /// 开始一个 SSE 事件。
    ///
    /// # 参数
    ///
    /// - `id`：可空事件 ID 的 UTF-16 `char[]`。
    /// - `event`：可空事件类型的 UTF-16 `char[]`。
    fn start_event(&mut self, id: Option<&[u16]>, event: Option<&[u16]>);

    /// 结束当前 SSE 事件并完成可能待写出的边界内容。
    ///
    /// # 错误
    ///
    /// 底层 Java `Writer` 对应输出失败时返回 I/O 错误。
    fn end_event(&mut self) -> io::Result<()>;
}
