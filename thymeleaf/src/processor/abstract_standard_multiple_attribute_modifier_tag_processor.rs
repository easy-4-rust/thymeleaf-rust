use crate::TemplateMode;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{AssignationUtils, StandardExpressionExecutionContext, TemplateValue};
use crate::model::IProcessableElementTag;
use crate::util::{EscapedAttributeUtils, EvaluationUtils, JavaEvaluationValue, Utf16String};

use super::{
    IProcessor, StandardAttributeCallback, StandardConditionalFixedValueTagProcessor,
    expression_processing_error, is_empty_or_java_whitespace,
};

/// 多属性赋值的修改方式。
///
/// 对应 Java:
/// `AbstractStandardMultipleAttributeModifierTagProcessor.ModificationType`。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModificationType {
    /// 替换属性。
    Substitution,
    /// 无分隔追加。
    Append,
    /// 无分隔前置。
    Prepend,
    /// 空格分隔追加。
    AppendWithSpace,
    /// 空格分隔前置。
    PrependWithSpace,
}

/// 解析 `name=value` 序列并批量修改标签属性的抽象 Processor。
///
/// 保留赋值序列预处理、restricted execution、NO-OP、HTML 固定值条件属性和
/// append/prepend 当前值组合语义。对应 Java:
/// `org.thymeleaf.standard.processor.AbstractStandardMultipleAttributeModifierTagProcessor`。
pub struct AbstractStandardMultipleAttributeModifierTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl AbstractStandardMultipleAttributeModifierTagProcessor {
    /// 创建多属性修改器。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`AbstractStandardMultipleAttributeModifierTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
        attr_name: Utf16String,
        precedence: i32,
        modification_type: ModificationType,
        restricted_expression_execution: bool,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException> {
        let callback: StandardAttributeCallback = Box::new(
            move |context, tag, _attribute_name, attribute_value, structure_handler| {
                let assignations = AssignationUtils::parse_assignation_sequence(
                    context,
                    attribute_value.as_ref(),
                    false,
                )
                .map_err(|error| {
                    expression_processing_error(
                        "Could not parse value as attribute assignations",
                        error,
                    )
                })?;
                let execution_context = if restricted_expression_execution {
                    StandardExpressionExecutionContext::RESTRICTED
                } else {
                    StandardExpressionExecutionContext::NORMAL
                };

                for assignation in assignations.get_assignations().iter() {
                    let assignation = assignation.as_ref().ok_or_else(|| {
                        Box::new(TemplateProcessingException::new(Some(
                            "Assignation list cannot contain any nulls".to_owned(),
                        ))) as Box<dyn TemplateEngineException>
                    })?;
                    let left_value = assignation
                        .get_left()
                        .execute_with_context(context, execution_context)
                        .map_err(|error| {
                            expression_processing_error(
                                "Could not execute attribute name expression",
                                error,
                            )
                        })?;
                    let right_expression = assignation.get_right().ok_or_else(|| {
                        Box::new(TemplateProcessingException::new(Some(
                            "Attribute assignation has no right-side expression".to_owned(),
                        ))) as Box<dyn TemplateEngineException>
                    })?;
                    let right_value = right_expression
                        .execute_with_context(context, execution_context)
                        .map_err(|error| {
                            expression_processing_error(
                                "Could not execute attribute value expression",
                                error,
                            )
                        })?;

                    if right_value
                        .as_deref()
                        .is_some_and(|value| matches!(value, TemplateValue::NoOp))
                    {
                        continue;
                    }

                    let new_attribute_name = left_value
                        .as_deref()
                        .and_then(TemplateValue::to_utf16_string);
                    if is_empty_or_java_whitespace(new_attribute_name.as_ref()) {
                        return Err(Box::new(TemplateProcessingException::new(Some(format!(
                            "Attribute name expression evaluated as null or empty: \"{}\"",
                            assignation
                                .get_left()
                                .get_string_representation()
                                .map_or_else(|_| String::new(), |value| value.to_string_lossy())
                        )))));
                    }
                    let new_attribute_name =
                        new_attribute_name.expect("non-empty attribute name was checked");

                    if template_mode == TemplateMode::HTML
                        && modification_type == ModificationType::Substitution
                        && StandardConditionalFixedValueTagProcessor::ATTR_NAMES
                            .iter()
                            .any(|name| {
                                new_attribute_name
                                    .as_utf16()
                                    .iter()
                                    .copied()
                                    .eq(name.encode_utf16())
                            })
                    {
                        let evaluation_value = right_value.as_deref().map_or(
                            JavaEvaluationValue::Null,
                            TemplateValue::to_evaluation_value,
                        );
                        if EvaluationUtils::evaluate_as_boolean(&evaluation_value).map_err(
                            |error| {
                                Box::new(TemplateProcessingException::with_cause(
                                    Some("Could not evaluate conditional attribute".to_owned()),
                                    error,
                                ))
                                    as Box<dyn TemplateEngineException>
                            },
                        )? {
                            structure_handler.set_attribute(
                                new_attribute_name.clone(),
                                Some(new_attribute_name),
                                None,
                            );
                        } else {
                            structure_handler.remove_attribute(new_attribute_name);
                        }
                        continue;
                    }

                    let raw_value = right_value
                        .as_deref()
                        .and_then(TemplateValue::to_utf16_string);
                    let escaped = EscapedAttributeUtils::escape_attribute(
                        Some(template_mode),
                        raw_value.as_ref(),
                    )
                    .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
                    let Some(escaped) = escaped.filter(|value| !value.is_empty()) else {
                        if modification_type == ModificationType::Substitution {
                            structure_handler.remove_attribute(new_attribute_name);
                        }
                        continue;
                    };

                    let current =
                        tag.get_attribute_value(&new_attribute_name)
                            .map_err(|error| {
                                Box::new(TemplateProcessingException::with_cause(
                                    Some("Could not read current attribute value".to_owned()),
                                    error,
                                ))
                                    as Box<dyn TemplateEngineException>
                            })?;
                    let output = if modification_type == ModificationType::Substitution
                        || current.is_none_or(Utf16String::is_empty)
                    {
                        escaped
                    } else {
                        let current = current.expect("non-empty current value was checked");
                        match modification_type {
                            ModificationType::Substitution => escaped,
                            ModificationType::Append => concat(current, None, &escaped),
                            ModificationType::AppendWithSpace => {
                                concat(current, Some(0x20), &escaped)
                            }
                            ModificationType::Prepend => concat(&escaped, None, current),
                            ModificationType::PrependWithSpace => {
                                concat(&escaped, Some(0x20), current)
                            }
                        }
                    };
                    structure_handler.set_attribute(new_attribute_name, Some(output), None);
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

impl IProcessor for AbstractStandardMultipleAttributeModifierTagProcessor {
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

impl IElementProcessor for AbstractStandardMultipleAttributeModifierTagProcessor {
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

impl IElementTagProcessor for AbstractStandardMultipleAttributeModifierTagProcessor {
    fn process(
        &self,
        context: &dyn crate::context::ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}

fn concat(left: &Utf16String, separator: Option<u16>, right: &Utf16String) -> Utf16String {
    let mut units = Vec::with_capacity(left.len() + right.len() + usize::from(separator.is_some()));
    units.extend_from_slice(left.as_utf16());
    if let Some(separator) = separator {
        units.push(separator);
    }
    units.extend_from_slice(right.as_utf16());
    Utf16String::from_utf16(units)
}
