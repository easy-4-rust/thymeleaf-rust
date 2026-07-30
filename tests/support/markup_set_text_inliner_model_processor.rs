use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeModelProcessor, IElementModelProcessor, IElementModelStructureHandler,
    IElementProcessor, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::engine::AttributeName;
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::inline::StandardTextInliner;
use thymeleaf::model::IModel;
use thymeleaf::processor::IProcessor;
use thymeleaf::util::JavaString;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &mut dyn IModel,
    &AttributeName,
    Option<JavaString>,
    &mut dyn IElementModelStructureHandler,
) -> ProcessResult;

/// 为当前模型作用域设置 Standard TEXT inliner。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.elementprocessors.dialect.MarkupSetTextInlinerModelProcessor`。
pub struct MarkupSetTextInlinerModelProcessor {
    processor: AbstractAttributeModelProcessor<ProcessCallback>,
}

impl MarkupSetTextInlinerModelProcessor {
    /// 创建 `markup:set-text-inliner` Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(JavaString::from_rust_str),
                None,
                false,
                Some(JavaString::from_rust_str("set-text-inliner")),
                true,
                600,
                true,
                "org.thymeleaf.templateengine.elementprocessors.dialect.MarkupSetTextInlinerModelProcessor",
                set_inliner as ProcessCallback,
            )
            .expect("the fixed markup inliner processor configuration is valid"),
        }
    }
}

impl IProcessor for MarkupSetTextInlinerModelProcessor {
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
impl IElementProcessor for MarkupSetTextInlinerModelProcessor {
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
impl IElementModelProcessor for MarkupSetTextInlinerModelProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, model, structure_handler)
    }
}

fn set_inliner(
    context: &dyn ITemplateContext,
    _model: &mut dyn IModel,
    _attribute_name: &AttributeName,
    _attribute_value: Option<JavaString>,
    structure_handler: &mut dyn IElementModelStructureHandler,
) -> ProcessResult {
    structure_handler.set_inliner(Some(Arc::new(StandardTextInliner::new(
        context.get_configuration(),
    ))));
    Ok(())
}
