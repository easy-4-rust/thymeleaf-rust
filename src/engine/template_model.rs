use std::fmt::{Display, Formatter};
use std::io;
use std::sync::Arc;

use crate::exceptions::TemplateEngineException;
use crate::model::{IModel, IModelError, IModelVisitor, ITemplateEvent};
use crate::util::{FastStringWriter, JavaWriter};
use crate::{IEngineConfiguration, TemplateMode};

use super::{
    ITemplateHandler, TemplateData, model::Model, template_flow_controller::TemplateFlowController,
};

/// 解析完成后供模板缓存保存的不可变完整模板模型。
///
/// 队列必须以 `TemplateStart` 开始并以 `TemplateEnd` 结束；所有修改入口均拒绝操作，
/// 从而保持缓存一致性。对应 Java: `org.thymeleaf.engine.TemplateModel`。
#[derive(Clone)]
pub struct TemplateModel {
    configuration: Arc<dyn IEngineConfiguration>,
    template_data: Arc<TemplateData>,
    queue: Vec<Arc<dyn ITemplateEvent>>,
}

impl TemplateModel {
    /// 从解析器事件队列创建不可变模板模型。
    ///
    /// 参数 `queue` 至少包含模板开始和结束事件，否则返回边界错误。
    pub(crate) fn new(
        configuration: Arc<dyn IEngineConfiguration>,
        template_data: Arc<TemplateData>,
        queue: Vec<Arc<dyn ITemplateEvent>>,
    ) -> Result<Self, IModelError> {
        if queue.len() < 2
            || !queue.first().is_some_and(|event| event.is_template_start())
            || !queue.last().is_some_and(|event| event.is_template_end())
        {
            return Err(IModelError::TemplateBoundaryInsertion);
        }
        Ok(Self {
            configuration,
            template_data,
            queue,
        })
    }

    /// 返回当前完整模板的解析元数据。
    #[must_use]
    pub fn get_template_data(&self) -> &TemplateData {
        self.template_data.as_ref()
    }

    /// 将完整事件队列依次交给模板处理器。
    pub(crate) fn process(
        &self,
        handler: &mut dyn ITemplateHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        for event in &self.queue {
            Arc::clone(event).be_handled(handler)?;
        }
        Ok(())
    }

    /// 从偏移量开始处理，遇到流控停止标志时暂停。
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
        let mut processed = 0;
        for event in self.queue.iter().skip(offset) {
            if controller.stop_processing {
                break;
            }
            Arc::clone(event).be_handled(handler)?;
            processed += 1;
        }
        Ok(processed)
    }

    fn immutable_model_error<T>() -> Result<T, IModelError> {
        Err(IModelError::ImmutableModel)
    }
}

impl IModel for TemplateModel {
    fn get_template_data(&self) -> Option<&TemplateData> {
        Some(self.template_data.as_ref())
    }

    fn get_template_data_arc(&self) -> Option<Arc<TemplateData>> {
        Some(Arc::clone(&self.template_data))
    }

    fn get_configuration(&self) -> &dyn IEngineConfiguration {
        self.configuration.as_ref()
    }

    fn get_template_mode(&self) -> TemplateMode {
        self.template_data
            .get_template_mode()
            .expect("TemplateModel requires non-null template mode")
    }

    fn size(&self) -> usize {
        self.queue.len()
    }

    fn get(&self, pos: usize) -> Arc<dyn ITemplateEvent> {
        Arc::clone(&self.queue[pos])
    }

    fn add(&mut self, _event: Option<Arc<dyn ITemplateEvent>>) -> Result<(), IModelError> {
        Self::immutable_model_error()
    }

    fn insert(
        &mut self,
        _pos: usize,
        _event: Option<Arc<dyn ITemplateEvent>>,
    ) -> Result<(), IModelError> {
        Self::immutable_model_error()
    }

    fn replace(
        &mut self,
        _pos: usize,
        _event: Option<Arc<dyn ITemplateEvent>>,
    ) -> Result<(), IModelError> {
        Self::immutable_model_error()
    }

    fn add_model(&mut self, _model: Option<&dyn IModel>) -> Result<(), IModelError> {
        Self::immutable_model_error()
    }

    fn insert_model(
        &mut self,
        _pos: usize,
        _model: Option<&dyn IModel>,
    ) -> Result<(), IModelError> {
        Self::immutable_model_error()
    }

    fn remove(&mut self, _pos: usize) -> Result<(), IModelError> {
        Self::immutable_model_error()
    }

    fn reset(&mut self) -> Result<(), IModelError> {
        Self::immutable_model_error()
    }

    fn clone_model(&self) -> Box<dyn IModel> {
        // Java `new Model(TemplateModel)` 明确去除首尾模板边界事件。
        let mut model = Model::new(Arc::clone(&self.configuration), self.get_template_mode());
        model.queue = self.queue[1..self.queue.len() - 1].to_vec();
        Box::new(model)
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

impl Display for TemplateModel {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = FastStringWriter::new();
        self.write(&mut writer).map_err(|_| std::fmt::Error)?;
        formatter.write_str(&writer.to_string().to_string_lossy())
    }
}
