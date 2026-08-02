use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::expression::TemplateValue;
use thymeleaf::model::IText;
use thymeleaf::processor::IProcessor;
use thymeleaf::text::{AbstractTextProcessor, ITextProcessor, ITextStructureHandler};
use thymeleaf::util::JavaString;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback =
    fn(&dyn ITemplateContext, &dyn IText, &mut dyn ITextStructureHandler) -> ProcessResult;

/// 验证局部变量可到达元素正文，并把正文改为 `processed!`。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.noop.NoOpTextProcessor`。
pub struct NoOpTextProcessor {
    processor: AbstractTextProcessor<ProcessCallback>,
}

impl NoOpTextProcessor {
    /// 创建 precedence 1100 的 HTML Text Processor。
    pub fn new() -> Self {
        Self {
            processor: AbstractTextProcessor::new(
                Some(TemplateMode::HTML),
                1100,
                "org.thymeleaf.templateengine.processors.dialects.noop.NoOpTextProcessor",
                process_text as ProcessCallback,
            )
            .expect("the fixed no-op text processor configuration is valid"),
        }
    }
}

impl Default for NoOpTextProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl IProcessor for NoOpTextProcessor {
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

impl ITextProcessor for NoOpTextProcessor {
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
    context: &dyn ITemplateContext,
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
        .unwrap_or_else(|| JavaString::from_rust_str(""));
    if text == JavaString::from_rust_str("...") {
        return Ok(());
    }
    let valid = ["noop-tag", "noop-model"].iter().any(|name| {
        matches!(
            context
                .get_variable(Some(&JavaString::from_rust_str(name)))
                .as_deref(),
            Some(TemplateValue::Boolean(true))
        )
    });
    if !valid {
        return Err(Box::new(TemplateProcessingException::new(Some(
            "Local variable has not reached from one no-op operator to the body text".to_owned(),
        ))));
    }
    structure_handler.set_text(JavaString::from_rust_str("processed!"));
    Ok(())
}
