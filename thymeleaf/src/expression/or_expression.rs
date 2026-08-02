use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::{JavaString, ValidateError};

use super::{
    BinaryOperationExpression, ComplexExpression, IStandardExpression,
    StandardExpressionExecutionContext, StandardExpressionResult, TemplateValue,
    binary_operation_expression::evaluate_as_boolean,
};

/// Standard Expression 短路逻辑 OR。
///
/// 对应 Java: `org.thymeleaf.standard.expression.OrExpression`。
pub struct OrExpression {
    operation: BinaryOperationExpression,
}

impl OrExpression {
    /// 创建 OR 表达式。
    /// 对应 Java 语义：`OrExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        left: Option<Arc<dyn IStandardExpression>>,
        right: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<Self, ValidateError> {
        BinaryOperationExpression::new(left, right).map(|operation| Self { operation })
    }
    /// 返回左操作数。
    /// 对应 Java 语义：Java 接口/超类方法 `getLeft()` 的 Rust 移植（`OrExpression` 继承路径）。
    pub fn get_left(&self) -> &dyn IStandardExpression {
        self.operation.get_left()
    }
    /// 返回右操作数。
    /// 对应 Java 语义：Java 接口/超类方法 `getRight()` 的 Rust 移植（`OrExpression` 继承路径）。
    pub fn get_right(&self) -> &dyn IStandardExpression {
        self.operation.get_right()
    }
    /// 判断解析阶段是否允许该操作数。
    #[expect(dead_code, reason = "将在后续 ExpressionParsingUtil 组合阶段调用")]
    /// 对应 Java 语义：`OrExpression` 的 `is_operand_allowed` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn is_operand_allowed(operand: Option<&dyn IStandardExpression>) -> bool {
        operand.is_some_and(|operand| {
            !operand.is_token_expression() || operand.is_boolean_token_expression()
        })
    }
}

impl IStandardExpression for OrExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        self.operation
            .get_string_representation(Some(&JavaString::from_rust_str("or")))
    }
    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let left = self
            .operation
            .get_left()
            .execute_with_context(context, execution_context)?;
        if evaluate_as_boolean(left.as_ref())? {
            return Ok(Some(Arc::new(TemplateValue::Boolean(true))));
        }
        let right = self
            .operation
            .get_right()
            .execute_with_context(context, execution_context)?;
        Ok(Some(Arc::new(TemplateValue::Boolean(evaluate_as_boolean(
            right.as_ref(),
        )?))))
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl ComplexExpression for OrExpression {}
