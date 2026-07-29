use std::error::Error;
use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::JavaString;

use super::{StandardExpressionExecutionContext, TemplateValue};

/// Standard Expression 可观察错误通道。
pub type StandardExpressionError = Box<dyn Error + Send + Sync>;

/// Standard Expression 字符串化、解析和求值结果。
pub type StandardExpressionResult<T> = Result<T, StandardExpressionError>;

/// 所有 Thymeleaf Standard Expression 的公共合同。
///
/// 对应 Java: `org.thymeleaf.standard.expression.IStandardExpression`。
pub trait IStandardExpression: Send + Sync {
    /// 返回表达式的规范 UTF-16 字符串表示。
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString>;

    /// 使用 NORMAL 执行上下文求值。
    fn execute(
        &self,
        context: &dyn IExpressionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        self.execute_with_context(context, StandardExpressionExecutionContext::NORMAL)
    }

    /// 使用指定标准执行上下文求值。
    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>>;

    /// 判断字符串嵌入时是否需要 Java `ComplexExpression` 的括号。
    fn is_complex(&self) -> bool {
        false
    }
}
