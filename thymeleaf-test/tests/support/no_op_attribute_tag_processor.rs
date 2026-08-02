use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::engine::AttributeName;
use thymeleaf::exceptions::TemplateEngineException;
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

/// 设置标签级局部变量、但不修改标签的 NoOp Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.noop.NoOpAttributeTagProcessor`。
pub struct NoOpAttributeTagProcessor {
    processor: AbstractAttributeTagProcessor<ProcessCallback>,
}

impl NoOpAttributeTagProcessor {
    /// 创建 precedence 1000 的 `noop:noop` 属性 Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(JavaString::from_rust_str),
                None,
                false,
                Some(JavaString::from_rust_str("noop")),
                true,
                1000,
                false,
                "org.thymeleaf.templateengine.processors.dialects.noop.NoOpAttributeTagProcessor",
                process_tag as ProcessCallback,
            )
            .expect("the fixed no-op attribute processor configuration is valid"),
        }
    }
}

impl IProcessor for NoOpAttributeTagProcessor {
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

impl IElementProcessor for NoOpAttributeTagProcessor {
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

impl IElementTagProcessor for NoOpAttributeTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, tag, structure_handler)
    }
}

fn process_tag(
    _context: &dyn ITemplateContext,
    _tag: &dyn IProcessableElementTag,
    _attribute_name: &AttributeName,
    _attribute_value: Option<JavaString>,
    structure_handler: &mut dyn IElementTagStructureHandler,
) -> ProcessResult {
    structure_handler.set_local_variable(
        JavaString::from_rust_str("noop-tag"),
        Some(Arc::new(TemplateValue::Boolean(true))),
    );
    Ok(())
}
