use crate::TemplateMode;
use crate::element::AbstractAttributeTagProcessor;
use crate::util::JavaString;

use super::{StandardAttributeCallback, delegate_standard_element_tag_processor};

/// 删除 Standard Dialect 的 `xmlns:prefix` 命名空间声明。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardXmlNsTagProcessor`。
pub struct StandardXmlNsTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl StandardXmlNsTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// 命名空间属性前缀。
    pub const ATTR_NAME_PREFIX: &'static str = "xmlns:";

    /// 创建 Processor；属性移除由抽象基类完成。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        let name = JavaString::from_rust_str(&format!(
            "{}{}",
            Self::ATTR_NAME_PREFIX,
            dialect_prefix
                .as_ref()
                .map_or_else(String::new, JavaString::to_string_lossy)
        ));
        let callback: StandardAttributeCallback =
            Box::new(|_context, _tag, _name, _value, _handler| Ok(()));
        Ok(Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(template_mode),
                None,
                None,
                false,
                Some(name),
                false,
                Self::PRECEDENCE,
                true,
                "org.thymeleaf.standard.processor.StandardXmlNsTagProcessor",
                callback,
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardXmlNsTagProcessor, processor);
