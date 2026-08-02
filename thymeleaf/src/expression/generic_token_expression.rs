use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::JavaString;

use super::{
    IStandardExpression, StandardExpressionExecutionContext, StandardExpressionResult,
    TemplateValue, Token,
};

/// Standard Expression 通用 Token。
///
/// 对应 Java: `org.thymeleaf.standard.expression.GenericTokenExpression`。
pub struct GenericTokenExpression {
    value: Arc<JavaString>,
}

impl GenericTokenExpression {
    fn new(value: JavaString) -> Self {
        Self {
            value: Arc::new(value),
        }
    }

    /// 所有码元均满足 `Token#isTokenChar` 时创建通用 Token。
    /// 对应 Java: `GenericTokenExpression#parseGenericTokenExpression()`。
    pub fn parse_generic_token_expression(input: Option<&JavaString>) -> Option<Self> {
        let input = input?;
        for position in 0..input.len() {
            let position = i32::try_from(position).ok()?;
            if !Token::<JavaString>::is_token_char(Some(input), position).ok()? {
                return None;
            }
        }
        Some(Self::new(input.clone()))
    }

    /// 返回 Token 保存的同一字符串。
    /// 对应 Java 语义：Java 接口/超类方法 `getValue()` 的 Rust 移植（`GenericTokenExpression` 继承路径）。
    pub fn get_value(&self) -> &JavaString {
        self.value.as_ref()
    }
}

impl IStandardExpression for GenericTokenExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        Ok(self.value.as_ref().clone())
    }

    fn execute_with_context(
        &self,
        _context: &dyn IExpressionContext,
        _expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        Ok(Some(Arc::new(TemplateValue::String(Arc::clone(
            &self.value,
        )))))
    }

    fn is_token_expression(&self) -> bool {
        true
    }

    fn is_generic_token_expression(&self) -> bool {
        true
    }
}

impl super::SimpleExpression for GenericTokenExpression {}
