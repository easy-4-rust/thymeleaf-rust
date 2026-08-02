use std::sync::Arc;

use crate::TemplateMode;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::inline::{
    IInliner, NoOpInliner, StandardCSSInliner, StandardInlineMode, StandardJavaScriptInliner,
    StandardTextInliner,
};
use crate::util::JavaString;

use super::{
    AbstractStandardTextInlineSettingTagProcessor, delegate_standard_element_tag_processor,
};

/// TEXT、JAVASCRIPT 或 CSS 模式的 `th:inline` Processor。
///
/// 对应 Java: `org.thymeleaf.standard.processor.StandardInlineTextualTagProcessor`。
pub struct StandardInlineTextualTagProcessor {
    processor: AbstractStandardTextInlineSettingTagProcessor,
}

impl StandardInlineTextualTagProcessor {
    /// Java Processor precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// Standard 属性本地名称。
    pub const ATTR_NAME: &'static str = "inline";

    /// 创建文本模板模式 `th:inline` Processor。
    /// 对应 Java 语义：`StandardInlineTextualTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, TemplateProcessingException> {
        if !template_mode.is_text() {
            return Err(TemplateProcessingException::new(Some(
                "Template mode must be a textual one".to_owned(),
            )));
        }
        Ok(Self {
            processor: AbstractStandardTextInlineSettingTagProcessor::new(
                template_mode,
                dialect_prefix,
                JavaString::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                move |context, inline_mode| match inline_mode {
                    StandardInlineMode::NONE => Ok(NoOpInliner::shared()),
                    StandardInlineMode::TEXT if template_mode == TemplateMode::TEXT => Ok(
                        Arc::new(StandardTextInliner::new(context.get_configuration())),
                    ),
                    StandardInlineMode::JAVASCRIPT if template_mode == TemplateMode::JAVASCRIPT => {
                        StandardJavaScriptInliner::new(context.get_configuration())
                            .map(|value| Arc::new(value) as Arc<dyn IInliner>)
                            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)
                    }
                    StandardInlineMode::CSS if template_mode == TemplateMode::CSS => {
                        StandardCSSInliner::new(context.get_configuration())
                            .map(|value| Arc::new(value) as Arc<dyn IInliner>)
                            .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)
                    }
                    _ => Err(invalid_mode(template_mode, inline_mode)),
                },
                "org.thymeleaf.standard.processor.StandardInlineTextualTagProcessor",
            )?,
        })
    }
}

fn invalid_mode(
    template_mode: TemplateMode,
    inline_mode: StandardInlineMode,
) -> Box<dyn TemplateEngineException> {
    Box::new(TemplateProcessingException::new(Some(format!(
        "Invalid inline mode selected: {inline_mode}. Allowed inline modes in template mode {template_mode} are: \"{template_mode}\" and \"NONE\""
    ))))
}

delegate_standard_element_tag_processor!(StandardInlineTextualTagProcessor, processor);
