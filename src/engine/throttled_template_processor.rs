use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::context::IEngineContext;
use crate::exceptions::{TemplateEngineException, TemplateOutputException};
use crate::model::IModel;
use crate::util::{Charset, JavaString, JavaWriter};
use crate::{
    IThrottledTemplateProcessor, TemplateSpec, ThrottledTemplateResult, ThrottledTemplateStatus,
};

use super::engine_context_manager::EngineContextManager;
use super::i_throttled_template_writer_control::IThrottledTemplateWriterControl;
use super::isse_throttled_template_writer_control::ISSEThrottledTemplateWriterControl;
use super::sse_throttled_template_writer::SSEThrottledTemplateWriter;
use super::template_flow_controller::TemplateFlowController;
use super::throttled_template_writer::ThrottledTemplateWriter;
use super::{ITemplateHandler, ProcessorTemplateHandler, TemplateModel};

static IDENTIFIER_GENERATOR: AtomicI64 = AtomicI64::new(0);

/// 支持字符/字节背压及 SSE 输出的标准节流模板处理器。
///
/// 对应 Java: `org.thymeleaf.engine.ThrottledTemplateProcessor`。
pub struct ThrottledTemplateProcessor {
    identifier: JavaString,
    template_spec: TemplateSpec,
    context: Arc<dyn IEngineContext>,
    template_model: Arc<TemplateModel>,
    template_handler: Box<dyn ITemplateHandler>,
    processor_template_handler: ProcessorTemplateHandler,
    flow_controller: Arc<Mutex<TemplateFlowController>>,
    writer: Arc<Mutex<ThrottledWriter>>,
    offset: usize,
    event_processing_finished: bool,
    all_processing_finished: Arc<AtomicBool>,
}

impl ThrottledTemplateProcessor {
    /// 创建完整节流执行状态；模板上下文只在全部事件消费后释放。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        template_spec: TemplateSpec,
        context: Arc<dyn IEngineContext>,
        template_model: Arc<TemplateModel>,
        template_handler: Box<dyn ITemplateHandler>,
        processor_template_handler: ProcessorTemplateHandler,
        flow_controller: Arc<Mutex<TemplateFlowController>>,
        writer: Arc<Mutex<ThrottledWriter>>,
    ) -> Self {
        let identifier = IDENTIFIER_GENERATOR.fetch_add(1, Ordering::Relaxed);
        Self {
            identifier: JavaString::from_rust_str(&identifier.to_string()),
            template_spec,
            context,
            template_model,
            template_handler,
            processor_template_handler,
            flow_controller,
            writer,
            offset: 0,
            event_processing_finished: false,
            all_processing_finished: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 创建普通或 SSE Writer 的共享状态。
    pub(crate) fn create_writer(
        template_name: String,
        flow_controller: Arc<Mutex<TemplateFlowController>>,
        output_sse: bool,
    ) -> Arc<Mutex<ThrottledWriter>> {
        Arc::new(Mutex::new(if output_sse {
            ThrottledWriter::Sse(SSEThrottledTemplateWriter::new(
                template_name,
                flow_controller,
            ))
        } else {
            ThrottledWriter::Standard(ThrottledTemplateWriter::new(template_name, flow_controller))
        }))
    }

    /// 创建交给 OutputTemplateHandler 的共享 Writer 代理。
    pub(crate) fn writer_proxy(writer: Arc<Mutex<ThrottledWriter>>) -> Box<dyn JavaWriter> {
        Box::new(SharedThrottledWriter { writer })
    }

    fn compute_finish(&mut self) -> Result<bool, TemplateOutputException> {
        if self.all_processing_finished.load(Ordering::Acquire) {
            return Ok(true);
        }
        let pending = lock(&self.flow_controller).processor_template_handler_pending;
        let overflown = lock(&self.writer)
            .is_overflown()
            .map_err(|error| self.output_error("An error happened while checking output", error))?;
        let finished = self.event_processing_finished && !pending && !overflown;
        if finished {
            self.all_processing_finished.store(true, Ordering::Release);
        }
        Ok(finished)
    }

    fn process_internal(&mut self, max_output: i32) -> ThrottledTemplateResult<i32> {
        if self.all_processing_finished.load(Ordering::Acquire) || max_output == 0 {
            return Ok(0);
        }
        let initial_written_count = lock(&self.writer).get_written_count();
        let allow_result = { lock(&self.writer).allow(max_output) };
        if let Err(error) = allow_result {
            return self.fail(error);
        }
        if !self.compute_finish().map_err(box_engine_error)?
            && !lock(&self.writer).is_stopped().map_err(|error| {
                box_engine_error(
                    self.output_error("An error happened while checking throttled output", error),
                )
            })?
        {
            // 先读取 pending 标志并立即释放 flow controller 锁：若把临时
            // MutexGuard 留在 if 条件的短路链里，guard 会存活到整个 if
            // 表达式结束，handle_pending 处理事件链时会通过
            // ThrottledTemplateWriterWriterAdapter 重入同一 Mutex 而死锁。
            let handler_pending = lock(&self.flow_controller).processor_template_handler_pending;
            if handler_pending
                && let Err(error) = self.processor_template_handler.handle_pending()
            {
                return self.fail_boxed(error);
            }
            if !self.compute_finish().map_err(box_engine_error)?
                && !lock(&self.writer).is_stopped().map_err(|error| {
                    box_engine_error(
                        self.output_error(
                            "An error happened while checking throttled output",
                            error,
                        ),
                    )
                })?
            {
                let processed_result = self.template_model.process_throttled(
                    self.template_handler.as_mut(),
                    self.offset,
                    Some(&self.flow_controller),
                );
                let processed = match processed_result {
                    Ok(processed) => processed,
                    Err(error) => return self.fail_boxed(error),
                };
                self.offset += processed;
                if self.offset == self.template_model.size() {
                    EngineContextManager::dispose_engine_context(self.context.as_ref());
                    self.event_processing_finished = true;
                    self.compute_finish().map_err(box_engine_error)?;
                }
            }
        }
        let flush_result = { lock(&self.writer).flush() };
        if let Err(error) = flush_result {
            return self
                .fail(self.output_error("An error happened while flushing output writer", error));
        }
        Ok(lock(&self.writer)
            .get_written_count()
            .wrapping_sub(initial_written_count))
    }

    fn output_error(&self, message: &str, cause: io::Error) -> TemplateOutputException {
        TemplateOutputException::new(
            Some(message.to_owned()),
            Some(self.template_spec.get_template().to_owned()),
            -1,
            -1,
            cause,
        )
    }

    fn fail<E>(&mut self, error: E) -> ThrottledTemplateResult<i32>
    where
        E: TemplateEngineException,
    {
        self.event_processing_finished = true;
        self.all_processing_finished.store(true, Ordering::Release);
        Err(Box::new(error))
    }

    fn fail_boxed(
        &mut self,
        error: Box<dyn TemplateEngineException>,
    ) -> ThrottledTemplateResult<i32> {
        self.event_processing_finished = true;
        self.all_processing_finished.store(true, Ordering::Release);
        Err(Box::new(ThrottledEngineCause(error)))
    }
}

impl IThrottledTemplateProcessor for ThrottledTemplateProcessor {
    fn get_throttled_template_writer_control(
        &self,
    ) -> Box<dyn IThrottledTemplateWriterControl + Send> {
        Box::new(SharedThrottledWriterControl {
            writer: Arc::clone(&self.writer),
        })
    }

    fn get_processor_identifier(&self) -> &JavaString {
        &self.identifier
    }

    fn get_template_spec(&self) -> &TemplateSpec {
        &self.template_spec
    }

    fn is_finished(&self) -> bool {
        // Java 明确允许该观察与非并发的 process 调用来自不同线程；Acquire/Release
        // 保证完成标志及此前模板处理写入对观察线程可见。
        self.all_processing_finished.load(Ordering::Acquire)
    }

    fn get_completion_status(&self) -> ThrottledTemplateStatus {
        ThrottledTemplateStatus::new(Arc::clone(&self.all_processing_finished))
    }

    fn process_all_writer(&mut self, writer: Box<dyn JavaWriter>) -> ThrottledTemplateResult<i32> {
        lock(&self.writer)
            .set_output_writer(writer)
            .map_err(box_engine_error)?;
        self.process_internal(i32::MAX)
    }

    fn process_all_output_stream(
        &mut self,
        output_stream: Box<dyn Write + Send>,
        charset: &Charset,
    ) -> ThrottledTemplateResult<i32> {
        lock(&self.writer)
            .set_output_stream(output_stream, charset, i32::MAX)
            .map_err(box_engine_error)?;
        self.process_internal(i32::MAX)
    }

    fn process_writer(
        &mut self,
        max_output_in_chars: i32,
        writer: Box<dyn JavaWriter>,
    ) -> ThrottledTemplateResult<i32> {
        lock(&self.writer)
            .set_output_writer(writer)
            .map_err(box_engine_error)?;
        self.process_internal(max_output_in_chars)
    }

    fn process_output_stream(
        &mut self,
        max_output_in_bytes: i32,
        output_stream: Box<dyn Write + Send>,
        charset: &Charset,
    ) -> ThrottledTemplateResult<i32> {
        lock(&self.writer)
            .set_output_stream(output_stream, charset, max_output_in_bytes)
            .map_err(box_engine_error)?;
        self.process_internal(max_output_in_bytes)
    }
}

pub(crate) enum ThrottledWriter {
    Standard(ThrottledTemplateWriter),
    Sse(SSEThrottledTemplateWriter),
}

impl ThrottledWriter {
    fn set_output_writer(
        &mut self,
        writer: Box<dyn JavaWriter>,
    ) -> Result<(), TemplateOutputException> {
        match self {
            Self::Standard(value) => value.set_output_writer(writer),
            Self::Sse(value) => value.set_output_writer(writer),
        }
    }

    fn set_output_stream(
        &mut self,
        output_stream: Box<dyn Write + Send>,
        charset: &Charset,
        max_output_in_bytes: i32,
    ) -> Result<(), TemplateOutputException> {
        match self {
            Self::Standard(value) => {
                value.set_output_stream(output_stream, charset, max_output_in_bytes)
            }
            Self::Sse(value) => {
                value.set_output_stream(output_stream, charset, max_output_in_bytes)
            }
        }
    }

    fn allow(&mut self, limit: i32) -> Result<(), TemplateOutputException> {
        match self {
            Self::Standard(value) => value.allow(limit),
            Self::Sse(value) => value.allow(limit),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Standard(value) => value.flush(),
            Self::Sse(value) => value.flush(),
        }
    }
}

impl JavaWriter for ThrottledWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        match self {
            Self::Standard(value) => value.write_utf16(characters),
            Self::Sse(value) => value.write_utf16(characters),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Self::flush(self)
    }

    fn close(&mut self) -> io::Result<()> {
        match self {
            Self::Standard(value) => value.close(),
            Self::Sse(value) => value.close(),
        }
    }
}

impl IThrottledTemplateWriterControl for ThrottledWriter {
    fn is_overflown(&mut self) -> io::Result<bool> {
        match self {
            Self::Standard(value) => value.is_overflown(),
            Self::Sse(value) => value.is_overflown(),
        }
    }

    fn is_stopped(&mut self) -> io::Result<bool> {
        match self {
            Self::Standard(value) => value.is_stopped(),
            Self::Sse(value) => value.is_stopped(),
        }
    }

    fn get_written_count(&self) -> i32 {
        match self {
            Self::Standard(value) => value.get_written_count(),
            Self::Sse(value) => value.get_written_count(),
        }
    }

    fn get_max_overflow_size(&self) -> i32 {
        match self {
            Self::Standard(value) => value.get_max_overflow_size(),
            Self::Sse(value) => value.get_max_overflow_size(),
        }
    }

    fn get_overflow_grow_count(&self) -> i32 {
        match self {
            Self::Standard(value) => value.get_overflow_grow_count(),
            Self::Sse(value) => value.get_overflow_grow_count(),
        }
    }
}

struct SharedThrottledWriter {
    writer: Arc<Mutex<ThrottledWriter>>,
}

struct SharedThrottledWriterControl {
    writer: Arc<Mutex<ThrottledWriter>>,
}

impl IThrottledTemplateWriterControl for SharedThrottledWriterControl {
    fn as_sse_control(&mut self) -> Option<&mut dyn ISSEThrottledTemplateWriterControl> {
        let is_sse = matches!(*lock(&self.writer), ThrottledWriter::Sse(_));
        is_sse.then_some(self)
    }

    fn is_overflown(&mut self) -> io::Result<bool> {
        lock(&self.writer).is_overflown()
    }

    fn is_stopped(&mut self) -> io::Result<bool> {
        lock(&self.writer).is_stopped()
    }

    fn get_written_count(&self) -> i32 {
        lock(&self.writer).get_written_count()
    }

    fn get_max_overflow_size(&self) -> i32 {
        lock(&self.writer).get_max_overflow_size()
    }

    fn get_overflow_grow_count(&self) -> i32 {
        lock(&self.writer).get_overflow_grow_count()
    }
}

impl ISSEThrottledTemplateWriterControl for SharedThrottledWriterControl {
    fn start_event(&mut self, id: Option<&[u16]>, event: Option<&[u16]>) {
        if let ThrottledWriter::Sse(writer) = &mut *lock(&self.writer) {
            writer.start_event(id, event);
        }
    }

    fn end_event(&mut self) -> io::Result<()> {
        match &mut *lock(&self.writer) {
            ThrottledWriter::Sse(writer) => writer.end_event(),
            ThrottledWriter::Standard(_) => Ok(()),
        }
    }
}

impl JavaWriter for SharedThrottledWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        lock(&self.writer).write_utf16(characters)
    }

    fn flush(&mut self) -> io::Result<()> {
        lock(&self.writer).flush()
    }

    fn close(&mut self) -> io::Result<()> {
        lock(&self.writer).close()
    }
}

struct ThrottledEngineCause(Box<dyn TemplateEngineException>);

impl std::fmt::Debug for ThrottledEngineCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("ThrottledEngineCause")
            .field(&self.0.to_string())
            .finish()
    }
}

impl std::fmt::Display for ThrottledEngineCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.0.as_ref(), formatter)
    }
}

impl std::error::Error for ThrottledEngineCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

impl TemplateEngineException for ThrottledEngineCause {}

fn box_engine_error<E>(error: E) -> Box<dyn TemplateEngineException + Send + Sync>
where
    E: TemplateEngineException,
{
    Box::new(error)
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::lock;

    #[test]
    fn internal_lock_recovers_after_a_rust_panic() {
        let state = Arc::new(Mutex::new(1_i32));
        let panicking_state = Arc::clone(&state);
        let _ = std::thread::spawn(move || {
            let _guard = panicking_state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            panic!("poison throttled processor test state");
        })
        .join();

        *lock(&state) = 2;
        assert_eq!(*lock(&state), 2);
    }
}
