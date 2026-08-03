use std::cell::RefCell;
use std::rc::Weak;
use std::sync::{Arc, Mutex};

use crate::IEngineConfiguration;
use crate::context::IEngineContext;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IModel, IOpenElementTag,
    IProcessingInstruction, IStandaloneElementTag, ITemplateEvent, IText, IXMLDeclaration,
};

use super::gathering_model_execution_state::GatheringModelExecutionState;
use super::model::Model;
use super::processor_execution_vars::ProcessorExecutionVars;
use super::template_flow_controller::TemplateFlowController;
use super::{ITemplateHandler, SkipBody, TemplateHandlerHandle, TemplateModelController};

/// Gathering processable 共享的模型构建、skip 恢复和上下文状态。
///
/// 对应 Java: `org.thymeleaf.engine.AbstractGatheringModelProcessable`。
pub(crate) struct AbstractGatheringModelProcessable {
    processor_template_handler: TemplateHandlerHandle,
    context: Arc<dyn IEngineContext>,
    synthetic_model: Model,
    model_controller: Weak<RefCell<TemplateModelController>>,
    flow_controller: Option<Arc<Mutex<TemplateFlowController>>>,
    build_time_skip_body: SkipBody,
    build_time_skip_close_tag: bool,
    processor_execution_vars: ProcessorExecutionVars,
    gathering_finished: bool,
    model_level: i32,
}

impl AbstractGatheringModelProcessable {
    /// 创建空合成 Model 并克隆起始 Processor 状态。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`AbstractGatheringModelProcessable` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(
        configuration: Arc<dyn IEngineConfiguration>,
        processor_template_handler: TemplateHandlerHandle,
        context: Arc<dyn IEngineContext>,
        model_controller: Weak<RefCell<TemplateModelController>>,
        flow_controller: Option<Arc<Mutex<TemplateFlowController>>>,
        build_time_skip_body: SkipBody,
        build_time_skip_close_tag: bool,
        processor_execution_vars: &ProcessorExecutionVars,
    ) -> Self {
        let template_mode = context.get_template_mode();
        Self {
            processor_template_handler,
            context,
            synthetic_model: Model::new(configuration, template_mode),
            model_controller,
            flow_controller,
            build_time_skip_body,
            build_time_skip_close_tag,
            processor_execution_vars: processor_execution_vars.clone_vars(),
            gathering_finished: false,
            model_level: 0,
        }
    }

    /// 对应 Java: `AbstractGatheringModelProcessable#resetGatheredSkipFlagsAfterNoIterations()`。
    pub(crate) fn reset_gathered_skip_flags_after_no_iterations(&self) {
        if let Some(controller) = self.model_controller.upgrade() {
            let skip_body = if self.build_time_skip_body == SkipBody::ProcessOneElement {
                SkipBody::SkipElements
            } else {
                self.build_time_skip_body
            };
            controller
                .borrow_mut()
                .skip(skip_body, self.build_time_skip_close_tag);
        }
    }

    /// 对应 Java: `AbstractGatheringModelProcessable#resetGatheredSkipFlags()`。
    pub(crate) fn reset_gathered_skip_flags(&self) {
        if let Some(controller) = self.model_controller.upgrade() {
            controller
                .borrow_mut()
                .skip(self.build_time_skip_body, self.build_time_skip_close_tag);
        }
    }

    /// 对应 Java: `AbstractGatheringModelProcessable#prepareProcessing()`。
    pub(crate) fn prepare_processing(&self) {
        self.processor_template_handler
            .borrow_mut()
            .set_current_gathering_model(Some(GatheringModelExecutionState::new(
                clone_model(&self.synthetic_model),
                self.processor_execution_vars.clone_vars(),
                self.model_controller.clone(),
                self.build_time_skip_body,
                self.build_time_skip_close_tag,
            )));
        self.reset_gathered_skip_flags();
    }

    /// 对应 Java 语义：`AbstractGatheringModelProcessable` 的 `reentrant_processor_template_handler` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn reentrant_processor_template_handler(&self) -> Box<dyn ITemplateHandler> {
        let handler = self.get_processor_template_handler();
        handler
            .borrow()
            .create_reentrant_handler()
            .expect("ProcessorTemplateHandler always supplies a reentrant proxy")
    }

    /// 返回 ProcessorTemplateHandler 的共享句柄。
    ///
    /// 对应 Java:
    /// `AbstractGatheringModelProcessable#getProcessorTemplateHandler()`；Rust 返回共享
    /// trait 句柄，以支持不持有 `RefCell` 借用的重入模型回放。
    pub(crate) fn get_processor_template_handler(&self) -> TemplateHandlerHandle {
        self.processor_template_handler.clone()
    }

    /// 对应 Java 语义：`AbstractGatheringModelProcessable` 的 `flow_controller` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn flow_controller(&self) -> Option<Arc<Mutex<TemplateFlowController>>> {
        self.flow_controller.clone()
    }

    /// 对应 Java 语义：`AbstractGatheringModelProcessable` 的 `context` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn context(&self) -> Arc<dyn IEngineContext> {
        Arc::clone(&self.context)
    }

    pub(crate) const fn is_gathering_finished(&self) -> bool {
        self.gathering_finished
    }

    pub(crate) const fn inner_model(&self) -> &Model {
        &self.synthetic_model
    }

    #[expect(
        dead_code,
        reason = "保留 Java AbstractGatheringModelProcessable 的包级方法合同"
    )]
    /// 对应 Java: `AbstractGatheringModelProcessable#initializeProcessorExecutionVars()`。
    pub(crate) fn initialize_processor_execution_vars(&self) -> ProcessorExecutionVars {
        self.processor_execution_vars.clone_vars()
    }

    /// 对应 Java: `AbstractGatheringModelProcessable#gatherText()`。
    pub(crate) fn gather_text(
        &mut self,
        event: Arc<dyn IText>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.gather_event(event)
    }

    /// 对应 Java: `AbstractGatheringModelProcessable#gatherComment()`。
    pub(crate) fn gather_comment(
        &mut self,
        event: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.gather_event(event)
    }

    /// 对应 Java 语义：`AbstractGatheringModelProcessable` 的 `gather_cdata_section` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn gather_cdata_section(
        &mut self,
        event: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.gather_event(event)
    }

    /// 对应 Java: `AbstractGatheringModelProcessable#gatherStandaloneElement()`。
    pub(crate) fn gather_standalone_element(
        &mut self,
        event: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.ensure_not_finished()?;
        self.synthetic_model.add(Some(event)).map_err(model_error)?;
        if self.model_level == 0 {
            self.gathering_finished = true;
        }
        Ok(())
    }

    /// 对应 Java: `AbstractGatheringModelProcessable#gatherOpenElement()`。
    pub(crate) fn gather_open_element(
        &mut self,
        event: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.ensure_not_finished()?;
        self.synthetic_model.add(Some(event)).map_err(model_error)?;
        self.model_level = self.model_level.wrapping_add(1);
        Ok(())
    }

    /// 对应 Java: `AbstractGatheringModelProcessable#gatherCloseElement()`。
    pub(crate) fn gather_close_element(
        &mut self,
        event: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if event.is_unmatched() {
            return self.gather_unmatched_close_element(event);
        }
        self.ensure_not_finished()?;
        self.model_level = self.model_level.wrapping_sub(1);
        self.synthetic_model.add(Some(event)).map_err(model_error)?;
        if self.model_level == 0 {
            self.gathering_finished = true;
        }
        Ok(())
    }

    /// 对应 Java: `AbstractGatheringModelProcessable#gatherUnmatchedCloseElement()`。
    pub(crate) fn gather_unmatched_close_element(
        &mut self,
        event: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.gather_event(event)
    }

    /// 对应 Java: `AbstractGatheringModelProcessable#gatherDocType()`。
    pub(crate) fn gather_doc_type(
        &mut self,
        event: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.gather_event(event)
    }

    /// 对应 Java 语义：`AbstractGatheringModelProcessable` 的 `gather_xml_declaration` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn gather_xml_declaration(
        &mut self,
        event: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.gather_event(event)
    }

    /// 对应 Java: `AbstractGatheringModelProcessable#gatherProcessingInstruction()`。
    pub(crate) fn gather_processing_instruction(
        &mut self,
        event: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.gather_event(event)
    }

    fn gather_event<T>(&mut self, event: Arc<T>) -> Result<(), Box<dyn TemplateEngineException>>
    where
        T: ITemplateEvent + ?Sized + 'static,
        Arc<T>: Into<Arc<dyn ITemplateEvent>>,
    {
        self.ensure_not_finished()?;
        self.synthetic_model
            .add(Some(event.into()))
            .map_err(model_error)
    }

    fn ensure_not_finished(&self) -> Result<(), Box<dyn TemplateEngineException>> {
        if self.gathering_finished {
            return Err(Box::new(TemplateProcessingException::new(Some(
                "Gathering is finished already! We cannot gather more events".to_owned(),
            ))));
        }
        Ok(())
    }
}

fn clone_model(model: &Model) -> Model {
    let mut clone = Model::new(
        model.get_configuration_arc(),
        model.get_template_mode_value(),
    );
    clone.reset_as_clone_of(model);
    clone
}

fn model_error(error: crate::model::IModelError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some("Could not gather template event into synthetic model".to_owned()),
        error,
    ))
}
