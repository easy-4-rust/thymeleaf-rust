use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::engine::AttributeName;
use thymeleaf::exceptions::TemplateEngineException;
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

/// 在处理链末端把局部变量 `var` 写入元素正文。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.context.dialect.WriteVarAttributeTagProcessor`。
pub struct WriteVarAttributeTagProcessor {
    processor: AbstractAttributeTagProcessor<ProcessCallback>,
}

impl WriteVarAttributeTagProcessor {
    /// 创建 precedence 100000000 且执行后移除属性的 Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                None,
                false,
                Some(Utf16String::from_rust_str("writevar")),
                true,
                100_000_000,
                true,
                "org.thymeleaf.templateengine.context.dialect.WriteVarAttributeTagProcessor",
                write_variable as ProcessCallback,
            )
            .expect("the fixed context variable writer configuration is valid"),
        }
    }
}

impl IProcessor for WriteVarAttributeTagProcessor {
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
impl IElementProcessor for WriteVarAttributeTagProcessor {
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
impl IElementTagProcessor for WriteVarAttributeTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, tag, structure_handler)
    }
}

fn write_variable(
    context: &dyn ITemplateContext,
    _tag: &dyn IProcessableElementTag,
    _attribute_name: &AttributeName,
    _attribute_value: Option<Utf16String>,
    structure_handler: &mut dyn IElementTagStructureHandler,
) -> ProcessResult {
    let value = context
        .get_variable(Some(&Utf16String::from_rust_str("var")))
        .as_deref()
        .and_then(|value| value.to_utf16_string())
        .unwrap_or_else(|| Utf16String::from_rust_str(""));
    structure_handler.set_body_text(value, false);
    Ok(())
}
