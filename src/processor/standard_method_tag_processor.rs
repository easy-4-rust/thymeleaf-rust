use super::{
    AbstractStandardAttributeModifierTagProcessor, delegate_standard_element_tag_processor,
};
use crate::TemplateMode;
use crate::util::JavaString;
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
    pub fn new(
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardAttributeModifierTagProcessor::new(
                TemplateMode::HTML,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                true,
                false,
                "org.thymeleaf.standard.processor.StandardMethodTagProcessor",
            )?,
        })
    }
}
delegate_standard_element_tag_processor!(StandardMethodTagProcessor, processor);
