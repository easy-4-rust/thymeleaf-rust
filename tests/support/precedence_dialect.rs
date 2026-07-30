use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::PrecedenceModifyLocalVariableModelProcessor;

/// 验证方言与 Processor precedence 组合排序的测试方言。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.elementprocessors.dialect.PrecedenceDialect`。
pub struct PrecedenceDialect {
    dialect: AbstractProcessorDialect,
}

impl PrecedenceDialect {
    /// 创建指定方言 precedence 的 `precedence` 方言。
    pub fn new(precedence: i32) -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(
                Some("PrecedenceDialect"),
                Some("precedence"),
                precedence,
            )
            .expect("the fixed precedence dialect configuration is valid"),
        }
    }
}

impl IDialect for PrecedenceDialect {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for PrecedenceDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }

    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }

    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let mut processors = ProcessorSet::new();
        let processor: Arc<dyn IProcessor> = Arc::new(
            PrecedenceModifyLocalVariableModelProcessor::new(dialect_prefix),
        );
        processors.insert(Some(processor));
        Some(processors)
    }
}
