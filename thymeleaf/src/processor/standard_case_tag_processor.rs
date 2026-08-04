use crate::TemplateMode;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{
    EqualsExpression, IStandardExpression, StandardExpressions, TemplateValue,
};
use crate::util::{EvaluationUtils, EvaluationValue, Utf16String};

use super::{
    AbstractStandardConditionalVisibilityTagProcessor, StandardSwitchTagProcessor, SwitchStructure,
    delegate_standard_element_tag_processor, expression_processing_error,
};

/// 在最近 `th:switch` 环境中短路匹配 `th:case` 的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardCaseTagProcessor`。
pub struct StandardCaseTagProcessor {
    processor: AbstractStandardConditionalVisibilityTagProcessor,
}

impl StandardCaseTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 275;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "case";
    /// default case 标记。
    pub const CASE_DEFAULT_ATTRIBUTE_VALUE: &'static str = "*";

    /// 创建 Processor。
    /// 对应 Java 语义：`StandardCaseTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardConditionalVisibilityTagProcessor::new(
                template_mode,
                dialect_prefix,
                Utf16String::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                |context, _tag, attribute_name, attribute_value| {
                    let switch_value = context.get_variable(Some(&Utf16String::from_rust_str(
                        StandardSwitchTagProcessor::SWITCH_VARIABLE_NAME,
                    )));
                    let switch_structure = switch_value
                        .as_deref()
                        .and_then(|value| match value {
                            TemplateValue::Object(object) => {
                                object.as_any().downcast_ref::<SwitchStructure>()
                            }
                            _ => None,
                        })
                        .ok_or_else(|| {
                            Box::new(TemplateProcessingException::new(Some(format!(
                                "Cannot specify a \"{}\" attribute in an environment where no switch operator has been defined before.",
                                attribute_name.to_utf16_string().map_or_else(
                                    |_| String::new(),
                                    |value| value.to_string_lossy()
                                )
                            )))) as Box<dyn TemplateEngineException>
                        })?;
                    if switch_structure.is_executed() {
                        return Ok(false);
                    }
                    if attribute_value.is_some_and(is_default_case) {
                        switch_structure.set_executed(true);
                        return Ok(true);
                    }

                    let parser =
                        StandardExpressions::get_expression_parser(context.get_configuration())
                            .map_err(|error| {
                                expression_processing_error(
                                    "Could not obtain Standard Expression parser",
                                    error,
                                )
                            })?;
                    let case_expression = parser
                        .parse_expression(context, attribute_value)
                        .map_err(|error| {
                            expression_processing_error("Could not parse case expression", error)
                        })?;
                    let equals_expression = EqualsExpression::new(
                        Some(switch_structure.get_expression()),
                        Some(case_expression),
                    )
                    .map_err(|error| {
                        Box::new(TemplateProcessingException::with_cause(
                            Some("Could not create case equality expression".to_owned()),
                            error,
                        )) as Box<dyn TemplateEngineException>
                    })?;
                    let value = equals_expression.execute(context).map_err(|error| {
                        expression_processing_error("Could not execute case expression", error)
                    })?;
                    let evaluation_value = value
                        .as_deref()
                        .map_or(EvaluationValue::Null, TemplateValue::to_evaluation_value);
                    let visible = EvaluationUtils::evaluate_as_boolean(&evaluation_value).map_err(
                        |error| {
                            Box::new(TemplateProcessingException::with_cause(
                                Some("Could not evaluate case expression".to_owned()),
                                error,
                            )) as Box<dyn TemplateEngineException>
                        },
                    )?;
                    if visible {
                        switch_structure.set_executed(true);
                    }
                    Ok(visible)
                },
                "org.thymeleaf.standard.processor.StandardCaseTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardCaseTagProcessor, processor);

fn is_default_case(value: &Utf16String) -> bool {
    let units = value.as_utf16();
    let start = units
        .iter()
        .position(|unit| *unit > 0x20)
        .unwrap_or(units.len());
    let end = units
        .iter()
        .rposition(|unit| *unit > 0x20)
        .map_or(start, |position| position + 1);
    units[start..end] == [u16::from(b'*')]
}
