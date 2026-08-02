//! Sa-Token 安全方言 —— 把 sa-token-rust 的认证/授权状态接入 Thymeleaf 模板。
//!
//! 对应 Java `thymeleaf-extras-springsecurity6`：
//! - [`SaTokenDialect`]（`sec` 前缀）：`sec:authorize` 条件可见性、
//!   `sec:authentication` 身份输出
//! - `#authentication` / `#authorization` 表达式对象（由
//!   [`SaTokenExpressionObjectFactory`] 提供）
//! - [`SaTokenAuthentication`]：渲染前预取的同步安全快照
//! - [`load_authentication`]：async 预取入口
//!
//! # 用法
//!
//! ```text
//! let authentication = load_authentication(&manager, login_id).await?;
//! let mut context = WebContext::new(Some(exchange))?;
//! context.set_variable(
//!     Some(JavaString::from_rust_str(AUTHENTICATION_VARIABLE)),
//!     Some(Arc::new(SaTokenAuthenticationObject::to_template_value(
//!         Arc::new(authentication),
//!     ))),
//! );
//! engine.add_dialect(Arc::new(SaTokenDialect::new()));
//! ```
//!
//! 模板内：
//! ```html
//! <div sec:authorize="${hasRole('ADMIN')}">admin only</div>
//! <span sec:authentication="name">current user</span>
//! ${#authorization.hasPermission('orders:write')}
//! ```
//!
//! # NOT_APPLICABLE（Java 特性，未实现）
//!
//! - `sec:authorize-acl` / `sec:authorize-url`：依赖 Spring ACL/URL 域对象，sa-token
//!   无等价物。
//! - `sec:authorize-exprs`：Java 2.3 遗留语法。
//! - `#authentication.credentials` / `.details` / `.principal.authorities`：Java
//!   `Authentication` 特有字段，sa-token 的 [`SaTokenAuthentication`] 只有
//!   name/roles/permissions 等价物。
//!
//! 匿名（未登录）时 `#authentication` 求值为可安全访问的空对象、`#authorization`
//! 所有授权查询恒为 `false`（fail-closed）。

mod authentication;
mod authentication_object;
mod dialect;
mod expression_object;
mod processor;

pub use authentication::{AUTHENTICATION_VARIABLE, SaTokenAuthentication, load_authentication};
pub use authentication_object::SaTokenAuthenticationObject;
pub use dialect::{DIALECT_NAME, DIALECT_PREFIX, PROCESSOR_PRECEDENCE, SaTokenDialect};
pub use expression_object::{
    AUTHENTICATION_OBJECT_NAME, AUTHORIZATION_OBJECT_NAME, SaTokenExpressionObjectFactory,
};
pub use processor::{
    AUTHENTICATION_ATTR_NAME, AUTHENTICATION_PRECEDENCE, AUTHORIZE_ATTR_NAME, AUTHORIZE_PRECEDENCE,
    SecAuthenticationTagProcessor, SecAuthorizeTagProcessor,
};
