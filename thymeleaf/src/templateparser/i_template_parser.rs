use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::TemplateMode;
use crate::engine::ITemplateHandler;
use crate::exceptions::TemplateInputException;
use crate::templateresource::ITemplateResource;
use crate::util::JavaString;
use thiserror::Error;

/// 高层模板 Parser 的未检查参数错误或模板输入错误。
#[derive(Debug, Error)]
/// 对应 Java 语义：`ITemplateParser` 的 Rust 侧类型 `TemplateParserError`。
pub enum TemplateParserError {
    /// Java `IllegalArgumentException`。
    #[error("{message}")]
    IllegalArgument {
        /// 与上游校验一致的消息。
        message: String,
    },
    /// Java `TemplateInputException`。
    #[error(transparent)]
    Input(#[from] TemplateInputException),
}

/// TemplateEngine 使用的模板 Parser 合同。
///
/// 实现必须可安全共享；每次解析把事件同步交给独立 Handler 链。对应 Java:
/// `org.thymeleaf.templateparser.ITemplateParser`。
pub trait ITemplateParser: Send + Sync {
    /// 解析独立模板资源。
    ///
    /// # 参数
    ///
    /// - `configuration`：引擎配置。
    /// - `owner_template`：可空拥有者模板。
    /// - `template`：当前模板名。
    /// - `template_selectors`：可空选择器集合。
    /// - `resource`：模板资源。
    /// - `template_mode`：解析模式。
    /// - `use_decoupled_logic`：是否启用解耦逻辑。
    /// - `handler`：最终 Handler 链。
    ///
    /// 对应 Java: `ITemplateParser#parseStandalone`。
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<(), TemplateParserError>;

    /// 解析嵌入当前模板的字符串。
    ///
    /// 对应 Java: `ITemplateParser#parseString`。
    #[allow(clippy::too_many_arguments)]
    fn parse_string(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        owner_template: &JavaString,
        template: &JavaString,
        line_offset: i32,
        col_offset: i32,
        template_mode: TemplateMode,
        handler: Box<dyn ITemplateHandler>,
    ) -> Result<(), TemplateParserError>;
}
