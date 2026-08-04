use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractElementTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::engine::Text;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::{IProcessableElementTag, ITemplateEvent};
use thymeleaf::processor::IProcessor;
use thymeleaf::util::Utf16String;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &dyn IProcessableElementTag,
    &mut dyn IElementTagStructureHandler,
) -> ProcessResult;

/// 在 HTML `div` 正文开始处插入 Dialect02 标记的 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.aggregation.dialect.Dialect02DivProcessor`。
pub struct Dialect02DivProcessor {
    processor: AbstractElementTagProcessor<ProcessCallback>,
}

impl Dialect02DivProcessor {
    /// 使用实际生效的方言前缀创建 precedence 100 的 Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractElementTagProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                Some(Utf16String::from_rust_str("div")),
                true,
                None,
                false,
                100,
                "org.thymeleaf.templateengine.aggregation.dialect.Dialect02DivProcessor",
                process_div as ProcessCallback,
            )
            .expect("the fixed Dialect02 div processor configuration is valid"),
        }
    }
}

impl IProcessor for Dialect02DivProcessor {
    fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
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

impl IElementProcessor for Dialect02DivProcessor {
    fn as_element_tag_processor(&self) -> Option<&dyn IElementTagProcessor> {
        Some(self)
    }
    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.processor.get_matching_element_name()
    }
    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        self.processor.get_matching_attribute_name()
    }
}

impl IElementTagProcessor for Dialect02DivProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, tag, structure_handler)
    }
}

fn process_div(
    context: &dyn ITemplateContext,
    _tag: &dyn IProcessableElementTag,
    structure_handler: &mut dyn IElementTagStructureHandler,
) -> ProcessResult {
    let mut markup = context.get_model_factory().create_model();
    let text: Arc<dyn ITemplateEvent> = Arc::new(Text::new(Some(Arc::new(
        Utf16String::from_rust_str("[From Dialect 02]"),
    ))));
    markup.add(Some(text)).map_err(|error| {
        Box::new(TemplateProcessingException::with_cause(
            Some(error.to_string()),
            error,
        )) as Box<dyn TemplateEngineException>
    })?;
    structure_handler.insert_immediately_after(Arc::from(markup), true);
    Ok(())
}
