use super::{
    AbstractStandardAttributeModifierTagProcessor, delegate_standard_element_tag_processor,
};
use crate::TemplateMode;
use crate::util::JavaString;
/// HTML `th:xmlbase` 到 `xml:base` 的属性修改 Processor。对应 Java: `org.thymeleaf.standard.processor.StandardXmlBaseTagProcessor`。
pub struct StandardXmlBaseTagProcessor {
    processor: AbstractStandardAttributeModifierTagProcessor,
}
impl StandardXmlBaseTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// 匹配属性名。
    pub const ATTR_NAME: &'static str = "xmlbase";
    /// 目标属性名。
    pub const TARGET_ATTR_NAME: &'static str = "xml:base";
    /// 创建 Processor。
    /// 对应 Java 语义：`StandardXmlBaseTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardAttributeModifierTagProcessor::with_target(
                TemplateMode::HTML,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                JavaString::from_rust_str(Self::TARGET_ATTR_NAME),
                Self::PRECEDENCE,
                true,
                false,
                "org.thymeleaf.standard.processor.StandardXmlBaseTagProcessor",
            )?,
        })
    }
}
delegate_standard_element_tag_processor!(StandardXmlBaseTagProcessor, processor);
