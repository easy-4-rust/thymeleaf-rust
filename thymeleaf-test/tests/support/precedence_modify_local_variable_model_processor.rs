use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeModelProcessor, IElementModelProcessor, IElementModelStructureHandler,
    IElementProcessor, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::expression::TemplateValue;
use thymeleaf::model::IModel;
use thymeleaf::processor::IProcessor;
use thymeleaf::util::Utf16String;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &mut dyn IModel,
    &thymeleaf::engine::AttributeName,
    Option<Utf16String>,
    &mut dyn IElementModelStructureHandler,
) -> ProcessResult;

/// 用模型结构处理器修改局部变量的 precedence 测试 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.elementprocessors.dialect.PrecedenceModifyLocalVariableModelProcessor`。
pub struct PrecedenceModifyLocalVariableModelProcessor {
    processor: AbstractAttributeModelProcessor<ProcessCallback>,
}

impl PrecedenceModifyLocalVariableModelProcessor {
    /// 创建与 StandardTextTagProcessor 相同 precedence 的模型 Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(Utf16String::from_rust_str),
                None,
                false,
                Some(Utf16String::from_rust_str("modify-local-variable-model")),
                true,
                1300,
                true,
                "org.thymeleaf.templateengine.elementprocessors.dialect.PrecedenceModifyLocalVariableModelProcessor",
                process_model as ProcessCallback,
            )
            .expect("the fixed precedence processor configuration is valid"),
        }
    }
}

impl IProcessor for PrecedenceModifyLocalVariableModelProcessor {
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

impl IElementProcessor for PrecedenceModifyLocalVariableModelProcessor {
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

impl IElementModelProcessor for PrecedenceModifyLocalVariableModelProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, model, structure_handler)
    }
}

fn process_model(
    _context: &dyn ITemplateContext,
    _model: &mut dyn IModel,
    _attribute_name: &thymeleaf::engine::AttributeName,
    _attribute_value: Option<Utf16String>,
    structure_handler: &mut dyn IElementModelStructureHandler,
) -> ProcessResult {
    structure_handler.set_local_variable(
        Utf16String::from_rust_str("local"),
        Some(Arc::new(TemplateValue::string(Utf16String::from_rust_str(
            "modified!",
        )))),
    );
    Ok(())
}
