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

    /// 执行内建表达式但不应用 `Expression.execute` 的 LiteralValue 解包。
    ///
    /// 默认与公开执行一致；TextLiteral 等需要保留内部包装的对象覆盖此入口。
    fn execute_raw(
        &self,
        context: &dyn IExpressionContext,
        expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        self.execute_with_context(context, expression_context)
    }

    /// 判断字符串嵌入时是否需要 Java `ComplexExpression` 的括号。
    fn is_complex(&self) -> bool {
        false
    }

    /// 判断是否属于 Java `Token` 抽象类族。
    fn is_token_expression(&self) -> bool {
        false
    }

    /// 判断是否为 NumberTokenExpression。
    fn is_number_token_expression(&self) -> bool {
        false
    }

    /// 判断是否为 BooleanTokenExpression。
    fn is_boolean_token_expression(&self) -> bool {
        false
    }

    /// 判断是否为 GenericTokenExpression。
    fn is_generic_token_expression(&self) -> bool {
        false
    }

    /// 判断是否为 TextLiteralExpression。
    fn is_text_literal_expression(&self) -> bool {
        false
    }

    /// 判断是否为 FragmentExpression；用于禁止属性级缓存。
    fn is_fragment_expression(&self) -> bool {
        false
    }

    /// 若当前对象就是纯 FragmentExpression，则返回其动态能力。
    ///
    /// Java 使用 `instanceof FragmentExpression` 触发避免多余资源 exists 查询的
    /// 快捷路径；Rust 用该显式 capability 保留同一分派语义。
    fn as_fragment_expression(&self) -> Option<&super::FragmentExpression> {
        None
    }
}
