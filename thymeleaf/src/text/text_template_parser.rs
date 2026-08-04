use std::sync::Arc;

use crate::engine::ITemplateHandler;
use crate::templateparser::{ITemplateParser, TemplateParserError};
use crate::templateresource::ITemplateResource;
use crate::util::Utf16String;
use crate::{IEngineConfiguration, TemplateMode};

use super::AbstractTextTemplateParser;

/// TEXT 模式高层模板 Parser。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.TextTemplateParser`。
pub struct TextTemplateParser {
    delegate: AbstractTextTemplateParser,
}

impl TextTemplateParser {
    /// 创建 TEXT Parser；TEXT 模式不启用 JS/CSS 注释与字面量扫描。
    /// 对应 Java 语义：`TextTemplateParser` 的 `new` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn new(buffer_pool_size: i32, buffer_size: i32, standard_dialect_present: bool) -> Self {
        Self {
            delegate: AbstractTextTemplateParser::new(
                buffer_pool_size,
                buffer_size,
                false,
                standard_dialect_present,
            ),
        }
    }
}

impl ITemplateParser for TextTemplateParser {
    fn parse_standalone(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        owner_template: Option<&Utf16String>,
        template: &Utf16String,
        template_selectors: Option<&[Utf16String]>,
        resource: Arc<dyn ITemplateResource>,
        template_mode: TemplateMode,
        use_decoupled_logic: bool,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError> {
        self.delegate.parse_standalone(
            configuration,
            owner_template,
            template,
            template_selectors,
            resource,
            template_mode,
            use_decoupled_logic,
            handler,
        )
    }

    fn parse_string(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        owner_template: &Utf16String,
        template: &Utf16String,
        line_offset: i32,
        col_offset: i32,
        template_mode: TemplateMode,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError> {
        self.delegate.parse_string(
            configuration,
            owner_template,
            template,
            line_offset,
            col_offset,
            template_mode,
            handler,
        )
    }
}
