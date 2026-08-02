use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::{JavaString, ValidateError};

use super::{
    ComplexExpression, IStandardExpression, StandardExpressionExecutionContext,
    StandardExpressionResult, TemplateValue, binary_operation_expression::evaluate_as_boolean,
};

/// Standard Expression 三元条件表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.ConditionalExpression`。
pub struct ConditionalExpression {
    condition_expression: Arc<dyn IStandardExpression>,
    then_expression: Arc<dyn IStandardExpression>,
    else_expression: Arc<dyn IStandardExpression>,
}

impl ConditionalExpression {
    /// 创建条件表达式，并按 Java 顺序校验 condition、then、else。
    /// 对应 Java 语义：`ConditionalExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        condition_expression: Option<Arc<dyn IStandardExpression>>,
        then_expression: Option<Arc<dyn IStandardExpression>>,
        else_expression: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<Self, ValidateError> {
        let condition_expression =
            condition_expression.ok_or_else(|| ValidateError::IllegalArgument {
                message: Some("Condition expression cannot be null".to_owned()),
            })?;
        let then_expression = then_expression.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Then expression cannot be null".to_owned()),
        })?;
        let else_expression = else_expression.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Else expression cannot be null".to_owned()),
        })?;
        Ok(Self {
            condition_expression,
            then_expression,
            else_expression,
        })
    }
    /// 返回条件表达式。
    /// 对应 Java: `ConditionalExpression#getConditionExpression()`。
    pub fn get_condition_expression(&self) -> &dyn IStandardExpression {
        self.condition_expression.as_ref()
    }
    /// 返回 then 表达式。
    /// 对应 Java: `ConditionalExpression#getThenExpression()`。
    pub fn get_then_expression(&self) -> &dyn IStandardExpression {
        self.then_expression.as_ref()
    }
    /// 返回 else 表达式。
    /// 对应 Java: `ConditionalExpression#getElseExpression()`。
    pub fn get_else_expression(&self) -> &dyn IStandardExpression {
        self.else_expression.as_ref()
    }
}

impl IStandardExpression for ConditionalExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        let mut units = Vec::new();
        append(&mut units, self.condition_expression.as_ref())?;
        units.extend_from_slice(&[b'?' as u16, b' ' as u16]);
        append(&mut units, self.then_expression.as_ref())?;
        units.extend_from_slice(&[b' ' as u16, b':' as u16, b' ' as u16]);
        append(&mut units, self.else_expression.as_ref())?;
        Ok(JavaString::from_utf16(units))
    }
    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let condition = self
            .condition_expression
            .execute_with_context(context, execution_context)?;
        if evaluate_as_boolean(condition.as_ref())? {
            self.then_expression
                .execute_with_context(context, execution_context)
        } else {
            self.else_expression
                .execute_with_context(context, execution_context)
        }
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl ComplexExpression for ConditionalExpression {}

fn append(
    units: &mut Vec<u16>,
    expression: &dyn IStandardExpression,
) -> StandardExpressionResult<()> {
    if expression.is_complex() {
        units.push(b'(' as u16);
    }
    units.extend_from_slice(expression.get_string_representation()?.as_utf16());
    if expression.is_complex() {
        units.push(b')' as u16);
    }
    Ok(())
}
