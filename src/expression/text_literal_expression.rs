#![expect(
    dead_code,
    reason = "包级解析辅助将在后续 StandardExpressionParser 主链中消费"
)]

use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::{JavaString, ValidateError};

use super::{
    IStandardExpression, LiteralValue, StandardExpressionError, StandardExpressionExecutionContext,
    StandardExpressionResult, TemplateValue, TokenError,
};

/// Standard Expression 文本字面量。
///
/// 对应 Java: `org.thymeleaf.standard.expression.TextLiteralExpression`。
///
/// 字面量结果保留 `LiteralValue` 包装，避免 `"4"`、`"2."` 等文本在算术表达式中
/// 再次按数字求值。
pub struct TextLiteralExpression {
    value: Arc<LiteralValue>,
}

impl TextLiteralExpression {
    /// Java 字面量转义前缀。
    pub const ESCAPE_PREFIX: u16 = b'\\' as u16;
    /// Java 文本字面量定界符。
    pub const DELIMITER: u16 = b'\'' as u16;

    /// 创建文本字面量；外层成对单引号会移除，`\\'` 与 `\\\\` 会解转义。
    pub fn new(value: Option<&JavaString>) -> Result<Self, ValidateError> {
        let value = value.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Value cannot be null".to_owned()),
        })?;
        Ok(Self {
            value: Arc::new(LiteralValue::new(Some(unwrap_literal(value)))),
        })
    }

    /// 返回同一 LiteralValue 包装对象。
    pub fn get_value(&self) -> &LiteralValue {
        self.value.as_ref()
    }

    /// 解析文本字面量。上游该入口始终调用构造器，不额外验证定界符。
    pub(crate) fn parse_text_literal_expression(input: &JavaString) -> Self {
        Self::new(Some(input)).expect("non-null parser input")
    }

    /// 把可空字符串包装成单引号字面量，并转义其中每个单引号。
    pub fn wrap_string_into_literal(value: Option<&JavaString>) -> Option<JavaString> {
        let value = value?;
        let quote_count = value
            .as_utf16()
            .iter()
            .filter(|unit| **unit == Self::DELIMITER)
            .count();
        let mut units = Vec::with_capacity(value.len() + quote_count + 2);
        units.push(Self::DELIMITER);
        for unit in value.as_utf16() {
            if *unit == Self::DELIMITER {
                units.push(Self::ESCAPE_PREFIX);
            }
            units.push(*unit);
        }
        units.push(Self::DELIMITER);
        Some(JavaString::from_utf16(units))
    }

    /// 判断指定定界符前是否存在奇数个连续反斜杠。
    pub(crate) fn is_delimiter_escaped(
        input: Option<&JavaString>,
        position: i32,
    ) -> Result<bool, TokenError> {
        let input = input.ok_or(TokenError::NullPointer)?;
        let position = usize::try_from(position)
            .map_err(|_| TokenError::StringIndexOutOfBounds { position })?;
        if position >= input.len() {
            return Err(TokenError::StringIndexOutOfBounds {
                position: position as i32,
            });
        }
        if position == 0 || input.as_utf16()[position - 1] != Self::ESCAPE_PREFIX {
            return Ok(false);
        }
        let mut current = position;
        let mut odd = false;
        while current > 0 {
            current -= 1;
            if input.as_utf16()[current] == Self::ESCAPE_PREFIX {
                odd = !odd;
            } else {
                return Ok(odd);
            }
        }
        Ok(odd)
    }
}

impl IStandardExpression for TextLiteralExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        let value = self
            .value
            .get_value()
            .ok_or_else(|| Box::new(TokenError::NullPointer) as StandardExpressionError)?;
        Ok(Self::wrap_string_into_literal(Some(value)).expect("non-null literal value"))
    }

    fn execute_with_context(
        &self,
        _context: &dyn IExpressionContext,
        _expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        Ok(self
            .value
            .get_value()
            .cloned()
            .map(TemplateValue::string)
            .map(Arc::new))
    }

    fn execute_raw(
        &self,
        _context: &dyn IExpressionContext,
        _expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        Ok(Some(Arc::new(TemplateValue::Literal(Arc::clone(
            &self.value,
        )))))
    }

    fn is_text_literal_expression(&self) -> bool {
        true
    }
}

impl super::SimpleExpression for TextLiteralExpression {}

fn unwrap_literal(input: &JavaString) -> JavaString {
    let units = input.as_utf16();
    if units.len() > 1
        && units[0] == TextLiteralExpression::DELIMITER
        && units[units.len() - 1] == TextLiteralExpression::DELIMITER
    {
        return unescape_literal(&units[1..units.len() - 1]);
    }
    input.clone()
}

fn unescape_literal(text: &[u16]) -> JavaString {
    let mut result = Vec::with_capacity(text.len());
    let mut position = 0;
    while position < text.len() {
        let unit = text[position];
        if unit == TextLiteralExpression::ESCAPE_PREFIX && position + 1 < text.len() {
            let next = text[position + 1];
            if next == TextLiteralExpression::DELIMITER
                || next == TextLiteralExpression::ESCAPE_PREFIX
            {
                result.push(next);
                position += 2;
                continue;
            }
        }
        result.push(unit);
        position += 1;
    }
    JavaString::from_utf16(result)
}
