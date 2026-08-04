use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::{Utf16String, ValidateError};

use super::{
    BinaryOperationExpression, ComplexExpression, IStandardExpression,
    StandardExpressionExecutionContext, StandardExpressionResult, TemplateValue,
    binary_operation_expression::{collapse_java_null, execute_operands, values_equal},
};

/// Standard Expression 不相等表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.NotEqualsExpression`。
pub struct NotEqualsExpression {
    operation: BinaryOperationExpression,
}

impl NotEqualsExpression {
    /// 创建不相等表达式。
    /// 对应 Java 语义：`NotEqualsExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        left: Option<Arc<dyn IStandardExpression>>,
        right: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<Self, ValidateError> {
        BinaryOperationExpression::new(left, right).map(|operation| Self { operation })
    }
    /// 返回左操作数。
    /// 对应 Java 语义：Java 接口/超类方法 `getLeft()` 的 Rust 移植（`NotEqualsExpression` 继承路径）。
    pub fn get_left(&self) -> &dyn IStandardExpression {
        self.operation.get_left()
    }
    /// 返回右操作数。
    /// 对应 Java 语义：Java 接口/超类方法 `getRight()` 的 Rust 移植（`NotEqualsExpression` 继承路径）。
    pub fn get_right(&self) -> &dyn IStandardExpression {
        self.operation.get_right()
    }
}

impl IStandardExpression for NotEqualsExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<Utf16String> {
        self.operation
            .get_string_representation(Some(&Utf16String::from_rust_str("!=")))
    }
    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let (left, right) = execute_operands(&self.operation, context, execution_context)?;
        let left = collapse_java_null(left);
        let right = collapse_java_null(right);
        Ok(Some(Arc::new(TemplateValue::Boolean(!values_equal(
            left.as_ref(),
            right.as_ref(),
        )?))))
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl ComplexExpression for NotEqualsExpression {}
impl super::EqualsNotEqualsExpression for NotEqualsExpression {}
