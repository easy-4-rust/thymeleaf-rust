use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex};

use thiserror::Error;

use crate::exceptions::TemplateProcessingException;
use crate::expression::{TemplateObject, TemplateValue};
use crate::util::JavaString;

use super::{DataDrivenTemplateSignal, IThrottledTemplateWriterControl};

const SSE_HEAD_EVENT_NAME: &[u16] = &[104, 101, 97, 100];
const SSE_MESSAGE_EVENT_NAME: &[u16] = &[109, 101, 115, 115, 97, 103, 101];
const SSE_TAIL_EVENT_NAME: &[u16] = &[116, 97, 105, 108];

/// 数据驱动迭代器的 Java 集合合同错误。
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
/// 对应 Java 语义：`DataDrivenTemplateIterator` 的 Rust 侧类型 `DataDrivenTemplateIteratorError`。
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
    writer_control: Option<Box<dyn IThrottledTemplateWriterControl + Send>>,
    sse_events_prefix: Option<Vec<u16>>,
    sse_events_id: i64,
    in_step: bool,
    feeding_complete: bool,
    queried: bool,
    signal: DataDrivenTemplateSignal,
}

impl<T> DataDrivenTemplateIterator<T> {
    /// 创建容量为十、等待数据喂入的空迭代器。
    #[must_use]
    /// 对应 Java 语义：`DataDrivenTemplateIterator` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new() -> Self {
        Self {
            values: VecDeque::with_capacity(10),
            writer_control: None,
            sse_events_prefix: None,
            sse_events_id: 0,
            in_step: false,
            feeding_complete: false,
            queried: false,
            signal: DataDrivenTemplateSignal::new(),
        }
    }

    /// 绑定普通或 SSE 节流 Writer 控制器。
    /// 对应 Java: `DataDrivenTemplateIterator#setWriterControl()`。
    pub fn set_writer_control(
        &mut self,
        writer_control: Box<dyn IThrottledTemplateWriterControl + Send>,
    ) {
        self.writer_control = Some(writer_control);
    }

    /// 设置 SSE 事件名前缀；`null` 或空字符串会清除前缀。
    /// 对应 Java: `DataDrivenTemplateIterator#setSseEventsPrefix()`。
    pub fn set_sse_events_prefix(&mut self, sse_events_prefix: Option<&JavaString>) {
        self.sse_events_prefix = sse_events_prefix
            .filter(|prefix| !prefix.is_empty())
            .map(|prefix| prefix.as_utf16().to_vec());
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
    /// 对应 Java: `DataDrivenTemplateIterator#hasNext()`。
    pub fn has_next(&mut self) -> bool {
        self.queried = true;
        !self.values.is_empty()
    }

    /// 按 Java `Iterator#next()` 语义取出队首；空队列返回 NoSuchElement。
    /// 对应 Java 语义：`DataDrivenTemplateIterator` 的 `next_java` 行为（Rust 侧辅助/私有路径）。
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
    ///
    /// Java `startIteration` 使用 `setSseEventsPrefix` 预组合的缓存事件名；
    /// 这里以原始名交给 `start_step` 组合一次（`start_step` 内部对 id 与
    /// 事件名各组合一次），结果与 Java 一致。
    /// 对应 Java: `DataDrivenTemplateIterator#startIteration()`。
    pub fn start_iteration(&mut self) {
        self.start_step(SSE_MESSAGE_EVENT_NAME);
    }

    /// 完成当前 message 数据迭代。
    /// 对应 Java: `DataDrivenTemplateIterator#finishIteration()`。
    pub fn finish_iteration(&mut self) -> Result<(), TemplateProcessingException> {
        self.finish_step()
    }

    /// 判断模板是否至少调用过一次 `hasNext` 或 `next`。
    #[must_use]
    pub const fn has_been_queried(&self) -> bool {
        self.queried
    }

    /// 判断模板是否正在等待尚未喂入的数据。
    /// 对应 Java: `DataDrivenTemplateIterator#isPaused()`。
    pub(crate) fn is_paused(&mut self) -> bool {
        self.queried = true;
        self.values.is_empty() && !self.feeding_complete
    }

    /// 判断当前缓冲区是否足以继续执行。
    #[must_use]
    /// 对应 Java: `DataDrivenTemplateIterator#continueBufferExecution()`。
    pub fn continue_buffer_execution(&self) -> bool {
        !self.values.is_empty()
    }

    /// 把一批新元素追加至缓冲区。
    /// 对应 Java: `DataDrivenTemplateIterator#feedBuffer()`。
    pub fn feed_buffer<I>(&mut self, new_elements: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.values.extend(new_elements);
        self.signal.notify();
    }

    /// 开始模板 head 输出阶段。
    /// 对应 Java: `DataDrivenTemplateIterator#startHead()`。
    pub fn start_head(&mut self) {
        self.start_step(SSE_HEAD_EVENT_NAME);
    }

    /// 标记不会再有数据喂入。
    /// 对应 Java: `DataDrivenTemplateIterator#feedingComplete()`。
    pub fn feeding_complete(&mut self) {
        self.feeding_complete = true;
        self.signal.notify();
    }

    /// 返回供响应式整合等待数据到达的共享信号。
    #[must_use]
    /// 对应 Java 语义：`DataDrivenTemplateIterator` 的 `get_signal` 行为（Rust 侧辅助/私有路径）。
    pub fn get_signal(&self) -> DataDrivenTemplateSignal {
        self.signal.clone()
    }

    /// 开始模板 tail 输出阶段。
    /// 对应 Java: `DataDrivenTemplateIterator#startTail()`。
    pub fn start_tail(&mut self) {
        self.start_step(SSE_TAIL_EVENT_NAME);
    }

    /// 完成当前 head/message/tail 阶段。
    /// 对应 Java: `DataDrivenTemplateIterator#finishStep()`。
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
    /// 对应 Java: `DataDrivenTemplateIterator#isStepOutputFinished()`。
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
        // Java: `composedToken[prefix.length] = '-'`（45），非下划线
        result.push(45);
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

impl DataDrivenTemplateIterator<Arc<TemplateValue>> {
    /// 创建可由宿主继续喂入、同时可直接放入模板 Context 的共享迭代器。
    ///
    /// 返回的第一个值用于调用 `feed_buffer`、`feeding_complete` 等控制方法；第二个
    /// 值保持同一对象身份并作为 `th:each` 的迭代目标。
    #[must_use]
    /// 对应 Java 语义：`DataDrivenTemplateIterator` 的 `shared_template_value` 行为（Rust 侧辅助/私有路径）。
    pub fn shared_template_value() -> (Arc<Mutex<Self>>, Arc<TemplateValue>) {
        let iterator = Arc::new(Mutex::new(Self::new()));
        let object: Arc<dyn TemplateObject> = iterator.clone();
        let value = Arc::new(TemplateValue::Object(object));
        (iterator, value)
    }

    /// 把已有共享迭代器转换为保持同一身份的模板动态值。
    #[must_use]
    /// 对应 Java 语义：`DataDrivenTemplateIterator` 的 `to_template_value` 行为（Rust 侧辅助/私有路径）。
    pub fn to_template_value(iterator: &Arc<Mutex<Self>>) -> Arc<TemplateValue> {
        let object: Arc<dyn TemplateObject> = iterator.clone();
        Arc::new(TemplateValue::Object(object))
    }
}

impl<T> Iterator for DataDrivenTemplateIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.queried = true;
        self.values.pop_front()
    }
}

impl TemplateObject for Mutex<DataDrivenTemplateIterator<Arc<TemplateValue>>> {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.engine.DataDrivenTemplateIterator"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str("org.thymeleaf.engine.DataDrivenTemplateIterator")
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
