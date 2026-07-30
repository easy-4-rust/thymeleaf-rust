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
        // `TemplateValue::NoOp` 本身是无状态规范值；宿主对象允许非 Send 用户对象后，
        // 不能把整个动态值枚举放入进程级 OnceLock。枚举判别值仍保留 Java 单例的
        // 所有模板可观察语义。
        Ok(Some(Arc::new(TemplateValue::NoOp)))
    }

    fn is_token_expression(&self) -> bool {
        true
    }
}

impl super::SimpleExpression for NoOpTokenExpression {}
