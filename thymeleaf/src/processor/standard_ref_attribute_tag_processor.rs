use crate::TemplateMode;
use crate::element::AbstractAttributeTagProcessor;
use crate::util::Utf16String;

use super::{StandardAttributeCallback, delegate_standard_element_tag_processor};

/// 删除仅用于模板定位的 `th:ref` 标记属性。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardRefAttributeTagProcessor`。
pub struct StandardRefAttributeTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl StandardRefAttributeTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 10000;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "ref";

    /// 创建 Processor；属性移除由抽象基类在回调后完成。
    /// 对应 Java 语义：`StandardRefAttributeTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        let callback: StandardAttributeCallback =
            Box::new(|_context, _tag, _name, _value, _handler| Ok(()));
        Ok(Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(template_mode),
                dialect_prefix,
                None,
                false,
                Some(Utf16String::from_rust_str(Self::ATTR_NAME)),
                true,
                Self::PRECEDENCE,
                true,
                "org.thymeleaf.standard.processor.StandardRefAttributeTagProcessor",
                callback,
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardRefAttributeTagProcessor, processor);
