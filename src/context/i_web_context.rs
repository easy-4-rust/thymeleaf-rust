use crate::web::IWebExchange;

use super::IContext;

/// Web 模板处理上下文合同。
///
/// 对应 Java: `org.thymeleaf.context.IWebContext`。
///
/// 该接口只增加框架中立的 Web exchange，不依赖 Servlet、Actix Web、Axum 或其他
/// 具体宿主。
pub trait IWebContext: IContext {
    /// 返回与当前模板执行关联的 Web exchange。
    fn get_exchange(&self) -> &dyn IWebExchange;
}
