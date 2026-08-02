use std::sync::Arc;

use crate::TemplateMode;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::inline::{
    IInliner, NoOpInliner, StandardCSSInliner, StandardHTMLInliner, StandardInlineMode,
    StandardJavaScriptInliner, StandardTextInliner,
};
use crate::util::JavaString;

use super::{
    AbstractStandardTextInlineSettingTagProcessor, delegate_standard_element_tag_processor,
};

/// HTML 模式 `th:inline` Processor。
///
/// 对应 Java: `org.thymeleaf.standard.processor.StandardInlineHTMLTagProcessor`。
pub struct StandardInlineHTMLTagProcessor {
    processor: AbstractStandardTextInlineSettingTagProcessor,
}

impl StandardInlineHTMLTagProcessor {
    /// Java Processor precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// Standard 属性本地名称。
    pub const ATTR_NAME: &'static str = "inline";

    /// 创建 HTML 模式 `th:inline` Processor。
    /// 对应 Java 语义：`StandardInlineHTMLTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(dialect_prefix: Option<JavaString>) -> Result<Self, TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardTextInlineSettingTagProcessor::new(
                TemplateMode::HTML,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                |context, inline_mode| match inline_mode {
                    StandardInlineMode::NONE => Ok(NoOpInliner::shared()),
                    StandardInlineMode::HTML => Ok(Arc::new(StandardHTMLInliner::new(
                        context.get_configuration(),
                    ))),
                    StandardInlineMode::TEXT => Ok(Arc::new(StandardTextInliner::new(
                        context.get_configuration(),
                    ))),
                    StandardInlineMode::JAVASCRIPT => {
                        StandardJavaScriptInliner::new(context.get_configuration())
                            .map(|value| Arc::new(value) as Arc<dyn IInliner>)
                            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)
                    }
                    StandardInlineMode::CSS => StandardCSSInliner::new(context.get_configuration())
                        .map(|value| Arc::new(value) as Arc<dyn IInliner>)
                        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>),
                    StandardInlineMode::XML => Err(invalid_mode(inline_mode)),
                },
                "org.thymeleaf.standard.processor.StandardInlineHTMLTagProcessor",
            )?,
        })
    }
}

fn invalid_mode(inline_mode: StandardInlineMode) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::new(Some(format!(
        "Invalid inline mode selected: {inline_mode}. Allowed inline modes in template mode HTML are: \"HTML\", \"TEXT\", \"JAVASCRIPT\", \"CSS\" and \"NONE\""
    ))))
}

delegate_standard_element_tag_processor!(StandardInlineHTMLTagProcessor, processor);
