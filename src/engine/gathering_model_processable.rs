use std::cell::RefCell;
use std::rc::Weak;
use std::sync::{Arc, Mutex};

use crate::IEngineConfiguration;
use crate::context::IEngineContext;
use crate::exceptions::TemplateEngineException;
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IOpenElementTag, IProcessingInstruction,
    IStandaloneElementTag, IText, IXMLDeclaration,
};

use super::abstract_gathering_model_processable::AbstractGatheringModelProcessable;
use super::i_engine_processable::EngineProcessableResult;
use super::i_gathering_model_processable::IGatheringModelProcessable;
use super::model::Model;
use super::processor_execution_vars::ProcessorExecutionVars;
use super::template_flow_controller::TemplateFlowController;
use super::{IEngineProcessable, SkipBody, TemplateHandlerHandle, TemplateModelController};

/// 收集一次延迟元素 Model 并按原 Processor handler 重放。
///
/// 对应 Java: `org.thymeleaf.engine.GatheringModelProcessable`。
pub(crate) struct GatheringModelProcessable {
    base: AbstractGatheringModelProcessable,
    offset: usize,
}

impl GatheringModelProcessable {
    /// 创建尚未收集事件的延迟 Model。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        configuration: Arc<dyn IEngineConfiguration>,
        processor_template_handler: TemplateHandlerHandle,
        context: Arc<dyn IEngineContext>,
        model_controller: Weak<RefCell<TemplateModelController>>,
        flow_controller: Option<Arc<Mutex<TemplateFlowController>>>,
        gathered_skip_body: SkipBody,
        gathered_skip_close_tag: bool,
        processor_execution_vars: &ProcessorExecutionVars,
    ) -> Self {
        Self {
            base: AbstractGatheringModelProcessable::new(
                configuration,
                processor_template_handler,
                context,
                model_controller,
                flow_controller,
                gathered_skip_body,
                gathered_skip_close_tag,
                processor_execution_vars,
            ),
            offset: 0,
        }
    }
}

impl IEngineProcessable for GatheringModelProcessable {
    fn process(&mut self) -> EngineProcessableResult {
        let flow = self.base.flow_controller();
        if flow.as_ref().is_some_and(|controller| {
            controller
                .lock()
                .expect("template flow controller lock poisoned")
                .stop_processing
        }) {
            return Ok(false);
        }
        if self.offset == 0 {
            self.base.prepare_processing();
        }
        let mut handler = self.base.reentrant_processor_template_handler();
        let processed = {
            let flow_borrow = flow.as_ref().map(|controller| {
                controller
                    .lock()
                    .expect("template flow controller lock poisoned")
            });
            self.base.inner_model().process_throttled(
                handler.as_mut(),
                self.offset,
                flow_borrow.as_deref(),
            )?
        };
        self.offset += processed;
        let completed = self.offset == self.base.inner_model().queue.len()
            && flow.as_ref().is_none_or(|controller| {
                !controller
                    .lock()
                    .expect("template flow controller lock poisoned")
                    .stop_processing
            });
        if completed {
            self.base.context().decrease_level();
        }
        Ok(completed)
    }
}

impl IGatheringModelProcessable for GatheringModelProcessable {
    fn is_gathering_finished(&self) -> bool {
        self.base.is_gathering_finished()
    }

    fn get_inner_model(&self) -> &Model {
        self.base.inner_model()
    }

    fn reset_gathered_skip_flags(&self) {
        self.base.reset_gathered_skip_flags();
    }

    fn initialize_processor_execution_vars(&self) -> ProcessorExecutionVars {
        self.base.initialize_processor_execution_vars()
    }

    fn gather_text(
        &mut self,
        text: Arc<dyn IText>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.gather_text(text)
    }

    fn gather_comment(
        &mut self,
        comment: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.gather_comment(comment)
    }

    fn gather_cdata_section(
        &mut self,
        cdata_section: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.gather_cdata_section(cdata_section)
    }

    fn gather_standalone_element(
        &mut self,
        tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.gather_standalone_element(tag)
    }

    fn gather_open_element(
        &mut self,
        tag: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.gather_open_element(tag)
    }

    fn gather_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.gather_close_element(tag)
    }

    fn gather_unmatched_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.gather_unmatched_close_element(tag)
    }

    fn gather_doc_type(
        &mut self,
        doc_type: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.gather_doc_type(doc_type)
    }

    fn gather_xml_declaration(
        &mut self,
        declaration: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.gather_xml_declaration(declaration)
    }

    fn gather_processing_instruction(
        &mut self,
        instruction: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.gather_processing_instruction(instruction)
    }
}
