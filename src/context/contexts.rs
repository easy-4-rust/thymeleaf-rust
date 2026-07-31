use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::web::IWebExchange;

use super::{IContext, IEngineContext, IWebContext};

/// 上下文运行时 capability 检查与安全转换工具。
///
/// Rust 通过 `IContext` 显式 capability 复现 Java `instanceof` 和强制转换；错误
/// 转换与 Java `ClassCastException` 一样立即失败。
///
/// 对应 Java: `org.thymeleaf.context.Contexts`。
pub struct Contexts;

/// Context capability 强制转换失败。
///
/// 对应 Java: `java.lang.ClassCastException`。Rust 引用不能表示 Java `null`，而不兼容
/// capability 的强制转换仍按 Java unchecked runtime failure 抛出本错误。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextsError {
    /// 当前 Context 不实现目标 engine/Web capability。
    ContextCast {
        /// 目标 Java 接口的简单名称。
        target: &'static str,
    },
    /// 当前 Web exchange 不具备 Servlet capability。
    ServletExchangeCast,
}

impl ContextsError {
    /// 返回 Java 对应异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        "java.lang.ClassCastException"
    }
}

impl Display for ContextsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContextCast { target } => write!(formatter, "Context cannot be cast to {target}"),
            Self::ServletExchangeCast => {
                formatter.write_str("Web exchange cannot be cast to IServletWebExchange")
            }
        }
    }
}

impl Error for ContextsError {}

impl Contexts {
    /// 判断上下文是否为引擎内部上下文。
    ///
    /// 对应 Java: `Contexts#isEngineContext(IContext)`。
    #[must_use]
    pub fn is_engine_context(context: &dyn IContext) -> bool {
        context.as_engine_context().is_some()
    }

    /// 转换为引擎内部上下文；类型不匹配时失败。
    ///
    /// 对应 Java: `Contexts#asEngineContext(IContext)`。
    #[must_use]
    pub fn as_engine_context(context: &dyn IContext) -> &dyn IEngineContext {
        context.as_engine_context().unwrap_or_else(|| {
            std::panic::panic_any(ContextsError::ContextCast {
                target: "IEngineContext",
            })
        })
    }

    /// 判断上下文是否具有 Web capability。
    ///
    /// 对应 Java: `Contexts#isWebContext(IContext)`。
    #[must_use]
    pub fn is_web_context(context: &dyn IContext) -> bool {
        context.as_web_context().is_some()
    }

    /// 转换为 Web 上下文；类型不匹配时失败。
    ///
    /// 对应 Java: `Contexts#asWebContext(IContext)`。
    #[must_use]
    pub fn as_web_context(context: &dyn IContext) -> &dyn IWebContext {
        context.as_web_context().unwrap_or_else(|| {
            std::panic::panic_any(ContextsError::ContextCast {
                target: "IWebContext",
            })
        })
    }

    /// 返回 Web exchange；非 Web 上下文时失败。
    ///
    /// 对应 Java: `Contexts#getWebExchange(IContext)`。
    #[must_use]
    pub fn get_web_exchange(context: &dyn IContext) -> &dyn IWebExchange {
        Self::as_web_context(context).get_exchange()
    }

    /// 判断上下文是否由 Rust Web 宿主 exchange 支撑。
    ///
    /// 对应 Java: `Contexts#isServletWebContext(IContext)`；Rust 将 Servlet capability
    /// 映射为中立 `IWebExchange`，实际宿主类型留在 `thymeleaf-{framework}`。
    #[must_use]
    pub fn is_servlet_web_context(context: &dyn IContext) -> bool {
        context.as_web_context().is_some_and(|web_context| {
            web_context
                .get_exchange()
                .as_servlet_web_exchange()
                .is_some()
        })
    }

    /// 返回中立宿主 exchange；类型不匹配时失败。
    ///
    /// 对应 Java: `Contexts#getServletWebExchange(IContext)` 的 Rust Host 映射。
    #[must_use]
    pub fn get_servlet_web_exchange(context: &dyn IContext) -> &dyn IWebExchange {
        Self::get_web_exchange(context)
            .as_servlet_web_exchange()
            .unwrap_or_else(|| std::panic::panic_any(ContextsError::ServletExchangeCast))
    }
}
