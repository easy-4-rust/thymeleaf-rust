use std::sync::Arc;

use crate::ExecutionAttributeValue;

use super::IDialect;

/// 方言执行属性映射。
///
/// Java `Map<String,Object>` 允许 null key、null value，并且具体 Map 的遍历顺序
/// 由方言实现决定。顺序化条目向量同时保留这些边界与实现返回的迭代次序。
pub type ExecutionAttributeMap = Vec<(Option<String>, Option<Arc<ExecutionAttributeValue>>)>;

/// 向模板引擎配置贡献执行属性的方言合同。
///
/// 对应 Java: `org.thymeleaf.dialect.IExecutionAttributeDialect`。
///
/// 执行属性供处理器、表达式对象和内部服务共享；配置聚合阶段负责检查不同方言
/// 对同名属性的冲突，本接口不提前改变方言返回的 null 或条目顺序。
pub trait IExecutionAttributeDialect: IDialect {
    /// 返回此方言贡献的执行属性。
    ///
    /// 对应 Java: `IExecutionAttributeDialect#getExecutionAttributes()`。
    ///
    /// # 返回
    ///
    /// `None` 对应方言实现返回 null Map；条目中的 `None` 分别对应 null key/value。
    fn get_execution_attributes(&self) -> Option<ExecutionAttributeMap>;
}
