use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeModelProcessor, IElementModelProcessor, IElementModelStructureHandler,
    IElementProcessor, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::engine::AttributeName;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::{AttributeValueQuotes, IModel, ITemplateEvent};
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

/// 把目标模型的元素名改为 `ctx`，同时保留完整属性映射与边界形态。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.context.dialect.ModelAttributeTagProcessor`。
pub struct ModelAttributeTagProcessor {
    processor: AbstractAttributeModelProcessor<ProcessCallback>,
}

impl ModelAttributeTagProcessor {
    /// 创建 precedence 250 的 `ctxvar:model` Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                None,
                false,
                Some(Utf16String::from_rust_str("model")),
                true,
                250,
                true,
                "org.thymeleaf.templateengine.context.dialect.ModelAttributeTagProcessor",
                rename_model as ProcessCallback,
            )
            .expect("the fixed context model renaming configuration is valid"),
        }
    }
}

impl IProcessor for ModelAttributeTagProcessor {
    fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
        Some(self)
    }
    fn java_class_name(&self) -> &'static str {
        self.processor.java_class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.processor.get_template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.processor.get_precedence()
    }
}
impl IElementProcessor for ModelAttributeTagProcessor {
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
impl IElementModelProcessor for ModelAttributeTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, model, structure_handler)
    }
}

fn rename_model(
    context: &dyn ITemplateContext,
    model: &mut dyn IModel,
    _attribute_name: &AttributeName,
    _attribute_value: Option<Utf16String>,
    _structure_handler: &mut dyn IElementModelStructureHandler,
) -> ProcessResult {
    let first_event = model.get(0);
    let last_event = model.get(model.size() - 1);
    let first_tag = Arc::clone(&first_event)
        .into_processable_element_tag()
        .ok_or_else(|| processing_error("Model first event is not an element tag"))?;
    let attributes = first_tag.get_attribute_map();
    let factory = context.get_model_factory();

    if Arc::ptr_eq(&first_event, &last_event) {
        let standalone = factory
            .create_standalone_element_tag(
                Utf16String::from_rust_str("ctx"),
                Some(&attributes),
                AttributeValueQuotes::DOUBLE,
                false,
                false,
            )
            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
        let event: Arc<dyn ITemplateEvent> = standalone;
        model.replace(0, Some(event)).map_err(model_error)?;
    } else {
        let open = factory
            .create_open_element_tag(
                Utf16String::from_rust_str("ctx"),
                Some(&attributes),
                AttributeValueQuotes::DOUBLE,
                false,
            )
            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
        let close = factory
            .create_close_element_tag(Utf16String::from_rust_str("ctx"), false, false)
            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
        let open_event: Arc<dyn ITemplateEvent> = open;
        let close_event: Arc<dyn ITemplateEvent> = close;
        model.replace(0, Some(open_event)).map_err(model_error)?;
        model
            .replace(model.size() - 1, Some(close_event))
            .map_err(model_error)?;
    }
    Ok(())
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
