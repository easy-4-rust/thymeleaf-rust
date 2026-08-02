use crate::TemplateMode;
use crate::util::JavaString;

use super::{
    AbstractStandardAttributeModifierTagProcessor, delegate_standard_element_tag_processor,
};

/// 提前处理不可因空值删除属性的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardNonRemovableAttributeTagProcessor`。
pub struct StandardNonRemovableAttributeTagProcessor {
    processor: AbstractStandardAttributeModifierTagProcessor,
}

impl StandardNonRemovableAttributeTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// StandardDialect 注册的属性集合。
    pub const ATTR_NAMES: &'static [&'static str] = &["name", "type"];

    /// 创建指定属性 Processor。
    pub fn new(
        dialect_prefix: Option<JavaString>,
        attr_name: JavaString,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardAttributeModifierTagProcessor::new(
                TemplateMode::HTML,
                dialect_prefix,
                attr_name,
                Self::PRECEDENCE,
                false,
                false,
                "org.thymeleaf.standard.processor.StandardNonRemovableAttributeTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardNonRemovableAttributeTagProcessor, processor);
