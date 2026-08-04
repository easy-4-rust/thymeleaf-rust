use crate::TemplateMode;
use crate::util::Utf16String;

use super::{AbstractStandardAssertionTagProcessor, delegate_standard_element_tag_processor};

/// `th:assert` 表达式序列断言 Processor。
///
/// 对应 Java: `org.thymeleaf.standard.processor.StandardAssertTagProcessor`。
pub struct StandardAssertTagProcessor {
    processor: AbstractStandardAssertionTagProcessor,
}

impl StandardAssertTagProcessor {
    /// Java Processor precedence。
    pub const PRECEDENCE: i32 = 1550;
    /// Standard 属性本地名称。
    pub const ATTR_NAME: &'static str = "assert";

    /// 创建指定模板模式和方言前缀的 `th:assert` Processor。
    /// 对应 Java 语义：`StandardAssertTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardAssertionTagProcessor::new(
                template_mode,
                dialect_prefix,
                Utf16String::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                "org.thymeleaf.standard.processor.StandardAssertTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardAssertTagProcessor, processor);
