use crate::TemplateMode;
use crate::util::JavaString;

use super::{AbstractStandardTargetSelectionTagProcessor, delegate_standard_element_tag_processor};

/// `th:object` selection target Processor。
///
/// 对应 Java: `org.thymeleaf.standard.processor.StandardObjectTagProcessor`。
pub struct StandardObjectTagProcessor {
    processor: AbstractStandardTargetSelectionTagProcessor,
}

impl StandardObjectTagProcessor {
    /// Java Processor precedence。
    pub const PRECEDENCE: i32 = 500;
    /// Standard 属性本地名称。
    pub const ATTR_NAME: &'static str = "object";

    /// 创建指定模板模式和方言前缀的 `th:object` Processor。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardTargetSelectionTagProcessor::new(
                template_mode,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                |_context, _tag, _attribute_name, _attribute_value, _expression| Ok(()),
                |_context, _tag, _attribute_name, _attribute_value, _expression| Ok(None),
                "org.thymeleaf.standard.processor.StandardObjectTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardObjectTagProcessor, processor);
