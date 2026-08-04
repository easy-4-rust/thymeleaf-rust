use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::util::{JavaBigDecimal, JavaNumber, Utf16String, ValidateError};

use super::{
    ComplexExpression, IStandardExpression, StandardExpressionExecutionContext,
    StandardExpressionResult, TemplateValue,
    binary_operation_expression::{literal_unwrapped_string, normalized_null_value},
};

/// Standard Expression 数值负号表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.MinusExpression`。
pub struct MinusExpression {
    operand: Arc<dyn IStandardExpression>,
}

impl MinusExpression {
    /// 创建数值负号表达式。
    /// 对应 Java 语义：`MinusExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(operand: Option<Arc<dyn IStandardExpression>>) -> Result<Self, ValidateError> {
        operand
            .map(|operand| Self { operand })
            .ok_or_else(|| ValidateError::IllegalArgument {
                message: Some("Operand cannot be null".to_owned()),
            })
    }
    /// 返回操作数。
    /// 对应 Java: `MinusExpression#getOperand()`。
    pub fn get_operand(&self) -> &dyn IStandardExpression {
        self.operand.as_ref()
    }
}

impl IStandardExpression for MinusExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<Utf16String> {
        let mut units = vec![b'-' as u16];
        if self.operand.is_complex() {
            units.push(b'(' as u16);
        }
        units.extend_from_slice(self.operand.get_string_representation()?.as_utf16());
        if self.operand.is_complex() {
            units.push(b')' as u16);
        }
        Ok(Utf16String::from_utf16(units))
    }
    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let operand = normalized_null_value(
            self.operand
                .execute_with_context(context, execution_context)?,
        );
        if let TemplateValue::Number(number) = operand.as_ref() {
            // Java/OGNL 的一元负号保留包装数字类型；这对接收 Integer 参数的
            // `#numbers.sequence(from, to, step)` 等方法尤为重要。
            let negated = match number {
                JavaNumber::Byte(value) => JavaNumber::Integer(-i32::from(*value)),
                JavaNumber::Short(value) => JavaNumber::Integer(-i32::from(*value)),
                JavaNumber::Integer(value) => JavaNumber::Integer(value.wrapping_neg()),
                JavaNumber::Long(value) => JavaNumber::Long(value.wrapping_neg()),
                JavaNumber::Float(value) => JavaNumber::Float(-value),
                JavaNumber::Double(value) => JavaNumber::Double(-value),
                JavaNumber::BigInteger(value) => JavaNumber::BigInteger(-value),
                JavaNumber::BigDecimal(value) => {
                    JavaNumber::BigDecimal(value.multiply_java(&JavaBigDecimal::parse("-1")?)?)
                }
                JavaNumber::Other {
                    class_name,
                    double_value,
                } => JavaNumber::Other {
                    class_name: class_name.clone(),
                    double_value: -double_value,
                },
            };
            return Ok(Some(Arc::new(TemplateValue::Number(negated))));
        }
        let display = literal_unwrapped_string(operand.as_ref())
            .unwrap_or_else(|| Utf16String::from_rust_str("null"))
            .to_string_lossy();
        Err(Box::new(TemplateProcessingException::new(Some(format!(
            "Cannot execute minus: operand is \"{display}\""
        )))))
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl ComplexExpression for MinusExpression {}
