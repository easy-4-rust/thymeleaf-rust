use crate::TemplateMode;
use crate::util::Utf16String;

use super::{
    AbstractStandardConditionalVisibilityTagProcessor, delegate_standard_element_tag_processor,
    evaluate_standard_expression_as_boolean,
};

/// `th:unless` 反向条件可见性 Processor。
///
/// 对应 Java: `org.thymeleaf.standard.processor.StandardUnlessTagProcessor`。
pub struct StandardUnlessTagProcessor {
    processor: AbstractStandardConditionalVisibilityTagProcessor,
}

impl StandardUnlessTagProcessor {
    /// Java Processor precedence。
    pub const PRECEDENCE: i32 = 400;
    /// Standard 属性本地名称。
    pub const ATTR_NAME: &'static str = "unless";

    /// 创建指定模板模式和方言前缀的 `th:unless` Processor。
    /// 对应 Java 语义：`StandardUnlessTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardConditionalVisibilityTagProcessor::new(
                template_mode,
                dialect_prefix,
                Utf16String::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                |context, _tag, _attribute_name, attribute_value| {
                    evaluate_standard_expression_as_boolean(context, attribute_value)
                        .map(std::ops::Not::not)
                },
                "org.thymeleaf.standard.processor.StandardUnlessTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardUnlessTagProcessor, processor);
