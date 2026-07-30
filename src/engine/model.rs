#![expect(
    dead_code,
    reason = "构造与处理入口由后续迁移的 ModelFactory、TemplateManager 统一消费"
)]

use std::fmt::{Display, Formatter};
use std::io;
use std::sync::Arc;

use crate::exceptions::TemplateEngineException;
use crate::model::{IModel, IModelError, IModelVisitor, ITemplateEvent};
use crate::util::{FastStringWriter, JavaWriter};
use crate::{IEngineConfiguration, TemplateMode};

use super::{ITemplateHandler, template_flow_controller::TemplateFlowController};

const INITIAL_EVENT_QUEUE_SIZE: usize = 50;

/// 可变模板事件模型。
///
/// 事件对象保持不可变并以共享引用保存；克隆模型或插入另一个模型不会复制事件本身，
/// 与 Java 数组复制后的对象身份语义一致。对应 Java: `org.thymeleaf.engine.Model`。
pub(crate) struct Model {
    configuration: Arc<dyn IEngineConfiguration>,
    template_mode: TemplateMode,
    pub(crate) queue: Vec<Arc<dyn ITemplateEvent>>,
}

impl Model {
    /// 返回配置共享身份，供同包 processable 克隆模型。
    pub(crate) fn get_configuration_arc(&self) -> Arc<dyn IEngineConfiguration> {
        Arc::clone(&self.configuration)
    }

    /// 返回模板模式，供同包 processable 克隆模型。
    pub(crate) const fn get_template_mode_value(&self) -> TemplateMode {
        self.template_mode
    }
    /// 使用引擎配置和模板模式创建空模型。
    pub(crate) fn new(
        configuration: Arc<dyn IEngineConfiguration>,
        template_mode: TemplateMode,
    ) -> Self {
        Self {
            configuration,
            template_mode,
            queue: Vec::with_capacity(INITIAL_EVENT_QUEUE_SIZE),
        }
    }

    /// 将全部事件依次交给处理器链。
    pub(crate) fn process(
        &self,
        handler: &mut dyn ITemplateHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        for event in &self.queue {
            Arc::clone(event).be_handled(handler)?;
        }
        Ok(())
    }

    /// 从偏移量开始处理，遇到流控停止标志时暂停并返回本次处理数量。
    pub(crate) fn process_throttled(
        &self,
        handler: &mut dyn ITemplateHandler,
        offset: usize,
        controller: Option<&TemplateFlowController>,
    ) -> Result<usize, Box<dyn TemplateEngineException>> {
        if controller.is_none() {
            self.process(handler)?;
            return Ok(self.queue.len());
        }
        if offset >= self.queue.len() {
            return Ok(0);
        }

        let controller = controller.expect("controller was checked");
        let mut index = offset;
        while index < self.queue.len() && !controller.stop_processing {
            Arc::clone(&self.queue[index]).be_handled(handler)?;
            index += 1;
        }
        Ok(index - offset)
    }

    /// 把当前模型恢复为另一个模型的浅克隆，事件对象身份保持不变。
    pub(crate) fn reset_as_clone_of(&mut self, model: &Self) {
        self.configuration = Arc::clone(&model.configuration);
        self.template_mode = model.template_mode;
        self.queue.clone_from(&model.queue);
    }

    /// 仅按事件对象身份与顺序判断两个模型是否完全未发生变化。
    pub(crate) fn same_as(&self, model: &Self) -> bool {
        self.queue.len() == model.queue.len()
            && self
                .queue
                .iter()
                .zip(&model.queue)
                .all(|(left, right)| Arc::ptr_eq(left, right))
    }

    fn validate_event(event: &dyn ITemplateEvent) -> Result<(), IModelError> {
        if event.is_template_start() || event.is_template_end() {
            return Err(IModelError::TemplateBoundaryInsertion);
        }
        Ok(())
    }

    fn validate_position(&self, pos: usize, allow_end: bool) -> Result<(), IModelError> {
        let valid = pos < self.queue.len() || (allow_end && pos == self.queue.len());
        if valid {
            Ok(())
        } else {
            Err(IModelError::IndexOutOfBounds(pos))
        }
    }
}

impl IModel for Model {
    fn get_configuration(&self) -> &dyn IEngineConfiguration {
        self.configuration.as_ref()
    }

    fn get_template_mode(&self) -> TemplateMode {
        self.template_mode
    }

    fn size(&self) -> usize {
        self.queue.len()
    }

    fn get(&self, pos: usize) -> Arc<dyn ITemplateEvent> {
        Arc::clone(&self.queue[pos])
    }

    fn add(&mut self, event: Option<Arc<dyn ITemplateEvent>>) -> Result<(), IModelError> {
        let pos = self.queue.len();
        self.insert(pos, event)
    }

    fn insert(
        &mut self,
        pos: usize,
        event: Option<Arc<dyn ITemplateEvent>>,
    ) -> Result<(), IModelError> {
        let Some(event) = event else {
            return Ok(());
        };
        Self::validate_event(event.as_ref())?;
        self.validate_position(pos, true)?;
        self.queue.insert(pos, event);
        Ok(())
    }

    fn replace(
        &mut self,
        pos: usize,
        event: Option<Arc<dyn ITemplateEvent>>,
    ) -> Result<(), IModelError> {
        let Some(event) = event else {
            return Ok(());
        };
        Self::validate_event(event.as_ref())?;
        self.validate_position(pos, false)?;
        self.queue[pos] = event;
        Ok(())
    }

    fn add_model(&mut self, model: Option<&dyn IModel>) -> Result<(), IModelError> {
        self.insert_model(self.queue.len(), model)
    }

    fn insert_model(&mut self, pos: usize, model: Option<&dyn IModel>) -> Result<(), IModelError> {
        let Some(model) = model else {
            return Ok(());
        };
        if model.size() == 0 {
            return Ok(());
        }
        self.validate_position(pos, true)?;
        if !std::ptr::eq(self.configuration.as_ref(), model.get_configuration()) {
            return Err(IModelError::DifferentConfiguration);
        }
        if self.template_mode != model.get_template_mode() {
            return Err(IModelError::DifferentTemplateMode);
        }

        // TemplateModel 的边界事件不可嵌套；普通模型则完整插入。
        let skip_start = model.get(0).is_template_start();
        let skip_end = model.get(model.size() - 1).is_template_end();
        let start = usize::from(skip_start);
        let end = model.size().saturating_sub(usize::from(skip_end));
        let events = (start..end).map(|index| model.get(index));
        self.queue.splice(pos..pos, events);
        Ok(())
    }

    fn remove(&mut self, pos: usize) -> Result<(), IModelError> {
        self.validate_position(pos, false)?;
        self.queue.remove(pos);
        Ok(())
    }

    fn reset(&mut self) -> Result<(), IModelError> {
        self.queue.clear();
        Ok(())
    }

    fn clone_model(&self) -> Box<dyn IModel> {
        Box::new(Self {
            configuration: Arc::clone(&self.configuration),
            template_mode: self.template_mode,
            queue: self.queue.clone(),
        })
    }

    fn accept(&self, visitor: &mut dyn IModelVisitor) {
        for event in &self.queue {
            event.accept(visitor);
        }
    }

    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        for event in &self.queue {
            event.write(writer)?;
        }
        Ok(())
    }
}

impl Display for Model {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = FastStringWriter::new();
        self.write(&mut writer).map_err(|_| std::fmt::Error)?;
        formatter.write_str(&writer.to_string().to_string_lossy())
    }
}
