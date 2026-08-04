use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractElementTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::{AttributeValueQuotes, IProcessableElementTag};
use thymeleaf::processor::IProcessor;
use thymeleaf::util::Utf16String;

use super::element_stack_text_processor::element_stack_text;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &dyn IProcessableElementTag,
    &mut dyn IElementTagStructureHandler,
) -> ProcessResult;

/// 把当前元素栈写入 `stack` 属性的测试 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.features.elementstack.ElementStackAttrProcessor`。
pub struct ElementStackAttrProcessor {
    processor: AbstractElementTagProcessor<ProcessCallback>,
}

impl ElementStackAttrProcessor {
    /// 使用实际方言前缀创建匹配所有 HTML 标签的 Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractElementTagProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                None,
                false,
                None,
                false,
                10000,
                "org.thymeleaf.templateengine.features.elementstack.ElementStackAttrProcessor",
                process_tag as ProcessCallback,
            )
            .expect("the fixed test processor configuration is valid"),
        }
    }
}

impl IProcessor for ElementStackAttrProcessor {
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

impl IElementProcessor for ElementStackAttrProcessor {
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

impl IElementTagProcessor for ElementStackAttrProcessor {
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
    context: &dyn ITemplateContext,
    _tag: &dyn IProcessableElementTag,
    structure_handler: &mut dyn IElementTagStructureHandler,
) -> ProcessResult {
    let escaped = html_escape::encode_safe(&element_stack_text(context)).into_owned();
    structure_handler.set_attribute(
        Utf16String::from_rust_str("stack"),
        Some(Utf16String::from_rust_str(&escaped)),
        Some(AttributeValueQuotes::DOUBLE),
    );
    Ok(())
}
