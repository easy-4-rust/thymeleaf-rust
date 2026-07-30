use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::{JavaString, ValidateError};

use super::{
    ComplexExpression, IStandardExpression, StandardExpressionExecutionContext,
    StandardExpressionResult, TemplateValue, binary_operation_expression::evaluate_as_boolean,
};

/// Standard Expression 布尔取反表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.NegationExpression`。
pub struct NegationExpression {
    operand: Arc<dyn IStandardExpression>,
}

impl NegationExpression {
    /// 创建取反表达式。
    pub fn new(operand: Option<Arc<dyn IStandardExpression>>) -> Result<Self, ValidateError> {
        operand
            .map(|operand| Self { operand })
            .ok_or_else(|| ValidateError::IllegalArgument {
                message: Some("Operand cannot be null".to_owned()),
            })
    }
    /// 返回操作数。
    pub fn get_operand(&self) -> &dyn IStandardExpression {
        self.operand.as_ref()
    }
}

impl IStandardExpression for NegationExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        let mut units = vec![b'!' as u16];
        if self.operand.is_complex() {
            units.push(b'(' as u16);
        }
        units.extend_from_slice(self.operand.get_string_representation()?.as_utf16());
        if self.operand.is_complex() {
            units.push(b')' as u16);
        }
        Ok(JavaString::from_utf16(units))
    }
    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let operand = self
            .operand
            .execute_with_context(context, execution_context)?;
        Ok(Some(Arc::new(TemplateValue::Boolean(
            !evaluate_as_boolean(operand.as_ref())?,
        ))))
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl ComplexExpression for NegationExpression {}
