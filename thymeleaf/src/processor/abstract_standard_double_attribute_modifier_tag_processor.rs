use crate::TemplateMode;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::TemplateValue;
use crate::util::{EscapedAttributeUtils, Utf16String};

use super::{
    AbstractStandardExpressionAttributeTagProcessor, delegate_standard_element_tag_processor,
};

/// 使用一次表达式结果同步修改两个目标属性的抽象 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.standard.processor.AbstractStandardDoubleAttributeModifierTagProcessor`。
pub struct AbstractStandardDoubleAttributeModifierTagProcessor {
    processor: AbstractStandardExpressionAttributeTagProcessor,
}

impl AbstractStandardDoubleAttributeModifierTagProcessor {
    /// 创建双目标属性修改器。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`AbstractStandardDoubleAttributeModifierTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
        attr_name: Utf16String,
        precedence: i32,
        attribute_one_complete_name: Utf16String,
        attribute_two_complete_name: Utf16String,
        remove_if_empty: bool,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardExpressionAttributeTagProcessor::new(
                template_mode,
                dialect_prefix,
                attr_name,
                precedence,
                true,
                crate::expression::StandardExpressionExecutionContext::NORMAL,
                move |_context,
                      _tag,
                      _attribute_name,
                      _attribute_value,
                      expression_result,
                      structure_handler| {
                    let value = expression_result
                        .as_deref()
                        .and_then(TemplateValue::to_utf16_string);
                    let escaped = EscapedAttributeUtils::escape_attribute(
                        Some(template_mode),
                        value.as_ref(),
                    )
                    .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?;
                    if remove_if_empty && escaped.as_ref().is_none_or(Utf16String::is_empty) {
                        structure_handler.remove_attribute(attribute_one_complete_name.clone());
                        structure_handler.remove_attribute(attribute_two_complete_name.clone());
                    } else {
                        structure_handler.set_attribute(
                            attribute_one_complete_name.clone(),
                            escaped.clone(),
                            None,
                        );
                        structure_handler.set_attribute(
                            attribute_two_complete_name.clone(),
                            escaped,
                            None,
                        );
                    }
                    Ok(())
                },
                processor_class_name,
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(
    AbstractStandardDoubleAttributeModifierTagProcessor,
    processor
);
