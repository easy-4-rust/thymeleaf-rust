use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractElementModelProcessor, IElementModelProcessor, IElementModelStructureHandler,
    IElementProcessor, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::expression::TemplateValue;
use thymeleaf::model::IModel;
use thymeleaf::processor::IProcessor;
use thymeleaf::util::JavaString;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &mut dyn IModel,
    &mut dyn IElementModelStructureHandler,
) -> ProcessResult;

/// 设置模型级局部变量、但不修改模型的 NoOp Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.noop.NoOpModelProcessor`。
pub struct NoOpModelProcessor {
    processor: AbstractElementModelProcessor<ProcessCallback>,
}

impl NoOpModelProcessor {
    /// 创建 precedence 1000 的 `noop:noop` 模型 Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractElementModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(JavaString::from_rust_str),
                Some(JavaString::from_rust_str("noop")),
                true,
                None,
                false,
                1000,
                "org.thymeleaf.templateengine.processors.dialects.noop.NoOpModelProcessor",
                process_model as ProcessCallback,
            )
            .expect("the fixed no-op model processor configuration is valid"),
        }
    }
}

impl IProcessor for NoOpModelProcessor {
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

impl IElementProcessor for NoOpModelProcessor {
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

impl IElementModelProcessor for NoOpModelProcessor {
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
    structure_handler: &mut dyn IElementModelStructureHandler,
) -> ProcessResult {
    structure_handler.set_local_variable(
        JavaString::from_rust_str("noop-model"),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    Ok(())
}
