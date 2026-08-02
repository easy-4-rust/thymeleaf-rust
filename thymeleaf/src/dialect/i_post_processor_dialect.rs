use std::sync::Arc;

use crate::postprocessor::IPostProcessor;

use super::IDialect;

/// Java `Set<IPostProcessor>` 的顺序化、可空元素表示。
pub type PostProcessorSet = Vec<Option<Arc<dyn IPostProcessor>>>;

/// 提供模板后处理器的方言合同。
///
/// 对应 Java: `org.thymeleaf.dialect.IPostProcessorDialect`。
pub trait IPostProcessorDialect: IDialect {
    /// 返回方言级后处理器优先级。
    ///
    /// 对应 Java: `IPostProcessorDialect#getDialectPostProcessorPrecedence()`。
    fn get_dialect_post_processor_precedence(&self) -> i32;

    /// 返回该方言声明的后处理器集合快照。
    ///
    /// 对应 Java: `IPostProcessorDialect#getPostProcessors()`。
    ///
    /// `None` 对应 Java `null` Set；集合中的 `None` 对应被聚合器拒绝的 null 元素。
    fn get_post_processors(&self) -> Option<PostProcessorSet>;
}
