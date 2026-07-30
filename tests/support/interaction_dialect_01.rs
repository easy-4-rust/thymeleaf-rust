use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::{
    InteractionDialect01CDATASectionProcessor, InteractionDialect01CommentProcessor,
    InteractionDialect01TextProcessor,
};

/// 注册 HTML、JavaScript 与 CSS 三种模式交互 Processor 的测试方言。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.features.interaction.InteractionDialect01`。
pub struct InteractionDialect01 {
    dialect: AbstractProcessorDialect,
}

impl InteractionDialect01 {
    /// 创建无 prefix、方言 precedence 1000 的交互方言。
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(Some("InteractionDialect01"), None, 1000)
                .expect("the fixed interaction dialect configuration is valid"),
        }
    }
}

impl Default for InteractionDialect01 {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for InteractionDialect01 {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for InteractionDialect01 {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }

    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }

    fn get_processors(&self, _dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let mut processors = ProcessorSet::new();
        for mode in [
            TemplateMode::HTML,
            TemplateMode::JAVASCRIPT,
            TemplateMode::CSS,
        ] {
            let text: Arc<dyn IProcessor> = Arc::new(InteractionDialect01TextProcessor::new(mode));
            let cdata: Arc<dyn IProcessor> =
                Arc::new(InteractionDialect01CDATASectionProcessor::new(mode));
            let comment: Arc<dyn IProcessor> =
                Arc::new(InteractionDialect01CommentProcessor::new(mode));
            processors.insert(Some(text));
            processors.insert(Some(cdata));
            processors.insert(Some(comment));
        }
        Some(processors)
    }
}
