use thymeleaf::TemplateMode;
use thymeleaf::comment::{AbstractCommentProcessor, ICommentProcessor, ICommentStructureHandler};
use thymeleaf::context::ITemplateContext;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::IComment;
use thymeleaf::processor::IProcessor;
use thymeleaf::util::Utf16String;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback =
    fn(&dyn ITemplateContext, &dyn IComment, &mut dyn ICommentStructureHandler) -> ProcessResult;

/// 在 Standard 内联之后为注释内容添加 `||` 边界的交互测试 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.features.interaction.InteractionDialect01CommentProcessor`。
pub struct InteractionDialect01CommentProcessor {
    processor: AbstractCommentProcessor<ProcessCallback>,
}

impl InteractionDialect01CommentProcessor {
    /// 创建指定模式、precedence 1010 的注释 Processor。
    pub fn new(template_mode: TemplateMode) -> Self {
        Self {
            processor: AbstractCommentProcessor::new(
                Some(template_mode),
                1010,
                "org.thymeleaf.templateengine.features.interaction.InteractionDialect01CommentProcessor",
                process_comment as ProcessCallback,
            )
            .expect("the fixed interaction comment processor configuration is valid"),
        }
    }
}

impl IProcessor for InteractionDialect01CommentProcessor {
    fn as_comment_processor(&self) -> Option<&dyn ICommentProcessor> {
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

impl ICommentProcessor for InteractionDialect01CommentProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        comment: &dyn IComment,
        structure_handler: &mut dyn ICommentStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, comment, structure_handler)
    }
}

fn process_comment(
    _context: &dyn ITemplateContext,
    comment: &dyn IComment,
    structure_handler: &mut dyn ICommentStructureHandler,
) -> ProcessResult {
    let content = comment
        .get_content()
        .map_err(|error| {
            Box::new(TemplateProcessingException::with_cause(
                Some("Could not read interaction comment".to_owned()),
                error,
            )) as Box<dyn TemplateEngineException>
        })?
        .unwrap_or_else(|| Utf16String::from_rust_str(""));
    structure_handler.set_content(Utf16String::from_rust_str(&format!(
        "||{}||",
        content.to_string_lossy()
    )));
    Ok(())
}
