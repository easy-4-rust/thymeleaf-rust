use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::JavaString;

use super::{
    IStandardExpression, StandardExpressionExecutionContext, StandardExpressionResult,
    TemplateValue,
};

/// Standard Expression null Token。
///
/// 对应 Java: `org.thymeleaf.standard.expression.NullTokenExpression`。
pub struct NullTokenExpression;

impl NullTokenExpression {
    /// 创建 null Token；Java 公开构造器每次产生新对象。
    pub const fn new() -> Self {
        Self
    }

    /// 忽略大小写解析 null，并复用内部规范单例。
    /// 对应 Java: `NullTokenExpression#parseNullTokenExpression()`。
    pub fn parse_null_token_expression(input: Option<&JavaString>) -> Option<Arc<Self>> {
        let input = input?;
        input
            .to_string_lossy()
            .eq_ignore_ascii_case("null")
            .then(Self::singleton)
    }

    fn singleton() -> Arc<Self> {
        static SINGLETON: std::sync::OnceLock<Arc<NullTokenExpression>> =
            std::sync::OnceLock::new();
        Arc::clone(SINGLETON.get_or_init(|| Arc::new(Self)))
    }
}

impl Default for NullTokenExpression {
    fn default() -> Self {
        Self::new()
    }
}

impl IStandardExpression for NullTokenExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        Ok(JavaString::from_rust_str("null"))
    }

    fn execute_with_context(
        &self,
        _context: &dyn IExpressionContext,
        _expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        Ok(None)
    }

    fn is_token_expression(&self) -> bool {
        true
    }
}

impl super::SimpleExpression for NullTokenExpression {}
