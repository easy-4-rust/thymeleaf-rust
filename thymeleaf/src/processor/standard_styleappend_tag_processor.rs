use crate::TemplateMode;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::TemplateValue;
use crate::util::{EscapedAttributeUtils, JavaString};

use super::{
    AbstractStandardExpressionAttributeTagProcessor, delegate_standard_element_tag_processor,
};

/// 将表达式结果以空格分隔追加到 HTML `style` 属性的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardStyleappendTagProcessor`。
pub struct StandardStyleappendTagProcessor {
    processor: AbstractStandardExpressionAttributeTagProcessor,
}

impl StandardStyleappendTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1100;
    /// 匹配属性名。
    pub const ATTR_NAME: &'static str = "styleappend";
    /// 目标属性名。
    pub const TARGET_ATTR_NAME: &'static str = "style";

    /// 创建 Processor。
    /// 对应 Java 语义：`StandardStyleappendTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(dialect_prefix: Option<JavaString>) -> Result<Self, TemplateProcessingException> {
        let target_name = JavaString::from_rust_str(Self::TARGET_ATTR_NAME);
        Ok(Self {
            processor: AbstractStandardExpressionAttributeTagProcessor::with_restricted_execution(
                TemplateMode::HTML,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                true,
                false,
                move |_context,
                      tag,
                      _attribute_name,
                      _attribute_value,
                      expression_result,
                      structure_handler| {
                    let raw = expression_result
                        .as_deref()
                        .and_then(TemplateValue::to_java_string);
                    let Some(mut escaped) = EscapedAttributeUtils::escape_attribute(
                        Some(TemplateMode::HTML),
                        raw.as_ref(),
                    )
                    .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?
                    .filter(|value| !value.is_empty()) else {
                        return Ok(());
                    };
                    if let Some(current) =
                        tag.get_attribute_value(&target_name).map_err(|error| {
                            Box::new(TemplateProcessingException::with_cause(
                                Some("Could not read current style attribute".to_owned()),
                                error,
                            )) as Box<dyn TemplateEngineException>
                        })?
                        && !current.is_empty()
                    {
                        escaped = join_with_space(current, &escaped);
                    }
                    structure_handler.set_attribute(target_name.clone(), Some(escaped), None);
                    Ok(())
                },
                "org.thymeleaf.standard.processor.StandardStyleappendTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardStyleappendTagProcessor, processor);

fn join_with_space(left: &JavaString, right: &JavaString) -> JavaString {
    let mut units = Vec::with_capacity(left.len() + right.len() + 1);
    units.extend_from_slice(left.as_utf16());
    units.push(0x20);
    units.extend_from_slice(right.as_utf16());
    JavaString::from_utf16(units)
}
