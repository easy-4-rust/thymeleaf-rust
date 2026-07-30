use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::IXMLDeclaration;
use thymeleaf::processor::IProcessor;
use thymeleaf::xmldeclaration::{AbstractXMLDeclarationProcessor, IXMLDeclarationProcessor, IXMLDeclarationStructureHandler};

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback =
    fn(&dyn ITemplateContext, &dyn IXMLDeclaration, &mut dyn IXMLDeclarationStructureHandler) -> ProcessResult;

/// 删除遇到的 XMLDeclaration 模板事件。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.remove.RemoveXMLDeclarationProcessor`。
pub struct RemoveXMLDeclarationProcessor {
    processor: AbstractXMLDeclarationProcessor<ProcessCallback>,
}

impl RemoveXMLDeclarationProcessor {
    /// 创建 HTML 模式、precedence 1000 的事件处理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            processor: AbstractXMLDeclarationProcessor::new(
                Some(TemplateMode::HTML),
                1000,
                "org.thymeleaf.templateengine.processors.dialects.remove.RemoveXMLDeclarationProcessor",
                remove_event as ProcessCallback,
            )
            .expect("the fixed remove processor configuration is valid"),
        }
    }
}

impl Default for RemoveXMLDeclarationProcessor {
    fn default() -> Self { Self::new() }
}

impl IProcessor for RemoveXMLDeclarationProcessor {
    fn as_xml_declaration_processor(&self) -> Option<&dyn IXMLDeclarationProcessor> { Some(self) }
    fn java_class_name(&self) -> &'static str { self.processor.java_class_name() }
    fn get_template_mode(&self) -> Option<TemplateMode> { self.processor.get_template_mode() }
    fn get_precedence(&self) -> i32 { self.processor.get_precedence() }
}

impl IXMLDeclarationProcessor for RemoveXMLDeclarationProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        event: &dyn IXMLDeclaration,
        structure_handler: &mut dyn IXMLDeclarationStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, event, structure_handler)
    }
}

fn remove_event(
    _context: &dyn ITemplateContext,
    _event: &dyn IXMLDeclaration,
    structure_handler: &mut dyn IXMLDeclarationStructureHandler,
) -> ProcessResult {
    structure_handler.remove_xml_declaration();
    Ok(())
}

