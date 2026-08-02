use std::fmt::{Display, Formatter};
use std::io;
use std::sync::{Arc, Mutex};

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
    /// 对应 Java 语义：`TemplateModel` 的 `new` 行为（Rust 侧辅助/私有路径）。
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
    /// 对应 Java: `TemplateModel#getTemplateData()`。
    pub fn get_template_data(&self) -> &TemplateData {
        self.template_data.as_ref()
    }

    /// 将完整事件队列依次交给模板处理器。
    /// 对应 Java: `TemplateModel#process()`。
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
    /// 对应 Java 语义：Java 接口/超类方法 `processThrottled()` 的 Rust 移植（`TemplateModel` 继承路径）。
    pub(crate) fn process_throttled(
        &self,
        handler: &mut dyn ITemplateHandler,
        offset: usize,
        controller: Option<&Arc<Mutex<TemplateFlowController>>>,
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
            // Java 控制器是普通共享对象；Rust 只在读取标志时持锁，避免处理器链写入
            // 同一控制器时发生不可重入 Mutex 自锁。
            if controller
                .lock()
                .expect("template flow controller lock poisoned")
                .stop_processing
            {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::TemplateModel;
    use crate::engine::{TemplateData, TemplateEnd, TemplateStart};
    use crate::model::{IModel, IModelError, ITemplateEvent};
    use crate::{ITemplateEngine, TemplateEngine, TemplateMode};

    fn golden(key: &str) -> &str {
        include_str!("../../tests/fixtures/model_golden.txt")
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))
            .expect("Java Golden record")
    }

    fn template_model() -> TemplateModel {
        let engine = TemplateEngine::new();
        let configuration = engine.get_configuration().expect("configuration");
        let template_data = Arc::new(TemplateData::new(
            Some(crate::util::JavaString::from_rust_str("template")),
            None,
            None,
            Some(TemplateMode::HTML),
            None,
        ));
        let queue: Vec<Arc<dyn ITemplateEvent>> =
            vec![TemplateStart::instance(), TemplateEnd::instance()];
        TemplateModel::new(configuration, template_data, queue).expect("bounded template model")
    }

    #[test]
    fn immutable_contract_and_mutable_clone_match_java_golden() {
        let mut model = template_model();
        assert_eq!(
            format!(
                "{},{},{},{}",
                model.size(),
                model.get_template_mode(),
                [
                    model.add(None),
                    model.insert(0, None),
                    model.replace(0, None),
                    model.add_model(None),
                    model.insert_model(0, None),
                    model.remove(0),
                    model.reset(),
                ]
                .iter()
                .all(|result| matches!(result, Err(IModelError::ImmutableModel))),
                model.clone_model().size(),
            ),
            golden("templateModel")
        );
        assert_eq!(
            format!(
                "UnsupportedOperationException:{}",
                IModelError::ImmutableModel
            ),
            golden("immutable")
        );
    }
}
