use crate::TemplateMode;
use crate::util::Utf16String;

use super::{
    AbstractStandardFragmentInsertionTagProcessor, delegate_standard_element_tag_processor,
};

/// 使用 Fragment 模型替换宿主元素的 `th:replace` Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardReplaceTagProcessor`。
pub struct StandardReplaceTagProcessor {
    processor: AbstractStandardFragmentInsertionTagProcessor,
}

impl StandardReplaceTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 100;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "replace";

    /// 创建 Processor。
    /// 对应 Java 语义：`StandardReplaceTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardFragmentInsertionTagProcessor::new(
                template_mode,
                dialect_prefix,
                Utf16String::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                true,
                "org.thymeleaf.standard.processor.StandardReplaceTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardReplaceTagProcessor, processor);
