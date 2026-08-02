use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::engine::{AttributeName, AttributeNameValue, AttributeNames, ElementNames};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::IProcessableElementTag;
use crate::processor::{AbstractProcessor, IProcessor};
use crate::util::{EscapedAttributeUtils, JavaString};

use super::{
    IElementProcessor, IElementTagProcessor, IElementTagStructureHandler, MatchingAttributeName,
    MatchingElementName,
};

/// 按指定属性匹配标签、反转义属性值并可在执行后删除该属性的抽象 Processor。
///
/// 对应 Java: `org.thymeleaf.processor.element.AbstractAttributeTagProcessor`。
pub struct AbstractAttributeTagProcessor<F> {
    processor: AbstractProcessor,
    processor_class_name: &'static str,
    dialect_prefix: Option<JavaString>,
    matching_element_name: Option<MatchingElementName>,
    matching_attribute_name: MatchingAttributeName,
    attribute_name: AttributeNameValue,
    remove_attribute: bool,
    do_process: F,
}

impl<F> AbstractAttributeTagProcessor<F> {
    /// 创建以闭包实现 Java 抽象 `doProcess` 的属性标签 Processor。
    ///
    /// # 错误
    /// 模式、属性名或匹配名称配置非法时返回处理异常。
    #[allow(clippy::too_many_arguments)]
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
        if attribute_name.as_ref().is_none_or(JavaString::is_empty) {
            return Err(TemplateProcessingException::new(Some(
                "Attribute name cannot be null or empty in Attribute Tag Processor".to_owned(),
            )));
        }
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
    pub fn get_dialect_prefix(&self) -> Option<&JavaString> {
        self.dialect_prefix.as_ref()
    }
}

impl<F> IProcessor for AbstractAttributeTagProcessor<F>
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

impl<F> IElementProcessor for AbstractAttributeTagProcessor<F>
where
    F: Fn(
            &dyn ITemplateContext,
            &dyn IProcessableElementTag,
            &AttributeName,
            Option<JavaString>,
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
        Some(&self.matching_attribute_name)
    }
}

impl<F> IElementTagProcessor for AbstractAttributeTagProcessor<F>
where
    F: Fn(
            &dyn ITemplateContext,
            &dyn IProcessableElementTag,
            &AttributeName,
            Option<JavaString>,
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
        let attribute_name = self.attribute_name.as_attribute_name();
        let mut operation = || -> Result<(), Box<dyn TemplateEngineException>> {
            let attribute_value = EscapedAttributeUtils::unescape_attribute(
                Some(context.get_template_mode()),
                tag.get_attribute_value_by_name(attribute_name),
            )
            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
            (self.do_process)(
                context,
                tag,
                attribute_name,
                attribute_value,
                structure_handler,
            )?;
            if self.remove_attribute {
                structure_handler.remove_attribute_name(self.attribute_name.clone());
            }
            Ok(())
        };
        match operation() {
            Ok(()) => Ok(()),
            Err(mut error) => {
                if let Some(processing) = error.as_processing_exception_mut() {
                    enrich_tag_attribute_location(processing, tag, Some(attribute_name));
                    return Err(error);
                }
                let (line, col) = attribute_location(tag, Some(attribute_name));
                Err(Box::new(
                    TemplateProcessingException::with_location_and_cause(
                        Some(format!(
                            "Error during execution of processor '{}'",
                            self.processor_class_name
                        )),
                        tag.get_template_name().map(JavaString::to_string_lossy),
                        line,
                        col,
                        ProcessorCause(error),
                    ),
                ))
            }
        }
    }
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

fn enrich_tag_attribute_location(
    error: &mut TemplateProcessingException,
    tag: &dyn IProcessableElementTag,
    attribute_name: Option<&AttributeName>,
) {
    if !tag.has_location() {
        return;
    }
    if !error.has_template_name() {
        error.set_template_name(tag.get_template_name().map(JavaString::to_string_lossy));
    }
    if !error.has_line_and_col() {
        let (line, col) = attribute_location(tag, attribute_name);
        error.set_line_and_col(line, col);
    }
}

fn attribute_location(
    tag: &dyn IProcessableElementTag,
    attribute_name: Option<&AttributeName>,
) -> (i32, i32) {
    attribute_name
        .and_then(|name| tag.get_attribute_by_name(name))
        .map_or_else(
            || (tag.get_line(), tag.get_col()),
            |attribute| (attribute.get_line(), attribute.get_col()),
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
