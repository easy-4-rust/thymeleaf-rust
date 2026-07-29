use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::JavaString;

use super::{
    IStandardExpression, StandardExpressionExecutionContext, StandardExpressionResult,
    TemplateValue, TokenError,
};

/// Standard Expression 布尔 Token。
///
/// 对应 Java: `org.thymeleaf.standard.expression.BooleanTokenExpression`。
pub struct BooleanTokenExpression {
    value: Option<bool>,
}

impl BooleanTokenExpression {
    /// 按 Java `Boolean.valueOf(String)` 创建布尔 Token；null 字符串得到 false。
    pub fn from_string(value: Option<&JavaString>) -> Self {
        let value = value.is_some_and(|value| value.to_string_lossy().eq_ignore_ascii_case("true"));
        Self { value: Some(value) }
    }

    /// 从可空 Java Boolean 创建布尔 Token。
    pub const fn from_boolean(value: Option<bool>) -> Self {
        Self { value }
    }

    /// 返回可空 Boolean 值。
    pub const fn get_value(&self) -> Option<bool> {
        self.value
    }

    /// 解析忽略大小写的 true/false；其他输入不匹配。
    pub fn parse_boolean_token_expression(input: Option<&JavaString>) -> Option<Self> {
        let input = input?;
        let text = input.to_string_lossy();
        (text.eq_ignore_ascii_case("true") || text.eq_ignore_ascii_case("false"))
            .then(|| Self::from_string(Some(input)))
    }
}

impl IStandardExpression for BooleanTokenExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        self.value
            .map(|value| JavaString::from_rust_str(if value { "true" } else { "false" }))
            .ok_or_else(|| Box::new(TokenError::NullPointer) as _)
    }

    fn execute_with_context(
        &self,
        _context: &dyn IExpressionContext,
        _expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        Ok(self
            .value
            .map(|value| Arc::new(TemplateValue::Boolean(value))))
    }
}
