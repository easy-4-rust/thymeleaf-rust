#![expect(
    dead_code,
    reason = "由后续 ProcessorTemplateHandler 的待处理队列创建并消费"
)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::i_engine_processable::EngineProcessableResult;
use super::model::Model;
use super::template_flow_controller::TemplateFlowController;
use super::{IEngineProcessable, ITemplateHandler};

/// 从保存的 offset 增量处理普通 Model 的待处理任务。
///
/// 每次调用先检查流控停止标志，再从上次 offset 继续向同一 Handler 发送事件；只有
/// 全部事件已经处理且流控未停止时返回 `true`。
///
/// 对应 Java: `org.thymeleaf.engine.SimpleModelProcessable`。
pub(crate) struct SimpleModelProcessable {
    model: Rc<RefCell<Model>>,
    model_handler: Rc<RefCell<Box<dyn ITemplateHandler>>>,
    flow_controller: Arc<Mutex<TemplateFlowController>>,
    offset: usize,
}

impl SimpleModelProcessable {
    /// 创建 offset 为零的模型处理任务。
    pub(crate) fn new(
        model: Rc<RefCell<Model>>,
        model_handler: Rc<RefCell<Box<dyn ITemplateHandler>>>,
        flow_controller: Arc<Mutex<TemplateFlowController>>,
    ) -> Self {
        Self {
            model,
            model_handler,
            flow_controller,
            offset: 0,
        }
    }

    /// 返回处理事件使用的同一 Handler。
    pub(crate) fn get_model_handler(&self) -> Rc<RefCell<Box<dyn ITemplateHandler>>> {
        Rc::clone(&self.model_handler)
    }

    /// 返回待处理的同一 Model。
    pub(crate) fn get_model(&self) -> Rc<RefCell<Model>> {
        Rc::clone(&self.model)
    }
}

impl IEngineProcessable for SimpleModelProcessable {
    fn process(&mut self) -> EngineProcessableResult {
        if self
            .flow_controller
            .lock()
            .expect("template flow controller lock poisoned")
            .stop_processing
        {
            return Ok(false);
        }
        let processed = {
            self.model.borrow().process_throttled(
                self.model_handler.borrow_mut().as_mut(),
                self.offset,
                Some(&self.flow_controller),
            )?
        };
        self.offset += processed;
        Ok(self.offset == self.model.borrow().queue.len()
            && !self
                .flow_controller
                .lock()
                .expect("template flow controller lock poisoned")
                .stop_processing)
    }
}
