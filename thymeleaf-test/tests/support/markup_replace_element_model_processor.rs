use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeModelProcessor, IElementModelProcessor, IElementModelStructureHandler,
    IElementProcessor, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::engine::AttributeName;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::IModel;
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

/// 用固定可处理 markup 替换整个聚合元素模型。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.elementprocessors.dialect.MarkupReplaceElementModelProcessor`。
pub struct MarkupReplaceElementModelProcessor {
    processor: AbstractAttributeModelProcessor<ProcessCallback>,
}

impl MarkupReplaceElementModelProcessor {
    /// 创建 `markup:replace` Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                None,
                false,
                Some(Utf16String::from_rust_str("replace")),
                true,
                1000,
                true,
                "org.thymeleaf.templateengine.elementprocessors.dialect.MarkupReplaceElementModelProcessor",
                replace_element as ProcessCallback,
            )
            .expect("the fixed markup replacement processor configuration is valid"),
        }
    }
}

impl IProcessor for MarkupReplaceElementModelProcessor {
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
impl IElementProcessor for MarkupReplaceElementModelProcessor {
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
impl IElementModelProcessor for MarkupReplaceElementModelProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, model, structure_handler)
    }
}

fn replace_element(
    context: &dyn ITemplateContext,
    model: &mut dyn IModel,
    _attribute_name: &AttributeName,
    _attribute_value: Option<Utf16String>,
    _structure_handler: &mut dyn IElementModelStructureHandler,
) -> ProcessResult {
    let replacement = context
        .get_model_factory()
        .parse(
            &context.get_template_data(),
            &Utf16String::from_rust_str(
                "<p>This is a <span th:text=\"replacement\">prototype</span></p>",
            ),
        )
        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
    model.reset().map_err(model_error)?;
    model
        .add_model(Some(replacement.as_ref()))
        .map_err(model_error)
}

fn model_error(error: thymeleaf::model::IModelError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some(error.to_string()),
        error,
    ))
}
