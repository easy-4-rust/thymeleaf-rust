use std::sync::Arc;
use std::sync::OnceLock;

use super::ILazyContextVariable;
use crate::expression::{TemplateObject, TemplateValue};
use crate::util::Utf16String;

/// 只执行一次加载逻辑的基础惰性上下文变量。
///
/// 对应 Java: `org.thymeleaf.context.LazyContextVariable<T>`。
///
/// Java 版本通过双重检查和 `volatile initialized` 保证 `loadValue()` 每个模板
/// 执行只调用一次。Rust 版本用 `OnceLock` 提供相同的跨线程发布与缓存语义，并以
/// 构造时传入的闭包承载 Java 子类对抽象 `loadValue()` 的实现。
pub struct LazyContextVariable<T, F>
where
    F: Fn() -> T,
{
    value: OnceLock<T>,
    load_value: F,
}

impl<T, F> LazyContextVariable<T, F>
where
    F: Fn() -> T,
{
    /// 创建尚未求值的惰性上下文变量。
    ///
    /// 对应 Java: protected `LazyContextVariable#LazyContextVariable()` 与子类
    /// `loadValue()` 实现的 Rust 组合形式。
    ///
    /// # 参数
    ///
    /// - `load_value`：第一次访问时执行的实际解析逻辑。
    ///
    /// # 返回
    ///
    /// 未初始化且不预先执行闭包的惰性变量。
    #[must_use]
    pub const fn new(load_value: F) -> Self {
        Self {
            value: OnceLock::new(),
            load_value,
        }
    }

    /// 惰性解析并返回缓存值。
    ///
    /// 对应 Java: `LazyContextVariable#getValue()`。
    ///
    /// # 返回
    ///
    /// 首次调用执行一次加载逻辑；之后始终返回同一值的共享借用。
    pub fn get_value(&self) -> &T {
        self.value.get_or_init(|| (self.load_value)())
    }
}

impl<T, F> ILazyContextVariable<T> for LazyContextVariable<T, F>
where
    F: Fn() -> T,
{
    fn get_value(&self) -> &T {
        Self::get_value(self)
    }
}

impl<F> TemplateObject for LazyContextVariable<Option<Arc<TemplateValue>>, F>
where
    F: Fn() -> Option<Arc<TemplateValue>> + Send + Sync + 'static,
{
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.context.LazyContextVariable"
    }

    fn to_utf16_string(&self) -> Utf16String {
        self.get_value()
            .as_deref()
            .and_then(TemplateValue::to_utf16_string)
            .unwrap_or_else(|| Utf16String::from_rust_str("null"))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn resolve_lazy_context_variable(&self) -> Option<Option<Arc<TemplateValue>>> {
        Some(self.get_value().clone())
    }
}
