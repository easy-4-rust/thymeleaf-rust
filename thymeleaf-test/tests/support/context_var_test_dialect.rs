use std::sync::Arc;

use thymeleaf::dialect::{AbstractProcessorDialect, IDialect, IProcessorDialect};
use thymeleaf::processor::{IProcessor, ProcessorSet};

use super::{
    Attr2ModelAttributeTagProcessor, AttrModelAttributeTagProcessor, ModelAttributeTagProcessor,
    SetVarAttributeTagProcessor, WriteVarAttributeTagProcessor,
};

/// 验证标签处理全过程局部变量生命周期的测试方言。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.context.dialect.ContextVarTestDialect`。
pub struct ContextVarTestDialect {
    dialect: AbstractProcessorDialect,
}

impl ContextVarTestDialect {
    /// 创建默认 prefix `ctxvar`、方言 precedence 1000 的方言。
    pub fn new() -> Self {
        Self {
            dialect: AbstractProcessorDialect::new(
                Some("ContextVarTestDialect"),
                Some("ctxvar"),
                1000,
            )
            .expect("the fixed context-variable dialect configuration is valid"),
        }
    }
}

impl Default for ContextVarTestDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for ContextVarTestDialect {
    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }
    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IProcessorDialect for ContextVarTestDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.dialect.get_prefix()
    }
    fn get_dialect_processor_precedence(&self) -> i32 {
        self.dialect.get_dialect_processor_precedence()
    }
    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        let mut processors = ProcessorSet::new();
        for processor in [
            Arc::new(SetVarAttributeTagProcessor::new(dialect_prefix)) as Arc<dyn IProcessor>,
            Arc::new(WriteVarAttributeTagProcessor::new(dialect_prefix)) as Arc<dyn IProcessor>,
            Arc::new(ModelAttributeTagProcessor::new(dialect_prefix)) as Arc<dyn IProcessor>,
            Arc::new(AttrModelAttributeTagProcessor::new(dialect_prefix)) as Arc<dyn IProcessor>,
            Arc::new(Attr2ModelAttributeTagProcessor::new(dialect_prefix)) as Arc<dyn IProcessor>,
        ] {
            processors.insert(Some(processor));
        }
        Some(processors)
    }
}
