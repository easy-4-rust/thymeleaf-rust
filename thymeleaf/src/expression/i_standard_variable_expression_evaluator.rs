use std::sync::Arc;

use super::{
    IStandardVariableExpression, StandardExpressionExecutionContext, StandardExpressionResult,
    TemplateValue,
};
use crate::context::IExpressionContext;

/// Standard Variable Expression 求值器合同。
///
/// 对应 Java:
/// `org.thymeleaf.standard.expression.IStandardVariableExpressionEvaluator`。
pub trait IStandardVariableExpressionEvaluator: Send + Sync {
    /// 求值变量表达式并保留 Java null 与异常语义。
    fn evaluate(
        &self,
        context: &dyn IExpressionContext,
        expression: &dyn IStandardVariableExpression,
        expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>>;
}
