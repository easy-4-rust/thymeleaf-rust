#![expect(
    dead_code,
    reason = "构造与处理入口由后续迁移的 ModelFactory、TemplateManager 统一消费"
)]

use std::fmt::{Display, Formatter};
use std::io;
use std::sync::{Arc, Mutex};

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
    /// 对应 Java 语义：`Model` 的 `get_configuration_arc` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn get_configuration_arc(&self) -> Arc<dyn IEngineConfiguration> {
        Arc::clone(&self.configuration)
    }

    /// 返回模板模式，供同包 processable 克隆模型。
    pub(crate) const fn get_template_mode_value(&self) -> TemplateMode {
        self.template_mode
    }
    /// 使用引擎配置和模板模式创建空模型。
    /// 对应 Java 语义：`Model` 的 `new` 行为（Rust 侧辅助/私有路径）。
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
    /// 对应 Java: `Model#process()`。
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
    ///
    /// 控制器由调用方以 `Arc<Mutex<...>>` 共享，与 `TemplateModel` 的节流入口
    /// 保持同一语义：Java 的 `TemplateFlowController` 是无锁普通共享对象，Rust
    /// 只在读取停止标志的瞬间持锁，避免处理器链在事件写出（Throttled writer
    /// 会设置 stop_processing）时对同一控制器发生不可重入 Mutex 自锁。
    /// 对应 Java 语义：Java 接口/超类方法 `processThrottled()` 的 Rust 移植（`Model` 继承路径）。
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
        let mut index = offset;
        while index < self.queue.len() {
            let stop = controller
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stop_processing;
            if stop {
                break;
            }
            Arc::clone(&self.queue[index]).be_handled(handler)?;
            index += 1;
        }
        Ok(index - offset)
    }

    /// 把当前模型恢复为另一个模型的浅克隆，事件对象身份保持不变。
    /// 对应 Java: `Model#resetAsCloneOf()`。
    pub(crate) fn reset_as_clone_of(&mut self, model: &Self) {
        self.configuration = Arc::clone(&model.configuration);
        self.template_mode = model.template_mode;
        self.queue.clone_from(&model.queue);
    }

    /// 仅按事件对象身份与顺序判断两个模型是否完全未发生变化。
    /// 对应 Java: `Model#sameAs()`。
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
            return Err(IModelError::DifferentTemplateMode {
                model_mode: model.get_template_mode(),
                current_mode: self.template_mode,
            });
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::Model;
    use crate::engine::template_flow_controller::TemplateFlowController;
    use crate::engine::{ITemplateHandler, TemplateEnd, TemplateStart, Text};
    use crate::model::{
        ICDATASection, ICloseElementTag, IComment, IDocType, IModel, IModelError, IModelVisitor,
        IOpenElementTag, IProcessingInstruction, IStandaloneElementTag, ITemplateEnd,
        ITemplateEvent, ITemplateStart, IText, IXMLDeclaration,
    };
    use crate::util::{FastStringWriter, JavaString};
    use crate::{ITemplateEngine, TemplateEngine, TemplateMode};

    /// 只记录 Text 的最小处理器；其余事件方法保持成功，模拟 Java 可链接 Handler。
    struct RecordingHandler(Vec<String>);

    impl ITemplateHandler for RecordingHandler {
        fn set_next(&mut self, _: Option<crate::engine::TemplateHandlerHandle>) {}
        fn set_context(&mut self, _: Arc<dyn crate::context::ITemplateContext>) {}
        fn handle_template_start(
            &mut self,
            _: Arc<dyn ITemplateStart>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_template_end(
            &mut self,
            _: Arc<dyn ITemplateEnd>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_xml_declaration(
            &mut self,
            _: Arc<dyn IXMLDeclaration>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_doc_type(
            &mut self,
            _: Arc<dyn IDocType>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_cdata_section(
            &mut self,
            _: Arc<dyn ICDATASection>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_comment(
            &mut self,
            _: Arc<dyn IComment>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_text(
            &mut self,
            text: Arc<dyn IText>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            self.0.push(
                text.get_text()
                    .expect("text access")
                    .expect("non-null text")
                    .to_string_lossy(),
            );
            Ok(())
        }
        fn handle_standalone_element(
            &mut self,
            _: Arc<dyn IStandaloneElementTag>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_open_element(
            &mut self,
            _: Arc<dyn IOpenElementTag>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_close_element(
            &mut self,
            _: Arc<dyn ICloseElementTag>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_processing_instruction(
            &mut self,
            _: Arc<dyn IProcessingInstruction>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
    }

    /// 只记录 Text 的最小访问器，用于验证事件分派顺序。
    struct RecordingVisitor(Vec<String>);

    impl IModelVisitor for RecordingVisitor {
        fn visit_template_start(&mut self, _: &dyn ITemplateStart) {}
        fn visit_template_end(&mut self, _: &dyn ITemplateEnd) {}
        fn visit_xml_declaration(&mut self, _: &dyn IXMLDeclaration) {}
        fn visit_doc_type(&mut self, _: &dyn IDocType) {}
        fn visit_cdata_section(&mut self, _: &dyn ICDATASection) {}
        fn visit_comment(&mut self, _: &dyn IComment) {}
        fn visit_text(&mut self, text: &dyn IText) {
            self.0.push(
                text.get_text()
                    .expect("text access")
                    .expect("non-null text")
                    .to_string_lossy(),
            );
        }
        fn visit_standalone_element_tag(&mut self, _: &dyn IStandaloneElementTag) {}
        fn visit_open_element_tag(&mut self, _: &dyn IOpenElementTag) {}
        fn visit_close_element_tag(&mut self, _: &dyn ICloseElementTag) {}
        fn visit_processing_instruction(&mut self, _: &dyn IProcessingInstruction) {}
    }

    /// 记录模板边界事件的最小 Handler。
    #[derive(Default)]
    struct BoundaryHandler {
        starts: usize,
        ends: usize,
    }

    impl ITemplateHandler for BoundaryHandler {
        fn set_next(&mut self, _: Option<crate::engine::TemplateHandlerHandle>) {}
        fn set_context(&mut self, _: Arc<dyn crate::context::ITemplateContext>) {}
        fn handle_template_start(
            &mut self,
            _: Arc<dyn ITemplateStart>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            self.starts += 1;
            Ok(())
        }
        fn handle_template_end(
            &mut self,
            _: Arc<dyn ITemplateEnd>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            self.ends += 1;
            Ok(())
        }
        fn handle_xml_declaration(
            &mut self,
            _: Arc<dyn IXMLDeclaration>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_doc_type(
            &mut self,
            _: Arc<dyn IDocType>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_cdata_section(
            &mut self,
            _: Arc<dyn ICDATASection>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_comment(
            &mut self,
            _: Arc<dyn IComment>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_text(
            &mut self,
            _: Arc<dyn IText>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_standalone_element(
            &mut self,
            _: Arc<dyn IStandaloneElementTag>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_open_element(
            &mut self,
            _: Arc<dyn IOpenElementTag>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_close_element(
            &mut self,
            _: Arc<dyn ICloseElementTag>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
        fn handle_processing_instruction(
            &mut self,
            _: Arc<dyn IProcessingInstruction>,
        ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
            Ok(())
        }
    }

    /// 记录模板边界事件的最小 Visitor。
    #[derive(Default)]
    struct BoundaryVisitor {
        starts: usize,
        ends: usize,
    }

    impl IModelVisitor for BoundaryVisitor {
        fn visit_template_start(&mut self, _: &dyn ITemplateStart) {
            self.starts += 1;
        }
        fn visit_template_end(&mut self, _: &dyn ITemplateEnd) {
            self.ends += 1;
        }
        fn visit_xml_declaration(&mut self, _: &dyn IXMLDeclaration) {}
        fn visit_doc_type(&mut self, _: &dyn IDocType) {}
        fn visit_cdata_section(&mut self, _: &dyn ICDATASection) {}
        fn visit_comment(&mut self, _: &dyn IComment) {}
        fn visit_text(&mut self, _: &dyn IText) {}
        fn visit_standalone_element_tag(&mut self, _: &dyn IStandaloneElementTag) {}
        fn visit_open_element_tag(&mut self, _: &dyn IOpenElementTag) {}
        fn visit_close_element_tag(&mut self, _: &dyn ICloseElementTag) {}
        fn visit_processing_instruction(&mut self, _: &dyn IProcessingInstruction) {}
    }

    fn event(value: &str) -> Arc<dyn ITemplateEvent> {
        Arc::new(Text::new(Some(Arc::new(JavaString::from_rust_str(value)))))
    }

    fn golden(key: &str) -> &str {
        include_str!("../../tests/fixtures/model_golden.txt")
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))
            .expect("Java Golden record")
    }

    #[test]
    fn mutable_model_preserves_java_edit_identity_and_configuration_contracts() {
        let engine = TemplateEngine::new();
        let configuration = engine.get_configuration().expect("configuration");
        let mut model = Model::new(Arc::clone(&configuration), TemplateMode::HTML);
        let a = event("a");
        let b = event("b");
        let c = event("c");
        model.add(Some(Arc::clone(&a))).expect("add a");
        model.insert(0, Some(Arc::clone(&b))).expect("insert b");
        model.replace(1, Some(Arc::clone(&c))).expect("replace c");
        assert_eq!(
            format!("{},{},true,true", model.size(), model),
            golden("edited")
        );
        assert!(Arc::ptr_eq(&model.get(0), &b));
        assert!(Arc::ptr_eq(&model.get(1), &c));
        assert_eq!(model.insert(3, None), Ok(()));
        assert_eq!(format!("{},{}", model.size(), model), golden("nullInsert"));
        assert_eq!(model.remove(2), Err(IModelError::IndexOutOfBounds(2)));

        let clone = model.clone_model();
        assert!(Arc::ptr_eq(&model.get(0), &clone.get(0)));
        assert_eq!(clone.size(), 2);
        assert_eq!(golden("clone"), "true,bc");
        let mut reset_clone = Model::new(Arc::clone(&configuration), TemplateMode::XML);
        reset_clone
            .add(Some(event("different")))
            .expect("populate reset clone");
        reset_clone.reset_as_clone_of(&model);
        assert_eq!(
            format!(
                "{},{},{}",
                reset_clone.get_template_mode(),
                reset_clone,
                reset_clone.same_as(&model)
            ),
            golden("resetClone")
        );
        reset_clone
            .replace(0, Some(event("x")))
            .expect("replace cloned event");
        assert_eq!(
            reset_clone.same_as(&model).to_string(),
            golden("sameAsChanged")
        );
        let mut writer = FastStringWriter::new();
        model.write(&mut writer).expect("write events");
        assert_eq!(writer.to_string().to_string_lossy(), "bc");

        let mut same_configuration = Model::new(Arc::clone(&configuration), TemplateMode::HTML);
        same_configuration
            .add_model(Some(&model))
            .expect("append model");
        assert_eq!(same_configuration.to_string(), golden("sameConfig"));
        let other_engine = TemplateEngine::new();
        let other_configuration = other_engine
            .get_configuration()
            .expect("other configuration");
        let mut other = Model::new(other_configuration, TemplateMode::HTML);
        other
            .add(Some(event("other")))
            .expect("populate other model");
        let error = model
            .add_model(Some(&other))
            .expect_err("different configuration must be rejected");
        assert_eq!(error, IModelError::DifferentConfiguration);
        assert_eq!(
            error.to_string(),
            golden("differentConfig")
                .split_once(':')
                .expect("Java error class/message")
                .1
        );
        let mut different_mode = Model::new(Arc::clone(&configuration), TemplateMode::XML);
        different_mode
            .add(Some(event("xml")))
            .expect("populate different-mode model");
        let error = model
            .add_model(Some(&different_mode))
            .expect_err("different template mode must be rejected");
        assert_eq!(
            error,
            IModelError::DifferentTemplateMode {
                model_mode: TemplateMode::XML,
                current_mode: TemplateMode::HTML,
            }
        );
        assert_eq!(
            error.to_string(),
            golden("differentMode")
                .split_once(':')
                .expect("Java error class/message")
                .1
        );
        model.reset().expect("reset");
        assert_eq!(format!("{},{}", model.size(), model), golden("reset"));
    }

    #[test]
    fn dispatches_events_in_order_and_honors_throttled_offset_and_stop() {
        let engine = TemplateEngine::new();
        let configuration = engine.get_configuration().expect("configuration");
        let mut model = Model::new(configuration, TemplateMode::HTML);
        model.add(Some(event("b"))).expect("b");
        model.add(Some(event("c"))).expect("c");
        let mut handler = RecordingHandler(Vec::new());
        model.process(&mut handler).expect("full processing");
        assert_eq!(handler.0.concat(), golden("dispatch"));
        handler.0.clear();
        let controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        assert_eq!(
            model
                .process_throttled(&mut handler, 1, Some(&controller))
                .expect("offset"),
            1
        );
        assert_eq!(
            handler.0.concat(),
            golden("throttled").split_once(',').expect("count/text").1
        );
        let stopped = Arc::new(Mutex::new(TemplateFlowController::new()));
        stopped
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop_processing = true;
        assert_eq!(
            model
                .process_throttled(&mut handler, 0, Some(&stopped))
                .expect("stopped"),
            0
        );
        let mut visitor = RecordingVisitor(Vec::new());
        model.accept(&mut visitor);
        assert_eq!(visitor.0.concat(), golden("visitor"));
    }

    #[test]
    fn template_boundaries_are_singletons_and_dispatch_like_java() {
        let start = TemplateStart::instance();
        let end = TemplateEnd::instance();
        let mut handler = BoundaryHandler::default();
        Arc::clone(&start)
            .be_handled(&mut handler)
            .expect("start handling");
        Arc::clone(&end)
            .be_handled(&mut handler)
            .expect("end handling");
        let mut visitor = BoundaryVisitor::default();
        start.accept(&mut visitor);
        end.accept(&mut visitor);
        let mut writer = FastStringWriter::new();
        start.write(&mut writer).expect("start write");
        end.write(&mut writer).expect("end write");
        assert_eq!(
            format!(
                "{},{},{},{},{},{},{}",
                Arc::ptr_eq(&start, &TemplateStart::instance()),
                Arc::ptr_eq(&end, &TemplateEnd::instance()),
                writer.to_string().to_string_lossy(),
                handler.starts,
                handler.ends,
                visitor.starts,
                visitor.ends,
            ),
            golden("boundaries")
        );
    }
}
