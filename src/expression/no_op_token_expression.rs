use std::sync::{Arc, OnceLock};

use crate::context::IExpressionContext;
use crate::util::JavaString;

use super::{
    IStandardExpression, StandardExpressionExecutionContext, StandardExpressionResult,
    TemplateValue,
};

/// Standard Expression NO-OP Token。
///
/// 对应 Java: `org.thymeleaf.standard.expression.NoOpTokenExpression`。
pub struct NoOpTokenExpression;

impl NoOpTokenExpression {
    /// 创建 NO-OP Token。
    pub const fn new() -> Self {
        Self
    }

    /// 只接受单个下划线，并复用内部规范单例。
    pub fn parse_no_op_token_expression(input: Option<&JavaString>) -> Option<Arc<Self>> {
        let input = input?;
        (input.as_utf16() == [b'_' as u16]).then(Self::singleton)
    }

    fn singleton() -> Arc<Self> {
        static SINGLETON: OnceLock<Arc<NoOpTokenExpression>> = OnceLock::new();
        Arc::clone(SINGLETON.get_or_init(|| Arc::new(Self)))
    }
}

impl Default for NoOpTokenExpression {
    fn default() -> Self {
        Self::new()
    }
}

impl IStandardExpression for NoOpTokenExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        Ok(JavaString::from_rust_str("_"))
    }

    fn execute_with_context(
        &self,
        _context: &dyn IExpressionContext,
        _expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        static VALUE: OnceLock<Arc<TemplateValue>> = OnceLock::new();
        Ok(Some(Arc::clone(
            VALUE.get_or_init(|| Arc::new(TemplateValue::NoOp)),
        )))
    }
}
