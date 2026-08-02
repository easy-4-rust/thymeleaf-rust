use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::{Dialect02Div2Processor, Dialect02DivProcessor, Dialect02TextProcessor};

/// 聚合测试中的第二个无前缀方言。
///
/// 对应 Java: `org.thymeleaf.templateengine.aggregation.dialect.Dialect02`。
pub struct Dialect02 {
    dialect: AbstractProcessorDialect,
}

impl Dialect02 {
    /// 创建方言 precedence 100 的 `Dialect02`。
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(Some("Dialect02"), None, 100)
                .expect("the fixed Dialect02 configuration is valid"),
        }
    }
}

impl Default for Dialect02 {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for Dialect02 {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }
    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for Dialect02 {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }
    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }
    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let mut processors = ProcessorSet::new();
        for processor in [
            Arc::new(Dialect02DivProcessor::new(dialect_prefix)) as Arc<dyn IProcessor>,
            Arc::new(Dialect02Div2Processor::new(dialect_prefix)) as Arc<dyn IProcessor>,
            Arc::new(Dialect02TextProcessor::new()) as Arc<dyn IProcessor>,
        ] {
            processors.insert(Some(processor));
        }
        Some(processors)
    }
}
