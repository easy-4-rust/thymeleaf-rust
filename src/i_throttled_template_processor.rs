use std::io::Write;

use crate::TemplateSpec;
use crate::exceptions::TemplateEngineException;
use crate::util::{Charset, JavaString, JavaWriter};

/// 节流模板处理调用的统一错误结果。
///
/// Java API 通过 `TemplateOutputException`、`TemplateProcessingException` 等运行时
/// 异常保留具体子类型；Rust 使用公共 `TemplateEngineException` trait object 保持
/// 同样的动态错误分类。
pub type ThrottledTemplateResult<T> =
    Result<T, Box<dyn TemplateEngineException + Send + Sync + 'static>>;

/// 调节模板引擎输出速率的处理器合同。
///
/// 调用方为每次执行提供能够接收的最大字符数或字节数，从而对模板执行施加背压。
/// 同一处理器的 `process` 调用不得并发，但 `is_finished` 必须支持跨线程可见性。
///
/// 对应 Java: `org.thymeleaf.IThrottledTemplateProcessor`。
pub trait IThrottledTemplateProcessor {
    /// 返回用于跨线程追踪处理器执行的稳定标识。
    fn get_processor_identifier(&self) -> &JavaString;

    /// 返回本处理器正在执行的模板规格。
    fn get_template_spec(&self) -> &TemplateSpec;

    /// 判断全部模板事件、待处理器工作和 Writer 溢出是否已经完成。
    fn is_finished(&self) -> bool;

    /// 不限制字符数，处理全部剩余模板并返回本次写出的 UTF-16 代码单元数。
    fn process_all_writer(&mut self, writer: &mut dyn JavaWriter) -> ThrottledTemplateResult<i32>;

    /// 不限制字节数，按指定字符集处理全部剩余模板并返回写出字节数。
    fn process_all_output_stream(
        &mut self,
        output_stream: &mut dyn Write,
        charset: &Charset,
    ) -> ThrottledTemplateResult<i32>;

    /// 最多写出 `max_output_in_chars` 个 UTF-16 代码单元。
    ///
    /// 负数或 `i32::MAX` 表示不设上限；零表示本次不推进。
    fn process_writer(
        &mut self,
        max_output_in_chars: i32,
        writer: &mut dyn JavaWriter,
    ) -> ThrottledTemplateResult<i32>;

    /// 最多按指定字符集写出 `max_output_in_bytes` 个字节。
    ///
    /// 负数或 `i32::MAX` 表示不设上限；零表示本次不推进。
    fn process_output_stream(
        &mut self,
        max_output_in_bytes: i32,
        output_stream: &mut dyn Write,
        charset: &Charset,
    ) -> ThrottledTemplateResult<i32>;
}
