//! GTVG Web 宿主 —— 对应 Java `GTVGFilter` 中的 `JakartaServletWebApplication`
//! / `JakartaServletWebExchange` / `HttpServletRequest` / `HttpSession` 角色。
//!
//! 引擎保持 Web 框架中立；示例用最小实现提供请求（路径 + 参数）、会话（user
//! 属性）与应用三个作用域。对应 Java `GTVGFilter#addUserToSession` 的
//! “模拟真实用户会话”逻辑在 `GtvgWebSession::with_user` 中体现。

pub mod gtvg_web_application;
pub mod gtvg_web_exchange;
pub mod gtvg_web_request;
pub mod gtvg_web_session;
