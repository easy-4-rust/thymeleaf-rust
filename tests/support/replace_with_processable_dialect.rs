use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::{
    ReplaceWithProcessableCDATASectionProcessor, ReplaceWithProcessableCommentProcessor,
    ReplaceWithProcessableDocTypeProcessor, ReplaceWithProcessableProcessingInstructionProcessor,
    ReplaceWithProcessableTextProcessor, ReplaceWithProcessableXMLDeclarationProcessor,
};

/// 注册继续处理替换模型的事件处理器。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.processors.dialects.replacewithprocessable.ReplaceWithProcessableDialect`。
pub struct ReplaceWithProcessableDialect {
    dialect: AbstractProcessorDialect,
}

impl ReplaceWithProcessableDialect {
    /// 创建 prefix `precedence`、方言 precedence 1000 的方言。
    #[must_use]
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(
                Some("ReplaceWithProcessableDialect"),
                Some("precedence"),
                1000,
            )
            .expect("the fixed processable replacement dialect is valid"),
        }
    }
}

impl Default for ReplaceWithProcessableDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for ReplaceWithProcessableDialect {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }
    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for ReplaceWithProcessableDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }
    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }
    fn get_processors(&self, _dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let processors: Vec<Arc<dyn IProcessor>> = vec![
            Arc::new(ReplaceWithProcessableCDATASectionProcessor::new()),
            Arc::new(ReplaceWithProcessableCommentProcessor::new()),
            Arc::new(ReplaceWithProcessableDocTypeProcessor::new()),
            Arc::new(ReplaceWithProcessableProcessingInstructionProcessor::new()),
            Arc::new(ReplaceWithProcessableTextProcessor::new()),
            Arc::new(ReplaceWithProcessableXMLDeclarationProcessor::new()),
        ];
        let mut result = ProcessorSet::new();
        for processor in processors {
            result.insert(Some(processor));
        }
        Some(result)
    }
}
