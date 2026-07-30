use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::{
    MarkupAddLocalVariableModelProcessor, MarkupDoNothingModelProcessor,
    MarkupPrintAfterElementModelProcessor, MarkupPrintBeforeElementModelProcessor,
    MarkupReplaceBodyElementModelProcessor, MarkupReplaceElementModelProcessor,
    MarkupSetTextInlinerModelProcessor,
};

/// 注册完整模型聚合、替换、快照和上下文操作的测试方言。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.elementprocessors.dialect.MarkupDialect`。
pub struct MarkupDialect {
    dialect: AbstractProcessorDialect,
}

impl MarkupDialect {
    /// 创建默认 prefix `markup`、方言 precedence 100 的方言。
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(Some("MarkupDialect"), Some("markup"), 100)
                .expect("the fixed markup dialect configuration is valid"),
        }
    }
}

impl Default for MarkupDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for MarkupDialect {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }
    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for MarkupDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }
    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }
    fn get_processors(&self, _dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        // Java fixture intentionally uses the declared PREFIX instead of the supplied argument.
        let prefix = Some("markup");
        let mut processors = ProcessorSet::new();
        for processor in [
            Arc::new(MarkupPrintBeforeElementModelProcessor::new(prefix)) as Arc<dyn IProcessor>,
            Arc::new(MarkupPrintAfterElementModelProcessor::new(prefix)) as Arc<dyn IProcessor>,
            Arc::new(MarkupReplaceElementModelProcessor::new(prefix)) as Arc<dyn IProcessor>,
            Arc::new(MarkupReplaceBodyElementModelProcessor::new(prefix)) as Arc<dyn IProcessor>,
            Arc::new(MarkupAddLocalVariableModelProcessor::new(prefix)) as Arc<dyn IProcessor>,
            Arc::new(MarkupSetTextInlinerModelProcessor::new(prefix)) as Arc<dyn IProcessor>,
            Arc::new(MarkupDoNothingModelProcessor::new(prefix)) as Arc<dyn IProcessor>,
        ] {
            processors.insert(Some(processor));
        }
        Some(processors)
    }
}
