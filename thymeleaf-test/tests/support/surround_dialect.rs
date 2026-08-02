use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::SurroundProcessor;

/// 注册元素模型前后环绕注释的测试方言。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.surround.SurroundDialect`。
pub struct SurroundDialect {
    dialect: AbstractProcessorDialect,
}

impl SurroundDialect {
    /// 创建 prefix `surround`、方言 precedence 1000 的方言。
    #[must_use]
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(Some("SurroundDialect"), Some("surround"), 1000)
                .expect("the fixed surround dialect configuration is valid"),
        }
    }
}

impl Default for SurroundDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for SurroundDialect {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }
    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for SurroundDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }
    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }
    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let mut processors = ProcessorSet::new();
        processors.insert(Some(
            Arc::new(SurroundProcessor::new(dialect_prefix)) as Arc<dyn IProcessor>
        ));
        Some(processors)
    }
}
