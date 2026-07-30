use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// 数据驱动模板迭代器的跨线程唤醒信号。
///
/// 这是 Rust 响应式整合使用的等价适配对象：Java Spring WebFlux 由 Reactive
/// Streams 回调重新驱动节流处理器，Rust 框架适配器通过该信号等待 `feed_buffer`
/// 或 `feeding_complete`，避免轮询和空转。
#[derive(Clone)]
pub struct DataDrivenTemplateSignal {
    state: Arc<(Mutex<u64>, Condvar)>,
}

impl DataDrivenTemplateSignal {
    /// 创建初始修订号为零的信号。
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new((Mutex::new(0), Condvar::new())),
        }
    }

    /// 返回当前修订号；调用方应在尝试推进模板之前保存该值。
    #[must_use]
    pub fn revision(&self) -> u64 {
        *self
            .state
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// 通知等待方已有新数据或上游已经结束。
    pub fn notify(&self) {
        let (revision, condition) = self.state.as_ref();
        let mut revision = revision
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *revision = revision.wrapping_add(1);
        condition.notify_all();
    }

    /// 等待修订号变化，或在超时后返回以便适配器检查响应是否已取消。
    ///
    /// # 参数
    /// - `previous_revision`：推进模板之前保存的修订号。
    /// - `timeout`：单次最长等待时间。
    ///
    /// # 返回
    /// 修订号已经变化时返回 `true`，超时时返回 `false`。
    #[must_use]
    pub fn wait_for_change(&self, previous_revision: u64, timeout: Duration) -> bool {
        let (revision, condition) = self.state.as_ref();
        let revision = revision
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *revision != previous_revision {
            return true;
        }
        let (revision, _) = condition
            .wait_timeout_while(revision, timeout, |revision| *revision == previous_revision)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *revision != previous_revision
    }
}

impl Default for DataDrivenTemplateSignal {
    fn default() -> Self {
        Self::new()
    }
}
