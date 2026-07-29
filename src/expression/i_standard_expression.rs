use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::util::JavaString;

use super::{StandardExpressionExecutionContext, TemplateValue};

/// 所有 Thymeleaf Standard Expression 的公共合同。
///
/// 对应 Java: `org.thymeleaf.standard.expression.IStandardExpression`。
pub trait IStandardExpression: Send + Sync {
    /// 返回表达式的规范 UTF-16 字符串表示。
    fn get_string_representation(&self) -> JavaString;

    /// 使用 NORMAL 执行上下文求值。
    fn execute(
        &self,
        context: &dyn IExpressionContext,
    ) -> Result<Option<Arc<TemplateValue>>, TemplateProcessingException> {
        self.execute_with_context(context, StandardExpressionExecutionContext::NORMAL)
    }

    /// 使用指定标准执行上下文求值。
    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        expression_context: &'static StandardExpressionExecutionContext,
    ) -> Result<Option<Arc<TemplateValue>>, TemplateProcessingException>;

    /// 判断字符串嵌入时是否需要 Java `ComplexExpression` 的括号。
    fn is_complex(&self) -> bool {
        false
    }
}
