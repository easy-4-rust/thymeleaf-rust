use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::IXMLDeclaration;
use thymeleaf::processor::IProcessor;
use thymeleaf::util::Utf16String;
use thymeleaf::xmldeclaration::{
    AbstractXMLDeclarationProcessor, IXMLDeclarationProcessor, IXMLDeclarationStructureHandler,
};

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &dyn IXMLDeclaration,
    &mut dyn IXMLDeclarationStructureHandler,
) -> ProcessResult;

/// 用固定 `<replaced th:text="one"/>` 模型替换 XMLDeclaration 事件，替换模型继续处理。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.replacewithprocessable.ReplaceWithProcessableXMLDeclarationProcessor`。
pub struct ReplaceWithProcessableXMLDeclarationProcessor {
    processor: AbstractXMLDeclarationProcessor<ProcessCallback>,
}

impl ReplaceWithProcessableXMLDeclarationProcessor {
    /// 创建 HTML 模式、precedence 1000 的事件处理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            processor: AbstractXMLDeclarationProcessor::new(
                Some(TemplateMode::HTML),
                1000,
                "org.thymeleaf.templateengine.processors.dialects.replacewithprocessable.ReplaceWithProcessableXMLDeclarationProcessor",
                replace_event as ProcessCallback,
            )
            .expect("the fixed replacement processor configuration is valid"),
        }
    }
}

impl Default for ReplaceWithProcessableXMLDeclarationProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl IProcessor for ReplaceWithProcessableXMLDeclarationProcessor {
    fn as_xml_declaration_processor(&self) -> Option<&dyn IXMLDeclarationProcessor> {
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

impl IXMLDeclarationProcessor for ReplaceWithProcessableXMLDeclarationProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        event: &dyn IXMLDeclaration,
        structure_handler: &mut dyn IXMLDeclarationStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, event, structure_handler)
    }
}

fn replace_event(
    context: &dyn ITemplateContext,
    event: &dyn IXMLDeclaration,
    structure_handler: &mut dyn IXMLDeclarationStructureHandler,
) -> ProcessResult {
    let _ = event;
    let replacement = context
        .get_model_factory()
        .parse(
            &context.get_template_data(),
            &Utf16String::from_rust_str("<replaced th:text=\"one\"/>"),
        )
        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
    structure_handler.replace_with(Arc::from(replacement), true);
    Ok(())
}
