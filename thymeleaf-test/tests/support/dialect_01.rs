use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::{Dialect01DivProcessor, Dialect01TextProcessor};

/// 不包含 Standard Dialect 的聚合测试方言。
///
/// 对应 Java: `org.thymeleaf.templateengine.aggregation.dialect.Dialect01`。
pub struct Dialect01 {
    dialect: AbstractProcessorDialect,
}

impl Dialect01 {
    /// 创建名称、空前缀和 precedence 与 Java 完全相同的方言。
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(Some("Dialect01"), None, 100)
                .expect("the fixed test dialect configuration is valid"),
        }
    }
}

impl IDialect for Dialect01 {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for Dialect01 {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }

    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }

    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let mut processors = ProcessorSet::new();
        let div: Arc<dyn IProcessor> = Arc::new(Dialect01DivProcessor::new(dialect_prefix));
        let text: Arc<dyn IProcessor> = Arc::new(Dialect01TextProcessor::new());
        processors.insert(Some(div));
        processors.insert(Some(text));
        Some(processors)
    }
}
