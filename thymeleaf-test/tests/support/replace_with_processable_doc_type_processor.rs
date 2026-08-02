use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::doctype::{AbstractDocTypeProcessor, IDocTypeProcessor, IDocTypeStructureHandler};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::IDocType;
use thymeleaf::processor::IProcessor;
use thymeleaf::util::JavaString;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback =
    fn(&dyn ITemplateContext, &dyn IDocType, &mut dyn IDocTypeStructureHandler) -> ProcessResult;

/// 用固定 `<replaced th:text="one"/>` 模型替换 DocType 事件，替换模型继续处理。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.replacewithprocessable.ReplaceWithProcessableDocTypeProcessor`。
pub struct ReplaceWithProcessableDocTypeProcessor {
    processor: AbstractDocTypeProcessor<ProcessCallback>,
}

impl ReplaceWithProcessableDocTypeProcessor {
    /// 创建 HTML 模式、precedence 1000 的事件处理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            processor: AbstractDocTypeProcessor::new(
                Some(TemplateMode::HTML),
                1000,
                "org.thymeleaf.templateengine.processors.dialects.replacewithprocessable.ReplaceWithProcessableDocTypeProcessor",
                replace_event as ProcessCallback,
            )
            .expect("the fixed replacement processor configuration is valid"),
        }
    }
}

impl Default for ReplaceWithProcessableDocTypeProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl IProcessor for ReplaceWithProcessableDocTypeProcessor {
    fn as_doc_type_processor(&self) -> Option<&dyn IDocTypeProcessor> {
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

impl IDocTypeProcessor for ReplaceWithProcessableDocTypeProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        event: &dyn IDocType,
        structure_handler: &mut dyn IDocTypeStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, event, structure_handler)
    }
}

fn replace_event(
    context: &dyn ITemplateContext,
    event: &dyn IDocType,
    structure_handler: &mut dyn IDocTypeStructureHandler,
) -> ProcessResult {
    let _ = event;
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
