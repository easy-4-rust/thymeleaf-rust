use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::JavaString;

use super::{IStandardExpression, StandardExpressionResult};

/// Standard Expression 解析器合同。
///
/// 对应 Java: `org.thymeleaf.standard.expression.IStandardExpressionParser`。
///
/// 实现必须能在线程之间安全共享。
pub trait IStandardExpressionParser: Send + Sync {
    /// 解析指定输入并返回表达式对象。
    fn parse_expression(
        &self,
        context: &dyn IExpressionContext,
        input: Option<&JavaString>,
    ) -> StandardExpressionResult<Arc<dyn IStandardExpression>>;
}
