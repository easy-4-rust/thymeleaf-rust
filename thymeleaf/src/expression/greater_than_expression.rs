use std::cmp::Ordering;
use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::util::{JavaString, ValidateError};

use super::{
    BinaryOperationExpression, ComplexExpression, IStandardExpression,
    StandardExpressionExecutionContext, StandardExpressionResult, TemplateValue,
    binary_operation_expression::{collapse_java_null, compare_java_values, execute_operands},
};

/// Standard Expression 大于表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.GreaterThanExpression`。
pub struct GreaterThanExpression {
    operation: BinaryOperationExpression,
}

impl GreaterThanExpression {
    /// 创建大于表达式。
    /// 对应 Java 语义：`GreaterThanExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        left: Option<Arc<dyn IStandardExpression>>,
        right: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<Self, ValidateError> {
        BinaryOperationExpression::new(left, right).map(|operation| Self { operation })
    }

    /// 返回左操作数。
    /// 对应 Java 语义：Java 接口/超类方法 `getLeft()` 的 Rust 移植（`GreaterThanExpression` 继承路径）。
    pub fn get_left(&self) -> &dyn IStandardExpression {
        self.operation.get_left()
    }

    /// 返回右操作数。
    /// 对应 Java 语义：Java 接口/超类方法 `getRight()` 的 Rust 移植（`GreaterThanExpression` 继承路径）。
    pub fn get_right(&self) -> &dyn IStandardExpression {
        self.operation.get_right()
    }
}

impl IStandardExpression for GreaterThanExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        self.operation
            .get_string_representation(Some(&JavaString::from_rust_str(">")))
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
                "GREATER THAN",
                left.as_deref(),
                right.as_deref(),
            ));
        };
        match compare_java_values(left, right)? {
            Some(ordering) => Ok(Some(Arc::new(TemplateValue::Boolean(
                ordering == Ordering::Greater,
            )))),
            None => Err(operation_error(
                "GREATER THAN",
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

impl ComplexExpression for GreaterThanExpression {}
impl super::GreaterLesserExpression for GreaterThanExpression {}

/// 对应 Java 语义：`GreaterThanExpression` 的 `comparison_null_error` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn comparison_null_error(
    operation: &str,
    left: Option<&TemplateValue>,
    right: Option<&TemplateValue>,
) -> crate::expression::StandardExpressionError {
    Box::new(TemplateProcessingException::new(Some(format!(
        "Cannot execute {operation} comparison: operands are \"{}\" and \"{}\"",
        display_value(left),
        display_value(right)
    ))))
}

/// 对应 Java 语义：`GreaterThanExpression` 的 `operation_error` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn operation_error(
    operation: &str,
    expression: JavaString,
    left: &TemplateValue,
    right: &TemplateValue,
) -> crate::expression::StandardExpressionError {
    Box::new(TemplateProcessingException::new(Some(format!(
        "Cannot execute {operation} from Expression \"{}\". Left is \"{}\", right is \"{}\"",
        expression.to_string_lossy(),
        display_value(Some(left)),
        display_value(Some(right))
    ))))
}

fn display_value(value: Option<&TemplateValue>) -> String {
    value
        .and_then(TemplateValue::to_java_string)
        .unwrap_or_else(|| JavaString::from_rust_str("null"))
        .to_string_lossy()
}
