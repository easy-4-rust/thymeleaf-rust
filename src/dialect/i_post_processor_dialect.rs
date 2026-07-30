use std::sync::Arc;

use crate::postprocessor::IPostProcessor;

use super::IDialect;

/// 提供模板后处理器的方言合同。
///
/// 对应 Java: `org.thymeleaf.dialect.IPostProcessorDialect`。
pub trait IPostProcessorDialect: IDialect {
    /// 返回方言级后处理器优先级。
    fn get_dialect_post_processor_precedence(&self) -> i32;

    /// 返回该方言声明的后处理器集合快照。
    fn get_post_processors(&self) -> Vec<Arc<dyn IPostProcessor>>;
}
