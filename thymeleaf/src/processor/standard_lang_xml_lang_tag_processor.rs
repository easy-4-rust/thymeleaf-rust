use super::{
    AbstractStandardDoubleAttributeModifierTagProcessor, delegate_standard_element_tag_processor,
};
use crate::TemplateMode;
use crate::util::Utf16String;
/// 同时设置 `lang` 和 `xml:lang` 的 Processor。对应 Java: `org.thymeleaf.standard.processor.StandardLangXmlLangTagProcessor`。
pub struct StandardLangXmlLangTagProcessor {
    processor: AbstractStandardDoubleAttributeModifierTagProcessor,
}
impl StandardLangXmlLangTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 990;
    /// 匹配属性名。
    pub const ATTR_NAME: &'static str = "lang-xmllang";
    /// 创建 Processor。
    /// 对应 Java 语义：`StandardLangXmlLangTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardDoubleAttributeModifierTagProcessor::new(
                TemplateMode::HTML,
                dialect_prefix,
                Utf16String::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                Utf16String::from_rust_str("lang"),
                Utf16String::from_rust_str("xml:lang"),
                true,
                "org.thymeleaf.standard.processor.StandardLangXmlLangTagProcessor",
            )?,
        })
    }
}
delegate_standard_element_tag_processor!(StandardLangXmlLangTagProcessor, processor);
