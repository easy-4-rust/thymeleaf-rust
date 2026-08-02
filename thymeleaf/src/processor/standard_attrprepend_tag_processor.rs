use crate::TemplateMode;
use crate::util::JavaString;

use super::{
    AbstractStandardMultipleAttributeModifierTagProcessor, ModificationType,
    delegate_standard_element_tag_processor,
};

/// 批量前置 `th:attrprepend` 指定属性的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardAttrprependTagProcessor`。
pub struct StandardAttrprependTagProcessor {
    processor: AbstractStandardMultipleAttributeModifierTagProcessor,
}

impl StandardAttrprependTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 800;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "attrprepend";

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
                ModificationType::Prepend,
                true,
                "org.thymeleaf.standard.processor.StandardAttrprependTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardAttrprependTagProcessor, processor);
