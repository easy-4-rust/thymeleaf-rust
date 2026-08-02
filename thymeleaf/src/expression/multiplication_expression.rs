use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::util::{JavaNumber, JavaString, ValidateError};

use super::{
    BinaryOperationExpression, ComplexExpression, IStandardExpression,
    StandardExpressionExecutionContext, StandardExpressionResult, TemplateValue,
    binary_operation_expression::{
        evaluate_as_number, execute_operands, literal_unwrapped_string, normalized_null_value,
    },
};

/// Standard Expression 乘法表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.MultiplicationExpression`。
pub struct MultiplicationExpression {
    operation: BinaryOperationExpression,
}

impl MultiplicationExpression {
    /// 创建乘法表达式。
    /// 对应 Java 语义：`MultiplicationExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        left: Option<Arc<dyn IStandardExpression>>,
        right: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<Self, ValidateError> {
        BinaryOperationExpression::new(left, right).map(|operation| Self { operation })
    }
    /// 返回左操作数。
    /// 对应 Java 语义：Java 接口/超类方法 `getLeft()` 的 Rust 移植（`MultiplicationExpression` 继承路径）。
    pub fn get_left(&self) -> &dyn IStandardExpression {
        self.operation.get_left()
    }
    /// 返回右操作数。
    /// 对应 Java 语义：Java 接口/超类方法 `getRight()` 的 Rust 移植（`MultiplicationExpression` 继承路径）。
    pub fn get_right(&self) -> &dyn IStandardExpression {
        self.operation.get_right()
    }
}

impl IStandardExpression for MultiplicationExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        self.operation
            .get_string_representation(Some(&JavaString::from_rust_str("*")))
    }

    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let (left, right) = execute_operands(&self.operation, context, execution_context)?;
        let left = normalized_null_value(left);
        let right = normalized_null_value(right);
        if let (Some(left_number), Some(right_number)) = (
            evaluate_as_number(Some(&left))?,
            evaluate_as_number(Some(&right))?,
        ) {
            let result = left_number.multiply_java(&right_number)?;
            return Ok(Some(Arc::new(TemplateValue::Number(
                JavaNumber::BigDecimal(result),
            ))));
        }
        Err(Box::new(TemplateProcessingException::new(Some(format!(
            "Cannot execute multiplication: operands are \"{}\" and \"{}\"",
            display_unwrapped(left.as_ref()),
            display_unwrapped(right.as_ref())
        )))))
    }

    fn is_complex(&self) -> bool {
        true
    }
}

impl ComplexExpression for MultiplicationExpression {}
impl super::MultiplicationDivisionRemainderExpression for MultiplicationExpression {}

fn display_unwrapped(value: &TemplateValue) -> String {
    literal_unwrapped_string(value)
        .unwrap_or_else(|| JavaString::from_rust_str("null"))
        .to_string_lossy()
}
