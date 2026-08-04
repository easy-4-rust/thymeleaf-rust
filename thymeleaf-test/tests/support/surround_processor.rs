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

/// 在聚合元素模型前后插入 `surround` 注释。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.surround.SurroundProcessor`。
pub struct SurroundProcessor {
    processor: AbstractAttributeModelProcessor<ProcessCallback>,
}

impl SurroundProcessor {
    /// 创建匹配 `surround:surround`、precedence 1000 的模型处理器。
    #[must_use]
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                None,
                false,
                Some(Utf16String::from_rust_str("surround")),
                true,
                1000,
                true,
                "org.thymeleaf.templateengine.processors.dialects.surround.SurroundProcessor",
                surround_model as ProcessCallback,
            )
            .expect("the fixed surround processor configuration is valid"),
        }
    }
}

impl IProcessor for SurroundProcessor {
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

impl IElementProcessor for SurroundProcessor {
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

impl IElementModelProcessor for SurroundProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, model, structure_handler)
    }
}

fn surround_model(
    context: &dyn ITemplateContext,
    model: &mut dyn IModel,
    _attribute_name: &AttributeName,
    _attribute_value: Option<Utf16String>,
    _structure_handler: &mut dyn IElementModelStructureHandler,
) -> ProcessResult {
    let model_factory = context.get_model_factory();
    let after: Arc<dyn ITemplateEvent> = model_factory
        .create_comment(Utf16String::from_rust_str("surround"))
        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
    model.add(Some(after)).map_err(model_error)?;
    let before: Arc<dyn ITemplateEvent> = model_factory
        .create_comment(Utf16String::from_rust_str("surround"))
        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
    model.insert(0, Some(before)).map_err(model_error)
}

fn model_error(error: thymeleaf::model::IModelError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some(error.to_string()),
        error,
    ))
}
