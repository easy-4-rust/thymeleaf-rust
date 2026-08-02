use super::{
    AbstractStandardDoubleAttributeModifierTagProcessor, delegate_standard_element_tag_processor,
};
use crate::TemplateMode;
use crate::util::JavaString;
/// 同时设置 `alt` 和 `title` 的 Processor。对应 Java: `org.thymeleaf.standard.processor.StandardAltTitleTagProcessor`。
pub struct StandardAltTitleTagProcessor {
    processor: AbstractStandardDoubleAttributeModifierTagProcessor,
}
impl StandardAltTitleTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 990;
    /// 匹配属性名。
    pub const ATTR_NAME: &'static str = "alt-title";
    /// 创建 Processor。
    /// 对应 Java 语义：`StandardAltTitleTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardDoubleAttributeModifierTagProcessor::new(
                TemplateMode::HTML,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                JavaString::from_rust_str("alt"),
                JavaString::from_rust_str("title"),
                true,
                "org.thymeleaf.standard.processor.StandardAltTitleTagProcessor",
            )?,
        })
    }
}
delegate_standard_element_tag_processor!(StandardAltTitleTagProcessor, processor);
