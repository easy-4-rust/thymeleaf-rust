use crate::IEngineConfiguration;
use crate::expression::IExpressionObjects;

use super::IContext;

/// 表达式求值所需的上下文合同。
///
/// 对应 Java: `org.thymeleaf.context.IExpressionContext`。
pub trait IExpressionContext: IContext {
    /// 返回当前模板引擎配置。
    fn get_configuration(&self) -> &dyn IEngineConfiguration;
    /// 返回表达式工具对象容器。
    fn get_expression_objects(&self) -> &dyn IExpressionObjects;
}
