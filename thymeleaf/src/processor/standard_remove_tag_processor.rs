use crate::TemplateMode;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::TemplateValue;
use crate::util::Utf16String;

use super::{
    AbstractStandardExpressionAttributeTagProcessor, delegate_standard_element_tag_processor,
};

/// 按 `th:remove` 值删除元素、标签或正文的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardRemoveTagProcessor`。
pub struct StandardRemoveTagProcessor {
    processor: AbstractStandardExpressionAttributeTagProcessor,
}

impl StandardRemoveTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1600;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "remove";

    /// 创建 Processor。
    /// 对应 Java 语义：`StandardRemoveTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardExpressionAttributeTagProcessor::with_restricted_execution(
                template_mode,
                dialect_prefix,
                Utf16String::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                true,
                false,
                |_context, _tag, attribute_name, attribute_value, result, structure_handler| {
                    let Some(value) = result.as_deref().and_then(TemplateValue::to_utf16_string)
                    else {
                        return Ok(());
                    };
                    match value.to_string_lossy().to_ascii_lowercase().as_str() {
                        "all" => structure_handler.remove_element(),
                        "tag" | "tags" => structure_handler.remove_tags(),
                        "all-but-first" => structure_handler.remove_all_but_first_child(),
                        "body" => structure_handler.remove_body(),
                        "none" => {}
                        _ => {
                            return Err(Box::new(TemplateProcessingException::new(Some(format!(
                                "Invalid value specified for \"{}\": only 'all', 'tag', 'body', 'none' and 'all-but-first' are allowed, but \"{}\" was specified.",
                                attribute_name
                                    .to_utf16_string()
                                    .map_or_else(|_| String::new(), |name| name.to_string_lossy()),
                                attribute_value
                                    .map_or_else(String::new, Utf16String::to_string_lossy)
                            ))))
                                as Box<dyn TemplateEngineException>);
                        }
                    }
                    Ok(())
                },
                "org.thymeleaf.standard.processor.StandardRemoveTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardRemoveTagProcessor, processor);
