use thymeleaf::TemplateMode;
use thymeleaf::cdatasection::{
    AbstractCDATASectionProcessor, ICDATASectionProcessor, ICDATASectionStructureHandler,
};
use thymeleaf::context::ITemplateContext;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::ICDATASection;
use thymeleaf::processor::IProcessor;
use thymeleaf::util::Utf16String;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &dyn ICDATASection,
    &mut dyn ICDATASectionStructureHandler,
) -> ProcessResult;

/// 在 Standard 内联之后为 CDATA 内容添加 `||` 边界的交互测试 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.features.interaction.InteractionDialect01CDATASectionProcessor`。
pub struct InteractionDialect01CDATASectionProcessor {
    processor: AbstractCDATASectionProcessor<ProcessCallback>,
}

impl InteractionDialect01CDATASectionProcessor {
    /// 创建指定模式、precedence 1010 的 CDATA Processor。
    pub fn new(template_mode: TemplateMode) -> Self {
        Self {
            processor: AbstractCDATASectionProcessor::new(
                Some(template_mode),
                1010,
                "org.thymeleaf.templateengine.features.interaction.InteractionDialect01CDATASectionProcessor",
                process_cdata as ProcessCallback,
            )
            .expect("the fixed interaction CDATA processor configuration is valid"),
        }
    }
}

impl IProcessor for InteractionDialect01CDATASectionProcessor {
    fn as_cdata_section_processor(&self) -> Option<&dyn ICDATASectionProcessor> {
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

impl ICDATASectionProcessor for InteractionDialect01CDATASectionProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        cdata_section: &dyn ICDATASection,
        structure_handler: &mut dyn ICDATASectionStructureHandler,
    ) -> ProcessResult {
        self.processor
            .process(context, cdata_section, structure_handler)
    }
}

fn process_cdata(
    _context: &dyn ITemplateContext,
    cdata_section: &dyn ICDATASection,
    structure_handler: &mut dyn ICDATASectionStructureHandler,
) -> ProcessResult {
    let content = cdata_section
        .get_content()
        .map_err(|error| {
            Box::new(TemplateProcessingException::with_cause(
                Some("Could not read interaction CDATA".to_owned()),
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
