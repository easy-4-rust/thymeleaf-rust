use std::cmp::Ordering;
use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::{JavaString, ValidateError};

use super::{
    BinaryOperationExpression, ComplexExpression, IStandardExpression,
    StandardExpressionExecutionContext, StandardExpressionResult, TemplateValue,
    binary_operation_expression::{collapse_java_null, compare_java_values, execute_operands},
    greater_than_expression::{comparison_null_error, operation_error},
};

/// Standard Expression 小于或等于表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.LessOrEqualToExpression`。
pub struct LessOrEqualToExpression {
    operation: BinaryOperationExpression,
}

impl LessOrEqualToExpression {
    /// 创建小于或等于表达式。
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
}

impl IStandardExpression for LessOrEqualToExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        self.operation
            .get_string_representation(Some(&JavaString::from_rust_str("<=")))
    }

    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let (left, right) = execute_operands(&self.operation, context, execution_context)?;
        let left = collapse_java_null(left);
        let right = collapse_java_null(right);
        let (Some(left), Some(right)) = (left.as_ref(), right.as_ref()) else {
            return Err(comparison_null_error(
                "LESS OR EQUAL TO",
                left.as_deref(),
                right.as_deref(),
            ));
        };
        match compare_java_values(left, right)? {
            Some(ordering) => Ok(Some(Arc::new(TemplateValue::Boolean(
                ordering != Ordering::Greater,
            )))),
            None => Err(operation_error(
                "LESS OR EQUAL TO",
                self.get_string_representation()?,
                left.as_ref(),
                right.as_ref(),
            )),
        }
    }

    fn is_complex(&self) -> bool {
        true
    }
}

impl ComplexExpression for LessOrEqualToExpression {}
impl super::GreaterLesserExpression for LessOrEqualToExpression {}
