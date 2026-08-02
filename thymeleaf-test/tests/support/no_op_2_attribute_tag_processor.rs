use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::engine::AttributeName;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::expression::TemplateValue;
use thymeleaf::model::IProcessableElementTag;
use thymeleaf::processor::IProcessor;
use thymeleaf::util::JavaString;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &dyn IProcessableElementTag,
    &AttributeName,
    Option<JavaString>,
    &mut dyn IElementTagStructureHandler,
) -> ProcessResult;

/// 验证标签级局部变量已跨越 NoOp Processor 传播的第二阶段 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.noop.NoOp2AttributeTagProcessor`。
pub struct NoOp2AttributeTagProcessor {
    processor: AbstractAttributeTagProcessor<ProcessCallback>,
}

impl NoOp2AttributeTagProcessor {
    /// 创建 precedence 1100 的 `noop:noop` 属性 Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(JavaString::from_rust_str),
                None,
                false,
                Some(JavaString::from_rust_str("noop")),
                true,
                1100,
                false,
                "org.thymeleaf.templateengine.processors.dialects.noop.NoOp2AttributeTagProcessor",
                verify_tag_variable as ProcessCallback,
            )
            .expect("the fixed second no-op attribute processor configuration is valid"),
        }
    }
}

impl IProcessor for NoOp2AttributeTagProcessor {
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

impl IElementProcessor for NoOp2AttributeTagProcessor {
    fn as_element_tag_processor(&self) -> Option<&dyn IElementTagProcessor> {
        Some(self)
    }
    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.processor.get_matching_element_name()
    }
    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        self.processor.get_matching_attribute_name()
    }
}

impl IElementTagProcessor for NoOp2AttributeTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, tag, structure_handler)
    }
}

fn verify_tag_variable(
    context: &dyn ITemplateContext,
    _tag: &dyn IProcessableElementTag,
    _attribute_name: &AttributeName,
    _attribute_value: Option<JavaString>,
    _structure_handler: &mut dyn IElementTagStructureHandler,
) -> ProcessResult {
    if !matches!(
        context
            .get_variable(Some(&JavaString::from_rust_str("noop-tag")))
            .as_deref(),
        Some(TemplateValue::Boolean(true))
    ) {
        return Err(Box::new(TemplateProcessingException::new(Some(
            "Local variable has not reached from one no-op attr operator to the next one"
                .to_owned(),
        ))));
    }
    Ok(())
}
