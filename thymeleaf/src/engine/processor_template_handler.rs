use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};

use crate::IEngineConfiguration;
use crate::TemplateMode;
use crate::cdatasection::ICDATASectionStructureHandler;
use crate::comment::ICommentStructureHandler;
use crate::context::{IEngineContext, ITemplateContext};
use crate::doctype::IDocTypeStructureHandler;
use crate::element::{IElementModelStructureHandler, IElementTagStructureHandler};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IModel, IOpenElementTag,
    IProcessableElementTag, IProcessingInstruction, IStandaloneElementTag, ITemplateEnd,
    ITemplateEvent, ITemplateStart, IText, IXMLDeclaration,
};
use crate::processinginstruction::IProcessingInstructionStructureHandler;
use crate::templateboundaries::ITemplateBoundariesStructureHandler;
use crate::text::ITextStructureHandler;
use crate::xmldeclaration::IXMLDeclarationStructureHandler;

use super::cdata_section_structure_handler::CDATASectionStructureHandler;
use super::comment_structure_handler::CommentStructureHandler;
use super::decrease_context_level_processable::DecreaseContextLevelProcessable;
use super::doc_type_structure_handler::DocTypeStructureHandler;
use super::element_model_structure_handler::ElementModelStructureHandler;
use super::element_tag_structure_handler::ElementTagStructureHandler;
use super::gathering_model_execution_state::GatheringModelExecutionState;
use super::i_engine_processable::EngineProcessableResult;
use super::i_gathering_model_processable::IGatheringModelProcessable;
use super::model::Model;
use super::open_element_tag_model_processable::OpenElementTagModelProcessable;
use super::processing_instruction_structure_handler::ProcessingInstructionStructureHandler;
use super::processor_execution_vars::ProcessorExecutionVars;
use super::simple_model_processable::SimpleModelProcessable;
use super::standalone_element_tag_model_processable::StandaloneElementTagModelProcessable;
use super::template_boundaries_structure_handler::TemplateBoundariesStructureHandler;
use super::template_end_model_processable::TemplateEndModelProcessable;
use super::template_flow_controller::TemplateFlowController;
use super::text_structure_handler::TextStructureHandler;
use super::xml_declaration_structure_handler::XMLDeclarationStructureHandler;
use super::{
    Attribute, Attributes, CDATASection, Comment, DocType, IEngineProcessable, ITemplateHandler,
    OpenElementTag, ProcessingInstruction, StandaloneElementTag, TemplateHandlerHandle,
    TemplateModelController, Text, XMLDeclaration,
};

type ProcessableHandle = Rc<RefCell<Box<dyn IEngineProcessable>>>;

struct SharedGatheringModelProcessable {
    inner: Rc<RefCell<dyn IGatheringModelProcessable>>,
}

impl IEngineProcessable for SharedGatheringModelProcessable {
    fn process(&mut self) -> EngineProcessableResult {
        self.inner.borrow_mut().process()
    }
}

/// 执行全部适用 Processor 并应用 StructureHandler 动作的核心模板 Handler。
///
/// 预处理器位于该 Handler 之前，后处理器和输出 Handler 位于其后。共享内部状态让
/// 普通处理、gathering 重放和节流恢复始终看到同一个 Processor 游标、上下文层级
/// 与待处理栈。
///
/// 对应 Java: `org.thymeleaf.engine.ProcessorTemplateHandler`。
#[derive(Clone)]
pub struct ProcessorTemplateHandler {
    state: Rc<RefCell<ProcessorTemplateHandlerState>>,
}

struct ProcessorTemplateHandlerProxy {
    state: Weak<RefCell<ProcessorTemplateHandlerState>>,
}

/// 对应 Java 语义：`ProcessorTemplateHandler` 的 Rust 侧类型 `ProcessorTemplateHandlerState`。
pub(crate) struct ProcessorTemplateHandlerState {
    self_handler: Option<TemplateHandlerHandle>,
    next: Option<TemplateHandlerHandle>,
    configuration: Option<Arc<dyn IEngineConfiguration>>,
    template_mode: Option<TemplateMode>,
    context: Option<Arc<dyn ITemplateContext>>,
    engine_context: Option<Arc<dyn IEngineContext>>,
    flow_controller: Option<Arc<Mutex<TemplateFlowController>>>,
    model_controller: Option<Rc<RefCell<TemplateModelController>>>,
    current_gathering_model: Option<GatheringModelExecutionState>,
    initial_context_level: Option<i32>,
    throttle_engine: bool,
    pending_processings: Vec<ProcessableHandle>,
    queued_events_model: Option<Rc<RefCell<Model>>>,
    queued_events_processable: Option<ProcessableHandle>,
    template_boundaries_structure_handler: TemplateBoundariesStructureHandler,
    element_tag_structure_handler: ElementTagStructureHandler,
    element_model_structure_handler: ElementModelStructureHandler,
    cdata_section_structure_handler: CDATASectionStructureHandler,
    comment_structure_handler: CommentStructureHandler,
    doc_type_structure_handler: DocTypeStructureHandler,
    processing_instruction_structure_handler: ProcessingInstructionStructureHandler,
    text_structure_handler: TextStructureHandler,
    xml_declaration_structure_handler: XMLDeclarationStructureHandler,
}

impl ProcessorTemplateHandler {
    /// 创建尚未连接下一 Handler、上下文和流控器的处理器。
    ///
    /// 对应 Java: `ProcessorTemplateHandler#ProcessorTemplateHandler()`。
    #[must_use]
    pub fn new() -> Self {
        let state = Rc::new(RefCell::new(ProcessorTemplateHandlerState {
            self_handler: None,
            next: None,
            configuration: None,
            template_mode: None,
            context: None,
            engine_context: None,
            flow_controller: None,
            model_controller: None,
            current_gathering_model: None,
            initial_context_level: None,
            throttle_engine: false,
            pending_processings: Vec::new(),
            queued_events_model: None,
            queued_events_processable: None,
            template_boundaries_structure_handler: TemplateBoundariesStructureHandler::new(),
            element_tag_structure_handler: ElementTagStructureHandler::new(),
            element_model_structure_handler: ElementModelStructureHandler::new(),
            cdata_section_structure_handler: CDATASectionStructureHandler::new(),
            comment_structure_handler: CommentStructureHandler::new(),
            doc_type_structure_handler: DocTypeStructureHandler::new(),
            processing_instruction_structure_handler: ProcessingInstructionStructureHandler::new(),
            text_structure_handler: TextStructureHandler::new(),
            xml_declaration_structure_handler: XMLDeclarationStructureHandler::new(),
        }));
        let proxy = ProcessorTemplateHandlerProxy {
            state: Rc::downgrade(&state),
        };
        let self_handler: TemplateHandlerHandle =
            Rc::new(RefCell::new(Box::new(proxy) as Box<dyn ITemplateHandler>));
        state.borrow_mut().self_handler = Some(self_handler);
        Self { state }
    }

    /// 将处理器放入 Handler 链共享句柄。
    #[must_use]
    /// 对应 Java 语义：`ProcessorTemplateHandler` 的 `into_handle` 行为（Rust 侧辅助/私有路径）。
    pub fn into_handle(self) -> TemplateHandlerHandle {
        Rc::new(RefCell::new(Box::new(self)))
    }

    /// 设置可选节流流控器。
    ///
    /// 对应 Java: `ProcessorTemplateHandler#setFlowController`。
    pub(crate) fn set_flow_controller(
        &self,
        flow_controller: Option<Arc<Mutex<TemplateFlowController>>>,
    ) {
        let mut state = self.state.borrow_mut();
        state.throttle_engine = flow_controller.is_some();
        state.flow_controller = flow_controller.clone();
        if let Some(controller) = &state.model_controller {
            controller
                .borrow_mut()
                .set_template_flow_controller(flow_controller);
        }
    }

    /// 继续执行节流期间留下的嵌套待处理栈。
    ///
    /// 对应 Java: `ProcessorTemplateHandler#handlePending()`。
    pub(crate) fn handle_pending(&self) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_pending_state(&self.state)
    }
}

impl Default for ProcessorTemplateHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ITemplateHandler for ProcessorTemplateHandler {
    fn create_reentrant_handler(&self) -> Option<Box<dyn ITemplateHandler>> {
        Some(Box::new(ProcessorTemplateHandlerProxy {
            state: Rc::downgrade(&self.state),
        }))
    }

    fn set_next(&mut self, next: Option<TemplateHandlerHandle>) {
        self.state.borrow_mut().next = next;
    }

    fn set_context(&mut self, context: Arc<dyn ITemplateContext>) {
        set_context_state(&self.state, context);
    }

    fn set_current_gathering_model(&mut self, state: Option<GatheringModelExecutionState>) {
        self.state.borrow_mut().current_gathering_model = state;
    }

    fn handle_template_start(
        &mut self,
        event: Arc<dyn ITemplateStart>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_template_start_state(&self.state, event)
    }

    fn handle_template_end(
        &mut self,
        event: Arc<dyn ITemplateEnd>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_template_end_state(&self.state, event)
    }

    fn handle_xml_declaration(
        &mut self,
        event: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_xml_declaration_state(&self.state, event)
    }

    fn handle_doc_type(
        &mut self,
        event: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_doc_type_state(&self.state, event)
    }

    fn handle_cdata_section(
        &mut self,
        event: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_cdata_section_state(&self.state, event)
    }

    fn handle_comment(
        &mut self,
        event: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_comment_state(&self.state, event)
    }

    fn handle_text(
        &mut self,
        event: Arc<dyn IText>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_text_state(&self.state, event)
    }

    fn handle_standalone_element(
        &mut self,
        event: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_standalone_element_state(&self.state, event)
    }

    fn handle_open_element(
        &mut self,
        event: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_open_element_state(&self.state, event)
    }

    fn handle_close_element(
        &mut self,
        event: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_close_element_state(&self.state, event)
    }

    fn handle_processing_instruction(
        &mut self,
        event: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        handle_processing_instruction_state(&self.state, event)
    }
}

macro_rules! proxy_handler_method {
    ($name:ident, $ty:ty, $delegate:ident) => {
        fn $name(&mut self, event: Arc<$ty>) -> Result<(), Box<dyn TemplateEngineException>> {
            self.state
                .upgrade()
                .map_or(Ok(()), |state| $delegate(&state, event))
        }
    };
}

impl ITemplateHandler for ProcessorTemplateHandlerProxy {
    fn create_reentrant_handler(&self) -> Option<Box<dyn ITemplateHandler>> {
        Some(Box::new(Self {
            state: self.state.clone(),
        }))
    }

    fn set_next(&mut self, next: Option<TemplateHandlerHandle>) {
        if let Some(state) = self.state.upgrade() {
            state.borrow_mut().next = next;
        }
    }

    fn set_context(&mut self, context: Arc<dyn ITemplateContext>) {
        if let Some(state) = self.state.upgrade() {
            set_context_state(&state, context);
        }
    }

    fn set_current_gathering_model(&mut self, current: Option<GatheringModelExecutionState>) {
        if let Some(state) = self.state.upgrade() {
            state.borrow_mut().current_gathering_model = current;
        }
    }

    proxy_handler_method!(
        handle_template_start,
        dyn ITemplateStart,
        handle_template_start_state
    );
    proxy_handler_method!(
        handle_template_end,
        dyn ITemplateEnd,
        handle_template_end_state
    );
    proxy_handler_method!(
        handle_xml_declaration,
        dyn IXMLDeclaration,
        handle_xml_declaration_state
    );
    proxy_handler_method!(handle_doc_type, dyn IDocType, handle_doc_type_state);
    proxy_handler_method!(
        handle_cdata_section,
        dyn ICDATASection,
        handle_cdata_section_state
    );
    proxy_handler_method!(handle_comment, dyn IComment, handle_comment_state);
    proxy_handler_method!(handle_text, dyn IText, handle_text_state);
    proxy_handler_method!(
        handle_standalone_element,
        dyn IStandaloneElementTag,
        handle_standalone_element_state
    );
    proxy_handler_method!(
        handle_open_element,
        dyn IOpenElementTag,
        handle_open_element_state
    );
    proxy_handler_method!(
        handle_close_element,
        dyn ICloseElementTag,
        handle_close_element_state
    );
    proxy_handler_method!(
        handle_processing_instruction,
        dyn IProcessingInstruction,
        handle_processing_instruction_state
    );
}

fn set_context_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    context: Arc<dyn ITemplateContext>,
) {
    let configuration = context.get_configuration_arc();
    let template_mode = context.get_template_mode();
    let engine_context = context.get_engine_context_arc();
    let self_handler = state
        .borrow()
        .self_handler
        .clone()
        .expect("processor self handler is initialized in constructor");
    let model_controller = TemplateModelController::new(
        Arc::clone(&configuration),
        template_mode,
        self_handler,
        engine_context.clone(),
    );
    let mut state = state.borrow_mut();
    model_controller
        .borrow_mut()
        .set_template_flow_controller(state.flow_controller.clone());
    state.configuration = Some(configuration);
    state.template_mode = Some(template_mode);
    state.engine_context = engine_context;
    state.model_controller = Some(model_controller);
    state.context = Some(context);
}

fn handle_template_start_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<dyn ITemplateStart>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if queue_event_if_stopped(state, event.clone())? {
        return Ok(());
    }
    let (next, model, model_handler) = {
        let mut state = state.borrow_mut();
        if let Some(context) = &state.engine_context {
            state.initial_context_level = Some(context.level());
        }
        let context = require_context(&state)?;
        let template_mode = require_template_mode(&state)?;
        let processors = context
            .get_configuration()
            .get_template_boundaries_processors(template_mode);
        let mut model = None;
        let mut processable = true;
        for processor in processors {
            state.template_boundaries_structure_handler.reset();
            processor.process_template_start(
                context.as_ref(),
                event.as_ref(),
                &mut state.template_boundaries_structure_handler,
            )?;
            if let Some(engine_context) = &state.engine_context {
                state
                    .template_boundaries_structure_handler
                    .apply_context_modifications(engine_context.as_ref());
            }
            if state.template_boundaries_structure_handler.insert_text {
                model = Some(new_model(&state)?);
                let text = state
                    .template_boundaries_structure_handler
                    .insert_text_value
                    .clone()
                    .expect("insertText action requires text");
                let text: Arc<dyn ITemplateEvent> = Arc::new(Text::new(Some(Arc::new(text))));
                model
                    .as_mut()
                    .expect("model was initialized")
                    .add(Some(text))
                    .map_err(model_error)?;
                processable = state
                    .template_boundaries_structure_handler
                    .insert_text_processable;
            } else if state.template_boundaries_structure_handler.insert_model {
                model = Some(new_model(&state)?);
                model
                    .as_mut()
                    .expect("model was initialized")
                    .add_model(
                        state
                            .template_boundaries_structure_handler
                            .insert_model_value
                            .as_deref(),
                    )
                    .map_err(model_error)?;
                processable = state
                    .template_boundaries_structure_handler
                    .insert_model_processable;
            }
        }
        let next = require_next(&state)?;
        let model_handler = if processable {
            require_self_handler(&state)?
        } else {
            next.clone()
        };
        (next, model, model_handler)
    };
    next.borrow_mut().handle_template_start(event)?;
    process_optional_model(state, model, model_handler)
}

fn handle_template_end_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<dyn ITemplateEnd>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if queue_event_if_stopped(state, event.clone())? {
        return Ok(());
    }
    let has_processors = {
        let state_ref = state.borrow();
        let context = require_context(&state_ref)?;
        !context
            .get_configuration()
            .get_template_boundaries_processors(require_template_mode(&state_ref)?)
            .is_empty()
    };
    if !has_processors {
        return require_next(&state.borrow())?
            .borrow_mut()
            .handle_template_end(event);
    }
    let (next, model, model_handler) = {
        let mut state = state.borrow_mut();
        let context = require_context(&state)?;
        let template_mode = require_template_mode(&state)?;
        let processors = context
            .get_configuration()
            .get_template_boundaries_processors(template_mode);
        let mut model = None;
        let mut processable = true;
        for processor in processors {
            state.template_boundaries_structure_handler.reset();
            processor.process_template_end(
                context.as_ref(),
                event.as_ref(),
                &mut state.template_boundaries_structure_handler,
            )?;
            if let Some(engine_context) = &state.engine_context {
                state
                    .template_boundaries_structure_handler
                    .apply_context_modifications(engine_context.as_ref());
            }
            if state.template_boundaries_structure_handler.insert_text {
                model = Some(new_model(&state)?);
                let text = state
                    .template_boundaries_structure_handler
                    .insert_text_value
                    .clone()
                    .expect("insertText action requires text");
                let text: Arc<dyn ITemplateEvent> = Arc::new(Text::new(Some(Arc::new(text))));
                model
                    .as_mut()
                    .expect("model was initialized")
                    .add(Some(text))
                    .map_err(model_error)?;
                processable = state
                    .template_boundaries_structure_handler
                    .insert_text_processable;
            } else if state.template_boundaries_structure_handler.insert_model {
                model = Some(new_model(&state)?);
                model
                    .as_mut()
                    .expect("model was initialized")
                    .add_model(
                        state
                            .template_boundaries_structure_handler
                            .insert_model_value
                            .as_deref(),
                    )
                    .map_err(model_error)?;
                processable = state
                    .template_boundaries_structure_handler
                    .insert_model_processable;
            }
        }
        let next = require_next(&state)?;
        let model_handler = if processable {
            require_self_handler(&state)?
        } else {
            next.clone()
        };
        (next, model, model_handler)
    };
    if state.borrow().throttle_engine && model.as_ref().is_some_and(|model| !model.queue.is_empty())
    {
        let flow_controller = state
            .borrow()
            .flow_controller
            .clone()
            .expect("throttled engine has flow controller");
        return queue_processable(
            state,
            Box::new(TemplateEndModelProcessable::new(
                event,
                model.expect("non-empty template-end model exists"),
                model_handler,
                Rc::downgrade(state),
                next,
                flow_controller,
            )),
        );
    }
    process_optional_model(state, model, model_handler)?;
    next.borrow_mut().handle_template_end(event.clone())?;
    perform_teardown_checks(state, event.as_ref())
}

fn handle_text_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<dyn IText>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if queue_event_if_stopped(state, event.clone())? {
        return Ok(());
    }
    {
        let controller = require_model_controller(&state.borrow())?;
        if !controller.borrow_mut().should_process_text(event.clone())? {
            return Ok(());
        }
    }
    let (next, output, replacement, replacement_handler) = {
        let mut state = state.borrow_mut();
        let context = require_context(&state)?;
        let processors = context
            .get_configuration()
            .get_text_processors(require_template_mode(&state)?);
        let mut output: Arc<dyn IText> = event;
        let mut replacement = None;
        let mut processable = true;
        let mut discard = false;
        for processor in processors {
            if discard {
                break;
            }
            state.text_structure_handler.reset();
            processor.process(
                context.as_ref(),
                output.as_ref(),
                &mut state.text_structure_handler,
            )?;
            if state.text_structure_handler.set_text {
                let value = state
                    .text_structure_handler
                    .set_text_value
                    .clone()
                    .expect("setText action requires text");
                output = Arc::new(Text::new(Some(value)));
            } else if state.text_structure_handler.replace_with_model {
                let mut model = new_model(&state)?;
                model
                    .add_model(
                        state
                            .text_structure_handler
                            .replace_with_model_value
                            .as_deref(),
                    )
                    .map_err(model_error)?;
                replacement = Some(model);
                processable = state.text_structure_handler.replace_with_model_processable;
                discard = true;
            } else if state.text_structure_handler.remove_text {
                replacement = None;
                discard = true;
            }
        }
        let next = require_next(&state)?;
        let replacement_handler = if processable {
            require_self_handler(&state)?
        } else {
            next.clone()
        };
        (
            next,
            (!discard).then_some(output),
            replacement,
            replacement_handler,
        )
    };
    if let Some(output) = output {
        next.borrow_mut().handle_text(output)?;
    }
    process_optional_model(state, replacement, replacement_handler)
}

fn handle_comment_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<dyn IComment>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if queue_event_if_stopped(state, event.clone())? {
        return Ok(());
    }
    {
        let controller = require_model_controller(&state.borrow())?;
        if !controller
            .borrow_mut()
            .should_process_comment(event.clone())?
        {
            return Ok(());
        }
    }
    let (next, output, replacement, replacement_handler) = {
        let mut state = state.borrow_mut();
        let context = require_context(&state)?;
        let processors = context
            .get_configuration()
            .get_comment_processors(require_template_mode(&state)?);
        let mut output = event;
        let mut replacement = None;
        let mut processable = true;
        let mut discard = false;
        for processor in processors {
            if discard {
                break;
            }
            state.comment_structure_handler.reset();
            processor.process(
                context.as_ref(),
                output.as_ref(),
                &mut state.comment_structure_handler,
            )?;
            if state.comment_structure_handler.set_content {
                let engine = output.as_engine_comment().ok_or_else(|| {
                    processing_error("Cannot preserve boundaries of a non-engine Comment")
                })?;
                output = Arc::new(Comment::with_boundaries(
                    engine.prefix().clone(),
                    Some(
                        state
                            .comment_structure_handler
                            .set_content_value
                            .clone()
                            .expect("setContent action requires content"),
                    ),
                    engine.suffix().clone(),
                ));
            } else if state.comment_structure_handler.replace_with_model {
                let mut model = new_model(&state)?;
                model
                    .add_model(
                        state
                            .comment_structure_handler
                            .replace_with_model_value
                            .as_deref(),
                    )
                    .map_err(model_error)?;
                replacement = Some(model);
                processable = state
                    .comment_structure_handler
                    .replace_with_model_processable;
                discard = true;
            } else if state.comment_structure_handler.remove_comment {
                replacement = None;
                discard = true;
            }
        }
        let next = require_next(&state)?;
        let replacement_handler = if processable {
            require_self_handler(&state)?
        } else {
            next.clone()
        };
        (
            next,
            (!discard).then_some(output),
            replacement,
            replacement_handler,
        )
    };
    if let Some(output) = output {
        next.borrow_mut().handle_comment(output)?;
    }
    process_optional_model(state, replacement, replacement_handler)
}

fn handle_cdata_section_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<dyn ICDATASection>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if queue_event_if_stopped(state, event.clone())? {
        return Ok(());
    }
    {
        let controller = require_model_controller(&state.borrow())?;
        if !controller
            .borrow_mut()
            .should_process_cdata_section(event.clone())?
        {
            return Ok(());
        }
    }
    let (next, output, replacement, replacement_handler) = {
        let mut state = state.borrow_mut();
        let context = require_context(&state)?;
        let processors = context
            .get_configuration()
            .get_cdata_section_processors(require_template_mode(&state)?);
        let mut output = event;
        let mut replacement = None;
        let mut processable = true;
        let mut discard = false;
        for processor in processors {
            if discard {
                break;
            }
            state.cdata_section_structure_handler.reset();
            processor.process(
                context.as_ref(),
                output.as_ref(),
                &mut state.cdata_section_structure_handler,
            )?;
            if state.cdata_section_structure_handler.set_content {
                let engine = output.as_engine_cdata_section().ok_or_else(|| {
                    processing_error("Cannot preserve boundaries of a non-engine CDATA section")
                })?;
                output = Arc::new(CDATASection::with_boundaries(
                    engine.prefix().clone(),
                    Some(
                        state
                            .cdata_section_structure_handler
                            .set_content_value
                            .clone()
                            .expect("setContent action requires content"),
                    ),
                    engine.suffix().clone(),
                ));
            } else if state.cdata_section_structure_handler.replace_with_model {
                let mut model = new_model(&state)?;
                model
                    .add_model(
                        state
                            .cdata_section_structure_handler
                            .replace_with_model_value
                            .as_deref(),
                    )
                    .map_err(model_error)?;
                replacement = Some(model);
                processable = state
                    .cdata_section_structure_handler
                    .replace_with_model_processable;
                discard = true;
            } else if state.cdata_section_structure_handler.remove_cdata_section {
                replacement = None;
                discard = true;
            }
        }
        let next = require_next(&state)?;
        let replacement_handler = if processable {
            require_self_handler(&state)?
        } else {
            next.clone()
        };
        (
            next,
            (!discard).then_some(output),
            replacement,
            replacement_handler,
        )
    };
    if let Some(output) = output {
        next.borrow_mut().handle_cdata_section(output)?;
    }
    process_optional_model(state, replacement, replacement_handler)
}

fn handle_doc_type_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<dyn IDocType>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if queue_event_if_stopped(state, event.clone())? {
        return Ok(());
    }
    {
        let controller = require_model_controller(&state.borrow())?;
        if !controller
            .borrow_mut()
            .should_process_doc_type(event.clone())?
        {
            return Ok(());
        }
    }
    let (next, output, replacement, replacement_handler) = {
        let mut state = state.borrow_mut();
        let context = require_context(&state)?;
        let processors = context
            .get_configuration()
            .get_doc_type_processors(require_template_mode(&state)?);
        let mut output = event;
        let mut replacement = None;
        let mut processable = true;
        let mut discard = false;
        for processor in processors {
            if discard {
                break;
            }
            state.doc_type_structure_handler.reset();
            processor.process(
                context.as_ref(),
                output.as_ref(),
                &mut state.doc_type_structure_handler,
            )?;
            if state.doc_type_structure_handler.set_doc_type {
                output = Arc::new(
                    DocType::with_components(
                        state
                            .doc_type_structure_handler
                            .set_doc_type_keyword
                            .clone(),
                        state
                            .doc_type_structure_handler
                            .set_doc_type_element_name
                            .clone(),
                        state
                            .doc_type_structure_handler
                            .set_doc_type_public_id
                            .clone(),
                        state
                            .doc_type_structure_handler
                            .set_doc_type_system_id
                            .clone(),
                        state
                            .doc_type_structure_handler
                            .set_doc_type_internal_subset
                            .clone(),
                    )
                    .map_err(|error| {
                        Box::new(TemplateProcessingException::with_cause(
                            Some("Could not apply DocType processor action".to_owned()),
                            error,
                        )) as Box<dyn TemplateEngineException>
                    })?,
                );
            } else if state.doc_type_structure_handler.replace_with_model {
                let mut model = new_model(&state)?;
                model
                    .add_model(
                        state
                            .doc_type_structure_handler
                            .replace_with_model_value
                            .as_deref(),
                    )
                    .map_err(model_error)?;
                replacement = Some(model);
                processable = state
                    .doc_type_structure_handler
                    .replace_with_model_processable;
                discard = true;
            } else if state.doc_type_structure_handler.remove_doc_type {
                replacement = None;
                discard = true;
            }
        }
        let next = require_next(&state)?;
        let replacement_handler = if processable {
            require_self_handler(&state)?
        } else {
            next.clone()
        };
        (
            next,
            (!discard).then_some(output),
            replacement,
            replacement_handler,
        )
    };
    if let Some(output) = output {
        next.borrow_mut().handle_doc_type(output)?;
    }
    process_optional_model(state, replacement, replacement_handler)
}

fn handle_processing_instruction_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<dyn IProcessingInstruction>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if queue_event_if_stopped(state, event.clone())? {
        return Ok(());
    }
    {
        let controller = require_model_controller(&state.borrow())?;
        if !controller
            .borrow_mut()
            .should_process_processing_instruction(event.clone())?
        {
            return Ok(());
        }
    }
    let (next, output, replacement, replacement_handler) = {
        let mut state = state.borrow_mut();
        let context = require_context(&state)?;
        let processors = context
            .get_configuration()
            .get_processing_instruction_processors(require_template_mode(&state)?);
        let mut output = event;
        let mut replacement = None;
        let mut processable = true;
        let mut discard = false;
        for processor in processors {
            if discard {
                break;
            }
            state.processing_instruction_structure_handler.reset();
            processor.process(
                context.as_ref(),
                output.as_ref(),
                &mut state.processing_instruction_structure_handler,
            )?;
            if state
                .processing_instruction_structure_handler
                .set_processing_instruction
            {
                output = Arc::new(ProcessingInstruction::new(
                    state
                        .processing_instruction_structure_handler
                        .set_processing_instruction_target
                        .clone(),
                    state
                        .processing_instruction_structure_handler
                        .set_processing_instruction_content
                        .clone(),
                ));
            } else if state
                .processing_instruction_structure_handler
                .replace_with_model
            {
                let mut model = new_model(&state)?;
                model
                    .add_model(
                        state
                            .processing_instruction_structure_handler
                            .replace_with_model_value
                            .as_deref(),
                    )
                    .map_err(model_error)?;
                replacement = Some(model);
                processable = state
                    .processing_instruction_structure_handler
                    .replace_with_model_processable;
                discard = true;
            } else if state
                .processing_instruction_structure_handler
                .remove_processing_instruction
            {
                replacement = None;
                discard = true;
            }
        }
        let next = require_next(&state)?;
        let replacement_handler = if processable {
            require_self_handler(&state)?
        } else {
            next.clone()
        };
        (
            next,
            (!discard).then_some(output),
            replacement,
            replacement_handler,
        )
    };
    if let Some(output) = output {
        next.borrow_mut().handle_processing_instruction(output)?;
    }
    process_optional_model(state, replacement, replacement_handler)
}

fn handle_xml_declaration_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<dyn IXMLDeclaration>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if queue_event_if_stopped(state, event.clone())? {
        return Ok(());
    }
    {
        let controller = require_model_controller(&state.borrow())?;
        if !controller
            .borrow_mut()
            .should_process_xml_declaration(event.clone())?
        {
            return Ok(());
        }
    }
    let (next, output, replacement, replacement_handler) = {
        let mut state = state.borrow_mut();
        let context = require_context(&state)?;
        let processors = context
            .get_configuration()
            .get_xml_declaration_processors(require_template_mode(&state)?);
        let mut output = event;
        let mut replacement = None;
        let mut processable = true;
        let mut discard = false;
        for processor in processors {
            if discard {
                break;
            }
            state.xml_declaration_structure_handler.reset();
            processor.process(
                context.as_ref(),
                output.as_ref(),
                &mut state.xml_declaration_structure_handler,
            )?;
            if state.xml_declaration_structure_handler.set_xml_declaration {
                output = Arc::new(XMLDeclaration::with_components(
                    state
                        .xml_declaration_structure_handler
                        .set_xml_declaration_keyword
                        .clone(),
                    state
                        .xml_declaration_structure_handler
                        .set_xml_declaration_version
                        .clone(),
                    state
                        .xml_declaration_structure_handler
                        .set_xml_declaration_encoding
                        .clone(),
                    state
                        .xml_declaration_structure_handler
                        .set_xml_declaration_standalone
                        .clone(),
                ));
            } else if state.xml_declaration_structure_handler.replace_with_model {
                let mut model = new_model(&state)?;
                model
                    .add_model(
                        state
                            .xml_declaration_structure_handler
                            .replace_with_model_value
                            .as_deref(),
                    )
                    .map_err(model_error)?;
                replacement = Some(model);
                processable = state
                    .xml_declaration_structure_handler
                    .replace_with_model_processable;
                discard = true;
            } else if state
                .xml_declaration_structure_handler
                .remove_xml_declaration
            {
                replacement = None;
                discard = true;
            }
        }
        let next = require_next(&state)?;
        let replacement_handler = if processable {
            require_self_handler(&state)?
        } else {
            next.clone()
        };
        (
            next,
            (!discard).then_some(output),
            replacement,
            replacement_handler,
        )
    };
    if let Some(output) = output {
        next.borrow_mut().handle_xml_declaration(output)?;
    }
    process_optional_model(state, replacement, replacement_handler)
}

fn handle_standalone_element_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<dyn IStandaloneElementTag>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if queue_event_if_stopped(state, event.clone())? {
        return Ok(());
    }
    let controller = require_model_controller(&state.borrow())?;
    if !controller
        .borrow_mut()
        .should_process_standalone_element(event.clone())?
    {
        return Ok(());
    }

    let (
        context,
        engine_context,
        configuration,
        next,
        self_handler,
        flow_controller,
        throttle_engine,
        current_gathering_model,
    ) = {
        let mut state_ref = state.borrow_mut();
        (
            require_context(&state_ref)?,
            state_ref.engine_context.clone(),
            state_ref
                .configuration
                .clone()
                .ok_or_else(|| processing_error("Processor context has not been set"))?,
            require_next(&state_ref)?,
            require_self_handler(&state_ref)?,
            state_ref.flow_controller.clone(),
            state_ref.throttle_engine,
            state_ref.current_gathering_model.take(),
        )
    };

    if current_gathering_model.is_some()
        && let Some(engine_context) = &engine_context
    {
        engine_context.set_element_tag(None);
    }

    let mut tag = normalize_standalone_tag(event, configuration.as_ref())?;
    let engine_tag = require_engine_processable_tag(tag.as_ref())?;
    if current_gathering_model.is_none()
        && !engine_tag
            .has_associated_processors()
            .map_err(|error| processor_cause("Could not obtain associated processors", error))?
    {
        let standalone = into_standalone_tag(tag)?;
        next.borrow_mut().handle_standalone_element(standalone)?;
        decrease_context_level_or_queue(state, engine_context, throttle_engine, flow_controller)?;
        return Ok(());
    }

    let mut vars = current_gathering_model
        .as_ref()
        .map_or_else(ProcessorExecutionVars::new, |gathering| {
            gathering.initialize_processor_execution_vars()
        });

    while !vars.discard_event {
        let engine_tag = require_engine_processable_tag(tag.as_ref())?;
        let Some(processor) = vars
            .processor_iterator
            .next(engine_tag)
            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?
        else {
            break;
        };

        let mut state_ref = state.borrow_mut();
        state_ref.element_tag_structure_handler.reset();
        state_ref.element_model_structure_handler.reset();

        if let Some(element_processor) = processor.as_element_tag_processor() {
            element_processor.process(
                context.as_ref(),
                tag.as_ref(),
                &mut state_ref.element_tag_structure_handler,
            )?;
            state_ref
                .element_tag_structure_handler
                .apply_context_modifications(engine_context.as_deref());
            tag = state_ref
                .element_tag_structure_handler
                .apply_attributes(configuration.get_attribute_definitions(), tag)
                .map_err(|error| {
                    processor_cause("Could not apply element attribute modifications", error)
                })?;

            if state_ref.element_tag_structure_handler.iterate_element {
                let iteration_variable = state_ref
                    .element_tag_structure_handler
                    .iter_variable_name
                    .clone()
                    .expect("iterateElement action requires an iteration variable");
                let status_variable = state_ref
                    .element_tag_structure_handler
                    .iter_status_variable_name
                    .clone();
                let iterated_object = state_ref
                    .element_tag_structure_handler
                    .iterated_object
                    .clone();
                drop(state_ref);
                let standalone = into_standalone_tag(tag)?;
                controller
                    .borrow_mut()
                    .start_gathering_iterated_standalone_model(
                        standalone,
                        &vars,
                        iteration_variable,
                        status_variable,
                        iterated_object,
                    )?;
                let gathered = take_gathered_model(&controller)?;
                return process_or_queue_gathering(state, gathered, throttle_engine);
            }

            if state_ref.element_tag_structure_handler.set_body_text
                || state_ref.element_tag_structure_handler.set_body_model
            {
                reset_model_slot(
                    &mut vars.model_after,
                    true,
                    configuration.clone(),
                    require_template_mode(&state_ref)?,
                )?;
                if state_ref.element_tag_structure_handler.set_body_text {
                    let value = state_ref
                        .element_tag_structure_handler
                        .set_body_text_value
                        .clone()
                        .expect("setBody text action requires text");
                    add_text_to_model(vars.model_after.as_mut(), value)?;
                    vars.model_after_processable = state_ref
                        .element_tag_structure_handler
                        .set_body_text_processable;
                } else {
                    vars.model_after
                        .as_mut()
                        .expect("modelAfter was initialized")
                        .add_model(
                            state_ref
                                .element_tag_structure_handler
                                .set_body_model_value
                                .as_deref(),
                        )
                        .map_err(model_error)?;
                    vars.model_after_processable = state_ref
                        .element_tag_structure_handler
                        .set_body_model_processable;
                }
                drop(state_ref);
                let standalone = into_standalone_tag(tag)?;
                let equivalent = controller
                    .borrow()
                    .create_standalone_equivalent_model(standalone.as_ref(), &vars)?;
                return process_or_queue_owned(state, Box::new(equivalent), throttle_engine);
            }

            apply_common_tag_actions(
                &mut vars,
                &state_ref.element_tag_structure_handler,
                configuration.clone(),
                require_template_mode(&state_ref)?,
                false,
            )?;
        } else if let Some(element_processor) = processor.as_element_model_processor() {
            if !vars.processor_iterator.last_was_repeated() {
                reject_modified_body_for_model_processor(
                    &vars,
                    processor.java_class_name(),
                    tag.as_ref(),
                )?;
                vars.processor_iterator
                    .set_last_to_be_repeated(require_engine_processable_tag(tag.as_ref())?)
                    .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
                drop(state_ref);
                let standalone = into_standalone_tag(tag)?;
                controller
                    .borrow_mut()
                    .start_gathering_delayed_standalone_model(standalone, &vars)?;
                let gathered = take_gathered_model(&controller)?;
                return process_or_queue_gathering(state, gathered, throttle_engine);
            }

            let gathering = current_gathering_model.as_ref().ok_or_else(|| {
                processing_error("Repeated element model processor has no gathered model")
            })?;
            let mut processed_model = clone_model(gathering.inner_model());
            element_processor.process(
                context.as_ref(),
                &mut processed_model,
                &mut state_ref.element_model_structure_handler,
            )?;
            state_ref
                .element_model_structure_handler
                .apply_context_modifications(engine_context.as_deref());
            gathering.reset_gathered_skip_flags();
            if !gathering.inner_model().same_as(&processed_model) {
                reset_model_slot(
                    &mut vars.model_after,
                    true,
                    configuration.clone(),
                    require_template_mode(&state_ref)?,
                )?;
                vars.model_after
                    .as_mut()
                    .expect("modelAfter was initialized")
                    .add_model(Some(&processed_model))
                    .map_err(model_error)?;
                vars.model_after_processable = true;
                vars.discard_event = true;
            }
        } else {
            return Err(processing_error(&format!(
                "An element has an associated processor of type {} which is neither a Tag Element Processor nor a Model Element Processor.",
                processor.java_class_name()
            )));
        }
    }

    let standalone = into_standalone_tag(tag)?;
    if throttle_engine
        && (model_has_events(&vars.model_before) || model_has_events(&vars.model_after))
    {
        let flow = flow_controller
            .ok_or_else(|| processing_error("Throttled engine has no flow controller"))?;
        return queue_processable(
            state,
            Box::new(StandaloneElementTagModelProcessable::new(
                standalone,
                vars,
                engine_context,
                controller,
                flow,
                self_handler,
                next,
            )),
        );
    }

    process_before_delegate_after_standalone(state, &standalone, &vars, &next)?;
    decrease_context_level_or_queue(state, engine_context, throttle_engine, flow_controller)?;
    Ok(())
}

fn process_before_delegate_after_standalone(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    tag: &Arc<dyn IStandaloneElementTag>,
    vars: &ProcessorExecutionVars,
    next: &TemplateHandlerHandle,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if let Some(model) = &vars.model_before {
        model.process(next.borrow_mut().as_mut())?;
    }
    if !vars.discard_event {
        next.borrow_mut()
            .handle_standalone_element(Arc::clone(tag))?;
    }
    if let Some(model) = &vars.model_after {
        if vars.model_after_processable {
            process_model_through_processor(state, model)?;
        } else {
            model.process(next.borrow_mut().as_mut())?;
        }
    }
    Ok(())
}

fn normalize_standalone_tag(
    tag: Arc<dyn IStandaloneElementTag>,
    configuration: &dyn IEngineConfiguration,
) -> Result<Arc<dyn IProcessableElementTag>, Box<dyn TemplateEngineException>> {
    if let Some(engine_tag) = Arc::clone(&tag).into_engine_standalone_element_tag() {
        return Ok(engine_tag);
    }
    let template_mode = tag.get_template_mode();
    let complete_name = tag.get_element_complete_name().clone();
    let element_definition = configuration
        .get_element_definitions()
        .for_name(Some(template_mode), Some(&complete_name))
        .map_err(|error| {
            processor_cause("Could not normalize standalone element definition", error)
        })?;
    let attributes = normalize_attributes(tag.as_ref(), configuration)?;
    let engine_tag = StandaloneElementTag::with_location(
        template_mode,
        element_definition,
        complete_name,
        attributes,
        tag.is_synthetic(),
        tag.is_minimized(),
        tag.get_template_name().cloned(),
        tag.get_line(),
        tag.get_col(),
    )
    .map_err(|error| processor_cause("Could not normalize standalone element tag", error))?;
    Ok(Arc::new(engine_tag))
}

fn normalize_open_tag(
    tag: Arc<dyn IOpenElementTag>,
    configuration: &dyn IEngineConfiguration,
) -> Result<Arc<dyn IProcessableElementTag>, Box<dyn TemplateEngineException>> {
    if let Some(engine_tag) = Arc::clone(&tag).into_engine_open_element_tag() {
        return Ok(engine_tag);
    }
    let template_mode = tag.get_template_mode();
    let complete_name = tag.get_element_complete_name().clone();
    let element_definition = configuration
        .get_element_definitions()
        .for_name(Some(template_mode), Some(&complete_name))
        .map_err(|error| processor_cause("Could not normalize open element definition", error))?;
    let attributes = normalize_attributes(tag.as_ref(), configuration)?;
    Ok(Arc::new(OpenElementTag::with_location(
        template_mode,
        element_definition,
        complete_name,
        attributes,
        tag.is_synthetic(),
        tag.get_template_name().cloned(),
        tag.get_line(),
        tag.get_col(),
    )))
}

fn normalize_attributes(
    tag: &dyn IProcessableElementTag,
    configuration: &dyn IEngineConfiguration,
) -> Result<Option<Arc<Attributes>>, Box<dyn TemplateEngineException>> {
    let original = tag.get_all_attributes();
    if original.is_empty() {
        return Ok(None);
    }
    let mut normalized = Vec::with_capacity(original.len());
    for attribute in original {
        let complete_name = attribute.get_attribute_complete_name().clone();
        let definition = configuration
            .get_attribute_definitions()
            .for_name(Some(tag.get_template_mode()), Some(&complete_name))
            .map_err(|error| {
                processor_cause("Could not normalize element attribute definition", error)
            })?;
        normalized.push(Arc::new(Attribute::new(
            definition,
            complete_name,
            attribute.get_operator().cloned(),
            attribute.get_value().cloned(),
            attribute.get_value_quotes(),
            attribute.get_template_name().cloned(),
            attribute.get_line(),
            attribute.get_col(),
        )));
    }
    let spaces = vec![crate::util::JavaString::from_rust_str(" "); normalized.len()];
    Ok(Some(Attributes::new(Some(normalized), Some(spaces))))
}

fn require_engine_processable_tag(
    tag: &dyn IProcessableElementTag,
) -> Result<&super::AbstractProcessableElementTag, Box<dyn TemplateEngineException>> {
    tag.as_engine_processable_element_tag()
        .ok_or_else(|| processing_error("Cannot process a non-engine processable element tag"))
}

fn into_standalone_tag(
    tag: Arc<dyn IProcessableElementTag>,
) -> Result<Arc<dyn IStandaloneElementTag>, Box<dyn TemplateEngineException>> {
    tag.into_standalone_element_tag().ok_or_else(|| {
        processing_error("Element attribute modification changed standalone tag kind")
    })
}

fn into_open_tag(
    tag: Arc<dyn IProcessableElementTag>,
) -> Result<Arc<dyn IOpenElementTag>, Box<dyn TemplateEngineException>> {
    tag.into_open_element_tag()
        .ok_or_else(|| processing_error("Element attribute modification changed open tag kind"))
}

fn reset_model_slot(
    slot: &mut Option<Model>,
    create_if_null: bool,
    configuration: Arc<dyn IEngineConfiguration>,
    template_mode: TemplateMode,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if let Some(model) = slot {
        model.reset().map_err(model_error)?;
    } else if create_if_null {
        *slot = Some(Model::new(configuration, template_mode));
    }
    Ok(())
}

fn add_text_to_model(
    model: Option<&mut Model>,
    value: Arc<dyn crate::util::JavaCharSequence>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    let text: Arc<dyn ITemplateEvent> = Arc::new(Text::new(Some(value)));
    model
        .expect("Processor action initialized its target model")
        .add(Some(text))
        .map_err(model_error)
}

fn apply_common_tag_actions(
    vars: &mut ProcessorExecutionVars,
    handler: &ElementTagStructureHandler,
    configuration: Arc<dyn IEngineConfiguration>,
    template_mode: TemplateMode,
    open_element: bool,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if handler.insert_before_model {
        reset_model_slot(
            &mut vars.model_before,
            true,
            Arc::clone(&configuration),
            template_mode,
        )?;
        vars.model_before
            .as_mut()
            .expect("modelBefore was initialized")
            .add_model(handler.insert_before_model_value.as_deref())
            .map_err(model_error)?;
    } else if handler.insert_immediately_after_model {
        if vars.model_after.is_none() {
            reset_model_slot(
                &mut vars.model_after,
                true,
                Arc::clone(&configuration),
                template_mode,
            )?;
        }
        vars.model_after_processable = handler.insert_immediately_after_model_processable;
        vars.model_after
            .as_mut()
            .expect("modelAfter was initialized")
            .insert_model(0, handler.insert_immediately_after_model_value.as_deref())
            .map_err(model_error)?;
    } else if handler.replace_with_text {
        reset_model_slot(
            &mut vars.model_after,
            true,
            Arc::clone(&configuration),
            template_mode,
        )?;
        vars.model_after_processable = handler.replace_with_text_processable;
        add_text_to_model(
            vars.model_after.as_mut(),
            Arc::new(
                handler
                    .replace_with_text_value
                    .clone()
                    .expect("replaceWith text action requires text"),
            ),
        )?;
        vars.discard_event = true;
        if open_element {
            vars.skip_body = super::SkipBody::SkipAll;
            vars.skip_close_tag = true;
        }
    } else if handler.replace_with_model {
        reset_model_slot(
            &mut vars.model_after,
            true,
            Arc::clone(&configuration),
            template_mode,
        )?;
        vars.model_after_processable = handler.replace_with_model_processable;
        vars.model_after
            .as_mut()
            .expect("modelAfter was initialized")
            .add_model(handler.replace_with_model_value.as_deref())
            .map_err(model_error)?;
        vars.discard_event = true;
        if open_element {
            vars.skip_body = super::SkipBody::SkipAll;
            vars.skip_close_tag = true;
        }
    } else if handler.remove_element {
        reset_model_slot(
            &mut vars.model_after,
            false,
            Arc::clone(&configuration),
            template_mode,
        )?;
        vars.discard_event = true;
        if open_element {
            vars.skip_body = super::SkipBody::SkipAll;
            vars.skip_close_tag = true;
        }
    } else if handler.remove_tags {
        vars.discard_event = true;
        if open_element {
            vars.skip_close_tag = true;
        }
    } else if open_element && handler.remove_body {
        reset_model_slot(
            &mut vars.model_after,
            false,
            Arc::clone(&configuration),
            template_mode,
        )?;
        vars.skip_body = super::SkipBody::SkipAll;
    } else if open_element && handler.remove_all_but_first_child {
        reset_model_slot(&mut vars.model_after, false, configuration, template_mode)?;
        vars.skip_body = super::SkipBody::ProcessOneElement;
    }
    Ok(())
}

fn reject_modified_body_for_model_processor(
    vars: &ProcessorExecutionVars,
    processor_name: &str,
    event: &dyn ITemplateEvent,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if model_has_events(&vars.model_before) || model_has_events(&vars.model_after) {
        return Err(location_error(
            format!(
                "Cannot execute model processor {processor_name} as the body of the target element has already been modified by a previously executed processor on the same tag. Model processors cannot execute on already-modified bodies as these might contain unprocessable events (e.g. as a result of a 'th:text' or similar)"
            ),
            event,
        ));
    }
    Ok(())
}

fn model_has_events(model: &Option<Model>) -> bool {
    model.as_ref().is_some_and(|model| !model.queue.is_empty())
}

fn clone_model(model: &Model) -> Model {
    let mut clone = Model::new(
        model.get_configuration_arc(),
        model.get_template_mode_value(),
    );
    clone.reset_as_clone_of(model);
    clone
}

fn take_gathered_model(
    controller: &Rc<RefCell<TemplateModelController>>,
) -> Result<Rc<RefCell<dyn IGatheringModelProcessable>>, Box<dyn TemplateEngineException>> {
    let gathered = controller
        .borrow()
        .get_gathered_model()
        .ok_or_else(|| processing_error("Model controller did not create a gathering model"))?;
    controller.borrow_mut().reset_gathering();
    Ok(gathered)
}

fn process_or_queue_gathering(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    gathering: Rc<RefCell<dyn IGatheringModelProcessable>>,
    throttle_engine: bool,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if throttle_engine {
        queue_processable(
            state,
            Box::new(SharedGatheringModelProcessable { inner: gathering }),
        )
    } else {
        gathering.borrow_mut().process().map(|_| ())
    }
}

fn process_or_queue_owned(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    mut processable: Box<dyn IEngineProcessable>,
    throttle_engine: bool,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if throttle_engine {
        queue_processable(state, processable)
    } else {
        processable.process().map(|_| ())
    }
}

fn decrease_context_level_or_queue(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    context: Option<Arc<dyn IEngineContext>>,
    throttle_engine: bool,
    flow_controller: Option<Arc<Mutex<TemplateFlowController>>>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    let stopped = flow_controller.as_ref().is_some_and(|flow| {
        flow.lock()
            .expect("template flow controller lock poisoned")
            .stop_processing
    });
    if !throttle_engine || !stopped {
        if let Some(context) = context {
            context.decrease_level();
        }
        return Ok(());
    }
    queue_processable(
        state,
        Box::new(DecreaseContextLevelProcessable::new(
            context,
            flow_controller.expect("throttled engine has flow controller"),
        )),
    )
}

fn processor_cause<E>(message: &str, error: E) -> Box<dyn TemplateEngineException>
where
    E: std::error::Error + Send + Sync + 'static,
{
    Box::new(TemplateProcessingException::with_cause(
        Some(message.to_owned()),
        error,
    ))
}

fn handle_open_element_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<dyn IOpenElementTag>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if queue_event_if_stopped(state, event.clone())? {
        return Ok(());
    }
    let controller = require_model_controller(&state.borrow())?;
    if !controller
        .borrow_mut()
        .should_process_open_element(event.clone())?
    {
        return Ok(());
    }

    let (
        context,
        engine_context,
        configuration,
        next,
        self_handler,
        flow_controller,
        throttle_engine,
        current_gathering_model,
    ) = {
        let mut state_ref = state.borrow_mut();
        (
            require_context(&state_ref)?,
            state_ref.engine_context.clone(),
            state_ref
                .configuration
                .clone()
                .ok_or_else(|| processing_error("Processor context has not been set"))?,
            require_next(&state_ref)?,
            require_self_handler(&state_ref)?,
            state_ref.flow_controller.clone(),
            state_ref.throttle_engine,
            state_ref.current_gathering_model.take(),
        )
    };

    if current_gathering_model.is_some()
        && let Some(engine_context) = &engine_context
    {
        engine_context.set_element_tag(None);
    }

    let mut tag = normalize_open_tag(event, configuration.as_ref())?;
    let engine_tag = require_engine_processable_tag(tag.as_ref())?;
    if current_gathering_model.is_none()
        && !engine_tag
            .has_associated_processors()
            .map_err(|error| processor_cause("Could not obtain associated processors", error))?
    {
        return next.borrow_mut().handle_open_element(into_open_tag(tag)?);
    }

    let mut vars = current_gathering_model
        .as_ref()
        .map_or_else(ProcessorExecutionVars::new, |gathering| {
            gathering.initialize_processor_execution_vars()
        });

    while !vars.discard_event {
        let engine_tag = require_engine_processable_tag(tag.as_ref())?;
        let Some(processor) = vars
            .processor_iterator
            .next(engine_tag)
            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?
        else {
            break;
        };

        let mut state_ref = state.borrow_mut();
        state_ref.element_tag_structure_handler.reset();
        state_ref.element_model_structure_handler.reset();

        if let Some(element_processor) = processor.as_element_tag_processor() {
            element_processor.process(
                context.as_ref(),
                tag.as_ref(),
                &mut state_ref.element_tag_structure_handler,
            )?;
            state_ref
                .element_tag_structure_handler
                .apply_context_modifications(engine_context.as_deref());
            tag = state_ref
                .element_tag_structure_handler
                .apply_attributes(configuration.get_attribute_definitions(), tag)
                .map_err(|error| {
                    processor_cause("Could not apply element attribute modifications", error)
                })?;

            if state_ref.element_tag_structure_handler.iterate_element {
                let iteration_variable = state_ref
                    .element_tag_structure_handler
                    .iter_variable_name
                    .clone()
                    .expect("iterateElement action requires an iteration variable");
                let status_variable = state_ref
                    .element_tag_structure_handler
                    .iter_status_variable_name
                    .clone();
                let iterated_object = state_ref
                    .element_tag_structure_handler
                    .iterated_object
                    .clone();
                drop(state_ref);
                controller
                    .borrow_mut()
                    .start_gathering_iterated_open_model(
                        into_open_tag(tag)?,
                        &vars,
                        iteration_variable,
                        status_variable,
                        iterated_object,
                    )?;
                return Ok(());
            }

            if state_ref.element_tag_structure_handler.set_body_text {
                reset_model_slot(
                    &mut vars.model_after,
                    true,
                    configuration.clone(),
                    require_template_mode(&state_ref)?,
                )?;
                vars.model_after_processable = state_ref
                    .element_tag_structure_handler
                    .set_body_text_processable;
                add_text_to_model(
                    vars.model_after.as_mut(),
                    state_ref
                        .element_tag_structure_handler
                        .set_body_text_value
                        .clone()
                        .expect("setBody text action requires text"),
                )?;
                vars.skip_body = super::SkipBody::SkipAll;
            } else if state_ref.element_tag_structure_handler.set_body_model {
                reset_model_slot(
                    &mut vars.model_after,
                    true,
                    configuration.clone(),
                    require_template_mode(&state_ref)?,
                )?;
                vars.model_after_processable = state_ref
                    .element_tag_structure_handler
                    .set_body_model_processable;
                vars.model_after
                    .as_mut()
                    .expect("modelAfter was initialized")
                    .add_model(
                        state_ref
                            .element_tag_structure_handler
                            .set_body_model_value
                            .as_deref(),
                    )
                    .map_err(model_error)?;
                vars.skip_body = super::SkipBody::SkipAll;
            } else {
                apply_common_tag_actions(
                    &mut vars,
                    &state_ref.element_tag_structure_handler,
                    configuration.clone(),
                    require_template_mode(&state_ref)?,
                    true,
                )?;
            }
        } else if let Some(element_processor) = processor.as_element_model_processor() {
            if !vars.processor_iterator.last_was_repeated() {
                reject_modified_body_for_model_processor(
                    &vars,
                    processor.java_class_name(),
                    tag.as_ref(),
                )?;
                vars.processor_iterator
                    .set_last_to_be_repeated(require_engine_processable_tag(tag.as_ref())?)
                    .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
                drop(state_ref);
                controller
                    .borrow_mut()
                    .start_gathering_delayed_open_model(into_open_tag(tag)?, &vars)?;
                return Ok(());
            }

            let gathering = current_gathering_model.as_ref().ok_or_else(|| {
                processing_error("Repeated element model processor has no gathered model")
            })?;
            let mut processed_model = clone_model(gathering.inner_model());
            element_processor.process(
                context.as_ref(),
                &mut processed_model,
                &mut state_ref.element_model_structure_handler,
            )?;
            state_ref
                .element_model_structure_handler
                .apply_context_modifications(engine_context.as_deref());
            gathering.reset_gathered_skip_flags();
            if !gathering.inner_model().same_as(&processed_model) {
                reset_model_slot(
                    &mut vars.model_after,
                    true,
                    configuration.clone(),
                    require_template_mode(&state_ref)?,
                )?;
                vars.model_after
                    .as_mut()
                    .expect("modelAfter was initialized")
                    .add_model(Some(&processed_model))
                    .map_err(model_error)?;
                vars.model_after_processable = true;
                vars.discard_event = true;
                vars.skip_body = super::SkipBody::SkipAll;
                vars.skip_close_tag = true;
            }
        } else {
            return Err(processing_error(&format!(
                "An element has an associated processor of type {} which is neither a Tag Element Processor nor a Model Element Processor.",
                processor.java_class_name()
            )));
        }
    }

    let open = into_open_tag(tag)?;
    if throttle_engine
        && (model_has_events(&vars.model_before) || model_has_events(&vars.model_after))
    {
        return queue_processable(
            state,
            Box::new(OpenElementTagModelProcessable::new(
                open,
                vars,
                controller,
                flow_controller
                    .ok_or_else(|| processing_error("Throttled engine has no flow controller"))?,
                self_handler,
                next,
            )),
        );
    }

    if let Some(model) = &vars.model_before {
        model.process(next.borrow_mut().as_mut())?;
    }
    if !vars.discard_event {
        next.borrow_mut().handle_open_element(open)?;
    }
    if let Some(model) = &vars.model_after {
        if vars.model_after_processable {
            process_model_through_processor(state, model)?;
        } else {
            model.process(next.borrow_mut().as_mut())?;
        }
    }
    controller
        .borrow_mut()
        .skip(vars.skip_body, vars.skip_close_tag);
    Ok(())
}

fn handle_close_element_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<dyn ICloseElementTag>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    if queue_event_if_stopped(state, event.clone())? {
        return Ok(());
    }
    let controller = require_model_controller(&state.borrow())?;
    let process = if event.is_unmatched() {
        controller
            .borrow_mut()
            .should_process_unmatched_close_element(event.clone())?
    } else {
        controller
            .borrow_mut()
            .should_process_close_element(event.clone())?
    };
    if process {
        require_next(&state.borrow())?
            .borrow_mut()
            .handle_close_element(event)?;
    } else if !event.is_unmatched() && controller.borrow().is_gathering_finished() {
        let gathered = take_gathered_model(&controller)?;
        let throttle_engine = state.borrow().throttle_engine;
        process_or_queue_gathering(state, gathered, throttle_engine)?;
    }
    Ok(())
}

fn process_optional_model(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    model: Option<Model>,
    handler: TemplateHandlerHandle,
) -> Result<(), Box<dyn TemplateEngineException>> {
    let Some(model) = model.filter(|model| !model.queue.is_empty()) else {
        return Ok(());
    };
    let state_ref = state.borrow();
    let flow = state_ref.flow_controller.clone();
    let process_through_self = state_ref
        .self_handler
        .as_ref()
        .is_some_and(|self_handler| Rc::ptr_eq(self_handler, &handler));
    drop(state_ref);
    if let Some(flow) = flow {
        queue_processable(
            state,
            Box::new(SimpleModelProcessable::new(
                Rc::new(RefCell::new(model)),
                handler,
                flow,
            )),
        )
    } else {
        if process_through_self {
            process_model_through_processor(state, &model)
        } else {
            model.process(handler.borrow_mut().as_mut())
        }
    }
}

fn process_model_through_processor(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    model: &Model,
) -> Result<(), Box<dyn TemplateEngineException>> {
    // 使用栈上代理执行可处理模型，避免持有 self_handler 的 RefCell 借用时发生递归
    // Processor 调用；Java 原实现允许同一 handler 重入。
    let mut proxy = ProcessorTemplateHandlerProxy {
        state: Rc::downgrade(state),
    };
    model.process(&mut proxy)
}

fn queue_event_if_stopped<T>(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: Arc<T>,
) -> Result<bool, Box<dyn TemplateEngineException>>
where
    T: ITemplateEvent + ?Sized + 'static,
    Arc<T>: Into<Arc<dyn ITemplateEvent>>,
{
    let stopped = {
        let state = state.borrow();
        state.throttle_engine
            && state.flow_controller.as_ref().is_some_and(|controller| {
                controller
                    .lock()
                    .expect("template flow controller lock poisoned")
                    .stop_processing
            })
    };
    if !stopped {
        return Ok(false);
    }
    let event = event.into();
    let mut state_ref = state.borrow_mut();
    if let Some(model) = &state_ref.queued_events_model {
        model.borrow_mut().add(Some(event)).map_err(model_error)?;
    } else {
        let mut model = new_model(&state_ref)?;
        model.add(Some(event)).map_err(model_error)?;
        let model = Rc::new(RefCell::new(model));
        let processable: ProcessableHandle =
            Rc::new(RefCell::new(Box::new(SimpleModelProcessable::new(
                Rc::clone(&model),
                require_self_handler(&state_ref)?,
                state_ref
                    .flow_controller
                    .clone()
                    .expect("stopped throttled engine has flow controller"),
            ))));
        state_ref.pending_processings.insert(0, processable.clone());
        state_ref.queued_events_model = Some(model);
        state_ref.queued_events_processable = Some(processable);
    }
    if let Some(flow) = &state_ref.flow_controller {
        flow.lock()
            .expect("template flow controller lock poisoned")
            .processor_template_handler_pending = true;
    }
    Ok(true)
}

fn queue_processable(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    processable: Box<dyn IEngineProcessable>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    let handle: ProcessableHandle = Rc::new(RefCell::new(processable));
    let should_process = {
        let mut state = state.borrow_mut();
        state.pending_processings.push(handle.clone());
        !state.flow_controller.as_ref().is_some_and(|controller| {
            controller
                .lock()
                .expect("template flow controller lock poisoned")
                .stop_processing
        })
    };
    if !should_process {
        if let Some(flow) = &state.borrow().flow_controller {
            flow.lock()
                .expect("template flow controller lock poisoned")
                .processor_template_handler_pending = true;
        }
        return Ok(());
    }
    let complete = handle.borrow_mut().process()?;
    let mut state = state.borrow_mut();
    if complete
        && state
            .pending_processings
            .last()
            .is_some_and(|last| Rc::ptr_eq(last, &handle))
    {
        state.pending_processings.pop();
    }
    if let Some(flow) = &state.flow_controller {
        flow.lock()
            .expect("template flow controller lock poisoned")
            .processor_template_handler_pending = !complete;
    }
    Ok(())
}

fn handle_pending_state(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    loop {
        let handle = {
            let state_ref = state.borrow();
            let Some(flow) = &state_ref.flow_controller else {
                return Ok(());
            };
            if flow
                .lock()
                .expect("template flow controller lock poisoned")
                .stop_processing
            {
                flow.lock()
                    .expect("template flow controller lock poisoned")
                    .processor_template_handler_pending = true;
                return Ok(());
            }
            state_ref.pending_processings.last().cloned()
        };
        let Some(handle) = handle else {
            if let Some(flow) = &state.borrow().flow_controller {
                flow.lock()
                    .expect("template flow controller lock poisoned")
                    .processor_template_handler_pending = false;
            }
            return Ok(());
        };
        if !handle.borrow_mut().process()? {
            if let Some(flow) = &state.borrow().flow_controller {
                flow.lock()
                    .expect("template flow controller lock poisoned")
                    .processor_template_handler_pending = true;
            }
            return Ok(());
        }
        let mut state_ref = state.borrow_mut();
        if state_ref
            .pending_processings
            .last()
            .is_some_and(|last| Rc::ptr_eq(last, &handle))
        {
            state_ref.pending_processings.pop();
        }
        if state_ref
            .queued_events_processable
            .as_ref()
            .is_some_and(|queued| Rc::ptr_eq(queued, &handle))
        {
            state_ref.queued_events_processable = None;
            state_ref.queued_events_model = None;
        }
    }
}

/// 对应 Java 语义：`ProcessorTemplateHandler` 的 `perform_teardown_checks` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn perform_teardown_checks(
    state: &Rc<RefCell<ProcessorTemplateHandlerState>>,
    event: &dyn ITemplateEnd,
) -> Result<(), Box<dyn TemplateEngineException>> {
    let state = state.borrow();
    let controller = require_model_controller(&state)?;
    let model_level = controller.borrow().get_model_level();
    if model_level != 0 {
        return Err(location_error(
            format!(
                "Bad markup or template processing sequence. Model level is != 0 ({model_level}) at template end."
            ),
            event,
        ));
    }
    if let (Some(context), Some(initial)) = (&state.engine_context, state.initial_context_level) {
        if context.level() != initial {
            return Err(location_error(
                format!(
                    "Bad markup or template processing sequence. Context level after processing ({}) does not correspond to context level before processing ({initial}).",
                    context.level()
                ),
                event,
            ));
        }
        let stack = context.get_element_stack_above(initial);
        if !stack.is_empty() {
            return Err(location_error(
                "Bad markup or template processing sequence. Element stack after processing is not empty."
                    .to_owned(),
                event,
            ));
        }
    }
    Ok(())
}

fn new_model(
    state: &ProcessorTemplateHandlerState,
) -> Result<Model, Box<dyn TemplateEngineException>> {
    Ok(Model::new(
        state
            .configuration
            .clone()
            .ok_or_else(|| processing_error("Processor context has not been set"))?,
        require_template_mode(state)?,
    ))
}

fn require_context(
    state: &ProcessorTemplateHandlerState,
) -> Result<Arc<dyn ITemplateContext>, Box<dyn TemplateEngineException>> {
    state
        .context
        .clone()
        .ok_or_else(|| processing_error("Processor context has not been set"))
}

fn require_template_mode(
    state: &ProcessorTemplateHandlerState,
) -> Result<TemplateMode, Box<dyn TemplateEngineException>> {
    state
        .template_mode
        .ok_or_else(|| processing_error("Processor template mode has not been set"))
}

fn require_next(
    state: &ProcessorTemplateHandlerState,
) -> Result<TemplateHandlerHandle, Box<dyn TemplateEngineException>> {
    state
        .next
        .clone()
        .ok_or_else(|| processing_error("Processor next handler has not been set"))
}

fn require_self_handler(
    state: &ProcessorTemplateHandlerState,
) -> Result<TemplateHandlerHandle, Box<dyn TemplateEngineException>> {
    state
        .self_handler
        .clone()
        .ok_or_else(|| processing_error("Processor self handler has not been initialized"))
}

fn require_model_controller(
    state: &ProcessorTemplateHandlerState,
) -> Result<Rc<RefCell<TemplateModelController>>, Box<dyn TemplateEngineException>> {
    state
        .model_controller
        .clone()
        .ok_or_else(|| processing_error("Processor model controller has not been initialized"))
}

fn model_error(error: crate::model::IModelError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some("Could not apply Processor model action".to_owned()),
        error,
    ))
}

fn processing_error(message: &str) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::new(Some(message.to_owned())))
}

fn location_error(message: String, event: &dyn ITemplateEvent) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_location(
        Some(message),
        event
            .get_template_name()
            .map(crate::util::JavaString::to_string_lossy),
        event.get_line(),
        event.get_col(),
    ))
}
