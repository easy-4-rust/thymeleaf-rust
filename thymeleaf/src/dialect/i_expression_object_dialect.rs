use std::sync::Arc;

use crate::expression::IExpressionObjectFactory;

use super::IDialect;

/// 提供表达式对象工厂的方言合同。
///
/// 对应 Java: `org.thymeleaf.dialect.IExpressionObjectDialect`。
pub trait IExpressionObjectDialect: IDialect {
    /// 返回该方言唯一、可共享的表达式对象工厂。
    ///
    /// 对应 Java: `IExpressionObjectDialect#getExpressionObjectFactory()`。
    ///
    /// # 返回
    ///
    /// `None` 对应第三方 Java 方言返回 `null`；配置聚合阶段会忽略该贡献。
    fn get_expression_object_factory(&self) -> Option<Arc<dyn IExpressionObjectFactory>>;
}
