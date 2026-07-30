use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::{
    NoOp2AttributeTagProcessor, NoOp2ModelProcessor, NoOpAttributeTagProcessor, NoOpModelProcessor,
    NoOpTextProcessor,
};

/// 注册局部变量传播验证 Processor 的 NoOp 测试方言。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.noop.NoOpDialect`。
pub struct NoOpDialect {
    dialect: AbstractProcessorDialect,
}

impl NoOpDialect {
    /// 创建默认 prefix `noop`、方言 precedence 1000 的测试方言。
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(Some("NoOpDialect"), Some("noop"), 1000)
                .expect("the fixed no-op dialect configuration is valid"),
        }
    }
}

impl Default for NoOpDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for NoOpDialect {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }
    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for NoOpDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }
    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }
    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let processors: Vec<Arc<dyn IProcessor>> = vec![
            Arc::new(NoOpModelProcessor::new(dialect_prefix)),
            Arc::new(NoOpAttributeTagProcessor::new(dialect_prefix)),
            Arc::new(NoOp2ModelProcessor::new(dialect_prefix)),
            Arc::new(NoOp2AttributeTagProcessor::new(dialect_prefix)),
            Arc::new(NoOpTextProcessor::new()),
        ];
        let mut processor_set = ProcessorSet::new();
        for processor in processors {
            processor_set.insert(Some(processor));
        }
        Some(processor_set)
    }
}
