use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use crate::context::ITemplateContext;
use crate::engine::AttributeName;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{StandardExpressions, TemplateValue};
use crate::model::IProcessableElementTag;
use crate::util::{EvaluationUtils, JavaEvaluationValue, Utf16String};

use crate::element::IElementTagStructureHandler;

/// Standard 属性 Processor 共用的动态处理回调。
///
/// 这是 Rust 组合继承适配类型，不对应独立 Java 主对象。
pub(crate) type StandardAttributeCallback = Box<
    dyn Fn(
            &dyn ITemplateContext,
            &dyn IProcessableElementTag,
            &AttributeName,
            Option<Utf16String>,
            &mut dyn IElementTagStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
>;

/// Standard 元素 Processor 共用的动态处理回调。
pub(crate) type StandardElementCallback = Box<
    dyn Fn(
            &dyn ITemplateContext,
            &dyn IProcessableElementTag,
            &mut dyn IElementTagStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
>;

/// 将 Standard Expression 错误保留为模板处理异常的 cause。
/// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
pub(crate) fn expression_processing_error(
    message: &'static str,
    error: Box<dyn Error + Send + Sync>,
) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::with_cause(
        Some(message.to_owned()),
        StandardExpressionCause(error),
    ))
}

struct StandardExpressionCause(Box<dyn Error + Send + Sync>);

impl Display for StandardExpressionCause {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

impl Debug for StandardExpressionCause {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("StandardExpressionCause")
            .field(&self.0.to_string())
            .finish()
    }
}

impl Error for StandardExpressionCause {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}

/// 判断 Java `StringUtils.isEmptyOrWhitespace` 所定义的空白文本。
/// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
pub(crate) fn is_empty_or_java_whitespace(value: Option<&Utf16String>) -> bool {
    value.is_none_or(|value| {
        value.is_empty()
            || value.as_utf16().iter().all(|unit| {
                *unit == u16::from(b' ')
                    || matches!(
                        *unit,
                        0x0009..=0x000D
                            | 0x001C..=0x001F
                            | 0x1680
                            | 0x2000..=0x2006
                            | 0x2008..=0x200A
                            | 0x2028
                            | 0x2029
                            | 0x205F
                            | 0x3000
                    )
            })
    })
}

/// 解析并执行 Standard Expression，然后按 Thymeleaf 真值规则求布尔值。
/// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
pub(crate) fn evaluate_standard_expression_as_boolean(
    context: &dyn ITemplateContext,
    input: Option<&Utf16String>,
) -> Result<bool, Box<dyn TemplateEngineException>> {
    let parser = StandardExpressions::get_expression_parser(context.get_configuration()).map_err(
        |error| expression_processing_error("Could not obtain Standard Expression parser", error),
    )?;
    let expression = parser.parse_expression(context, input).map_err(|error| {
        expression_processing_error("Could not parse Standard Expression", error)
    })?;
    let result = expression.execute(context).map_err(|error| {
        expression_processing_error("Could not execute Standard Expression", error)
    })?;
    let evaluation_value = result.as_deref().map_or(
        JavaEvaluationValue::Null,
        TemplateValue::to_evaluation_value,
    );
    EvaluationUtils::evaluate_as_boolean(&evaluation_value).map_err(|error| {
        Box::new(TemplateProcessingException::with_cause(
            Some("Could not evaluate Standard Expression as boolean".to_owned()),
            error,
        )) as Box<dyn TemplateEngineException>
    })
}

macro_rules! delegate_standard_element_tag_processor {
    ($object:ty, $field:ident) => {
        impl crate::processor::IProcessor for $object {
            fn as_element_processor(&self) -> Option<&dyn crate::element::IElementProcessor> {
                Some(self)
            }

            fn java_class_name(&self) -> &'static str {
                crate::processor::IProcessor::java_class_name(&self.$field)
            }

            fn get_template_mode(&self) -> Option<crate::TemplateMode> {
                crate::processor::IProcessor::get_template_mode(&self.$field)
            }

            fn get_precedence(&self) -> i32 {
                crate::processor::IProcessor::get_precedence(&self.$field)
            }
        }

        impl crate::element::IElementProcessor for $object {
            fn as_element_tag_processor(
                &self,
            ) -> Option<&dyn crate::element::IElementTagProcessor> {
                Some(self)
            }

            fn get_matching_element_name(&self) -> Option<&crate::element::MatchingElementName> {
                crate::element::IElementProcessor::get_matching_element_name(&self.$field)
            }

            fn get_matching_attribute_name(
                &self,
            ) -> Option<&crate::element::MatchingAttributeName> {
                crate::element::IElementProcessor::get_matching_attribute_name(&self.$field)
            }
        }

        impl crate::element::IElementTagProcessor for $object {
            fn process(
                &self,
                context: &dyn crate::context::ITemplateContext,
                tag: &dyn crate::model::IProcessableElementTag,
                structure_handler: &mut dyn crate::element::IElementTagStructureHandler,
            ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
                crate::element::IElementTagProcessor::process(
                    &self.$field,
                    context,
                    tag,
                    structure_handler,
                )
            }
        }
    };
}

pub(crate) use delegate_standard_element_tag_processor;
