use crate::TemplateMode;
use crate::util::JavaString;

use super::{
    AbstractStandardMultipleAttributeModifierTagProcessor, ModificationType,
    delegate_standard_element_tag_processor,
};

/// 批量追加 `th:attrappend` 指定属性的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardAttrappendTagProcessor`。
pub struct StandardAttrappendTagProcessor {
    processor: AbstractStandardMultipleAttributeModifierTagProcessor,
}

impl StandardAttrappendTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 900;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "attrappend";

    /// 创建 Processor。
    /// 对应 Java 语义：`StandardAttrappendTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
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
                ModificationType::Append,
                true,
                "org.thymeleaf.standard.processor.StandardAttrappendTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardAttrappendTagProcessor, processor);
