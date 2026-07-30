use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::AddContextVariableElementProcessor;

/// 注册 Web 上下文分层写入验证处理器。
///
/// 对应 Java: `org.thymeleaf.templateengine.context.dialect.ContextDialect`。
pub struct ContextDialect {
    dialect: AbstractProcessorDialect,
}

impl ContextDialect {
    /// 创建 prefix `context`、方言 precedence 100 的方言。
    #[must_use]
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(Some("ContextDialect"), Some("context"), 100)
                .expect("the fixed context dialect configuration is valid"),
        }
    }
}

impl Default for ContextDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for ContextDialect {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }
    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for ContextDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }
    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }
    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let mut processors = ProcessorSet::new();
        processors.insert(Some(
            Arc::new(AddContextVariableElementProcessor::new(dialect_prefix))
                as Arc<dyn IProcessor>,
        ));
        Some(processors)
    }
}
