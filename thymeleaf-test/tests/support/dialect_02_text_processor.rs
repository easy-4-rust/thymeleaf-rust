use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::IText;
use thymeleaf::processor::IProcessor;
use thymeleaf::text::{AbstractTextProcessor, ITextProcessor, ITextStructureHandler};
use thymeleaf::util::Utf16String;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback =
    fn(&dyn ITemplateContext, &dyn IText, &mut dyn ITextStructureHandler) -> ProcessResult;

/// 为 HTML 文本追加 `[02]` 的聚合测试 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.aggregation.dialect.Dialect02TextProcessor`。
pub struct Dialect02TextProcessor {
    processor: AbstractTextProcessor<ProcessCallback>,
}

impl Dialect02TextProcessor {
    /// 创建 precedence 1000 的文本 Processor。
    pub fn new() -> Self {
        Self {
            processor: AbstractTextProcessor::new(
                Some(TemplateMode::HTML),
                1000,
                "org.thymeleaf.templateengine.aggregation.dialect.Dialect02TextProcessor",
                process_text as ProcessCallback,
            )
            .expect("the fixed Dialect02 text processor configuration is valid"),
        }
    }
}

impl Default for Dialect02TextProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl IProcessor for Dialect02TextProcessor {
    fn as_text_processor(&self) -> Option<&dyn ITextProcessor> {
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

impl ITextProcessor for Dialect02TextProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        text: &dyn IText,
        structure_handler: &mut dyn ITextStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, text, structure_handler)
    }
}

fn process_text(
    _context: &dyn ITemplateContext,
    text: &dyn IText,
    structure_handler: &mut dyn ITextStructureHandler,
) -> ProcessResult {
    let text = text
        .get_text()
        .map_err(|error| {
            Box::new(TemplateProcessingException::with_cause(
                Some(error.to_string()),
                error,
            )) as Box<dyn TemplateEngineException>
        })?
        .unwrap_or_else(|| Utf16String::from_rust_str(""));
    structure_handler.set_text(Utf16String::from_rust_str(&format!(
        "{}[02]",
        text.to_string_lossy()
    )));
    Ok(())
}
