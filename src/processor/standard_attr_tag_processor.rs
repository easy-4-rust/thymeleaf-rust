use crate::TemplateMode;
use crate::util::JavaString;

use super::{
    AbstractStandardMultipleAttributeModifierTagProcessor, ModificationType,
    delegate_standard_element_tag_processor,
};

/// 批量替换 `th:attr` 指定属性的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardAttrTagProcessor`。
pub struct StandardAttrTagProcessor {
    processor: AbstractStandardMultipleAttributeModifierTagProcessor,
}

impl StandardAttrTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 700;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "attr";

    /// 创建 Processor。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardMultipleAttributeModifierTagProcessor::new(
                template_mode,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                ModificationType::Substitution,
                true,
                "org.thymeleaf.standard.processor.StandardAttrTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardAttrTagProcessor, processor);
