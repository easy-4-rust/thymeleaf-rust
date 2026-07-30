use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::ICDATASection;
use thymeleaf::processor::IProcessor;
use thymeleaf::cdatasection::{AbstractCDATASectionProcessor, ICDATASectionProcessor, ICDATASectionStructureHandler};

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback =
    fn(&dyn ITemplateContext, &dyn ICDATASection, &mut dyn ICDATASectionStructureHandler) -> ProcessResult;

/// 删除遇到的 CDATASection 模板事件。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.remove.RemoveCDATASectionProcessor`。
pub struct RemoveCDATASectionProcessor {
    processor: AbstractCDATASectionProcessor<ProcessCallback>,
}

impl RemoveCDATASectionProcessor {
    /// 创建 HTML 模式、precedence 1000 的事件处理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            processor: AbstractCDATASectionProcessor::new(
                Some(TemplateMode::HTML),
                1000,
                "org.thymeleaf.templateengine.processors.dialects.remove.RemoveCDATASectionProcessor",
                remove_event as ProcessCallback,
            )
            .expect("the fixed remove processor configuration is valid"),
        }
    }
}

impl Default for RemoveCDATASectionProcessor {
    fn default() -> Self { Self::new() }
}

impl IProcessor for RemoveCDATASectionProcessor {
    fn as_cdata_section_processor(&self) -> Option<&dyn ICDATASectionProcessor> { Some(self) }
    fn java_class_name(&self) -> &'static str { self.processor.java_class_name() }
    fn get_template_mode(&self) -> Option<TemplateMode> { self.processor.get_template_mode() }
    fn get_precedence(&self) -> i32 { self.processor.get_precedence() }
}

impl ICDATASectionProcessor for RemoveCDATASectionProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        event: &dyn ICDATASection,
        structure_handler: &mut dyn ICDATASectionStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, event, structure_handler)
    }
}

fn remove_event(
    _context: &dyn ITemplateContext,
    _event: &dyn ICDATASection,
    structure_handler: &mut dyn ICDATASectionStructureHandler,
) -> ProcessResult {
    structure_handler.remove_cdata_section();
    Ok(())
}

