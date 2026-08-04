use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};

use crate::IEngineConfiguration;
use crate::TemplateMode;
use crate::context::IEngineContext;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IElementTag, IOpenElementTag,
    IProcessableElementTag, IProcessingInstruction, IStandaloneElementTag, ITemplateEvent, IText,
    IXMLDeclaration,
};
use crate::util::Utf16String;

use super::TemplateHandlerHandle;
use super::gathering_model_processable::GatheringModelProcessable;
use super::i_gathering_model_processable::IGatheringModelProcessable;
use super::iterated_gathering_model_processable::IteratedGatheringModelProcessable;
use super::processor_execution_vars::ProcessorExecutionVars;
use super::template_flow_controller::TemplateFlowController;

const DEFAULT_MODEL_LEVELS: usize = 25;

/// 当前元素 body 与子事件的处理策略。
///
/// 对应 Java: `TemplateModelController.SkipBody`。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkipBody {
    Process,
    SkipAll,
    SkipElements,
    ProcessOneElement,
}

impl SkipBody {
    const fn process_elements(self) -> bool {
        matches!(self, Self::Process | Self::ProcessOneElement)
    }

    const fn process_non_elements(self) -> bool {
        !matches!(self, Self::SkipAll)
    }

    const fn process_children(self) -> bool {
        matches!(self, Self::Process | Self::ProcessOneElement)
    }
}

/// 控制 Model 层级、skip 状态和延迟事件收集。
///
/// 对应 Java: `org.thymeleaf.engine.TemplateModelController`。
pub(crate) struct TemplateModelController {
    self_reference: Weak<RefCell<TemplateModelController>>,
    configuration: Arc<dyn IEngineConfiguration>,
    template_mode: TemplateMode,
    processor_template_handler: TemplateHandlerHandle,
    context: Option<Arc<dyn IEngineContext>>,
    template_flow_controller: Option<Arc<Mutex<TemplateFlowController>>>,
    gathered_model: Option<Rc<RefCell<dyn IGatheringModelProcessable>>>,
    skip_body: SkipBody,
    skip_body_by_level: Vec<SkipBody>,
    skip_close_tag_by_level: Vec<bool>,
    unskipped_first_element_by_level: Vec<Option<Arc<dyn IProcessableElementTag>>>,
    last_event: Option<Arc<dyn ITemplateEvent>>,
    second_to_last_event: Option<Arc<dyn ITemplateEvent>>,
    model_level: usize,
}

impl TemplateModelController {
    /// 创建根层为正常处理的 Model controller。
    /// 对应 Java 语义：`TemplateModelController` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(
        configuration: Arc<dyn IEngineConfiguration>,
        template_mode: TemplateMode,
        processor_template_handler: TemplateHandlerHandle,
        context: Option<Arc<dyn IEngineContext>>,
    ) -> Rc<RefCell<Self>> {
        Rc::new_cyclic(|weak| {
            RefCell::new(Self {
                self_reference: weak.clone(),
                configuration,
                template_mode,
                processor_template_handler,
                context,
                template_flow_controller: None,
                gathered_model: None,
                skip_body: SkipBody::Process,
                skip_body_by_level: vec![SkipBody::Process; DEFAULT_MODEL_LEVELS],
                skip_close_tag_by_level: vec![false; DEFAULT_MODEL_LEVELS],
                unskipped_first_element_by_level: vec![None; DEFAULT_MODEL_LEVELS],
                last_event: None,
                second_to_last_event: None,
                model_level: 0,
            })
        })
    }

    /// 设置可选节流流控对象。
    /// 对应 Java: `TemplateModelController#setTemplateFlowController()`。
    pub(crate) fn set_template_flow_controller(
        &mut self,
        template_flow_controller: Option<Arc<Mutex<TemplateFlowController>>>,
    ) {
        self.template_flow_controller = template_flow_controller;
    }

    /// 返回当前 Model 嵌套层。
    pub(crate) const fn get_model_level(&self) -> usize {
        self.model_level
    }

    /// 从开放元素开始收集延迟 Model。
    /// 对应 Java 语义：`TemplateModelController` 的 `start_gathering_delayed_open_model` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn start_gathering_delayed_open_model(
        &mut self,
        first_tag: Arc<dyn IOpenElementTag>,
        processor_execution_vars: &ProcessorExecutionVars,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.model_level = self.model_level.checked_sub(1).ok_or_else(|| {
            Box::new(TemplateProcessingException::new(Some(
                "Cannot start delayed open model at model level zero".to_owned(),
            ))) as Box<dyn TemplateEngineException>
        })?;
        let mut gathering = GatheringModelProcessable::new(
            Arc::clone(&self.configuration),
            Rc::clone(&self.processor_template_handler),
            self.require_engine_context()?,
            self.self_reference.clone(),
            self.template_flow_controller.clone(),
            self.skip_body_by_level[self.model_level],
            self.skip_close_tag_by_level[self.model_level],
            processor_execution_vars,
        );
        gathering.gather_open_element(first_tag)?;
        self.gathered_model = Some(Rc::new(RefCell::new(gathering)));
        Ok(())
    }

    /// 从独立元素开始收集延迟 Model。
    /// 对应 Java 语义：`TemplateModelController` 的 `start_gathering_delayed_standalone_model` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn start_gathering_delayed_standalone_model(
        &mut self,
        first_tag: Arc<dyn IStandaloneElementTag>,
        processor_execution_vars: &ProcessorExecutionVars,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let gathered_skip_body =
            if self.skip_body_by_level[self.model_level] == SkipBody::SkipElements {
                SkipBody::ProcessOneElement
            } else {
                self.skip_body_by_level[self.model_level]
            };
        let mut gathering = GatheringModelProcessable::new(
            Arc::clone(&self.configuration),
            Rc::clone(&self.processor_template_handler),
            self.require_engine_context()?,
            self.self_reference.clone(),
            self.template_flow_controller.clone(),
            gathered_skip_body,
            self.skip_close_tag_by_level[self.model_level],
            processor_execution_vars,
        );
        gathering.gather_standalone_element(first_tag)?;
        self.gathered_model = Some(Rc::new(RefCell::new(gathering)));
        Ok(())
    }

    /// 从开放元素开始收集迭代 Model。
    ///
    /// 对应 Java:
    /// `TemplateModelController#startGatheringIteratedModel(IOpenElementTag,...)`。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn start_gathering_iterated_open_model(
        &mut self,
        first_tag: Arc<dyn IOpenElementTag>,
        processor_execution_vars: &ProcessorExecutionVars,
        iter_variable_name: Utf16String,
        iter_status_variable_name: Option<Utf16String>,
        iterated_object: Option<Arc<crate::expression::TemplateValue>>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.model_level = self.model_level.checked_sub(1).ok_or_else(|| {
            Box::new(TemplateProcessingException::new(Some(
                "Cannot start iterated open model at model level zero".to_owned(),
            ))) as Box<dyn TemplateEngineException>
        })?;
        let preceding_whitespace =
            self.compute_whitespace_preceding_iteration(first_tag.as_ref())?;
        let mut gathering = IteratedGatheringModelProcessable::new(
            Arc::clone(&self.configuration),
            Rc::clone(&self.processor_template_handler),
            self.require_engine_context()?,
            self.self_reference.clone(),
            self.template_flow_controller.clone(),
            self.skip_body_by_level[self.model_level],
            self.skip_close_tag_by_level[self.model_level],
            processor_execution_vars,
            iter_variable_name,
            iter_status_variable_name,
            iterated_object,
            preceding_whitespace,
        );
        gathering.gather_open_element(first_tag)?;
        self.gathered_model = Some(Rc::new(RefCell::new(gathering)));
        Ok(())
    }

    /// 从 standalone 元素开始收集迭代 Model。
    ///
    /// 对应 Java:
    /// `TemplateModelController#startGatheringIteratedModel(IStandaloneElementTag,...)`。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn start_gathering_iterated_standalone_model(
        &mut self,
        first_tag: Arc<dyn IStandaloneElementTag>,
        processor_execution_vars: &ProcessorExecutionVars,
        iter_variable_name: Utf16String,
        iter_status_variable_name: Option<Utf16String>,
        iterated_object: Option<Arc<crate::expression::TemplateValue>>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let gathered_skip_body =
            if self.skip_body_by_level[self.model_level] == SkipBody::SkipElements {
                SkipBody::ProcessOneElement
            } else {
                self.skip_body_by_level[self.model_level]
            };
        let preceding_whitespace =
            self.compute_whitespace_preceding_iteration(first_tag.as_ref())?;
        let mut gathering = IteratedGatheringModelProcessable::new(
            Arc::clone(&self.configuration),
            Rc::clone(&self.processor_template_handler),
            self.require_engine_context()?,
            self.self_reference.clone(),
            self.template_flow_controller.clone(),
            gathered_skip_body,
            self.skip_close_tag_by_level[self.model_level],
            processor_execution_vars,
            iter_variable_name,
            iter_status_variable_name,
            iterated_object,
            preceding_whitespace,
        );
        gathering.gather_standalone_element(first_tag)?;
        self.gathered_model = Some(Rc::new(RefCell::new(gathering)));
        Ok(())
    }

    /// 将 standalone 标签转换为字段等价的 open+close 合成 Model。
    ///
    /// 对应 Java: `TemplateModelController#createStandaloneEquivalentModel`。
    pub(crate) fn create_standalone_equivalent_model(
        &self,
        standalone_element_tag: &dyn IStandaloneElementTag,
        processor_execution_vars: &ProcessorExecutionVars,
    ) -> Result<GatheringModelProcessable, Box<dyn TemplateEngineException>> {
        let engine_tag = standalone_element_tag
            .as_engine_standalone_element_tag()
            .ok_or_else(|| {
                Box::new(TemplateProcessingException::new(Some(
                    "Cannot create standalone equivalent model from a non-engine tag".to_owned(),
                ))) as Box<dyn TemplateEngineException>
            })?;
        let gathered_skip_body =
            if self.skip_body_by_level[self.model_level] == SkipBody::SkipElements {
                SkipBody::ProcessOneElement
            } else {
                self.skip_body_by_level[self.model_level]
            };
        let mut gathering = GatheringModelProcessable::new(
            Arc::clone(&self.configuration),
            Rc::clone(&self.processor_template_handler),
            self.require_engine_context()?,
            self.self_reference.clone(),
            self.template_flow_controller.clone(),
            gathered_skip_body,
            self.skip_close_tag_by_level[self.model_level],
            processor_execution_vars,
        );
        gathering.gather_open_element(Arc::new(engine_tag.as_synthetic_open_equivalent()))?;
        gathering.gather_close_element(Arc::new(engine_tag.as_synthetic_close_equivalent()))?;
        Ok(gathering)
    }

    /// 判断当前收集对象是否已经抵达完整元素边界。
    /// 对应 Java: `TemplateModelController#isGatheringFinished()`。
    pub(crate) fn is_gathering_finished(&self) -> bool {
        self.gathered_model
            .as_ref()
            .is_some_and(|model| model.borrow().is_gathering_finished())
    }

    /// 返回当前收集对象的共享身份。
    /// 对应 Java: `TemplateModelController#getGatheredModel()`。
    pub(crate) fn get_gathered_model(&self) -> Option<Rc<RefCell<dyn IGatheringModelProcessable>>> {
        self.gathered_model.clone()
    }

    /// 清除已经进入处理队列的收集对象。
    /// 对应 Java: `TemplateModelController#resetGathering()`。
    pub(crate) fn reset_gathering(&mut self) {
        self.gathered_model = None;
    }

    /// 同时设置 body 与 close-tag skip 状态。
    /// 对应 Java: `TemplateModelController#skip()`。
    pub(crate) fn skip(&mut self, skip_body: SkipBody, skip_close_tag: bool) {
        self.skip_body_by_level[self.model_level] = skip_body;
        self.skip_body = skip_body;
        if skip_close_tag {
            assert!(
                self.model_level > 0,
                "Cannot set containing close tag to skip when model level is zero"
            );
            self.skip_close_tag_by_level[self.model_level - 1] = true;
        }
    }

    /// 根据当前 skip/收集状态处理 Text。
    /// 对应 Java: `TemplateModelController#shouldProcessText()`。
    pub(crate) fn should_process_text(
        &mut self,
        text: Arc<dyn IText>,
    ) -> Result<bool, Box<dyn TemplateEngineException>> {
        self.last_event = Some(text.clone());
        if let Some(gathering) = &self.gathered_model {
            gathering.borrow_mut().gather_text(text)?;
            return Ok(false);
        }
        Ok(self.skip_body.process_non_elements())
    }

    /// 根据当前 skip/收集状态处理 Comment。
    /// 对应 Java: `TemplateModelController#shouldProcessComment()`。
    pub(crate) fn should_process_comment(
        &mut self,
        comment: Arc<dyn IComment>,
    ) -> Result<bool, Box<dyn TemplateEngineException>> {
        self.last_event = Some(comment.clone());
        if let Some(gathering) = &self.gathered_model {
            gathering.borrow_mut().gather_comment(comment)?;
            return Ok(false);
        }
        Ok(self.skip_body.process_non_elements())
    }

    /// 根据当前 skip/收集状态处理 CDATA。
    /// 对应 Java 语义：`TemplateModelController` 的 `should_process_cdata_section` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn should_process_cdata_section(
        &mut self,
        cdata_section: Arc<dyn ICDATASection>,
    ) -> Result<bool, Box<dyn TemplateEngineException>> {
        self.last_event = Some(cdata_section.clone());
        if let Some(gathering) = &self.gathered_model {
            gathering.borrow_mut().gather_cdata_section(cdata_section)?;
            return Ok(false);
        }
        Ok(self.skip_body.process_non_elements())
    }

    /// 根据当前 skip/收集状态处理独立元素。
    /// 对应 Java: `TemplateModelController#shouldProcessStandaloneElement()`。
    pub(crate) fn should_process_standalone_element(
        &mut self,
        tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<bool, Box<dyn TemplateEngineException>> {
        self.second_to_last_event = self.last_event.take();
        self.last_event = Some(tag.clone());
        if let Some(gathering) = &self.gathered_model {
            gathering.borrow_mut().gather_standalone_element(tag)?;
            return Ok(false);
        }
        let mut process = self.skip_body.process_elements();
        if self.skip_body == SkipBody::ProcessOneElement {
            self.unskipped_first_element_by_level[self.model_level] = Some(tag.clone());
            self.skip(SkipBody::SkipElements, false);
            process = true;
        }
        if process && let Some(context) = &self.context {
            context.increase_level();
            context.set_element_tag(Some(tag));
        }
        Ok(process)
    }

    /// 根据当前 skip/收集状态处理开放元素并进入下一 Model 层。
    /// 对应 Java: `TemplateModelController#shouldProcessOpenElement()`。
    pub(crate) fn should_process_open_element(
        &mut self,
        tag: Arc<dyn IOpenElementTag>,
    ) -> Result<bool, Box<dyn TemplateEngineException>> {
        self.second_to_last_event = self.last_event.take();
        self.last_event = Some(tag.clone());
        if let Some(gathering) = &self.gathered_model {
            gathering.borrow_mut().gather_open_element(tag)?;
            return Ok(false);
        }
        let mut process = self.skip_body.process_elements();
        let transparent_first_child_wrapper =
            self.skip_body == SkipBody::ProcessOneElement && tag.is_synthetic();
        if self.skip_body == SkipBody::ProcessOneElement {
            if !transparent_first_child_wrapper {
                self.unskipped_first_element_by_level[self.model_level] = Some(tag.clone());
            }
        } else if self.skip_body == SkipBody::SkipElements
            && self.unskipped_first_element_by_level[self.model_level]
                .as_ref()
                .is_some_and(|first| {
                    Arc::ptr_eq(first, &(tag.clone() as Arc<dyn IProcessableElementTag>))
                })
        {
            self.skip(SkipBody::ProcessOneElement, false);
            process = true;
        }
        self.increase_model_level(tag);
        if transparent_first_child_wrapper {
            // HTML 自动补全的 `tbody` 等 synthetic 包装不应消耗
            // `th:remove="all-but-first"` 的“第一个真实子元素”名额。
            self.skip(SkipBody::ProcessOneElement, false);
        }
        Ok(process)
    }

    /// 处理匹配关闭元素并退出一层。
    /// 对应 Java: `TemplateModelController#shouldProcessCloseElement()`。
    pub(crate) fn should_process_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<bool, Box<dyn TemplateEngineException>> {
        if let Some(gathering) = &self.gathered_model {
            gathering.borrow_mut().gather_close_element(tag)?;
            return Ok(false);
        }
        let synthetic = tag.is_synthetic();
        let child_skip_body = self.skip_body;
        self.last_event = Some(tag);
        self.decrease_model_level();
        if synthetic
            && self.skip_body == SkipBody::ProcessOneElement
            && child_skip_body == SkipBody::SkipElements
        {
            self.skip(SkipBody::SkipElements, false);
            return Ok(true);
        }
        if self.skip_body == SkipBody::ProcessOneElement {
            self.skip(SkipBody::SkipElements, false);
            return Ok(!std::mem::take(
                &mut self.skip_close_tag_by_level[self.model_level],
            ));
        }
        if std::mem::take(&mut self.skip_close_tag_by_level[self.model_level]) {
            return Ok(false);
        }
        Ok(self.skip_body.process_elements())
    }

    /// 把不匹配关闭元素按非元素事件处理。
    /// 对应 Java: `TemplateModelController#shouldProcessUnmatchedCloseElement()`。
    pub(crate) fn should_process_unmatched_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<bool, Box<dyn TemplateEngineException>> {
        self.last_event = Some(tag.clone());
        if let Some(gathering) = &self.gathered_model {
            gathering.borrow_mut().gather_unmatched_close_element(tag)?;
            return Ok(false);
        }
        Ok(self.skip_body.process_non_elements())
    }

    /// 根据当前 skip/收集状态处理 DOCTYPE。
    /// 对应 Java: `TemplateModelController#shouldProcessDocType()`。
    pub(crate) fn should_process_doc_type(
        &mut self,
        event: Arc<dyn IDocType>,
    ) -> Result<bool, Box<dyn TemplateEngineException>> {
        self.last_event = Some(event.clone());
        if let Some(gathering) = &self.gathered_model {
            gathering.borrow_mut().gather_doc_type(event)?;
            return Ok(false);
        }
        Ok(self.skip_body.process_non_elements())
    }

    /// 根据当前 skip/收集状态处理 XML declaration。
    /// 对应 Java 语义：`TemplateModelController` 的 `should_process_xml_declaration` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn should_process_xml_declaration(
        &mut self,
        event: Arc<dyn IXMLDeclaration>,
    ) -> Result<bool, Box<dyn TemplateEngineException>> {
        self.last_event = Some(event.clone());
        if let Some(gathering) = &self.gathered_model {
            gathering.borrow_mut().gather_xml_declaration(event)?;
            return Ok(false);
        }
        Ok(self.skip_body.process_non_elements())
    }

    /// 根据当前 skip/收集状态处理 processing instruction。
    /// 对应 Java: `TemplateModelController#shouldProcessProcessingInstruction()`。
    pub(crate) fn should_process_processing_instruction(
        &mut self,
        event: Arc<dyn IProcessingInstruction>,
    ) -> Result<bool, Box<dyn TemplateEngineException>> {
        self.last_event = Some(event.clone());
        if let Some(gathering) = &self.gathered_model {
            gathering
                .borrow_mut()
                .gather_processing_instruction(event)?;
            return Ok(false);
        }
        Ok(self.skip_body.process_non_elements())
    }

    fn compute_whitespace_preceding_iteration(
        &self,
        iterated_element: &dyn IElementTag,
    ) -> Result<Option<Arc<dyn IText>>, Box<dyn TemplateEngineException>> {
        let applicable = self.template_mode == TemplateMode::XML
            || (self.template_mode == TemplateMode::HTML
                && is_iteration_whitespace_applicable(iterated_element));
        if !applicable {
            return Ok(None);
        }
        let Some(event) = self.second_to_last_event.clone() else {
            return Ok(None);
        };
        let Some(text) = event.into_text() else {
            return Ok(None);
        };
        let length = text.java_length().map_err(text_error)?;
        for index in 0..length {
            if !is_java_whitespace(text.java_char_at(index).map_err(text_error)?) {
                return Ok(None);
            }
        }
        Ok(Some(text))
    }

    fn increase_model_level(&mut self, tag: Arc<dyn IOpenElementTag>) {
        self.model_level += 1;
        if self.skip_body_by_level.len() == self.model_level {
            let new_len = self.skip_body_by_level.len() + DEFAULT_MODEL_LEVELS / 2;
            self.skip_body_by_level.resize(new_len, SkipBody::Process);
            self.skip_close_tag_by_level.resize(new_len, false);
            self.unskipped_first_element_by_level
                .resize_with(new_len, || None);
        }
        let child_skip = if self.skip_body.process_children() {
            SkipBody::Process
        } else {
            SkipBody::SkipAll
        };
        self.skip(child_skip, false);
        self.skip_close_tag_by_level[self.model_level] = false;
        self.unskipped_first_element_by_level[self.model_level] = None;
        if let Some(context) = &self.context {
            context.increase_level();
            context.set_element_tag(Some(tag));
        }
    }

    fn decrease_model_level(&mut self) {
        assert!(
            self.model_level > 0,
            "Cannot decrease model level below zero"
        );
        self.model_level -= 1;
        self.skip_body = self.skip_body_by_level[self.model_level];
        if let Some(context) = &self.context {
            context.decrease_level();
        }
    }

    fn require_engine_context(
        &self,
    ) -> Result<Arc<dyn IEngineContext>, Box<dyn TemplateEngineException>> {
        self.context.clone().ok_or_else(|| {
            Box::new(TemplateProcessingException::new(Some(
                "Neither iteration nor model gathering are supported because local variable support is DISABLED. This is due to the use of an implementation of the org.thymeleaf.context.ITemplateContext interface that does not provide local-variable support. In order to have local-variable support, the context implementation should also implement the org.thymeleaf.context.IEngineContext interface".to_owned(),
            ))) as Box<dyn TemplateEngineException>
        })
    }
}

fn is_iteration_whitespace_applicable(element: &dyn IElementTag) -> bool {
    const NAMES: &[&str] = &[
        "address",
        "article",
        "aside",
        "audio",
        "blockquote",
        "canvas",
        "dd",
        "div",
        "dl",
        "dt",
        "fieldset",
        "figcaption",
        "figure",
        "footer",
        "form",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "header",
        "hgroup",
        "hr",
        "li",
        "main",
        "nav",
        "noscript",
        "ol",
        "option",
        "output",
        "p",
        "pre",
        "section",
        "table",
        "tbody",
        "td",
        "tfoot",
        "th",
        "tr",
        "ul",
        "video",
    ];
    let name = element
        .get_element_definition()
        .get_element_name()
        .as_element_name();
    name.get_prefix().is_none()
        && NAMES
            .iter()
            .any(|candidate| name.get_element_name() == &Utf16String::from_rust_str(candidate))
}

fn is_java_whitespace(unit: u16) -> bool {
    matches!(
        unit,
        0x0009..=0x000D
            | 0x001C..=0x0020
            | 0x1680
            | 0x2000..=0x2006
            | 0x2008..=0x200A
            | 0x2028
            | 0x2029
            | 0x205F
            | 0x3000
    )
}

fn text_error(error: crate::util::TextUtilsError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some("Could not inspect whitespace preceding iteration".to_owned()),
        error,
    ))
}
