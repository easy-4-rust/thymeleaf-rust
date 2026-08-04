use super::{
    AbstractStandardAttributeModifierTagProcessor, delegate_standard_element_tag_processor,
};
use crate::TemplateMode;
use crate::util::Utf16String;
/// HTML `th:method` 属性修改 Processor。对应 Java: `org.thymeleaf.standard.processor.StandardMethodTagProcessor`。
pub struct StandardMethodTagProcessor {
    processor: AbstractStandardAttributeModifierTagProcessor,
}
impl StandardMethodTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "method";
    /// 创建 Processor。
    /// 对应 Java 语义：`StandardMethodTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardAttributeModifierTagProcessor::new(
                TemplateMode::HTML,
                dialect_prefix,
                Utf16String::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                true,
                false,
                "org.thymeleaf.standard.processor.StandardMethodTagProcessor",
            )?,
        })
    }
}
delegate_standard_element_tag_processor!(StandardMethodTagProcessor, processor);
