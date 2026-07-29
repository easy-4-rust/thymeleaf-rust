use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::engine::{AttributeNames, ElementNames};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::IModel;
use crate::processor::{AbstractProcessorAdapter, IProcessor};
use crate::util::{JavaString, ValidateError};

use super::{
    IElementModelProcessor, IElementModelStructureHandler, IElementProcessor,
    MatchingAttributeName, MatchingElementName,
};

/// 按元素名和/或属性名匹配并以完整模型为单位执行的抽象 Processor。
///
/// 对应 Java: `org.thymeleaf.processor.element.AbstractElementModelProcessor`。
pub struct AbstractElementModelProcessor<F> {
    adapter: AbstractProcessorAdapter<F>,
    dialect_prefix: Option<JavaString>,
    matching_element_name: Option<MatchingElementName>,
    matching_attribute_name: Option<MatchingAttributeName>,
}

impl<F> AbstractElementModelProcessor<F> {
    /// 创建以闭包表达 Java 抽象 `doProcess` 方法的模型 Processor。
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        template_mode: Option<TemplateMode>,
        dialect_prefix: Option<JavaString>,
        element_name: Option<JavaString>,
        prefix_element_name: bool,
        attribute_name: Option<JavaString>,
        prefix_attribute_name: bool,
        precedence: i32,
        processor_class_name: &'static str,
        do_process: F,
    ) -> Result<Self, TemplateProcessingException> {
        let mode = template_mode.ok_or_else(|| {
            TemplateProcessingException::new(Some("Template mode cannot be null".to_owned()))
        })?;
        let matching_element_name = element_name
            .as_ref()
            .map(|name| {
                let parsed = if prefix_element_name {
                    ElementNames::for_name_with_prefix(
                        Some(mode),
                        dialect_prefix.as_ref(),
                        Some(name),
                    )
                } else {
                    ElementNames::for_name(Some(mode), Some(name))
                }
                .map_err(configuration_error)?;
                MatchingElementName::for_element_name(Some(mode), Some(parsed))
                    .map_err(configuration_error)
            })
            .transpose()?;
        let matching_attribute_name = attribute_name
            .as_ref()
            .map(|name| {
                let parsed = if prefix_attribute_name {
                    AttributeNames::for_name_with_prefix(
                        Some(mode),
                        dialect_prefix.as_ref(),
                        Some(name),
                    )
                } else {
                    AttributeNames::for_name(Some(mode), Some(name))
                }
                .map_err(configuration_error)?;
                MatchingAttributeName::for_attribute_name(Some(mode), Some(parsed))
                    .map_err(configuration_error)
            })
            .transpose()?;
        let adapter =
            AbstractProcessorAdapter::new(Some(mode), precedence, processor_class_name, do_process)
                .map_err(validate_error)?;
        Ok(Self {
            adapter,
            dialect_prefix,
            matching_element_name,
            matching_attribute_name,
        })
    }

    /// 返回构造时保存的可空方言前缀。
    pub fn get_dialect_prefix(&self) -> Option<&JavaString> {
        self.dialect_prefix.as_ref()
    }
}

impl<F> IProcessor for AbstractElementModelProcessor<F> {
    fn java_class_name(&self) -> &'static str {
        self.adapter.processor_class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.adapter.template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.adapter.precedence()
    }
}

impl<F> IElementProcessor for AbstractElementModelProcessor<F> {
    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.matching_element_name.as_ref()
    }
    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        self.matching_attribute_name.as_ref()
    }
}

impl<F> IElementModelProcessor for AbstractElementModelProcessor<F>
where
    F: Fn(
        &dyn ITemplateContext,
        &mut dyn IModel,
        &mut dyn IElementModelStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>,
{
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let first_event = (model.size() > 0).then(|| model.get(0));
        self.adapter
            .execute_optional(first_event.as_deref(), |callback| {
                callback(context, model, structure_handler)
            })
    }
}

fn configuration_error(
    error: impl std::error::Error + Send + Sync + 'static,
) -> TemplateProcessingException {
    TemplateProcessingException::with_cause(
        Some("Invalid element processor matching configuration".to_owned()),
        error,
    )
}

fn validate_error(error: ValidateError) -> TemplateProcessingException {
    TemplateProcessingException::with_cause(Some(error.to_string()), error)
}
