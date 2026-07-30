use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::engine::EngineEventUtils;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::IText;
use thymeleaf::processor::IProcessor;
use thymeleaf::text::{AbstractTextProcessor, ITextProcessor, ITextStructureHandler};
use thymeleaf::util::JavaString;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback =
    fn(&dyn ITemplateContext, &dyn IText, &mut dyn ITextStructureHandler) -> ProcessResult;

/// 用固定 `<replaced th:text="one"/>` 模型替换 Text 事件，替换模型继续处理。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.replacewithprocessable.ReplaceWithProcessableTextProcessor`。
pub struct ReplaceWithProcessableTextProcessor {
    processor: AbstractTextProcessor<ProcessCallback>,
}

impl ReplaceWithProcessableTextProcessor {
    /// 创建 HTML 模式、precedence 1000 的事件处理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            processor: AbstractTextProcessor::new(
                Some(TemplateMode::HTML),
                1000,
                "org.thymeleaf.templateengine.processors.dialects.replacewithprocessable.ReplaceWithProcessableTextProcessor",
                replace_event as ProcessCallback,
            )
            .expect("the fixed replacement processor configuration is valid"),
        }
    }
}

impl Default for ReplaceWithProcessableTextProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl IProcessor for ReplaceWithProcessableTextProcessor {
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

impl ITextProcessor for ReplaceWithProcessableTextProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        event: &dyn IText,
        structure_handler: &mut dyn ITextStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, event, structure_handler)
    }
}

fn replace_event(
    context: &dyn ITemplateContext,
    event: &dyn IText,
    structure_handler: &mut dyn ITextStructureHandler,
) -> ProcessResult {
    let _ = event;
    let whitespace = EngineEventUtils::is_whitespace_text(Some(event)).map_err(|error| {
        Box::new(TemplateProcessingException::with_cause(
            Some(error.to_string()),
            error,
        )) as Box<dyn TemplateEngineException>
    })?;
    let preserved = event
        .get_text()
        .map_err(|error| {
            Box::new(TemplateProcessingException::with_cause(
                Some(error.to_string()),
                error,
            )) as Box<dyn TemplateEngineException>
        })?
        .is_some_and(|value| value.to_string_lossy() == "one");
    if whitespace || preserved {
        return Ok(());
    }
    let replacement = context
        .get_model_factory()
        .parse(
            &context.get_template_data(),
            &JavaString::from_rust_str("<replaced th:text=\"one\"/>"),
        )
        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
    structure_handler.replace_with(Arc::from(replacement), true);
    Ok(())
}
