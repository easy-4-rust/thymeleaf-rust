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
use thymeleaf::util::Utf16String;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &dyn IProcessableElementTag,
    &AttributeName,
    Option<Utf16String>,
    &mut dyn IElementTagStructureHandler,
) -> ProcessResult;

/// 用标签结构处理器修改局部变量的 precedence 测试 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.elementprocessors.dialect.PrecedenceModifyLocalVariableTagProcessor`。
pub struct PrecedenceModifyLocalVariableTagProcessor {
    processor: AbstractAttributeTagProcessor<ProcessCallback>,
}

impl PrecedenceModifyLocalVariableTagProcessor {
    /// 创建与 StandardTextTagProcessor 相同 precedence 的标签 Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                None,
                false,
                Some(Utf16String::from_rust_str("modify-local-variable-tag")),
                true,
                1300,
                true,
                "org.thymeleaf.templateengine.elementprocessors.dialect.PrecedenceModifyLocalVariableTagProcessor",
                process_tag as ProcessCallback,
            )
            .expect("the fixed precedence tag processor configuration is valid"),
        }
    }
}

impl IProcessor for PrecedenceModifyLocalVariableTagProcessor {
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
impl IElementProcessor for PrecedenceModifyLocalVariableTagProcessor {
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
impl IElementTagProcessor for PrecedenceModifyLocalVariableTagProcessor {
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
    _attribute_value: Option<Utf16String>,
    structure_handler: &mut dyn IElementTagStructureHandler,
) -> ProcessResult {
    structure_handler.set_local_variable(
        Utf16String::from_rust_str("local"),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "modified!",
        )))),
    );
    Ok(())
}
