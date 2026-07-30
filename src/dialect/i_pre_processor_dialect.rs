use std::sync::Arc;

use crate::preprocessor::IPreProcessor;

use super::IDialect;

/// 提供模板预处理器的方言合同。
///
/// 对应 Java: `org.thymeleaf.dialect.IPreProcessorDialect`。
pub trait IPreProcessorDialect: IDialect {
    /// 返回方言级预处理器优先级。
    fn get_dialect_pre_processor_precedence(&self) -> i32;

    /// 返回该方言声明的预处理器集合快照。
    fn get_pre_processors(&self) -> Vec<Arc<dyn IPreProcessor>>;
}
