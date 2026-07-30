use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractElementModelProcessor, IElementModelProcessor, IElementModelStructureHandler,
    IElementProcessor, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
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

/// 验证模型级局部变量已跨越 NoOp Processor 传播的第二阶段 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.noop.NoOp2ModelProcessor`。
pub struct NoOp2ModelProcessor {
    processor: AbstractElementModelProcessor<ProcessCallback>,
}

impl NoOp2ModelProcessor {
    /// 创建 precedence 1100 的 `noop:noop` 模型 Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractElementModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(JavaString::from_rust_str),
                Some(JavaString::from_rust_str("noop")),
                true,
                None,
                false,
                1100,
                "org.thymeleaf.templateengine.processors.dialects.noop.NoOp2ModelProcessor",
                verify_model_variable as ProcessCallback,
            )
            .expect("the fixed second no-op model processor configuration is valid"),
        }
    }
}

impl IProcessor for NoOp2ModelProcessor {
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

impl IElementProcessor for NoOp2ModelProcessor {
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

impl IElementModelProcessor for NoOp2ModelProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, model, structure_handler)
    }
}

fn verify_model_variable(
    context: &dyn ITemplateContext,
    _model: &mut dyn IModel,
    _structure_handler: &mut dyn IElementModelStructureHandler,
) -> ProcessResult {
    if !matches!(
        context
            .get_variable(Some(&JavaString::from_rust_str("noop-model")))
            .as_deref(),
        Some(TemplateValue::Boolean(true))
    ) {
        return Err(Box::new(TemplateProcessingException::new(Some(
            "Local variable has not reached from one no-op model operator to the next one"
                .to_owned(),
        ))));
    }
    Ok(())
}
