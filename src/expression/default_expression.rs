use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::{JavaString, ValidateError};

use super::{
    ComplexExpression, IStandardExpression, StandardExpressionExecutionContext,
    StandardExpressionResult, TemplateValue,
};

/// Standard Expression null 默认值表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.DefaultExpression`。
pub struct DefaultExpression {
    queried_expression: Arc<dyn IStandardExpression>,
    default_expression: Arc<dyn IStandardExpression>,
}

impl DefaultExpression {
    /// 创建默认值表达式，并按 Java 顺序校验两个操作数。
    pub fn new(
        queried_expression: Option<Arc<dyn IStandardExpression>>,
        default_expression: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<Self, ValidateError> {
        let queried_expression =
            queried_expression.ok_or_else(|| ValidateError::IllegalArgument {
                message: Some("Queried expression cannot be null".to_owned()),
            })?;
        let default_expression =
            default_expression.ok_or_else(|| ValidateError::IllegalArgument {
                message: Some("Default expression cannot be null".to_owned()),
            })?;
        Ok(Self {
            queried_expression,
            default_expression,
        })
    }
    /// 返回被查询表达式。
    pub fn get_queried_expression(&self) -> &dyn IStandardExpression {
        self.queried_expression.as_ref()
    }
    /// 返回默认表达式。
    pub fn get_default_expression(&self) -> &dyn IStandardExpression {
        self.default_expression.as_ref()
    }
}

impl IStandardExpression for DefaultExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        let mut units = Vec::new();
        append(&mut units, self.queried_expression.as_ref())?;
        units.extend_from_slice(&[b' ' as u16, b'?' as u16, b':' as u16, b' ' as u16]);
        append(&mut units, self.default_expression.as_ref())?;
        Ok(JavaString::from_utf16(units))
    }
    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let queried = self
            .queried_expression
            .execute_with_context(context, execution_context)?;
        if queried.is_none() {
            return self
                .default_expression
                .execute_with_context(context, execution_context);
        }
        Ok(queried)
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl ComplexExpression for DefaultExpression {}

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
