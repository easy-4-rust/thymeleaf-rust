use crate::TemplateMode;
use crate::element::AbstractElementTagProcessor;
use crate::util::JavaString;

use super::{StandardElementCallback, delegate_standard_element_tag_processor};

/// 删除 `th:block` 包装标签但保留正文的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardBlockTagProcessor`。
pub struct StandardBlockTagProcessor {
    processor: AbstractElementTagProcessor<StandardElementCallback>,
}

impl StandardBlockTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 100000;
    /// 默认元素名。
    pub const ELEMENT_NAME: &'static str = "block";

    /// 创建 Processor。
    /// 对应 Java 语义：`StandardBlockTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
        element_name: JavaString,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        let prefix_element_name = dialect_prefix.is_some();
        let callback: StandardElementCallback = Box::new(|_context, _tag, structure_handler| {
            structure_handler.remove_tags();
            Ok(())
        });
        Ok(Self {
            processor: AbstractElementTagProcessor::new(
                Some(template_mode),
                dialect_prefix,
                Some(element_name),
                prefix_element_name,
                None,
                false,
                Self::PRECEDENCE,
                "org.thymeleaf.standard.processor.StandardBlockTagProcessor",
                callback,
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardBlockTagProcessor, processor);
