use std::sync::Arc;

use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::engine::{AttributeName, AttributeNameValue, AttributeNames, ElementNames};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::{IAttribute, IModel, IProcessableElementTag, ITemplateEvent};
use crate::processor::{AbstractProcessor, IProcessor};
use crate::util::{EscapedAttributeUtils, JavaString};

use super::{
    IElementModelProcessor, IElementModelStructureHandler, IElementProcessor,
    MatchingAttributeName, MatchingElementName,
};

/// 按指定属性匹配完整元素模型并可在执行后删除该属性的抽象 Processor。
///
/// 对应 Java: `org.thymeleaf.processor.element.AbstractAttributeModelProcessor`。
pub struct AbstractAttributeModelProcessor<F> {
    processor: AbstractProcessor,
    processor_class_name: &'static str,
    dialect_prefix: Option<JavaString>,
    matching_element_name: Option<MatchingElementName>,
    matching_attribute_name: MatchingAttributeName,
    attribute_name: AttributeNameValue,
    remove_attribute: bool,
    do_process: F,
}

impl<F> AbstractAttributeModelProcessor<F> {
    /// 创建以闭包实现 Java 抽象 `doProcess` 的属性模型 Processor。
    ///
    /// # 错误
    /// 模式或匹配名称配置非法时返回处理异常。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`AbstractAttributeModelProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: Option<TemplateMode>,
        dialect_prefix: Option<JavaString>,
        element_name: Option<JavaString>,
        prefix_element_name: bool,
        attribute_name: Option<JavaString>,
        prefix_attribute_name: bool,
        precedence: i32,
        remove_attribute: bool,
        processor_class_name: &'static str,
        do_process: F,
    ) -> Result<Self, TemplateProcessingException> {
        let processor =
            AbstractProcessor::new(template_mode, precedence).map_err(configuration_error)?;
        let mode = processor.get_template_mode();
        let matching_element_name = build_matching_element(
            mode,
            dialect_prefix.as_ref(),
            element_name.as_ref(),
            prefix_element_name,
        )?;
        let normalized = if prefix_attribute_name {
            AttributeNames::for_name_with_prefix(
                Some(mode),
                dialect_prefix.as_ref(),
                attribute_name.as_ref(),
            )
        } else {
            AttributeNames::for_name(Some(mode), attribute_name.as_ref())
        }
        .map_err(configuration_error)?;
        let matching_attribute_name =
            MatchingAttributeName::for_attribute_name(Some(mode), Some(normalized.clone()))
                .map_err(configuration_error)?;
        Ok(Self {
            processor,
            processor_class_name,
            dialect_prefix,
            matching_element_name,
            matching_attribute_name,
            attribute_name: normalized,
            remove_attribute,
            do_process,
        })
    }

    /// 返回构造时保存的可空方言前缀。
    /// 对应 Java 语义：Java 接口/超类方法 `getDialectPrefix()` 的 Rust 移植（`AbstractAttributeModelProcessor` 继承路径）。
    pub fn get_dialect_prefix(&self) -> Option<&JavaString> {
        self.dialect_prefix.as_ref()
    }
}

impl<F> IProcessor for AbstractAttributeModelProcessor<F>
where
    F: Send + Sync,
{
    fn java_class_name(&self) -> &'static str {
        self.processor_class_name
    }

    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(self.processor.get_template_mode())
    }

    fn get_precedence(&self) -> i32 {
        self.processor.get_precedence()
    }
}

impl<F> IElementProcessor for AbstractAttributeModelProcessor<F>
where
    F: Fn(
            &dyn ITemplateContext,
            &mut dyn IModel,
            &AttributeName,
            Option<JavaString>,
            &mut dyn IElementModelStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
{
    fn as_element_model_processor(&self) -> Option<&dyn IElementModelProcessor> {
        Some(self)
    }

    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.matching_element_name.as_ref()
    }

    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        Some(&self.matching_attribute_name)
    }
}

impl<F> IElementModelProcessor for AbstractAttributeModelProcessor<F>
where
    F: Fn(
            &dyn ITemplateContext,
            &mut dyn IModel,
            &AttributeName,
            Option<JavaString>,
            &mut dyn IElementModelStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
{
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let first_event = (model.size() > 0).then(|| model.get(0));
        let first_tag = first_event
            .as_ref()
            .and_then(|event| event.clone().into_processable_element_tag());
        let attribute_name = self.attribute_name.as_attribute_name();
        let mut operation = || -> Result<(), Box<dyn TemplateEngineException>> {
            let first_tag = first_tag.as_ref().ok_or_else(|| {
                Box::new(TemplateProcessingException::new(Some(
                    "Model first event is not an IProcessableElementTag".to_owned(),
                ))) as Box<dyn TemplateEngineException>
            })?;
            let attribute_value = EscapedAttributeUtils::unescape_attribute(
                Some(context.get_template_mode()),
                first_tag.get_attribute_value_by_name(attribute_name),
            )
            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
            (self.do_process)(
                context,
                model,
                attribute_name,
                attribute_value,
                structure_handler,
            )?;
            if self.remove_attribute {
                let location = locate_first_event_in_model(model, first_event.as_ref());
                if let Some(location) = location {
                    let current_event = model.get(location);
                    let current_tag = current_event
                        .clone()
                        .into_processable_element_tag()
                        .ok_or_else(|| {
                            Box::new(TemplateProcessingException::new(Some(
                                "Located model event is not an IProcessableElementTag".to_owned(),
                            ))) as Box<dyn TemplateEngineException>
                        })?;
                    let new_first_event = context
                        .get_model_factory()
                        .remove_attribute(current_tag.clone(), attribute_name)
                        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
                    if !Arc::ptr_eq(&current_tag, &new_first_event) {
                        let replacement: Arc<dyn ITemplateEvent> = new_first_event;
                        model
                            .replace(location, Some(replacement))
                            .map_err(|error| {
                                Box::new(TemplateProcessingException::with_cause(
                                    Some(error.to_string()),
                                    error,
                                ))
                                    as Box<dyn TemplateEngineException>
                            })?;
                    }
                }
            }
            Ok(())
        };

        match operation() {
            Ok(()) => Ok(()),
            Err(mut error) => {
                if let Some(processing) = error.as_processing_exception_mut() {
                    enrich_model_attribute_location(
                        processing,
                        first_tag.as_deref(),
                        attribute_name,
                    );
                    return Err(error);
                }
                let (template_name, line, col) =
                    model_attribute_location(first_tag.as_deref(), attribute_name);
                Err(Box::new(
                    TemplateProcessingException::with_location_and_cause(
                        Some(format!(
                            "Error during execution of processor '{}'",
                            self.processor_class_name
                        )),
                        template_name,
                        line,
                        col,
                        ProcessorCause(error),
                    ),
                ))
            }
        }
    }
}

fn locate_first_event_in_model(
    model: &dyn IModel,
    first_event: Option<&Arc<dyn ITemplateEvent>>,
) -> Option<usize> {
    if let Some(first_event) = first_event {
        for index in 0..model.size() {
            if Arc::ptr_eq(first_event, &model.get(index)) {
                return Some(index);
            }
        }
    }
    if model.size() > 0 && model.get(0).into_processable_element_tag().is_some() {
        return Some(0);
    }
    None
}

fn build_matching_element(
    mode: TemplateMode,
    dialect_prefix: Option<&JavaString>,
    element_name: Option<&JavaString>,
    prefix_element_name: bool,
) -> Result<Option<MatchingElementName>, TemplateProcessingException> {
    element_name
        .map(|name| {
            let normalized = if prefix_element_name {
                ElementNames::for_name_with_prefix(Some(mode), dialect_prefix, Some(name))
            } else {
                ElementNames::for_name(Some(mode), Some(name))
            }
            .map_err(configuration_error)?;
            MatchingElementName::for_element_name(Some(mode), Some(normalized))
                .map_err(configuration_error)
        })
        .transpose()
}

fn enrich_model_attribute_location(
    error: &mut TemplateProcessingException,
    first_tag: Option<&dyn IProcessableElementTag>,
    attribute_name: &AttributeName,
) {
    let (template_name, line, col) = model_attribute_location(first_tag, attribute_name);
    if !error.has_template_name() {
        error.set_template_name(template_name);
    }
    if !error.has_line_and_col() && line != -1 && col != -1 {
        error.set_line_and_col(line, col);
    }
}

fn model_attribute_location(
    first_tag: Option<&dyn IProcessableElementTag>,
    attribute_name: &AttributeName,
) -> (Option<String>, i32, i32) {
    let Some(first_tag) = first_tag else {
        return (None, -1, -1);
    };
    let attribute = first_tag.get_attribute_by_name(attribute_name);
    (
        first_tag
            .get_template_name()
            .map(JavaString::to_string_lossy),
        attribute.map_or(-1, IAttribute::get_line),
        attribute.map_or(-1, IAttribute::get_col),
    )
}

fn configuration_error(
    error: impl std::error::Error + Send + Sync + 'static,
) -> TemplateProcessingException {
    TemplateProcessingException::with_cause(Some(error.to_string()), error)
}

struct ProcessorCause(Box<dyn TemplateEngineException>);

impl std::fmt::Display for ProcessorCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}

impl std::fmt::Debug for ProcessorCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}

impl std::error::Error for ProcessorCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}
