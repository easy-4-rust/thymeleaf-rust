use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::{ElementStackAttrProcessor, ElementStackModelProcessor, ElementStackTextProcessor};

/// 暴露元素栈的测试方言。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.features.elementstack.ElementStackDialect`。
pub struct ElementStackDialect {
    dialect: AbstractProcessorDialect,
}

impl ElementStackDialect {
    /// 创建与 Standard Dialect 使用相同方言 precedence 的 `stack` 方言。
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(
                Some("ElementStackDialect"),
                Some("stack"),
                1000,
            )
            .expect("the fixed test dialect configuration is valid"),
        }
    }
}

impl IDialect for ElementStackDialect {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for ElementStackDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }

    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }

    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let mut processors = ProcessorSet::new();
        let attr: Arc<dyn IProcessor> = Arc::new(ElementStackAttrProcessor::new(dialect_prefix));
        let text: Arc<dyn IProcessor> = Arc::new(ElementStackTextProcessor::new(dialect_prefix));
        let model: Arc<dyn IProcessor> = Arc::new(ElementStackModelProcessor::new(dialect_prefix));
        processors.insert(Some(attr));
        processors.insert(Some(text));
        processors.insert(Some(model));
        Some(processors)
    }
}
