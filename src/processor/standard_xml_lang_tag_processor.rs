use super::{
    AbstractStandardAttributeModifierTagProcessor, delegate_standard_element_tag_processor,
};
use crate::TemplateMode;
use crate::util::JavaString;
/// HTML `th:xmllang` 到 `xml:lang` 的属性修改 Processor。对应 Java: `org.thymeleaf.standard.processor.StandardXmlLangTagProcessor`。
pub struct StandardXmlLangTagProcessor {
    processor: AbstractStandardAttributeModifierTagProcessor,
}
impl StandardXmlLangTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// 匹配属性名。
    pub const ATTR_NAME: &'static str = "xmllang";
    /// 目标属性名。
    pub const TARGET_ATTR_NAME: &'static str = "xml:lang";
    /// 创建 Processor。
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
                "org.thymeleaf.standard.processor.StandardXmlLangTagProcessor",
            )?,
        })
    }
}
delegate_standard_element_tag_processor!(StandardXmlLangTagProcessor, processor);
