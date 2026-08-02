use crate::TemplateMode;
use crate::util::JavaString;

use super::{
    AbstractStandardFragmentInsertionTagProcessor, delegate_standard_element_tag_processor,
};

/// 将 Fragment 模型插入宿主正文的 `th:insert` Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardInsertTagProcessor`。
pub struct StandardInsertTagProcessor {
    processor: AbstractStandardFragmentInsertionTagProcessor,
}

impl StandardInsertTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 100;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "insert";

    /// 创建 Processor。
    /// 对应 Java 语义：`StandardInsertTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardFragmentInsertionTagProcessor::new(
                template_mode,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                false,
                "org.thymeleaf.standard.processor.StandardInsertTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardInsertTagProcessor, processor);
