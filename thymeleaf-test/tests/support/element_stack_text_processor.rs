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

/// 把当前元素栈附加到 HTML 文本事件的测试 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.features.elementstack.ElementStackTextProcessor`。
pub struct ElementStackTextProcessor {
    processor: AbstractTextProcessor<ProcessCallback>,
}

impl ElementStackTextProcessor {
    /// 创建 precedence 为 10000 的 HTML 文本 Processor。
    ///
    /// `dialect_prefix` 与 Java 构造器参数一致；文本 Processor 不按属性前缀匹配，
    /// 因而只保留调用合同而不读取该值。
    pub fn new(_dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractTextProcessor::new(
                Some(TemplateMode::HTML),
                10000,
                "org.thymeleaf.templateengine.features.elementstack.ElementStackTextProcessor",
                process_text as ProcessCallback,
            )
            .expect("the fixed test processor configuration is valid"),
        }
    }
}

impl IProcessor for ElementStackTextProcessor {
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

impl ITextProcessor for ElementStackTextProcessor {
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
        .unwrap_or_else(|| Utf16String::from_rust_str(""));
    structure_handler.set_text(Utf16String::from_rust_str(&format!(
        "{} {}",
        text.to_string_lossy(),
        element_stack_text(context)
    )));
    Ok(())
}

pub(super) fn element_stack_text(context: &dyn ITemplateContext) -> String {
    context
        .get_element_stack()
        .iter()
        .map(|tag| {
            let mut entry = tag.get_element_complete_name().to_string_lossy();
            for attribute in tag.get_all_attributes() {
                entry.push(' ');
                entry.push_str(&attribute.get_attribute_complete_name().to_string_lossy());
            }
            entry
        })
        .collect::<Vec<_>>()
        .join(", ")
}
