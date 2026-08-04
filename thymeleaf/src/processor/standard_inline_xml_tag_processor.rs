use std::sync::Arc;

use crate::TemplateMode;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::inline::{NoOpInliner, StandardInlineMode, StandardTextInliner, StandardXMLInliner};
use crate::util::Utf16String;

use super::{
    AbstractStandardTextInlineSettingTagProcessor, delegate_standard_element_tag_processor,
};

/// XML 模式 `th:inline` Processor。
///
/// 对应 Java: `org.thymeleaf.standard.processor.StandardInlineXMLTagProcessor`。
pub struct StandardInlineXMLTagProcessor {
    processor: AbstractStandardTextInlineSettingTagProcessor,
}

impl StandardInlineXMLTagProcessor {
    /// Java Processor precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// Standard 属性本地名称。
    pub const ATTR_NAME: &'static str = "inline";

    /// 创建 XML 模式 `th:inline` Processor。
    /// 对应 Java 语义：`StandardInlineXMLTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(dialect_prefix: Option<Utf16String>) -> Result<Self, TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardTextInlineSettingTagProcessor::new(
                TemplateMode::XML,
                dialect_prefix,
                Utf16String::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                |context, inline_mode| match inline_mode {
                    StandardInlineMode::NONE => Ok(NoOpInliner::shared()),
                    StandardInlineMode::XML => Ok(Arc::new(StandardXMLInliner::new(
                        context.get_configuration(),
                    ))),
                    StandardInlineMode::TEXT => Ok(Arc::new(StandardTextInliner::new(
                        context.get_configuration(),
                    ))),
                    _ => Err(invalid_mode(inline_mode)),
                },
                "org.thymeleaf.standard.processor.StandardInlineXMLTagProcessor",
            )?,
        })
    }
}

fn invalid_mode(inline_mode: StandardInlineMode) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::new(Some(format!(
        "Invalid inline mode selected: {inline_mode}. Allowed inline modes in template mode XML are: \"XML\", \"TEXT\", \"NONE\""
    ))))
}

delegate_standard_element_tag_processor!(StandardInlineXMLTagProcessor, processor);
