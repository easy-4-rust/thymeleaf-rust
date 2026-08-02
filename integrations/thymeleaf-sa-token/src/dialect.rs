//! Sa-Token 安全方言 —— `sec` 前缀的认证/授权模板方言。
//!
//! 对应 Java `thymeleaf-extras-springsecurity6` 的 `SpringSecurityDialect`：
//! - 处理器：`sec:authorize`、`sec:authentication`
//! - 表达式对象：`#authentication`、`#authorization`
//!
//! 未实现（登记 NOT_APPLICABLE，见 crate 文档）：
//! - `sec:authorize-acl` / `sec:authorize-url`（依赖 Spring ACL/URL 域对象）
//! - `sec:authorize-exprs`（Java 2.3 遗留语法）

use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::dialect::{
    AbstractProcessorDialect, IDialect, IExpressionObjectDialect, IProcessorDialect,
};
use thymeleaf::exceptions::TemplateProcessingException;
use thymeleaf::expression::IExpressionObjectFactory;
use thymeleaf::processor::{IProcessor, ProcessorSet};
use thymeleaf::util::JavaString;

use crate::expression_object::SaTokenExpressionObjectFactory;
use crate::processor::{SecAuthenticationTagProcessor, SecAuthorizeTagProcessor};

/// Sa-Token 安全方言名称。
pub const DIALECT_NAME: &str = "SaTokenSecurity";
/// Sa-Token 安全方言默认前缀。
pub const DIALECT_PREFIX: &str = "sec";
/// 跨方言 Processor precedence（Java `SpringSecurityDialect`：1000）。
pub const PROCESSOR_PRECEDENCE: i32 = 1000;

/// `sec` 前缀的安全方言。
pub struct SaTokenDialect {
    base: AbstractProcessorDialect,
    expression_object_factory: Arc<dyn IExpressionObjectFactory>,
}

impl SaTokenDialect {
    /// 创建使用默认 `sec` 前缀的安全方言。
    #[must_use]
    pub fn new() -> Self {
        Self::with_prefix(Some(JavaString::from_rust_str(DIALECT_PREFIX)))
    }

    /// 创建使用指定前缀的安全方言。
    #[must_use]
    pub fn with_prefix(dialect_prefix: Option<JavaString>) -> Self {
        Self {
            base: AbstractProcessorDialect::new(
                Some(DIALECT_NAME),
                dialect_prefix
                    .as_ref()
                    .map(JavaString::to_string_lossy)
                    .as_deref(),
                PROCESSOR_PRECEDENCE,
            )
            .expect("Sa-Token dialect constants are non-null"),
            expression_object_factory: Arc::new(SaTokenExpressionObjectFactory::new()),
        }
    }

    /// 创建指定实际前缀对应的 `sec` 处理器集合。
    pub fn create_processors_set(
        dialect_prefix: Option<&str>,
    ) -> Result<ProcessorSet, TemplateProcessingException> {
        let prefix = dialect_prefix.map(JavaString::from_rust_str);
        let mut processors = ProcessorSet::new();
        for mode in [
            TemplateMode::HTML,
            TemplateMode::XML,
            TemplateMode::TEXT,
            TemplateMode::JAVASCRIPT,
            TemplateMode::CSS,
        ] {
            insert(
                &mut processors,
                SecAuthorizeTagProcessor::new(mode, prefix.clone())?,
            );
            insert(
                &mut processors,
                SecAuthenticationTagProcessor::new(mode, prefix.clone())?,
            );
        }
        Ok(processors)
    }
}

impl Default for SaTokenDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for SaTokenDialect {
    fn java_class_name(&self) -> &'static str {
        "org.thymeleaf.extras.springsecurity6.dialect.SpringSecurityDialect"
    }

    fn is_standard_dialect(&self) -> bool {
        false
    }

    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }

    fn as_expression_object_dialect(&self) -> Option<&dyn IExpressionObjectDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some(self.base.get_name())
    }
}

impl IProcessorDialect for SaTokenDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.base.get_prefix()
    }

    fn get_dialect_processor_precedence(&self) -> i32 {
        self.base.get_dialect_processor_precedence()
    }

    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        Some(
            Self::create_processors_set(dialect_prefix).unwrap_or_else(|error| {
                panic!("Could not create Sa-Token security processors: {error}")
            }),
        )
    }
}

impl IExpressionObjectDialect for SaTokenDialect {
    fn get_expression_object_factory(&self) -> Option<Arc<dyn IExpressionObjectFactory>> {
        Some(Arc::clone(&self.expression_object_factory))
    }
}

fn insert<P>(processors: &mut ProcessorSet, processor: P)
where
    P: IProcessor + 'static,
{
    let processor: Arc<dyn IProcessor> = Arc::new(processor);
    processors.insert(Some(processor));
}
