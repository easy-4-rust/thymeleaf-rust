use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::{
    RemoveCDATASectionProcessor, RemoveCommentProcessor, RemoveDocTypeProcessor,
    RemoveProcessingInstructionProcessor, RemoveTextProcessor, RemoveXMLDeclarationProcessor,
};

/// 注册删除所有非保留模板事件的测试方言。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.remove.RemoveDialect`。
pub struct RemoveDialect {
    dialect: AbstractProcessorDialect,
}

impl RemoveDialect {
    /// 创建 prefix `precedence`、方言 precedence 1000 的方言。
    #[must_use]
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(Some("RemoveDialect"), Some("precedence"), 1000)
                .expect("the fixed remove dialect configuration is valid"),
        }
    }
}

impl Default for RemoveDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for RemoveDialect {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }
    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for RemoveDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }
    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }
    fn get_processors(&self, _dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let processors: Vec<Arc<dyn IProcessor>> = vec![
            Arc::new(RemoveCDATASectionProcessor::new()),
            Arc::new(RemoveCommentProcessor::new()),
            Arc::new(RemoveDocTypeProcessor::new()),
            Arc::new(RemoveProcessingInstructionProcessor::new()),
            Arc::new(RemoveTextProcessor::new()),
            Arc::new(RemoveXMLDeclarationProcessor::new()),
        ];
        let mut result = ProcessorSet::new();
        for processor in processors {
            result.insert(Some(processor));
        }
        Some(result)
    }
}
