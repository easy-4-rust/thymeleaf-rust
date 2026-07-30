use std::cell::RefCell;
use std::rc::Weak;
use std::sync::{Arc, Mutex};

use crate::model::ITemplateEnd;

use super::i_engine_processable::EngineProcessableResult;
use super::model::Model;
use super::processor_template_handler::{ProcessorTemplateHandlerState, perform_teardown_checks};
use super::template_flow_controller::TemplateFlowController;
use super::{IEngineProcessable, TemplateHandlerHandle};

/// 节流模式下先输出模板结束边界前插入的 Model，再委派 TemplateEnd 并执行收尾校验。
///
/// 对应 Java: `org.thymeleaf.engine.TemplateEndModelProcessable`。
pub(crate) struct TemplateEndModelProcessable {
    template_end: Arc<dyn ITemplateEnd>,
    model: Model,
    model_handler: TemplateHandlerHandle,
    processor_template_handler_state: Weak<RefCell<ProcessorTemplateHandlerState>>,
    next_handler: TemplateHandlerHandle,
    flow_controller: Arc<Mutex<TemplateFlowController>>,
    offset: usize,
}

impl TemplateEndModelProcessable {
    /// 创建从 Model 偏移零开始的模板结束任务。
    pub(crate) fn new(
        template_end: Arc<dyn ITemplateEnd>,
        model: Model,
        model_handler: TemplateHandlerHandle,
        processor_template_handler_state: Weak<RefCell<ProcessorTemplateHandlerState>>,
        next_handler: TemplateHandlerHandle,
        flow_controller: Arc<Mutex<TemplateFlowController>>,
    ) -> Self {
        Self {
            template_end,
            model,
            model_handler,
            processor_template_handler_state,
            next_handler,
            flow_controller,
            offset: 0,
        }
    }
}

impl IEngineProcessable for TemplateEndModelProcessable {
    fn process(&mut self) -> EngineProcessableResult {
        if self
            .flow_controller
            .lock()
            .expect("template flow controller lock poisoned")
            .stop_processing
        {
            return Ok(false);
        }
        let processed = self.model.process_throttled(
            self.model_handler.borrow_mut().as_mut(),
            self.offset,
            Some(
                &self
                    .flow_controller
                    .lock()
                    .expect("template flow controller lock poisoned"),
            ),
        )?;
        self.offset += processed;
        if self.offset < self.model.queue.len()
            || self
                .flow_controller
                .lock()
                .expect("template flow controller lock poisoned")
                .stop_processing
        {
            return Ok(false);
        }

        self.next_handler
            .borrow_mut()
            .handle_template_end(Arc::clone(&self.template_end))?;
        if let Some(state) = self.processor_template_handler_state.upgrade() {
            perform_teardown_checks(&state, self.template_end.as_ref())?;
        }
        Ok(true)
    }
}
