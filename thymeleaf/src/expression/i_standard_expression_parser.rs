use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::Utf16String;

use super::{IStandardExpression, StandardExpressionResult};

/// Standard Expression 解析器合同。
///
/// 对应 Java: `org.thymeleaf.standard.expression.IStandardExpressionParser`。
///
/// 实现必须能在线程之间安全共享。
pub trait IStandardExpressionParser: Send + Sync {
    /// 解析指定输入并返回表达式对象。
    fn parse_expression(
        &self,
        context: &dyn IExpressionContext,
        input: Option<&Utf16String>,
    ) -> StandardExpressionResult<Arc<dyn IStandardExpression>>;

    /// 是否为支持 `__...__` 预处理的标准解析器实现。
    ///
    /// Java 使用 `instanceof StandardExpressionParser`；Rust 以显式能力方法保留同一
    /// 动态分派语义，第三方解析器默认不启用预处理。
    fn supports_standard_preprocessing(&self) -> bool {
        false
    }
}
