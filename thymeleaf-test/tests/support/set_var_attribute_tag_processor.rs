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

/// 把 `ctxvar:setvar` 属性值放入当前标签的局部变量 `var`。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.context.dialect.SetVarAttributeTagProcessor`。
pub struct SetVarAttributeTagProcessor {
    processor: AbstractAttributeTagProcessor<ProcessCallback>,
}

impl SetVarAttributeTagProcessor {
    /// 创建 precedence 1 且执行后移除属性的 Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(JavaString::from_rust_str),
                None,
                false,
                Some(JavaString::from_rust_str("setvar")),
                true,
                1,
                true,
                "org.thymeleaf.templateengine.context.dialect.SetVarAttributeTagProcessor",
                set_variable as ProcessCallback,
            )
            .expect("the fixed context variable setter configuration is valid"),
        }
    }
}

impl IProcessor for SetVarAttributeTagProcessor {
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
impl IElementProcessor for SetVarAttributeTagProcessor {
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
impl IElementTagProcessor for SetVarAttributeTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, tag, structure_handler)
    }
}

fn set_variable(
    _context: &dyn ITemplateContext,
    _tag: &dyn IProcessableElementTag,
    _attribute_name: &AttributeName,
    attribute_value: Option<JavaString>,
    structure_handler: &mut dyn IElementTagStructureHandler,
) -> ProcessResult {
    structure_handler.set_local_variable(
        JavaString::from_rust_str("var"),
        attribute_value.map(TemplateValue::string).map(Arc::new),
    );
    Ok(())
}
