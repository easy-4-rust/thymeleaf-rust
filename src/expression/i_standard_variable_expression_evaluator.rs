use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;

use super::{IStandardVariableExpression, StandardExpressionExecutionContext, TemplateValue};

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
    ) -> Result<Option<Arc<TemplateValue>>, TemplateProcessingException>;
}
