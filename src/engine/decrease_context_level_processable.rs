#![expect(
    dead_code,
    reason = "由后续 ProcessorTemplateHandler 的待处理队列创建并消费"
)]

use std::sync::{Arc, Mutex};

use crate::context::IEngineContext;

use super::IEngineProcessable;
use super::i_engine_processable::EngineProcessableResult;
use super::template_flow_controller::TemplateFlowController;

/// 在处理队列恢复执行时减少引擎上下文层级的任务。
///
/// 若流控已经停止，本次不修改上下文并返回 `false`；否则对非空上下文调用一次
/// `decreaseLevel()` 并返回 `true`。
///
/// 对应 Java: `org.thymeleaf.engine.DecreaseContextLevelProcessable`。
pub(crate) struct DecreaseContextLevelProcessable {
    context: Option<Arc<dyn IEngineContext>>,
    flow_controller: Arc<Mutex<TemplateFlowController>>,
}

impl DecreaseContextLevelProcessable {
    /// 创建共享同一上下文与流控状态的待处理任务。
    pub(crate) fn new(
        context: Option<Arc<dyn IEngineContext>>,
        flow_controller: Arc<Mutex<TemplateFlowController>>,
    ) -> Self {
        Self {
            context,
            flow_controller,
        }
    }
}

impl IEngineProcessable for DecreaseContextLevelProcessable {
    fn process(&mut self) -> EngineProcessableResult {
        if self
            .flow_controller
            .lock()
            .expect("template flow controller lock poisoned")
            .stop_processing
        {
            return Ok(false);
        }
        if let Some(context) = &self.context {
            context.decrease_level();
        }
        Ok(true)
    }
}
