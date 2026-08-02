use std::sync::Arc;

use crate::web::IWebExchange;

use super::IContext;

/// Web 模板处理上下文合同。
///
/// 对应 Java: `org.thymeleaf.context.IWebContext`。
///
/// 该接口只增加框架中立的 Web exchange，不依赖 Servlet、Actix Web、Axum 或其他
/// 具体宿主。与 Java 3.1 合同一致，exchange 为本次模板执行提供 URL 重写及
/// request/session/application 访问能力。
pub trait IWebContext: IContext {
    /// 返回与当前模板执行关联的 Web exchange。
    ///
    /// 对应 Java: `IWebContext#getExchange()`。
    ///
    /// # 返回值
    ///
    /// 返回构造 Context 时传入的同一非空 exchange 对象。
    fn get_exchange(&self) -> &dyn IWebExchange;

    /// 返回同一 Web exchange 的共享身份。
    ///
    /// Java 直接把对象引用传给 `WebEngineContext`；Rust 需要显式克隆 `Arc`，才能
    /// 让新引擎上下文独立于原始 Web Context 的借用生命周期。实现必须让此方法与
    /// `get_exchange()` 指向同一逻辑 exchange。
    ///
    /// # 返回值
    ///
    /// 返回与 `get_exchange()` 共享同一分配的 `Arc`。
    fn get_exchange_arc(&self) -> Arc<dyn IWebExchange>;
}
