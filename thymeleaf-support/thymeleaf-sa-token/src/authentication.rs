//! 渲染前预取的同步安全快照 —— 模板 sec 方言的求值数据源。
//!
//! Thymeleaf 的 Processor 与表达式求值是同步的，而 sa-token 的角色/权限查询是
//! `async`。因此整合层在进入模板渲染之前（async 上下文）调用 [`load_authentication`]
//! 预取登录身份、角色与权限列表，把结果以 [`TemplateValue`] 注入模板上下文变量
//! （约定键 [`AUTHENTICATION_VARIABLE`]），sec 方言处理器与 `#authentication`/
//! `#authorization` 表达式对象全部基于该同步快照求值，不触发任何异步存储访问。

use std::sync::Arc;

use sa_token_core::{SaTokenError, SaTokenManager};

/// 模板上下文变量约定键：持有 [`SaTokenAuthentication`] 包装的 `TemplateValue`。
///
/// 用户可以在构造 WebContext/Context 时用 `set_variable` 显式注入，或由
/// `thymeleaf-vernal` 的 `VernalWebExchange` 自动从 `SecurityPrincipal` 构建。
pub const AUTHENTICATION_VARIABLE: &str = "saTokenAuthentication";

/// 一次模板执行使用的同步认证快照。
///
/// 对应 Java `thymeleaf-extras-springsecurity6` 中 `Authentication` 的模板可见
/// 子集：身份标识（`name`）、角色与权限集合。匿名（未登录）时 [`Self::login_id`]
/// 为 `None`，所有授权查询恒为 `false`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaTokenAuthentication {
    login_id: Option<String>,
    roles: Arc<[Arc<str>]>,
    permissions: Arc<[Arc<str>]>,
}

impl SaTokenAuthentication {
    /// 创建已认证快照。
    ///
    /// # 参数
    ///
    /// - `login_id`：非空登录标识。
    /// - `roles`：已预取的角色名集合。
    /// - `permissions`：已预取的权限名集合。
    #[must_use]
    pub fn new(login_id: String, roles: Arc<[Arc<str>]>, permissions: Arc<[Arc<str>]>) -> Self {
        Self {
            login_id: Some(login_id),
            roles,
            permissions,
        }
    }

    /// 创建匿名（未登录）快照。
    #[must_use]
    pub fn anonymous() -> Self {
        Self {
            login_id: None,
            roles: Arc::from([]),
            permissions: Arc::from([]),
        }
    }

    /// 当前请求是否具有已认证登录身份。
    #[must_use]
    pub fn is_authenticated(&self) -> bool {
        self.login_id.is_some()
    }

    /// 返回已认证的登录标识；匿名时为 `None`。
    #[must_use]
    pub fn login_id(&self) -> Option<&str> {
        self.login_id.as_deref()
    }

    /// 返回已预取的角色名集合。
    #[must_use]
    pub fn roles(&self) -> &[Arc<str>] {
        &self.roles
    }

    /// 返回已预取的权限名集合。
    #[must_use]
    pub fn permissions(&self) -> &[Arc<str>] {
        &self.permissions
    }

    /// 判断是否具有指定角色（精确匹配，与 sa-token `StpUtil::has_role` 一致）。
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|owned| owned.as_ref() == role)
    }

    /// 判断是否具有任意一个指定角色。
    #[must_use]
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// 判断是否同时具有全部指定角色。
    #[must_use]
    pub fn has_all_roles(&self, roles: &[&str]) -> bool {
        roles.iter().all(|role| self.has_role(role))
    }

    /// 判断是否具有指定权限。
    ///
    /// 匹配语义与 sa-token `StpUtil::has_permission` 一致（util.rs:546-568）：
    /// - 精确匹配；
    /// - 全局通配 `*` 匹配任意权限；
    /// - 前缀通配 `admin:*` 匹配所有 `admin:` 前缀权限。
    #[must_use]
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|owned| {
            let owned = owned.as_ref();
            if owned == "*" || owned == permission {
                return true;
            }
            owned
                .strip_suffix(":*")
                .is_some_and(|prefix| permission.starts_with(&format!("{prefix}:")))
        })
    }

    /// 判断是否具有任意一个指定权限。
    #[must_use]
    pub fn has_any_permission(&self, permissions: &[&str]) -> bool {
        permissions
            .iter()
            .any(|permission| self.has_permission(permission))
    }

    /// 判断是否同时具有全部指定权限。
    #[must_use]
    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions
            .iter()
            .all(|permission| self.has_permission(permission))
    }
}

/// 预取并构建安全快照。
///
/// 在进入模板渲染之前调用（async 上下文）。`login_id` 为 `None`（匿名）时返回
/// `Ok(None)`，调用方应注入匿名快照或直接跳过安全变量。
///
/// # 参数
///
/// - `manager`：共享的 `SaTokenManager`。
/// - `login_id`：当前请求的登录标识（来自 `VernalAuthentication` 或
///   `SaTokenContext::get_current()`）。
///
/// # Errors
///
/// 存储后端读取角色或权限失败时返回 [`SaTokenError`]。
pub async fn load_authentication(
    manager: &SaTokenManager,
    login_id: Option<&str>,
) -> Result<Option<SaTokenAuthentication>, SaTokenError> {
    let Some(login_id) = login_id else {
        return Ok(None);
    };
    let roles = manager.get_roles(login_id).await?;
    let permissions = manager.get_permissions(login_id).await?;
    Ok(Some(SaTokenAuthentication::new(
        login_id.to_owned(),
        roles.into_iter().map(Into::into).collect(),
        permissions.into_iter().map(Into::into).collect(),
    )))
}
