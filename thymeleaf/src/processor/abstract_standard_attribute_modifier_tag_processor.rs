use crate::TemplateMode;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::TemplateValue;
use crate::util::{EscapedAttributeUtils, Utf16String};

use super::{
    AbstractStandardExpressionAttributeTagProcessor, delegate_standard_element_tag_processor,
};

/// 将 Standard Expression 结果转义后替换目标属性的抽象 Processor。
///
/// 保留可空删除、目标属性改名和 restricted execution 构造语义。对应 Java:
/// `org.thymeleaf.standard.processor.AbstractStandardAttributeModifierTagProcessor`。
pub struct AbstractStandardAttributeModifierTagProcessor {
    processor: AbstractStandardExpressionAttributeTagProcessor,
}

impl AbstractStandardAttributeModifierTagProcessor {
    /// 创建目标属性与匹配属性同名的修改器。
    /// 对应 Java 语义：`AbstractStandardAttributeModifierTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
        attr_name: Utf16String,
        precedence: i32,
        remove_if_empty: bool,
        restricted_expression_execution: bool,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException> {
        Self::with_target(
            template_mode,
            dialect_prefix,
            attr_name.clone(),
            attr_name,
            precedence,
            remove_if_empty,
            restricted_expression_execution,
            processor_class_name,
        )
    }

    /// 创建可指定完整目标属性名的修改器。
    #[allow(clippy::too_many_arguments)]
    /// 对应 Java 语义：`AbstractStandardAttributeModifierTagProcessor` 的 `with_target` 行为（Rust 侧辅助/私有路径）。
    pub fn with_target(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
        attr_name: Utf16String,
        target_attr_complete_name: Utf16String,
        precedence: i32,
        remove_if_empty: bool,
        restricted_expression_execution: bool,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardExpressionAttributeTagProcessor::with_restricted_execution(
                template_mode,
                dialect_prefix,
                attr_name,
                precedence,
                false,
                restricted_expression_execution,
                move |_context,
                      tag,
                      attribute_name,
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
                        structure_handler.remove_attribute(target_attr_complete_name.clone());
                        structure_handler.remove_attribute_with_prefix(
                            attribute_name.get_prefix().cloned(),
                            attribute_name.get_attribute_name().clone(),
                        );
                    } else {
                        let value = escaped.or_else(|| Some(Utf16String::from_rust_str("")));
                        if let Some(source_attribute) = tag.get_attribute_by_name(attribute_name) {
                            // 对应 Java StandardProcessorUtils#replaceAttribute：在原
                            // 位置改名并继承输入模板的引号，而不是追加到属性末尾。
                            structure_handler.replace_attribute(
                                source_attribute
                                    .get_attribute_definition()
                                    .get_attribute_name()
                                    .clone(),
                                target_attr_complete_name.clone(),
                                value,
                                None,
                            );
                        } else {
                            // 第三方 IProcessableElementTag 若违反回调期间属性仍存在的
                            // 引擎约束，保持旧有的安全降级行为。
                            structure_handler.set_attribute(
                                target_attr_complete_name.clone(),
                                value,
                                None,
                            );
                        }
                    }
                    Ok(())
                },
                processor_class_name,
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(AbstractStandardAttributeModifierTagProcessor, processor);
