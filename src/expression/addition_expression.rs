use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::{JavaNumber, JavaString, ValidateError};

use super::{
    BinaryOperationExpression, ComplexExpression, IStandardExpression, LiteralValue,
    StandardExpressionError, StandardExpressionExecutionContext, StandardExpressionResult,
    TemplateValue, TokenError,
    binary_operation_expression::{
        evaluate_as_number, execute_raw_operands, literal_unwrapped_string, normalized_null_value,
        unwrap_literal_result,
    },
};

/// Standard Expression 加法/字符串连接表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.AdditionExpression`。
pub struct AdditionExpression {
    operation: BinaryOperationExpression,
}

impl AdditionExpression {
    /// 创建加法表达式。
    pub fn new(
        left: Option<Arc<dyn IStandardExpression>>,
        right: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<Self, ValidateError> {
        BinaryOperationExpression::new(left, right).map(|operation| Self { operation })
    }

    /// 返回左操作数。
    pub fn get_left(&self) -> &dyn IStandardExpression {
        self.operation.get_left()
    }

    /// 返回右操作数。
    pub fn get_right(&self) -> &dyn IStandardExpression {
        self.operation.get_right()
    }

    fn execute_raw_addition(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let (left, right) = execute_raw_operands(&self.operation, context, execution_context)?;
        let left = normalized_null_value(left);
        let right = normalized_null_value(right);
        if let Some(left_number) = evaluate_as_number(Some(&left))? {
            if let Some(right_number) = evaluate_as_number(Some(&right))? {
                return Ok(Some(Arc::new(TemplateValue::Number(
                    JavaNumber::BigDecimal(left_number.add_java(&right_number)),
                ))));
            }
        }
        let left = literal_unwrapped_string(left.as_ref())
            .ok_or_else(|| Box::new(TokenError::NullPointer) as StandardExpressionError)?;
        let right = literal_unwrapped_string(right.as_ref())
            .ok_or_else(|| Box::new(TokenError::NullPointer) as StandardExpressionError)?;
        let mut units = left.as_utf16().to_vec();
        units.extend_from_slice(right.as_utf16());
        Ok(Some(Arc::new(TemplateValue::Literal(Arc::new(
            LiteralValue::new(Some(JavaString::from_utf16(units))),
        )))))
    }
}

impl IStandardExpression for AdditionExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        self.operation
            .get_string_representation(Some(&JavaString::from_rust_str("+")))
    }

    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        Ok(unwrap_literal_result(
            self.execute_raw_addition(context, execution_context)?,
        ))
    }

    fn execute_raw(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        self.execute_raw_addition(context, execution_context)
    }

    fn is_complex(&self) -> bool {
        true
    }
}

impl ComplexExpression for AdditionExpression {}
impl super::AdditionSubtractionExpression for AdditionExpression {}
