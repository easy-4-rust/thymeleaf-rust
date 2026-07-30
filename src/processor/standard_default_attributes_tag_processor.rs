use std::sync::Arc;

use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::element::{
    IElementProcessor, IElementTagProcessor, IElementTagStructureHandler, MatchingAttributeName,
    MatchingElementName,
};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{
    FragmentExpression, StandardExpressionExecutionContext, StandardExpressions, TemplateValue,
};
use crate::model::{IAttribute, IProcessableElementTag};
use crate::processor::{AbstractProcessor, IProcessor};
use crate::util::{EscapedAttributeUtils, JavaString};

use super::expression_processing_error;

/// 处理 Standard 方言 prefix 下未被专用 Processor 捕获的所有属性。
///
/// 每个属性独立解析，始终使用 RESTRICTED 执行，并把 prefix 去除后写回。对应 Java:
/// `org.thymeleaf.standard.processor.StandardDefaultAttributesTagProcessor`。
pub struct StandardDefaultAttributesTagProcessor {
    processor: AbstractProcessor,
    dialect_prefix: Option<JavaString>,
    matching_attribute_name: MatchingAttributeName,
}

impl StandardDefaultAttributesTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = i32::MAX;

    /// 创建默认属性 Processor。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, TemplateProcessingException> {
        let matching_attribute_name = MatchingAttributeName::for_all_attributes_with_prefix(
            Some(template_mode),
            dialect_prefix.clone(),
        )
        .map_err(|error| {
            TemplateProcessingException::with_cause(
                Some("Invalid default attribute matching prefix".to_owned()),
                error,
            )
        })?;
        let processor =
            AbstractProcessor::new(Some(template_mode), Self::PRECEDENCE).map_err(|error| {
                TemplateProcessingException::with_cause(
                    Some("Could not create default attributes processor".to_owned()),
                    error,
                )
            })?;
        Ok(Self {
            processor,
            dialect_prefix,
            matching_attribute_name,
        })
    }
}

impl IProcessor for StandardDefaultAttributesTagProcessor {
    fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
        Some(self)
    }

    fn java_class_name(&self) -> &'static str {
        "org.thymeleaf.standard.processor.StandardDefaultAttributesTagProcessor"
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(self.processor.get_template_mode())
    }
    fn get_precedence(&self) -> i32 {
        self.processor.get_precedence()
    }
}

impl IElementProcessor for StandardDefaultAttributesTagProcessor {
    fn as_element_tag_processor(&self) -> Option<&dyn IElementTagProcessor> {
        Some(self)
    }
    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        None
    }
    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        Some(&self.matching_attribute_name)
    }
}

impl IElementTagProcessor for StandardDefaultAttributesTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let template_mode = self.processor.get_template_mode();
        for attribute in tag.get_all_attributes() {
            let attribute_name = attribute
                .get_attribute_definition()
                .get_attribute_name()
                .as_attribute_name();
            let Some(prefix) = attribute_name.get_prefix() else {
                continue;
            };
            if !prefix_matches(template_mode, prefix, self.dialect_prefix.as_ref()) {
                continue;
            }
            process_default_attribute(template_mode, context, tag, attribute, structure_handler)?;
        }
        Ok(())
    }
}

fn process_default_attribute(
    template_mode: TemplateMode,
    context: &dyn ITemplateContext,
    tag: &dyn IProcessableElementTag,
    attribute: &dyn IAttribute,
    structure_handler: &mut dyn IElementTagStructureHandler,
) -> Result<(), Box<dyn TemplateEngineException>> {
    let attribute_name = attribute
        .get_attribute_definition()
        .get_attribute_name()
        .as_attribute_name();
    let attribute_value = EscapedAttributeUtils::unescape_attribute(
        Some(context.get_template_mode()),
        attribute.get_value(),
    )
    .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
    let original_complete_name = attribute.get_attribute_complete_name();
    let canonical_name = attribute_name.get_attribute_name();
    let new_attribute_name = if original_complete_name
        .as_utf16()
        .ends_with(canonical_name.as_utf16())
    {
        canonical_name.clone()
    } else {
        let start = original_complete_name
            .len()
            .saturating_sub(canonical_name.len());
        JavaString::from_utf16(original_complete_name.as_utf16()[start..].to_vec())
    };

    let expression_result = if let Some(attribute_value) = attribute_value.as_ref() {
        let parser = StandardExpressions::get_expression_parser(context.get_configuration())
            .map_err(|error| {
                expression_processing_error("Could not obtain Standard Expression parser", error)
            })?;
        let expression = parser
            .parse_expression(context, Some(attribute_value))
            .map_err(|error| {
                expression_processing_error("Could not parse default attribute", error)
            })?;
        if let Some(fragment_expression) = expression.as_fragment_expression() {
            let executed = FragmentExpression::create_executed_fragment_expression(
                context,
                fragment_expression,
            )
            .map_err(|error| {
                expression_processing_error("Could not execute Fragment expression", error)
            })?;
            FragmentExpression::resolve_executed_fragment_expression(context, &executed, true)
                .map_err(|error| {
                    expression_processing_error("Could not resolve Fragment expression", error)
                })?
                .map(|fragment| Arc::new(TemplateValue::Object(fragment)))
        } else {
            expression
                .execute_with_context(context, StandardExpressionExecutionContext::RESTRICTED)
                .map_err(|error| {
                    expression_processing_error("Could not execute default attribute", error)
                })?
        }
    } else {
        None
    };

    if expression_result
        .as_deref()
        .is_some_and(|value| matches!(value, TemplateValue::NoOp))
    {
        structure_handler.remove_attribute(original_complete_name.clone());
        return Ok(());
    }
    let raw = expression_result
        .as_deref()
        .and_then(TemplateValue::to_java_string);
    let escaped = EscapedAttributeUtils::escape_attribute(Some(template_mode), raw.as_ref())
        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
    if escaped.as_ref().is_none_or(JavaString::is_empty) {
        structure_handler.remove_attribute(new_attribute_name);
        structure_handler.remove_attribute(original_complete_name.clone());
    } else {
        structure_handler.set_attribute(new_attribute_name, escaped, None);
        structure_handler.remove_attribute(original_complete_name.clone());
    }
    let _ = tag;
    Ok(())
}

fn prefix_matches(
    template_mode: TemplateMode,
    actual: &JavaString,
    expected: Option<&JavaString>,
) -> bool {
    let Some(expected) = expected else {
        return false;
    };
    if template_mode.is_case_sensitive() {
        actual == expected
    } else {
        actual
            .to_string_lossy()
            .eq_ignore_ascii_case(&expected.to_string_lossy())
    }
}
