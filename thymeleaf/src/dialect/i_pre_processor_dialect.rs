use std::sync::Arc;

use crate::preprocessor::IPreProcessor;

use super::IDialect;

/// Java `Set<IPreProcessor>` 的顺序化、可空元素表示。
pub type PreProcessorSet = Vec<Option<Arc<dyn IPreProcessor>>>;

/// 提供模板预处理器的方言合同。
///
/// 对应 Java: `org.thymeleaf.dialect.IPreProcessorDialect`。
pub trait IPreProcessorDialect: IDialect {
    /// 返回方言级预处理器优先级。
    ///
    /// 对应 Java: `IPreProcessorDialect#getDialectPreProcessorPrecedence()`。
    fn get_dialect_pre_processor_precedence(&self) -> i32;

    /// 返回该方言声明的预处理器集合快照。
    ///
    /// 对应 Java: `IPreProcessorDialect#getPreProcessors()`。
    ///
    /// `None` 对应 Java `null` Set；集合中的 `None` 对应被聚合器拒绝的 null 元素。
    fn get_pre_processors(&self) -> Option<PreProcessorSet>;
}
