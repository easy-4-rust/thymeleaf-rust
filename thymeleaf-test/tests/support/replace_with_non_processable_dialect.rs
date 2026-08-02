use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::{
    ReplaceWithNonProcessableCDATASectionProcessor, ReplaceWithNonProcessableCommentProcessor,
    ReplaceWithNonProcessableDocTypeProcessor,
    ReplaceWithNonProcessableProcessingInstructionProcessor,
    ReplaceWithNonProcessableTextProcessor, ReplaceWithNonProcessableXMLDeclarationProcessor,
};

/// 注册不可继续处理的事件替换处理器。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.replacewithnonprocessable.ReplaceWithNonProcessableDialect`。
pub struct ReplaceWithNonProcessableDialect {
    dialect: AbstractProcessorDialect,
}

impl ReplaceWithNonProcessableDialect {
    /// 创建 prefix `precedence`、方言 precedence 1000 的方言。
    #[must_use]
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(
                Some("ReplaceWithNonProcessableDialect"),
                Some("precedence"),
                1000,
            )
            .expect("the fixed non-processable replacement dialect is valid"),
        }
    }
}

impl Default for ReplaceWithNonProcessableDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for ReplaceWithNonProcessableDialect {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }
    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for ReplaceWithNonProcessableDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }
    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }
    fn get_processors(&self, _dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let processors: Vec<Arc<dyn IProcessor>> = vec![
            Arc::new(ReplaceWithNonProcessableCDATASectionProcessor::new()),
            Arc::new(ReplaceWithNonProcessableCommentProcessor::new()),
            Arc::new(ReplaceWithNonProcessableDocTypeProcessor::new()),
            Arc::new(ReplaceWithNonProcessableProcessingInstructionProcessor::new()),
            Arc::new(ReplaceWithNonProcessableTextProcessor::new()),
            Arc::new(ReplaceWithNonProcessableXMLDeclarationProcessor::new()),
        ];
        let mut result = ProcessorSet::new();
        for processor in processors {
            result.insert(Some(processor));
        }
        Some(result)
    }
}
