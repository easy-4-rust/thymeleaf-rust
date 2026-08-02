use crate::TemplateMode;
use crate::util::JavaString;

use super::{
    AbstractStandardAttributeModifierTagProcessor, delegate_standard_element_tag_processor,
};

/// HTML `th:src` 属性修改 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardSrcTagProcessor`。
pub struct StandardSrcTagProcessor {
    processor: AbstractStandardAttributeModifierTagProcessor,
}

impl StandardSrcTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "src";

    /// 创建 Processor。
    /// 对应 Java 语义：`StandardSrcTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardAttributeModifierTagProcessor::new(
                TemplateMode::HTML,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                false,
                true,
                "org.thymeleaf.standard.processor.StandardSrcTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardSrcTagProcessor, processor);
