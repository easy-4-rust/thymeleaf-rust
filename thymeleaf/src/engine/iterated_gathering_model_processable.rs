use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Weak;
use std::sync::{Arc, Mutex};

use crate::context::IEngineContext;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{TemplateObject, TemplateObjectPropertyError, TemplateValue};
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IModel, IOpenElementTag,
    IProcessingInstruction, IStandaloneElementTag, ITemplateEvent, IText, IXMLDeclaration,
};
use crate::util::{CharSequenceValue, NumberValue, Utf16String};
use crate::{IEngineConfiguration, TemplateMode};

use super::abstract_gathering_model_processable::AbstractGatheringModelProcessable;
use super::i_engine_processable::EngineProcessableResult;
use super::i_gathering_model_processable::IGatheringModelProcessable;
use super::model::Model;
use super::processor_execution_vars::ProcessorExecutionVars;
use super::template_flow_controller::TemplateFlowController;
use super::{
    DataDrivenTemplateIterator, IEngineProcessable, IterationStatusVar, SkipBody,
    TemplateHandlerHandle, TemplateModelController, Text,
};

const DEFAULT_STATUS_VAR_SUFFIX: &str = "Stat";

#[derive(Clone, Copy)]
enum IterationWhiteSpaceHandling {
    Zero,
    Single,
    Multiple,
}

struct IterationModels {
    first: Option<Model>,
    middle: Option<Model>,
    last: Option<Model>,
}

/// 收集一个元素模型并依次对迭代值重放。
///
/// 该对象保留 Java 对普通集合、Map、数组等价值、Iterable 快照与标量的迭代规则，
/// 复用同一个 `IterationStatusVar` 身份，并在每次迭代建立独立上下文层。文本模式
/// 首尾空白折叠以及标记模式前导空白复制与 Java 实现一致。
///
/// 对应 Java: `org.thymeleaf.engine.IteratedGatheringModelProcessable`。
pub(crate) struct IteratedGatheringModelProcessable {
    base: AbstractGatheringModelProcessable,
    context: Arc<dyn IEngineContext>,
    template_mode: TemplateMode,
    iter_variable_name: Utf16String,
    iter_status_variable_name: Utf16String,
    iter_status_variable: Arc<IterationStatusVar>,
    iterator: VecDeque<Arc<TemplateValue>>,
    data_driven_iterator: Option<Arc<dyn TemplateObject>>,
    preceding_whitespace: Option<Arc<dyn IText>>,
    iteration_models: Option<IterationModels>,
    iter: i32,
    iter_offset: usize,
    iter_model: Option<Model>,
}

impl IteratedGatheringModelProcessable {
    /// 创建尚未开始重放的迭代收集对象。
    ///
    /// 参数名称和构造顺序对应 Java 同名构造器；`iterated_object` 为 Java null 时
    /// 创建空迭代，普通标量按单元素集合处理。
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
        iter_variable_name: Utf16String,
        iter_status_variable_name: Option<Utf16String>,
        iterated_object: Option<Arc<TemplateValue>>,
        preceding_whitespace: Option<Arc<dyn IText>>,
    ) -> Self {
        let template_mode = context.get_template_mode();
        let (iterator, size, data_driven_iterator) = compute_iterated_object(iterated_object);
        let iter_status_variable_name = iter_status_variable_name
            .filter(|name| !is_empty_or_whitespace(name))
            .unwrap_or_else(|| {
                let mut units = iter_variable_name.as_utf16().to_vec();
                units.extend(DEFAULT_STATUS_VAR_SUFFIX.encode_utf16());
                Utf16String::from_utf16(units)
            });
        Self {
            base: AbstractGatheringModelProcessable::new(
                configuration,
                processor_template_handler,
                Arc::clone(&context),
                model_controller,
                flow_controller,
                gathered_skip_body,
                gathered_skip_close_tag,
                processor_execution_vars,
            ),
            context,
            template_mode,
            iter_variable_name,
            iter_status_variable_name,
            iter_status_variable: Arc::new(IterationStatusVar::new(size)),
            iterator,
            data_driven_iterator,
            preceding_whitespace,
            iteration_models: None,
            iter: 0,
            iter_offset: 0,
            iter_model: None,
        }
    }

    fn initialize_iteration_models(&mut self) -> Result<(), Box<dyn TemplateEngineException>> {
        let handling = if self.data_driven_iterator.is_some() {
            if self.data_driven_has_next() {
                self.iter_status_variable
                    .set_current(self.data_driven_next()?);
                IterationWhiteSpaceHandling::Single
            } else {
                IterationWhiteSpaceHandling::Zero
            }
        } else {
            match self.iterator.len() {
                0 => IterationWhiteSpaceHandling::Zero,
                1 => {
                    self.iter_status_variable
                        .set_current(self.iterator.pop_front());
                    IterationWhiteSpaceHandling::Single
                }
                _ => {
                    self.iter_status_variable
                        .set_current(self.iterator.pop_front());
                    IterationWhiteSpaceHandling::Multiple
                }
            }
        };
        self.iteration_models = Some(self.compute_iteration_models(handling)?);
        Ok(())
    }

    fn process_iteration_model(&mut self, iteration_is_new: bool) -> EngineProcessableResult {
        if iteration_is_new {
            self.context.increase_level();
            self.context.set_variable(
                Some(self.iter_variable_name.clone()),
                self.iter_status_variable.get_current(),
            );
            let status: Arc<dyn TemplateObject> = self.iter_status_variable.clone();
            self.context.set_variable(
                Some(self.iter_status_variable_name.clone()),
                Some(Arc::new(TemplateValue::Object(status))),
            );
            self.base.prepare_processing();
            if let Some(iterator) = self.data_driven() {
                lock_data_driven(iterator).start_iteration();
            }
        }

        let flow = self.base.flow_controller();
        let mut handler = self.base.reentrant_processor_template_handler();
        let processed = {
            self.iter_model
                .as_ref()
                .expect("iteration model must be selected before processing")
                .process_throttled(handler.as_mut(), self.iter_offset, flow.as_ref())?
        };
        self.iter_offset += processed;

        let model_size = self
            .iter_model
            .as_ref()
            .expect("iteration model remains selected until completion")
            .queue
            .len();
        if self.iter_offset < model_size
            || flow.as_ref().is_some_and(|controller| {
                controller
                    .lock()
                    .expect("template flow controller lock poisoned")
                    .stop_processing
            })
        {
            return Ok(false);
        }
        self.context.decrease_level();
        if let Some(iterator) = self.data_driven() {
            lock_data_driven(iterator)
                .finish_iteration()
                .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
        }
        Ok(true)
    }

    fn data_driven(&self) -> Option<&Mutex<DataDrivenTemplateIterator<Arc<TemplateValue>>>> {
        self.data_driven_iterator.as_ref().and_then(|value| {
            value
                .as_any()
                .downcast_ref::<Mutex<DataDrivenTemplateIterator<Arc<TemplateValue>>>>()
        })
    }

    fn data_driven_has_next(&self) -> bool {
        self.data_driven()
            .is_some_and(|iterator| lock_data_driven(iterator).has_next())
    }

    fn data_driven_next(
        &self,
    ) -> Result<Option<Arc<TemplateValue>>, Box<dyn TemplateEngineException>> {
        self.data_driven()
            .expect("data-driven object type was validated")
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .next_java()
            .map(Some)
            .map_err(|error| {
                Box::new(TemplateProcessingException::with_cause(
                    Some("Could not obtain next data-driven iteration value".to_owned()),
                    error,
                )) as Box<dyn TemplateEngineException>
            })
    }

    fn pause_if_data_driven_empty(&self) -> bool {
        let Some(iterator) = self.data_driven() else {
            return false;
        };
        if !lock_data_driven(iterator).is_paused() {
            return false;
        }
        if let Some(flow) = self.base.flow_controller() {
            flow.lock()
                .expect("template flow controller lock poisoned")
                .stop_processing = true;
            return true;
        }
        false
    }

    fn compute_iteration_models(
        &self,
        handling: IterationWhiteSpaceHandling,
    ) -> Result<IterationModels, Box<dyn TemplateEngineException>> {
        if matches!(handling, IterationWhiteSpaceHandling::Zero) {
            return Ok(IterationModels {
                first: None,
                middle: None,
                last: None,
            });
        }

        let inner = self.base.inner_model();
        if matches!(handling, IterationWhiteSpaceHandling::Single) {
            let model = clone_model(inner);
            return Ok(IterationModels {
                first: Some(clone_model(&model)),
                middle: Some(clone_model(&model)),
                last: Some(model),
            });
        }

        if !self.template_mode.is_text() {
            let first = clone_model(inner);
            if let Some(whitespace) = &self.preceding_whitespace {
                let mut repeated = clone_model(inner);
                let event: Arc<dyn ITemplateEvent> = whitespace.clone();
                repeated.insert(0, Some(event)).map_err(model_error)?;
                return Ok(IterationModels {
                    first: Some(first),
                    middle: Some(clone_model(&repeated)),
                    last: Some(repeated),
                });
            }
            return Ok(IterationModels {
                first: Some(clone_model(&first)),
                middle: Some(clone_model(&first)),
                last: Some(first),
            });
        }

        if inner.queue.len() <= 2
            || inner
                .queue
                .first()
                .and_then(|event| event.as_open_element_tag())
                .is_none()
            || inner
                .queue
                .last()
                .and_then(|event| event.as_close_element_tag())
                .is_none()
        {
            return Ok(same_iteration_models(inner));
        }

        let first_body = &inner.queue[1];
        let last_body_index = inner.queue.len() - 2;
        let last_body = &inner.queue[last_body_index];
        let Some(first_text) = first_body.as_text() else {
            return Ok(same_iteration_models(inner));
        };
        let first_value = first_text
            .get_text()
            .map_err(text_error)?
            .ok_or_else(null_text_error)?;
        let Some(first_cut) = first_body_cut_point(first_value.as_utf16()) else {
            return Ok(same_iteration_models(inner));
        };
        let Some(last_text) = last_body.as_text() else {
            return Ok(same_iteration_models(inner));
        };
        let last_value = last_text
            .get_text()
            .map_err(text_error)?
            .ok_or_else(null_text_error)?;
        let Some(last_cut) = last_body_cut_point(last_value.as_utf16()) else {
            return Ok(same_iteration_models(inner));
        };

        let mut first = clone_model(inner);
        let mut middle = clone_model(inner);
        let mut last = clone_model(inner);
        if Arc::ptr_eq(first_body, last_body) {
            replace_text(
                &mut first,
                1,
                first_value.java_sub_sequence(0, last_cut as i32),
            )?;
            replace_text(
                &mut middle,
                1,
                first_value.java_sub_sequence(first_cut as i32, last_cut as i32),
            )?;
            replace_text(
                &mut last,
                1,
                first_value.java_sub_sequence(first_cut as i32, first_value.len() as i32),
            )?;
        } else {
            if first_cut > 0 {
                let text =
                    first_value.java_sub_sequence(first_cut as i32, first_value.len() as i32);
                replace_text(&mut middle, 1, text.clone())?;
                replace_text(&mut last, 1, text)?;
            }
            if last_cut < last_value.len() {
                let text = last_value.java_sub_sequence(0, last_cut as i32);
                replace_text(&mut first, last_body_index, text.clone())?;
                replace_text(&mut middle, last_body_index, text)?;
            }
        }
        Ok(IterationModels {
            first: Some(first),
            middle: Some(middle),
            last: Some(last),
        })
    }
}

impl IEngineProcessable for IteratedGatheringModelProcessable {
    fn process(&mut self) -> EngineProcessableResult {
        if self
            .base
            .flow_controller()
            .as_ref()
            .is_some_and(|controller| {
                controller
                    .lock()
                    .expect("template flow controller lock poisoned")
                    .stop_processing
            })
        {
            return Ok(false);
        }
        if self.iter_model.is_none() && self.pause_if_data_driven_empty() {
            return Ok(false);
        }
        if self.iteration_models.is_none() {
            self.initialize_iteration_models()?;
        }

        if self.iter == 0 {
            let iteration_is_new = self.iter_model.is_none();
            if iteration_is_new {
                self.iter_model = self
                    .iteration_models
                    .as_mut()
                    .expect("iteration models were initialized")
                    .first
                    .take();
            }
            if self.iter_model.is_some() {
                if !self.process_iteration_model(iteration_is_new)? {
                    return Ok(false);
                }
                self.iter = self.iter.wrapping_add(1);
                self.iter_offset = 0;
                self.iter_model = None;
                if self.pause_if_data_driven_empty() {
                    return Ok(false);
                }
            } else {
                self.base.reset_gathered_skip_flags_after_no_iterations();
            }
        }

        while self.iter_model.is_some()
            || if self.data_driven_iterator.is_some() {
                self.data_driven_has_next()
            } else {
                !self.iterator.is_empty()
            }
        {
            let iteration_is_new = self.iter_model.is_none() && self.iter_offset == 0;
            if iteration_is_new {
                self.iter_status_variable.increment_index();
                let current = if self.data_driven_iterator.is_some() {
                    self.data_driven_next()?
                } else {
                    self.iterator.pop_front()
                };
                self.iter_status_variable.set_current(current);
            }
            if self.iter_model.is_none() {
                let models = self
                    .iteration_models
                    .as_ref()
                    .expect("iteration models were initialized");
                let has_more = if self.data_driven_iterator.is_some() {
                    self.data_driven_has_next()
                } else {
                    !self.iterator.is_empty()
                };
                self.iter_model = Some(if !has_more {
                    clone_model(
                        models
                            .last
                            .as_ref()
                            .expect("non-empty iteration has a last model"),
                    )
                } else {
                    clone_model(
                        models
                            .middle
                            .as_ref()
                            .expect("multiple iteration has a middle model"),
                    )
                });
            }
            if !self.process_iteration_model(iteration_is_new)? {
                return Ok(false);
            }
            self.iter = self.iter.wrapping_add(1);
            self.iter_offset = 0;
            self.iter_model = None;
            if self.pause_if_data_driven_empty() {
                return Ok(false);
            }
        }
        self.context.decrease_level();
        Ok(true)
    }
}

impl IGatheringModelProcessable for IteratedGatheringModelProcessable {
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
        self.base.initialize_processor_execution_vars().clone_vars()
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

struct MapEntryTemplateObject {
    key: Arc<TemplateValue>,
    value: Arc<TemplateValue>,
}

impl TemplateObject for MapEntryTemplateObject {
    fn java_class_name(&self) -> &str {
        "java.util.Map$Entry"
    }

    fn to_utf16_string(&self) -> Utf16String {
        let key = self
            .key
            .to_utf16_string()
            .unwrap_or_else(|| Utf16String::from_rust_str("null"));
        let value = self
            .value
            .to_utf16_string()
            .unwrap_or_else(|| Utf16String::from_rust_str("null"));
        let mut units = key.as_utf16().to_vec();
        units.push(u16::from(b'='));
        units.extend_from_slice(value.as_utf16());
        Utf16String::from_utf16(units)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        match property_name.to_string_lossy().as_str() {
            "key" => Some(Ok(Some(self.key.clone()))),
            "value" => Some(Ok(Some(self.value.clone()))),
            _ => None,
        }
    }
}

#[expect(
    clippy::type_complexity,
    reason = "三元组逐项保留 Java IteratedGatheringModelProcessable 的迭代状态"
)]
fn compute_iterated_object(
    iterated_object: Option<Arc<TemplateValue>>,
) -> (
    VecDeque<Arc<TemplateValue>>,
    Option<i32>,
    Option<Arc<dyn TemplateObject>>,
) {
    let Some(value) = iterated_object else {
        return (VecDeque::new(), Some(0), None);
    };
    match value.as_ref() {
        TemplateValue::Null => (VecDeque::new(), Some(0), None),
        TemplateValue::List(values) => (
            values.iter().cloned().collect(),
            Some(values.len() as i32),
            None,
        ),
        TemplateValue::Map(entries) => {
            let values = entries
                .iter()
                .map(|(key, value)| {
                    let entry: Arc<dyn TemplateObject> = Arc::new(MapEntryTemplateObject {
                        key: key.clone(),
                        value: value.clone(),
                    });
                    Arc::new(TemplateValue::Object(entry))
                })
                .collect::<VecDeque<_>>();
            (values, Some(entries.len() as i32), None)
        }
        TemplateValue::Bytes(values) => (
            values
                .iter()
                .map(|value| Arc::new(TemplateValue::Number(NumberValue::Byte(*value))))
                .collect(),
            Some(values.len() as i32),
            None,
        ),
        TemplateValue::Object(object) => {
            if object
                .as_any()
                .is::<Mutex<DataDrivenTemplateIterator<Arc<TemplateValue>>>>()
            {
                return (VecDeque::new(), None, Some(object.clone()));
            }
            if let Some(values) = object.java_iterable_values() {
                (values.into(), None, None)
            } else {
                (VecDeque::from([value]), Some(1), None)
            }
        }
        _ => (VecDeque::from([value]), Some(1), None),
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

fn same_iteration_models(model: &Model) -> IterationModels {
    IterationModels {
        first: Some(clone_model(model)),
        middle: Some(clone_model(model)),
        last: Some(clone_model(model)),
    }
}

fn lock_data_driven(
    iterator: &Mutex<DataDrivenTemplateIterator<Arc<TemplateValue>>>,
) -> std::sync::MutexGuard<'_, DataDrivenTemplateIterator<Arc<TemplateValue>>> {
    iterator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn replace_text(
    model: &mut Model,
    index: usize,
    text: Result<Utf16String, crate::util::TextUtilsError>,
) -> Result<(), Box<dyn TemplateEngineException>> {
    let text = text.map_err(text_error)?;
    let event: Arc<dyn ITemplateEvent> = Arc::new(Text::new(Some(Arc::new(text))));
    model.replace(index, Some(event)).map_err(model_error)
}

fn first_body_cut_point(units: &[u16]) -> Option<usize> {
    for (index, unit) in units.iter().copied().enumerate() {
        if unit == u16::from(b'\n') {
            return Some(index + 1);
        }
        if !is_java_whitespace(unit) {
            return None;
        }
    }
    None
}

fn last_body_cut_point(units: &[u16]) -> Option<usize> {
    for index in (0..units.len()).rev() {
        if units[index] == u16::from(b'\n') {
            return Some(index + 1);
        }
        if !is_java_whitespace(units[index]) {
            return None;
        }
    }
    None
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

fn is_empty_or_whitespace(value: &Utf16String) -> bool {
    value.is_empty()
        || value
            .as_utf16()
            .iter()
            .all(|unit| is_java_whitespace(*unit))
}

fn model_error(error: crate::model::IModelError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some("Could not prepare iteration model".to_owned()),
        error,
    ))
}

fn text_error(error: crate::util::TextUtilsError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some("Could not inspect iteration whitespace".to_owned()),
        error,
    ))
}

fn null_text_error() -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::new(Some(
        "Engine text returned null content while preparing iteration whitespace".to_owned(),
    )))
}
