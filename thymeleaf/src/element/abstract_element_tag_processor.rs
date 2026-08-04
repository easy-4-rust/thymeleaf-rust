use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::engine::{AttributeNames, ElementNames};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::IProcessableElementTag;
use crate::processor::{AbstractProcessorAdapter, IProcessor};
use crate::util::{Utf16String, ValidateError};

use super::{
    IElementProcessor, IElementTagProcessor, IElementTagStructureHandler, MatchingAttributeName,
    MatchingElementName,
};

/// 按元素名和/或属性名匹配并为标签异常补充位置的抽象 Processor。
///
/// 对应 Java: `org.thymeleaf.processor.element.AbstractElementTagProcessor`。
pub struct AbstractElementTagProcessor<F> {
    adapter: AbstractProcessorAdapter<F>,
    dialect_prefix: Option<Utf16String>,
    matching_element_name: Option<MatchingElementName>,
    matching_attribute_name: Option<MatchingAttributeName>,
}

impl<F> AbstractElementTagProcessor<F> {
    /// 创建以闭包表达 Java 抽象 `doProcess` 方法的标签 Processor。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`AbstractElementTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: Option<TemplateMode>,
        dialect_prefix: Option<Utf16String>,
        element_name: Option<Utf16String>,
        prefix_element_name: bool,
        attribute_name: Option<Utf16String>,
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
    /// 对应 Java: `AbstractElementTagProcessor#getDialectPrefix()`。
    pub fn get_dialect_prefix(&self) -> Option<&Utf16String> {
        self.dialect_prefix.as_ref()
    }
}

impl<F> IProcessor for AbstractElementTagProcessor<F>
where
    F: Send + Sync,
{
    fn class_name(&self) -> &'static str {
        self.adapter.processor_class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.adapter.template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.adapter.precedence()
    }
}

impl<F> IElementProcessor for AbstractElementTagProcessor<F>
where
    F: Fn(
            &dyn ITemplateContext,
            &dyn IProcessableElementTag,
            &mut dyn IElementTagStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
{
    fn as_element_tag_processor(&self) -> Option<&dyn IElementTagProcessor> {
        Some(self)
    }

    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.matching_element_name.as_ref()
    }
    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        self.matching_attribute_name.as_ref()
    }
}

impl<F> IElementTagProcessor for AbstractElementTagProcessor<F>
where
    F: Fn(
            &dyn ITemplateContext,
            &dyn IProcessableElementTag,
            &mut dyn IElementTagStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
{
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.adapter
            .execute(tag, |callback| callback(context, tag, structure_handler))
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
