use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::exceptions::{
    TemplateAssertionException, TemplateEngineException, TemplateProcessingException,
};
use crate::expression::ExpressionSequenceUtils;
use crate::model::IProcessableElementTag;
use crate::util::{EvaluationUtils, Utf16String};

use super::{
    IProcessor, StandardAttributeCallback, expression_processing_error, is_empty_or_java_whitespace,
};

/// 执行逗号分隔 Standard Expression 断言序列的抽象 Processor。
///
/// 对应 Java: `org.thymeleaf.standard.processor.AbstractStandardAssertionTagProcessor`。
pub struct AbstractStandardAssertionTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl AbstractStandardAssertionTagProcessor {
    /// 创建断言属性 Processor；空或全空白属性值不执行任何断言。
    /// 对应 Java 语义：`AbstractStandardAssertionTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
        attr_name: Utf16String,
        precedence: i32,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException> {
        let callback: StandardAttributeCallback = Box::new(
            move |context, tag, attribute_name, attribute_value, _structure_handler| {
                if is_empty_or_java_whitespace(attribute_value.as_ref()) {
                    return Ok(());
                }
                let sequence = ExpressionSequenceUtils::parse_expression_sequence(
                    context,
                    attribute_value.as_ref(),
                )
                .map_err(|error| {
                    expression_processing_error("Could not parse assertion sequence", error)
                })?;
                for expression in sequence.get_expressions().iter().flatten() {
                    let result = expression.execute(context).map_err(|error| {
                        expression_processing_error("Could not execute assertion expression", error)
                    })?;
                    let evaluation_value = result.as_deref().map_or(
                        crate::util::EvaluationValue::Null,
                        crate::expression::TemplateValue::to_evaluation_value,
                    );
                    let assertion_valid = EvaluationUtils::evaluate_as_boolean(&evaluation_value)
                        .map_err(|error| {
                        Box::new(TemplateProcessingException::with_cause(
                            Some("Could not evaluate assertion result as boolean".to_owned()),
                            error,
                        )) as Box<dyn TemplateEngineException>
                    })?;
                    if !assertion_valid {
                        let representation =
                            expression.get_string_representation().map_err(|error| {
                                expression_processing_error(
                                    "Could not render assertion expression",
                                    error,
                                )
                            })?;
                        let attribute = tag.get_attribute_by_name(attribute_name);
                        let (line, col) = attribute
                            .map_or((tag.get_line(), tag.get_col()), |value| {
                                (value.get_line(), value.get_col())
                            });
                        return Err(Box::new(TemplateAssertionException::with_location(
                            Some(&representation.to_string_lossy()),
                            tag.get_template_name()
                                .map(Utf16String::to_string_lossy)
                                .as_deref(),
                            line,
                            col,
                        )));
                    }
                }
                Ok(())
            },
        );
        Ok(Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(template_mode),
                dialect_prefix,
                None,
                false,
                Some(attr_name),
                true,
                precedence,
                true,
                processor_class_name,
                callback,
            )?,
        })
    }
}

impl IProcessor for AbstractStandardAssertionTagProcessor {
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

impl IElementProcessor for AbstractStandardAssertionTagProcessor {
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

impl IElementTagProcessor for AbstractStandardAssertionTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}
