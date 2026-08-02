use super::{
    AbstractStandardAttributeModifierTagProcessor, delegate_standard_element_tag_processor,
};
use crate::TemplateMode;
use crate::util::JavaString;
/// HTML `th:href` 属性修改 Processor。对应 Java: `org.thymeleaf.standard.processor.StandardHrefTagProcessor`。
pub struct StandardHrefTagProcessor {
    processor: AbstractStandardAttributeModifierTagProcessor,
}
impl StandardHrefTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "href";
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
                false,
                true,
                "org.thymeleaf.standard.processor.StandardHrefTagProcessor",
            )?,
        })
    }
}
delegate_standard_element_tag_processor!(StandardHrefTagProcessor, processor);
