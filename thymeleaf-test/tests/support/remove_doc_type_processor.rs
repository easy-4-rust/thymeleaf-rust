use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::doctype::{AbstractDocTypeProcessor, IDocTypeProcessor, IDocTypeStructureHandler};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::IDocType;
use thymeleaf::processor::IProcessor;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback =
    fn(&dyn ITemplateContext, &dyn IDocType, &mut dyn IDocTypeStructureHandler) -> ProcessResult;

/// 删除遇到的 DocType 模板事件。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.remove.RemoveDocTypeProcessor`。
pub struct RemoveDocTypeProcessor {
    processor: AbstractDocTypeProcessor<ProcessCallback>,
}

impl RemoveDocTypeProcessor {
    /// 创建 HTML 模式、precedence 1000 的事件处理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            processor: AbstractDocTypeProcessor::new(
                Some(TemplateMode::HTML),
                1000,
                "org.thymeleaf.templateengine.processors.dialects.remove.RemoveDocTypeProcessor",
                remove_event as ProcessCallback,
            )
            .expect("the fixed remove processor configuration is valid"),
        }
    }
}

impl Default for RemoveDocTypeProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl IProcessor for RemoveDocTypeProcessor {
    fn as_doc_type_processor(&self) -> Option<&dyn IDocTypeProcessor> {
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

impl IDocTypeProcessor for RemoveDocTypeProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        event: &dyn IDocType,
        structure_handler: &mut dyn IDocTypeStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, event, structure_handler)
    }
}

fn remove_event(
    _context: &dyn ITemplateContext,
    _event: &dyn IDocType,
    structure_handler: &mut dyn IDocTypeStructureHandler,
) -> ProcessResult {
    structure_handler.remove_doc_type();
    Ok(())
}
