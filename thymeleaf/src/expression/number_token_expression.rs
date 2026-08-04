use std::sync::Arc;

use num_bigint::BigInt;
use unicode_general_category::{GeneralCategory, get_general_category};

use crate::context::IExpressionContext;
use crate::util::{JavaBigDecimal, JavaNumber, Utf16String};

use super::{
    IStandardExpression, StandardExpressionExecutionContext, StandardExpressionResult,
    TemplateValue, TokenError,
};

/// Standard Expression 数字 Token。
///
/// 对应 Java: `org.thymeleaf.standard.expression.NumberTokenExpression`。
pub struct NumberTokenExpression {
    value: JavaNumber,
}

impl NumberTokenExpression {
    /// 小数点 UTF-16 代码单元。
    pub const DECIMAL_POINT: u16 = b'.' as u16;

    /// 按 Java `BigDecimal(String)` 后的 scale 规则创建数字 Token。
    /// 对应 Java 语义：`NumberTokenExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(value: Option<&Utf16String>) -> StandardExpressionResult<Self> {
        let value = value.ok_or_else(|| {
            Box::new(TokenError::NullPointer) as crate::expression::StandardExpressionError
        })?;
        let text = String::from_utf16(value.as_utf16())
            .map_err(|error| Box::new(error) as crate::expression::StandardExpressionError)?;
        let decimal = JavaBigDecimal::parse(&text)
            .map_err(|error| Box::new(error) as crate::expression::StandardExpressionError)?;
        let number = if decimal.scale() > 0 {
            JavaNumber::BigDecimal(decimal)
        } else {
            let plain = decimal.to_plain_string();
            let integer = BigInt::parse_bytes(plain.as_bytes(), 10).ok_or_else(|| {
                Box::new(TokenError::runtime(
                    "java.lang.NumberFormatException",
                    plain,
                )) as crate::expression::StandardExpressionError
            })?;
            JavaNumber::BigInteger(integer)
        };
        Ok(Self { value: number })
    }

    /// 返回保存的 Number。
    /// 对应 Java 语义：Java 接口/超类方法 `getValue()` 的 Rust 移植（`NumberTokenExpression` 继承路径）。
    pub fn get_value(&self) -> &JavaNumber {
        &self.value
    }

    /// 解析只含 Java digit 和至多一个小数点的数字。
    /// 对应 Java: `NumberTokenExpression#parseNumberTokenExpression()`。
    pub fn parse_number_token_expression(input: Option<&Utf16String>) -> Option<Self> {
        let input = input?;
        if input.is_empty() || input.as_utf16().iter().all(|unit| *unit <= 0x20) {
            return None;
        }
        let mut decimal_found = false;
        for unit in input.as_utf16() {
            if char::from_u32(u32::from(*unit))
                .is_some_and(|value| get_general_category(value) == GeneralCategory::DecimalNumber)
            {
                continue;
            }
            if *unit == Self::DECIMAL_POINT && !decimal_found {
                decimal_found = true;
                continue;
            }
            return None;
        }
        Self::new(Some(input)).ok()
    }
}

impl IStandardExpression for NumberTokenExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<Utf16String> {
        let text = match &self.value {
            JavaNumber::BigDecimal(value) => value.to_plain_string(),
            JavaNumber::BigInteger(value) => value.to_string(),
            _ => unreachable!("NumberTokenExpression only stores BigDecimal or BigInteger"),
        };
        Ok(Utf16String::from_rust_str(&text))
    }

    fn execute_with_context(
        &self,
        _context: &dyn IExpressionContext,
        _expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        Ok(Some(Arc::new(TemplateValue::Number(self.value.clone()))))
    }

    fn is_token_expression(&self) -> bool {
        true
    }

    fn is_number_token_expression(&self) -> bool {
        true
    }
}

impl super::SimpleExpression for NumberTokenExpression {}
