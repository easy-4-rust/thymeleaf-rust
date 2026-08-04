use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeModelProcessor, IElementModelProcessor, IElementModelStructureHandler,
    IElementProcessor, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::engine::AttributeName;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::{IModel, ITemplateEvent};
use thymeleaf::processor::IProcessor;
use thymeleaf::util::Utf16String;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &mut dyn IModel,
    &AttributeName,
    Option<Utf16String>,
    &mut dyn IElementModelStructureHandler,
) -> ProcessResult;

/// 在较晚模型阶段把局部变量复制为标签属性 `var`。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.context.dialect.AttrModelAttributeTagProcessor`。
pub struct AttrModelAttributeTagProcessor {
    processor: AbstractAttributeModelProcessor<ProcessCallback>,
}

impl AttrModelAttributeTagProcessor {
    /// 创建 precedence 150 的 `ctxvar:attrmodel` Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                None,
                false,
                Some(Utf16String::from_rust_str("attrmodel")),
                true,
                150,
                true,
                "org.thymeleaf.templateengine.context.dialect.AttrModelAttributeTagProcessor",
                set_attribute as ProcessCallback,
            )
            .expect("the fixed context attr model configuration is valid"),
        }
    }
}

impl IProcessor for AttrModelAttributeTagProcessor {
    fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
        Some(self)
    }
    fn class_name(&self) -> &'static str {
        self.processor.class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.processor.get_template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.processor.get_precedence()
    }
}
impl IElementProcessor for AttrModelAttributeTagProcessor {
    fn as_element_model_processor(&self) -> Option<&dyn IElementModelProcessor> {
        Some(self)
    }
    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.processor.get_matching_element_name()
    }
    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        self.processor.get_matching_attribute_name()
    }
}
impl IElementModelProcessor for AttrModelAttributeTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, model, structure_handler)
    }
}

fn set_attribute(
    context: &dyn ITemplateContext,
    model: &mut dyn IModel,
    _attribute_name: &AttributeName,
    _attribute_value: Option<Utf16String>,
    _structure_handler: &mut dyn IElementModelStructureHandler,
) -> ProcessResult {
    let first = model
        .get(0)
        .into_processable_element_tag()
        .ok_or_else(|| processing_error("Model first event is not an element tag"))?;
    let value = context
        .get_variable(Some(&Utf16String::from_rust_str("var")))
        .as_deref()
        .and_then(|value| value.to_utf16_string());
    let first = context
        .get_model_factory()
        .set_attribute(first, Utf16String::from_rust_str("var"), value, None)
        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
    let event: Arc<dyn ITemplateEvent> = first;
    model.replace(0, Some(event)).map_err(model_error)
}

fn model_error(error: thymeleaf::model::IModelError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some(error.to_string()),
        error,
    ))
}
fn processing_error(message: &str) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::new(Some(message.to_owned())))
}
