use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeModelProcessor, IElementModelProcessor, IElementModelStructureHandler,
    IElementProcessor, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::engine::AttributeName;
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::{IModel, ITemplateEvent};
use thymeleaf::processor::IProcessor;
use thymeleaf::util::{EscapedAttributeUtils, FastStringWriter, JavaString};

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &mut dyn IModel,
    &AttributeName,
    Option<JavaString>,
    &mut dyn IElementModelStructureHandler,
) -> ProcessResult;

/// 把执行前完整模型的 HTML 转义快照写入 `aggbefore` 属性。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.elementprocessors.dialect.MarkupPrintBeforeElementModelProcessor`。
pub struct MarkupPrintBeforeElementModelProcessor {
    processor: AbstractAttributeModelProcessor<ProcessCallback>,
}

impl MarkupPrintBeforeElementModelProcessor {
    /// 创建 `markup:printbefore` Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(JavaString::from_rust_str),
                None,
                false,
                Some(JavaString::from_rust_str("printbefore")),
                true,
                500,
                true,
                "org.thymeleaf.templateengine.elementprocessors.dialect.MarkupPrintBeforeElementModelProcessor",
                print_model as ProcessCallback,
            )
            .expect("the fixed print-before processor configuration is valid"),
        }
    }
}

impl IProcessor for MarkupPrintBeforeElementModelProcessor {
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
impl IElementProcessor for MarkupPrintBeforeElementModelProcessor {
    fn as_element_model_processor(&self) -> Option<&dyn IElementModelProcessor> {
        Some(self)
    }
    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.processor.get_matching_element_name()
    }
    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        self.processor.get_matching_attribute_name()
    }
}
impl IElementModelProcessor for MarkupPrintBeforeElementModelProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, model, structure_handler)
    }
}

fn print_model(
    context: &dyn ITemplateContext,
    model: &mut dyn IModel,
    _attribute_name: &AttributeName,
    _attribute_value: Option<JavaString>,
    _structure_handler: &mut dyn IElementModelStructureHandler,
) -> ProcessResult {
    let markup = escaped_model(model)?;
    let tag = model
        .get(0)
        .into_processable_element_tag()
        .ok_or_else(|| processing_error("Model first event is not an element tag"))?;
    let tag = context
        .get_model_factory()
        .set_attribute(
            tag,
            JavaString::from_rust_str("aggbefore"),
            Some(markup),
            None,
        )
        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
    let event: Arc<dyn ITemplateEvent> = tag;
    model.replace(0, Some(event)).map_err(model_error)
}

fn escaped_model(model: &dyn IModel) -> Result<JavaString, Box<dyn TemplateEngineException>> {
    let mut writer = FastStringWriter::new();
    model.write(&mut writer).map_err(|error| {
        Box::new(TemplateProcessingException::with_cause(
            Some(error.to_string()),
            error,
        )) as Box<dyn TemplateEngineException>
    })?;
    let normalized = writer
        .to_string()
        .to_string_lossy()
        .replace("\r\n", "\\n")
        .replace('\r', "\\n")
        .replace('\n', "\\n");
    EscapedAttributeUtils::escape_attribute(
        Some(TemplateMode::HTML),
        Some(&JavaString::from_rust_str(&normalized)),
    )
    .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?
    .ok_or_else(|| processing_error("Escaped model unexpectedly became null"))
}

fn model_error(error: thymeleaf::model::IModelError) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some(error.to_string()),
        error,
    ))
}

fn processing_error(message: &str) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::new(Some(message.to_owned())))
}
