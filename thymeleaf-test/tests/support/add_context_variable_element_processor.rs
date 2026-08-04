use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractElementTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::expression::TemplateValue;
use thymeleaf::model::IProcessableElementTag;
use thymeleaf::processor::IProcessor;
use thymeleaf::util::Utf16String;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &dyn IProcessableElementTag,
    &mut dyn IElementTagStructureHandler,
) -> ProcessResult;

/// 向引擎、exchange、应用与会话四层分别写入验证变量。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.context.dialect.AddContextVariableElementProcessor`。
pub struct AddContextVariableElementProcessor {
    processor: AbstractElementTagProcessor<ProcessCallback>,
}

impl AddContextVariableElementProcessor {
    /// 创建匹配 `context:add-context-variable`、precedence 100 的处理器。
    #[must_use]
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractElementTagProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                Some(Utf16String::from_rust_str("add-context-variable")),
                true,
                None,
                false,
                100,
                "org.thymeleaf.templateengine.context.dialect.AddContextVariableElementProcessor",
                add_context_variables as ProcessCallback,
            )
            .expect("the fixed context processor configuration is valid"),
        }
    }
}

impl IProcessor for AddContextVariableElementProcessor {
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

impl IElementProcessor for AddContextVariableElementProcessor {
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

impl IElementTagProcessor for AddContextVariableElementProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, tag, structure_handler)
    }
}

fn add_context_variables(
    context: &dyn ITemplateContext,
    _tag: &dyn IProcessableElementTag,
    structure_handler: &mut dyn IElementTagStructureHandler,
) -> ProcessResult {
    let engine_context = context.as_engine_context().ok_or_else(|| {
        Box::new(TemplateProcessingException::new(Some(
            "Context processor requires an IEngineContext".to_owned(),
        ))) as Box<dyn TemplateEngineException>
    })?;
    let web_exchange = context.get_web_exchange().ok_or_else(|| {
        Box::new(TemplateProcessingException::new(Some(
            "Context processor requires an IWebContext".to_owned(),
        ))) as Box<dyn TemplateEngineException>
    })?;

    engine_context.set_variable(
        Some(Utf16String::from_rust_str("newvar0")),
        string_value("LocalVariablesNewVar0"),
    );
    engine_context.set_variable(
        Some(Utf16String::from_rust_str("newvar1")),
        string_value("LocalVariablesNewVar1"),
    );
    web_exchange.set_attribute_value(
        Some(Utf16String::from_rust_str("newvar2")),
        string_value("RequestAttributesNewVar2"),
    );
    web_exchange.set_attribute_value(
        Some(Utf16String::from_rust_str("newvar3")),
        string_value("RequestAttributesNewVar3"),
    );
    web_exchange.get_application().set_attribute_value(
        Some(Utf16String::from_rust_str("newvar4")),
        string_value("ApplicationAttributesNewVar4"),
    );
    web_exchange.get_application().set_attribute_value(
        Some(Utf16String::from_rust_str("newvar5")),
        string_value("ApplicationAttributesNewVar5"),
    );
    let session = web_exchange.get_session().ok_or_else(|| {
        Box::new(TemplateProcessingException::new(Some(
            "Context processor requires an IWebSession".to_owned(),
        ))) as Box<dyn TemplateEngineException>
    })?;
    session.set_attribute_value(
        Some(Utf16String::from_rust_str("newvar6")),
        string_value("SessionAttributesNewVar6"),
    );
    session.set_attribute_value(
        Some(Utf16String::from_rust_str("newvar7")),
        string_value("SessionAttributesNewVar7"),
    );
    structure_handler.set_local_variable(Utf16String::from_rust_str("one"), string_value("one"));
    structure_handler.remove_element();
    Ok(())
}

fn string_value(value: &str) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
        value,
    ))))
}
