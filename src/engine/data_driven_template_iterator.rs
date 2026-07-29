use std::collections::VecDeque;
use std::io;

use thiserror::Error;

use crate::exceptions::TemplateProcessingException;
use crate::util::JavaString;

use super::IThrottledTemplateWriterControl;

const SSE_HEAD_EVENT_NAME: &[u16] = &[104, 101, 97, 100];
const SSE_MESSAGE_EVENT_NAME: &[u16] = &[109, 101, 115, 115, 97, 103, 101];
const SSE_TAIL_EVENT_NAME: &[u16] = &[116, 97, 105, 108];

/// 数据驱动迭代器的 Java 集合合同错误。
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DataDrivenTemplateIteratorError {
    /// 空队列调用 Java `Iterator#next()`。
    #[error("java.util.NoSuchElementException")]
    NoSuchElement,
    /// Java 对象明确禁止 `Iterator#remove()`。
    #[error("remove() is not supported in Throttled Iterator")]
    RemoveUnsupported,
}

/// 响应式集成使用的节流数据驱动迭代器。
///
/// 集成层可以分批喂入数据；模板已经查询但队列暂空时，对象报告暂停而不是结束。
/// 绑定支持 SSE 的 Writer 后，head、每次 message 迭代和 tail 会生成带递增 ID 的
/// 事件边界。
///
/// 对应 Java: `org.thymeleaf.engine.DataDrivenTemplateIterator`。
pub struct DataDrivenTemplateIterator<T> {
    values: VecDeque<T>,
    writer_control: Option<Box<dyn IThrottledTemplateWriterControl>>,
    sse_events_prefix: Option<Vec<u16>>,
    sse_events_composed_message_event_name: Vec<u16>,
    sse_events_id: i64,
    in_step: bool,
    feeding_complete: bool,
    queried: bool,
}

impl<T> DataDrivenTemplateIterator<T> {
    /// 创建容量为十、等待数据喂入的空迭代器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: VecDeque::with_capacity(10),
            writer_control: None,
            sse_events_prefix: None,
            sse_events_composed_message_event_name: SSE_MESSAGE_EVENT_NAME.to_vec(),
            sse_events_id: 0,
            in_step: false,
            feeding_complete: false,
            queried: false,
        }
    }

    /// 绑定普通或 SSE 节流 Writer 控制器。
    pub fn set_writer_control(&mut self, writer_control: Box<dyn IThrottledTemplateWriterControl>) {
        self.writer_control = Some(writer_control);
    }

    /// 设置 SSE 事件名前缀；`null` 或空字符串会清除前缀。
    pub fn set_sse_events_prefix(&mut self, sse_events_prefix: Option<&JavaString>) {
        self.sse_events_prefix = sse_events_prefix
            .filter(|prefix| !prefix.is_empty())
            .map(|prefix| prefix.as_utf16().to_vec());
        self.sse_events_composed_message_event_name = self.compose_token(SSE_MESSAGE_EVENT_NAME);
    }

    /// 设置首个 SSE 事件 ID。
    pub const fn set_sse_events_first_id(&mut self, sse_events_first_id: i64) {
        self.sse_events_id = sse_events_first_id;
    }

    /// 若当前 ID 为正数，则回退最近一次分配。
    pub const fn take_back_last_event_id(&mut self) {
        if self.sse_events_id > 0 {
            self.sse_events_id -= 1;
        }
    }

    /// 判断当前是否有下一项，并记录模板已经查询该迭代器。
    pub fn has_next(&mut self) -> bool {
        self.queried = true;
        !self.values.is_empty()
    }

    /// 按 Java `Iterator#next()` 语义取出队首；空队列返回 NoSuchElement。
    pub fn next_java(&mut self) -> Result<T, DataDrivenTemplateIteratorError> {
        self.queried = true;
        self.values
            .pop_front()
            .ok_or(DataDrivenTemplateIteratorError::NoSuchElement)
    }

    /// Java `Iterator#remove()` 在此对象上始终不受支持。
    pub const fn remove(&self) -> Result<(), DataDrivenTemplateIteratorError> {
        Err(DataDrivenTemplateIteratorError::RemoveUnsupported)
    }

    /// 开始一次 message 数据迭代。
    pub fn start_iteration(&mut self) {
        let event = self.sse_events_composed_message_event_name.clone();
        self.start_step(&event);
    }

    /// 完成当前 message 数据迭代。
    pub fn finish_iteration(&mut self) -> Result<(), TemplateProcessingException> {
        self.finish_step()
    }

    /// 判断模板是否至少调用过一次 `hasNext` 或 `next`。
    #[must_use]
    pub const fn has_been_queried(&self) -> bool {
        self.queried
    }

    /// 判断模板是否正在等待尚未喂入的数据。
    #[expect(
        dead_code,
        reason = "由后续 IteratedGatheringModelProcessable 同包对象调用"
    )]
    pub(crate) fn is_paused(&mut self) -> bool {
        self.queried = true;
        self.values.is_empty() && !self.feeding_complete
    }

    /// 判断当前缓冲区是否足以继续执行。
    #[must_use]
    pub fn continue_buffer_execution(&self) -> bool {
        !self.values.is_empty()
    }

    /// 把一批新元素追加至缓冲区。
    pub fn feed_buffer<I>(&mut self, new_elements: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.values.extend(new_elements);
    }

    /// 开始模板 head 输出阶段。
    pub fn start_head(&mut self) {
        self.start_step(SSE_HEAD_EVENT_NAME);
    }

    /// 标记不会再有数据喂入。
    pub const fn feeding_complete(&mut self) {
        self.feeding_complete = true;
    }

    /// 开始模板 tail 输出阶段。
    pub fn start_tail(&mut self) {
        self.start_step(SSE_TAIL_EVENT_NAME);
    }

    /// 完成当前 head/message/tail 阶段。
    pub fn finish_step(&mut self) -> Result<(), TemplateProcessingException> {
        if !self.in_step {
            return Ok(());
        }
        self.in_step = false;
        if let Some(sse_control) = self
            .writer_control
            .as_deref_mut()
            .and_then(IThrottledTemplateWriterControl::as_sse_control)
        {
            sse_control.end_event().map_err(Self::processing_error)?;
        }
        Ok(())
    }

    /// 判断阶段输出及其溢出缓冲是否已经全部写完。
    pub fn is_step_output_finished(&mut self) -> Result<bool, TemplateProcessingException> {
        if self.in_step {
            return Ok(false);
        }
        match self.writer_control.as_deref_mut() {
            Some(control) => control
                .is_overflown()
                .map(|overflown| !overflown)
                .map_err(Self::processing_error),
            None => Ok(true),
        }
    }

    fn start_step(&mut self, event: &[u16]) {
        self.in_step = true;
        let id_token: Vec<u16> = self.sse_events_id.to_string().encode_utf16().collect();
        let id = self.compose_token(&id_token);
        let event = self.compose_token(event);
        if let Some(sse_control) = self
            .writer_control
            .as_deref_mut()
            .and_then(IThrottledTemplateWriterControl::as_sse_control)
        {
            sse_control.start_event(Some(&id), Some(&event));
            self.sse_events_id = self.sse_events_id.wrapping_add(1);
        }
    }

    fn compose_token(&self, token: &[u16]) -> Vec<u16> {
        let Some(prefix) = self.sse_events_prefix.as_deref() else {
            return token.to_vec();
        };
        let mut result = Vec::with_capacity(prefix.len() + 1 + token.len());
        result.extend_from_slice(prefix);
        result.push(95);
        result.extend_from_slice(token);
        result
    }

    fn processing_error(cause: io::Error) -> TemplateProcessingException {
        TemplateProcessingException::with_cause(
            Some("Cannot signal end of SSE event".to_owned()),
            cause,
        )
    }
}

impl<T> Default for DataDrivenTemplateIterator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Iterator for DataDrivenTemplateIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.queried = true;
        self.values.pop_front()
    }
}
