use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::IText;
use thymeleaf::processor::IProcessor;
use thymeleaf::text::{AbstractTextProcessor, ITextProcessor, ITextStructureHandler};
use thymeleaf::util::JavaString;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback =
    fn(&dyn ITemplateContext, &dyn IText, &mut dyn ITextStructureHandler) -> ProcessResult;

/// 在 Standard 内联之后为文本内容添加 `||` 边界的交互测试 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.features.interaction.InteractionDialect01TextProcessor`。
pub struct InteractionDialect01TextProcessor {
    processor: AbstractTextProcessor<ProcessCallback>,
}

impl InteractionDialect01TextProcessor {
    /// 创建指定模式、precedence 1010 的文本 Processor。
    pub fn new(template_mode: TemplateMode) -> Self {
        Self {
            processor: AbstractTextProcessor::new(
                Some(template_mode),
                1010,
                "org.thymeleaf.templateengine.features.interaction.InteractionDialect01TextProcessor",
                process_text as ProcessCallback,
            )
            .expect("the fixed interaction text processor configuration is valid"),
        }
    }
}

impl IProcessor for InteractionDialect01TextProcessor {
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

impl ITextProcessor for InteractionDialect01TextProcessor {
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
    let content = text
        .get_text()
        .map_err(|error| {
            Box::new(TemplateProcessingException::with_cause(
                Some("Could not read interaction text".to_owned()),
                error,
            )) as Box<dyn TemplateEngineException>
        })?
        .unwrap_or_else(|| JavaString::from_rust_str(""));
    structure_handler.set_text(JavaString::from_rust_str(&format!(
        "||{}||",
        content.to_string_lossy()
    )));
    Ok(())
}
