use crate::TemplateMode;
use crate::util::JavaString;

use super::{
    AbstractStandardConditionalVisibilityTagProcessor, delegate_standard_element_tag_processor,
    evaluate_standard_expression_as_boolean,
};

/// `th:if` 条件可见性 Processor。
///
/// 对应 Java: `org.thymeleaf.standard.processor.StandardIfTagProcessor`。
pub struct StandardIfTagProcessor {
    processor: AbstractStandardConditionalVisibilityTagProcessor,
}

impl StandardIfTagProcessor {
    /// Java Processor precedence。
    pub const PRECEDENCE: i32 = 300;
    /// Standard 属性本地名称。
    pub const ATTR_NAME: &'static str = "if";

    /// 创建指定模板模式和方言前缀的 `th:if` Processor。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardConditionalVisibilityTagProcessor::new(
                template_mode,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                |context, _tag, _attribute_name, attribute_value| {
                    evaluate_standard_expression_as_boolean(context, attribute_value)
                },
                "org.thymeleaf.standard.processor.StandardIfTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardIfTagProcessor, processor);
