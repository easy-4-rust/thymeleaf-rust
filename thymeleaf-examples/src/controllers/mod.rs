//! 控制器层 —— 对应 Java `web/controller/` 包（8 个控制器 + 接口 + 映射）。

pub mod controller_mappings;
pub mod home;
pub mod order_details;
pub mod order_list;
pub mod product_comments;
pub mod product_list;
pub mod subscribe;
pub mod user_profile;

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::context::WebContext;
use thymeleaf::expression::{TemplateObject, TemplateValue};
use thymeleaf::util::{DateValue, Utf16String};
use thymeleaf::web::IWebExchange;

/// 控制器处理结果：渲染出的完整 HTML（对应 Java `Writer` 输出）。
pub type ControllerResult = Result<Utf16String, ControllerError>;

/// 控制器错误（对应 Java `process(...) throws Exception`）。
#[derive(Debug)]
pub struct ControllerError(pub String);

impl std::fmt::Display for ControllerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ControllerError {}

impl From<Box<dyn thymeleaf::TemplateEngineException + Send + Sync>> for ControllerError {
    fn from(error: Box<dyn thymeleaf::TemplateEngineException + Send + Sync>) -> Self {
        Self(format!("{error:?}"))
    }
}

/// 包装实体为 Java 对象模板值（对应 `ctx.setVariable(name, object)`）。
#[must_use]
pub fn template_object<T: TemplateObject>(value: T) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Object(Arc::new(value)))
}

/// 构造 Java `List` 模板值（对应 `ctx.setVariable(name, list)`）。
#[must_use]
pub fn template_list(values: Vec<Arc<TemplateValue>>) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::List(Arc::new(values)))
}

/// `IGTVGController#process(IWebExchange, ITemplateEngine, Writer)` 的 Rust 对应。
///
/// Java 控制器在内部 `new WebContext(webExchange, webExchange.getLocale())`；
/// 本 trait 保留该形状，`now` 由首页控制器注入“当前时间”（Java
/// `Calendar.getInstance()`），便于测试注入固定时刻。
pub trait GtvgController: Send + Sync {
    /// 渲染模板并返回完整输出。
    fn process(
        &self,
        web_exchange: Arc<dyn IWebExchange>,
        template_engine: &TemplateEngine,
        now: DateValue,
    ) -> ControllerResult;
}

/// `WebContext` 构造辅助 —— 对应 Java `new WebContext(webExchange, locale)`。
fn build_web_context(web_exchange: &Arc<dyn IWebExchange>) -> WebContext {
    WebContext::with_locale(Some(Arc::clone(web_exchange)), web_exchange.get_locale())
        .expect("web context construction")
}

/// `IContext#setVariable(String, Object)` 的便捷入口。
fn set_variable(context: &WebContext, name: &str, value: Option<Arc<TemplateValue>>) {
    context.set_variable(Some(Utf16String::from_rust_str(name)), value);
}
