use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeModelProcessor, IElementModelProcessor, IElementModelStructureHandler,
    IElementProcessor, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::engine::AttributeName;
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::expression::TemplateValue;
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

/// 为被聚合的元素模型设置局部变量。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.elementprocessors.dialect.MarkupAddLocalVariableModelProcessor`。
pub struct MarkupAddLocalVariableModelProcessor {
    processor: AbstractAttributeModelProcessor<ProcessCallback>,
}

impl MarkupAddLocalVariableModelProcessor {
    /// 创建 `markup:add-local-variable` Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                None,
                false,
                Some(Utf16String::from_rust_str("add-local-variable")),
                true,
                500,
                true,
                "org.thymeleaf.templateengine.elementprocessors.dialect.MarkupAddLocalVariableModelProcessor",
                process_model as ProcessCallback,
            )
            .expect("the fixed markup local-variable processor configuration is valid"),
        }
    }
}

impl IProcessor for MarkupAddLocalVariableModelProcessor {
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
impl IElementProcessor for MarkupAddLocalVariableModelProcessor {
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
impl IElementModelProcessor for MarkupAddLocalVariableModelProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, model, structure_handler)
    }
}

fn process_model(
    _context: &dyn ITemplateContext,
    _model: &mut dyn IModel,
    _attribute_name: &AttributeName,
    _attribute_value: Option<Utf16String>,
    structure_handler: &mut dyn IElementModelStructureHandler,
) -> ProcessResult {
    structure_handler.set_local_variable(
        Utf16String::from_rust_str("local"),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "a local value",
        )))),
    );
    Ok(())
}
