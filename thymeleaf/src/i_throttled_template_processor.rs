use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::TemplateSpec;
use crate::engine::IThrottledTemplateWriterControl;
use crate::exceptions::TemplateEngineException;
use crate::util::{Charset, JavaWriter, Utf16String};

/// 节流模板处理调用的统一错误结果。
///
/// Java API 通过 `TemplateOutputException`、`TemplateProcessingException` 等运行时
/// 异常保留具体子类型；Rust 使用公共 `TemplateEngineException` trait object 保持
/// 同样的动态错误分类。
pub type ThrottledTemplateResult<T> =
    Result<T, Box<dyn TemplateEngineException + Send + Sync + 'static>>;

/// 节流处理器完成状态的线程安全观察句柄。
///
/// Java 允许在处理线程之外并发调用
/// `IThrottledTemplateProcessor#isFinished()`。Rust 的事件处理链保持线程亲和，因此
/// 通过可克隆句柄提供同一可观察能力：处理器仍由单线程非交错驱动，任意观察线程可
/// 无锁检查完成状态。
///
/// 对应 Java: `IThrottledTemplateProcessor#isFinished()` 的跨线程观察能力。
#[derive(Clone, Debug)]
pub struct ThrottledTemplateStatus {
    finished: Arc<AtomicBool>,
}

impl ThrottledTemplateStatus {
    /// 从处理器共享的完成标志创建观察句柄。
    #[must_use]
    /// 对应 Java 语义：`IThrottledTemplateProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(finished: Arc<AtomicBool>) -> Self {
        Self { finished }
    }

    /// 判断模板处理是否已经全部完成。
    ///
    /// 返回 `true` 时，创建该句柄的处理器已经完成全部事件、待处理工作和溢出输出。
    /// Acquire 读取与处理器的 Release 写入配对，保证此前结果对观察线程可见。
    #[must_use]
    /// 对应 Java: `IThrottledTemplateProcessor#isFinished()`。
    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }
}

/// 调节模板引擎输出速率的处理器合同。
///
/// 调用方为每次执行提供能够接收的最大字符数或字节数，从而对模板执行施加背压。
/// 同一处理器的 `process` 调用不得并发，但 `is_finished` 必须支持跨线程可见性。
///
/// 对应 Java: `org.thymeleaf.IThrottledTemplateProcessor`。
pub trait IThrottledTemplateProcessor {
    /// 返回与当前处理器共享同一 Writer 状态的控制器。
    ///
    /// 数据驱动迭代器使用该控制器观察溢出状态，并在 SSE 模式下标记事件边界。
    /// 对应 Java: `ThrottledTemplateProcessor#getThrottledTemplateWriterControl()`。
    fn get_throttled_template_writer_control(
        &self,
    ) -> Box<dyn IThrottledTemplateWriterControl + Send>;

    /// 返回用于跨线程追踪处理器执行的稳定标识。
    ///
    /// 不要求构造级绝对唯一，但必须足以区分日志中的不同处理器执行。
    /// 对应 Java: `IThrottledTemplateProcessor#getProcessorIdentifier()`。
    fn get_processor_identifier(&self) -> &Utf16String;

    /// 返回本处理器正在执行的模板规格。
    ///
    /// 对应 Java: `IThrottledTemplateProcessor#getTemplateSpec()`。
    fn get_template_spec(&self) -> &TemplateSpec;

    /// 判断全部模板事件、待处理器工作和 Writer 溢出是否已经完成。
    ///
    /// 该观察允许与非并发的处理调用来自不同线程，实现必须保证完成状态的跨线程
    /// 可见性。返回 `true` 后继续处理必须返回零且不得重复输出。
    /// 对应 Java: `IThrottledTemplateProcessor#isFinished()`。
    fn is_finished(&self) -> bool;

    /// 返回可由其他线程并发观察的完成状态句柄。
    ///
    /// Rust 的处理器内部事件链保持线程亲和；调用方在开始处理前取得此句柄，即可
    /// 保留 Java `isFinished()` 的并发短路检查能力，而不允许并发执行 `process`。
    fn get_completion_status(&self) -> ThrottledTemplateStatus;

    /// 不限制字符数，处理全部剩余模板并返回本次写出的 UTF-16 代码单元数。
    ///
    /// `writer` 接收输出并在本次调用结束时刷新；输出或模板处理失败时返回保留具体
    /// 子类型的引擎错误。对应 Java: `IThrottledTemplateProcessor#processAll(Writer)`。
    fn process_all_writer(&mut self, writer: Box<dyn JavaWriter>) -> ThrottledTemplateResult<i32>;

    /// 不限制字节数，按指定字符集处理全部剩余模板并返回写出字节数。
    ///
    /// `output_stream` 接收编码后的字节，`charset` 决定编码；输出或模板处理失败时
    /// 返回保留具体子类型的引擎错误。
    /// 对应 Java: `IThrottledTemplateProcessor#processAll(OutputStream, Charset)`。
    fn process_all_output_stream(
        &mut self,
        output_stream: Box<dyn Write + Send>,
        charset: &Charset,
    ) -> ThrottledTemplateResult<i32>;

    /// 最多写出 `max_output_in_chars` 个 UTF-16 代码单元。
    ///
    /// 负数或 `i32::MAX` 表示不设上限；零表示本次不推进。
    /// 返回本次实际写出的 UTF-16 代码单元数。
    /// 对应 Java: `IThrottledTemplateProcessor#process(int, Writer)`。
    fn process_writer(
        &mut self,
        max_output_in_chars: i32,
        writer: Box<dyn JavaWriter>,
    ) -> ThrottledTemplateResult<i32>;

    /// 最多按指定字符集写出 `max_output_in_bytes` 个字节。
    ///
    /// 负数或 `i32::MAX` 表示不设上限；零表示本次不推进。
    /// 返回本次实际写出的字节数；同一处理器不能在字符输出与字节输出之间切换。
    /// 对应 Java: `IThrottledTemplateProcessor#process(int, OutputStream, Charset)`。
    fn process_output_stream(
        &mut self,
        max_output_in_bytes: i32,
        output_stream: Box<dyn Write + Send>,
        charset: &Charset,
    ) -> ThrottledTemplateResult<i32>;
}
