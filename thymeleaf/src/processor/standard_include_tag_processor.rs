use crate::TemplateMode;
use crate::util::JavaString;

use super::{
    AbstractStandardFragmentInsertionTagProcessor, delegate_standard_element_tag_processor,
};

/// 仅插入 Fragment 容器内容的 deprecated `th:include` Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardIncludeTagProcessor`。
pub struct StandardIncludeTagProcessor {
    processor: AbstractStandardFragmentInsertionTagProcessor,
}

#[allow(deprecated)]
impl StandardIncludeTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 100;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "include";

    /// 创建 Processor。
    /// 对应 Java 语义：`StandardIncludeTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardFragmentInsertionTagProcessor::with_insert_only_contents(
                template_mode,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                false,
                true,
                "org.thymeleaf.standard.processor.StandardIncludeTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardIncludeTagProcessor, processor);
