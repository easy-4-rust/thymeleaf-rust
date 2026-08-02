use std::sync::Arc;

use crate::engine::ITemplateHandler;
use crate::templateparser::{ITemplateParser, TemplateParserError};
use crate::templateresource::ITemplateResource;
use crate::util::JavaString;
use crate::{IEngineConfiguration, TemplateMode};

use super::AbstractMarkupTemplateParser;

/// HTML 模式模板 parser。
///
/// 对应 Java: `org.thymeleaf.templateparser.markup.HTMLTemplateParser`。
pub struct HTMLTemplateParser {
    delegate: AbstractMarkupTemplateParser,
}

impl HTMLTemplateParser {
    /// 创建 HTML parser，并保留 Java 的 buffer pool 参数合同。
    #[must_use]
    pub const fn new(buffer_pool_size: i32, buffer_size: i32) -> Self {
        Self {
            delegate: AbstractMarkupTemplateParser::new(true, buffer_pool_size, buffer_size),
        }
    }
}

impl ITemplateParser for HTMLTemplateParser {
    fn parse_standalone(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        owner_template: Option<&JavaString>,
        template: &JavaString,
        template_selectors: Option<&[JavaString]>,
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
        owner_template: &JavaString,
        template: &JavaString,
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
