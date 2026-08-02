use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::model::IOpenElementTag;

use super::i_engine_processable::EngineProcessableResult;
use super::processor_execution_vars::ProcessorExecutionVars;
use super::template_flow_controller::TemplateFlowController;
use super::{IEngineProcessable, TemplateHandlerHandle, TemplateModelController};

/// 分阶段输出开放标签的 before/delegate/after Model。
///
/// 对应 Java: `org.thymeleaf.engine.OpenElementTagModelProcessable`。
pub(crate) struct OpenElementTagModelProcessable {
    open_element_tag: Arc<dyn IOpenElementTag>,
    vars: ProcessorExecutionVars,
    flow_controller: Arc<Mutex<TemplateFlowController>>,
    model_controller: Rc<RefCell<TemplateModelController>>,
    processor_template_handler: TemplateHandlerHandle,
    next_template_handler: TemplateHandlerHandle,
    before_processed: bool,
    delegation_processed: bool,
    after_processed: bool,
    offset: usize,
}

impl OpenElementTagModelProcessable {
    /// 创建从 before 阶段开始的开放标签任务。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`OpenElementTagModelProcessable` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(
        open_element_tag: Arc<dyn IOpenElementTag>,
        vars: ProcessorExecutionVars,
        model_controller: Rc<RefCell<TemplateModelController>>,
        flow_controller: Arc<Mutex<TemplateFlowController>>,
        processor_template_handler: TemplateHandlerHandle,
        next_template_handler: TemplateHandlerHandle,
    ) -> Self {
        Self {
            open_element_tag,
            vars,
            flow_controller,
            model_controller,
            processor_template_handler,
            next_template_handler,
            before_processed: false,
            delegation_processed: false,
            after_processed: false,
            offset: 0,
        }
    }
}

impl IEngineProcessable for OpenElementTagModelProcessable {
    fn process(&mut self) -> EngineProcessableResult {
        if self
            .flow_controller
            .lock()
            .expect("template flow controller lock poisoned")
            .stop_processing
        {
            return Ok(false);
        }
        if !self.before_processed {
            if let Some(model_before) = &self.vars.model_before {
                let processed = model_before.process_throttled(
                    self.next_template_handler.borrow_mut().as_mut(),
                    self.offset,
                    Some(&self.flow_controller),
                )?;
                self.offset += processed;
                if self.offset < model_before.queue.len()
                    || self
                        .flow_controller
                        .lock()
                        .expect("template flow controller lock poisoned")
                        .stop_processing
                {
                    return Ok(false);
                }
            }
            self.before_processed = true;
            self.offset = 0;
        }
        if !self.delegation_processed {
            if !self.vars.discard_event {
                self.next_template_handler
                    .borrow_mut()
                    .handle_open_element(Arc::clone(&self.open_element_tag))?;
            }
            self.delegation_processed = true;
            self.offset = 0;
        }
        if self
            .flow_controller
            .lock()
            .expect("template flow controller lock poisoned")
            .stop_processing
        {
            return Ok(false);
        }
        if !self.after_processed {
            if let Some(model_after) = &self.vars.model_after {
                let handler = if self.vars.model_after_processable {
                    &self.processor_template_handler
                } else {
                    &self.next_template_handler
                };
                let processed = model_after.process_throttled(
                    handler.borrow_mut().as_mut(),
                    self.offset,
                    Some(&self.flow_controller),
                )?;
                self.offset += processed;
                if self.offset < model_after.queue.len()
                    || self
                        .flow_controller
                        .lock()
                        .expect("template flow controller lock poisoned")
                        .stop_processing
                {
                    return Ok(false);
                }
            }
            self.after_processed = true;
        }
        self.model_controller
            .borrow_mut()
            .skip(self.vars.skip_body, self.vars.skip_close_tag);
        Ok(true)
    }
}
