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

/// 保留元素边界、用固定可处理 markup 替换正文。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.elementprocessors.dialect.MarkupReplaceBodyElementModelProcessor`。
pub struct MarkupReplaceBodyElementModelProcessor {
    processor: AbstractAttributeModelProcessor<ProcessCallback>,
}

impl MarkupReplaceBodyElementModelProcessor {
    /// 创建 `markup:replacebody` Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                None,
                false,
                Some(Utf16String::from_rust_str("replacebody")),
                true,
                1000,
                true,
                "org.thymeleaf.templateengine.elementprocessors.dialect.MarkupReplaceBodyElementModelProcessor",
                replace_body as ProcessCallback,
            )
            .expect("the fixed markup body replacement processor configuration is valid"),
        }
    }
}

impl IProcessor for MarkupReplaceBodyElementModelProcessor {
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
impl IElementProcessor for MarkupReplaceBodyElementModelProcessor {
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
impl IElementModelProcessor for MarkupReplaceBodyElementModelProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, model, structure_handler)
    }
}

fn replace_body(
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

    // Java 从倒数第二个事件逆序删除到索引 1，保留开始/结束或 standalone 边界。
    for index in (1..model.size().saturating_sub(1)).rev() {
        model.remove(index).map_err(model_error)?;
    }
    model
        .insert_model(1, Some(replacement.as_ref()))
        .map_err(model_error)
}

fn model_error(error: thymeleaf::model::IModelError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some(error.to_string()),
        error,
    ))
}
