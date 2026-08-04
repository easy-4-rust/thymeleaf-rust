use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::engine::EngineEventUtils;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::IText;
use thymeleaf::processor::IProcessor;
use thymeleaf::text::{AbstractTextProcessor, ITextProcessor, ITextStructureHandler};

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback =
    fn(&dyn ITemplateContext, &dyn IText, &mut dyn ITextStructureHandler) -> ProcessResult;

/// 删除非空白且不等于 `...` 的文本事件。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.remove.RemoveTextProcessor`。
pub struct RemoveTextProcessor {
    processor: AbstractTextProcessor<ProcessCallback>,
}

impl RemoveTextProcessor {
    /// 创建 HTML 模式、precedence 1000 的文本处理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            processor: AbstractTextProcessor::new(
                Some(TemplateMode::HTML),
                1000,
                "org.thymeleaf.templateengine.processors.dialects.remove.RemoveTextProcessor",
                remove_text as ProcessCallback,
            )
            .expect("the fixed remove text processor configuration is valid"),
        }
    }
}

impl Default for RemoveTextProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl IProcessor for RemoveTextProcessor {
    fn as_text_processor(&self) -> Option<&dyn ITextProcessor> {
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

impl ITextProcessor for RemoveTextProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        text: &dyn IText,
        structure_handler: &mut dyn ITextStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, text, structure_handler)
    }
}

fn remove_text(
    _context: &dyn ITemplateContext,
    text: &dyn IText,
    structure_handler: &mut dyn ITextStructureHandler,
) -> ProcessResult {
    let whitespace = EngineEventUtils::is_whitespace_text(Some(text)).map_err(|error| {
        Box::new(TemplateProcessingException::with_cause(
            Some(error.to_string()),
            error,
        )) as Box<dyn TemplateEngineException>
    })?;
    let preserved = text
        .get_text()
        .map_err(|error| {
            Box::new(TemplateProcessingException::with_cause(
                Some(error.to_string()),
                error,
            )) as Box<dyn TemplateEngineException>
        })?
        .is_some_and(|value| value.to_string_lossy() == "...");
    if !whitespace && !preserved {
        structure_handler.remove_text();
    }
    Ok(())
}
