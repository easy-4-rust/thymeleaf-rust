//! Vernal 请求上下文到 Thymeleaf `IWebExchange` 的适配。
//!
//! 对应 Java `org.thymeleaf.web.IWebExchange` 的 Vernal 宿主实现：把
//! `vernal_web::RequestContext`（含 HttpRequestSnapshot、SecurityPrincipal、路由
//! 元数据）适配为 Thymeleaf Web 交换。`get_principal()` 把已认证的
//! `SecurityPrincipal` 包装为 `TemplateValue::Object`，使
//! `thymeleaf-sa-token` 的 sec 方言与 `#authentication` 表达式对象可以直接消费
//! Vernal 安全主体（无需单独预取权限；角色已在 `SecurityPrincipal` 中）。

use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::expression::{TemplateObject, TemplateValue};
use thymeleaf::util::{JavaLocale, JavaString};
use thymeleaf::web::{IWebApplication, IWebExchange, IWebRequest, IWebSession};
use vernal_web::{RequestContext, SecurityPrincipal};

use crate::web_request::VernalWebRequest;

/// 把 Vernal 请求上下文适配为 Thymeleaf Web 交换。
pub struct VernalWebExchange {
    request: VernalWebRequest,
    request_context: Arc<RequestContext>,
    principal_snapshot: RwLock<Option<Arc<SecurityPrincipal>>>,
    session: Option<Arc<VernalWebSession>>,
    application: Arc<VernalWebApplication>,
    attributes: RwLock<IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
}

impl VernalWebExchange {
    /// 从共享请求上下文创建 Web 交换。
    ///
    /// 快照从 `request_context.extensions()` 中查找 `HttpRequestSnapshot`；
    /// 缺失时构造空快照（请求仍可访问方法/URI 的兜底值）。
    ///
    /// # 参数
    ///
    /// - `request_context`：当前请求的共享 Vernal 上下文。
    /// - `snapshot`：请求的 HTTP 元数据快照。
    #[must_use]
    pub fn new(
        request_context: Arc<RequestContext>,
        snapshot: Arc<vernal_http::HttpRequestSnapshot>,
    ) -> Self {
        Self {
            request: VernalWebRequest::new(snapshot),
            request_context,
            principal_snapshot: RwLock::new(None),
            session: None,
            application: Arc::new(VernalWebApplication::default()),
            attributes: RwLock::new(IndexMap::new()),
        }
    }

    /// 返回被包装的 Vernal 请求上下文。
    #[must_use]
    pub const fn request_context(&self) -> &Arc<RequestContext> {
        &self.request_context
    }

    /// 同步注入已认证主体（渲染入口在 async 上下文已读取 principal 时使用）。
    ///
    /// 该路径避免模板渲染期间再次进入异步 principal 读取；`thymeleaf-sa-token`
    /// 的 sec 方言处理器是同步的，无法 `.await`。
    pub fn set_principal_snapshot(&self, principal: Option<Arc<SecurityPrincipal>>) {
        *self
            .principal_snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = principal;
    }

    /// 当前请求的已认证主体。
    ///
    /// 优先返回 [`Self::set_principal_snapshot`] 注入的同步快照；未注入时尝试从
    /// Vernal `RequestContext` 读取（仅在 Tokio 运行时外可用，此时使用
    /// `futures_executor::block_on`）。
    #[must_use]
    pub fn principal(&self) -> Option<Arc<SecurityPrincipal>> {
        if let Some(principal) = self
            .principal_snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
        {
            return Some(Arc::clone(principal));
        }
        // 运行时内不可 block_on；由调用方经 set_principal_snapshot 注入。
        if tokio::runtime::Handle::try_current().is_ok() {
            return None;
        }
        futures_executor::block_on(self.request_context.principal())
    }
}

impl IWebExchange for VernalWebExchange {
    fn get_request(&self) -> &dyn IWebRequest {
        &self.request
    }

    fn get_session(&self) -> Option<&dyn IWebSession> {
        self.session
            .as_ref()
            .map(|session| session.as_ref() as &dyn IWebSession)
    }

    fn get_application(&self) -> &dyn IWebApplication {
        self.application.as_ref()
    }

    fn get_principal(&self) -> Option<Arc<TemplateValue>> {
        self.principal().map(|principal| {
            Arc::new(TemplateValue::Object(Arc::new(VernalPrincipalObject::new(
                principal,
            ))))
        })
    }

    fn get_locale(&self) -> Option<JavaLocale> {
        // Vernal 不携带请求 Locale；保留 None（宿主可覆盖）。
        None
    }

    fn get_content_type(&self) -> Option<JavaString> {
        None
    }

    fn get_character_encoding(&self) -> Option<JavaString> {
        Some(JavaString::from_rust_str("UTF-8"))
    }

    fn contains_attribute(&self, name: Option<&JavaString>) -> bool {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(&name.cloned())
    }

    fn get_attribute_count(&self) -> i32 {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len() as i32
    }

    fn get_all_attribute_names(&self) -> Vec<Option<JavaString>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .keys()
            .cloned()
            .collect()
    }

    fn get_attribute_map(&self) -> IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn get_attribute_value(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&name.cloned())
            .cloned()
            .flatten()
    }

    fn set_attribute_value(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        let mut attributes = self
            .attributes
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if value.is_some() {
            attributes.insert(name, value);
        } else {
            attributes.shift_remove(&name);
        }
    }

    fn remove_attribute(&self, name: Option<&JavaString>) {
        self.attributes
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .shift_remove(&name.cloned());
    }

    fn transform_url(&self, url: Option<&JavaString>) -> Option<JavaString> {
        url.cloned()
    }
}

/// 把 `SecurityPrincipal` 包装为 Thymeleaf 动态对象（`subject`/`roles`/`hasRole`）。
struct VernalPrincipalObject {
    principal: Arc<SecurityPrincipal>,
}

impl VernalPrincipalObject {
    const fn new(principal: Arc<SecurityPrincipal>) -> Self {
        Self { principal }
    }
}

impl TemplateObject for VernalPrincipalObject {
    fn java_class_name(&self) -> &str {
        "org.springframework.security.core.Authentication"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str(self.principal.subject())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn java_serializable_properties(
        &self,
    ) -> Option<Vec<(JavaString, Option<Arc<TemplateValue>>)>> {
        Some(vec![
            (
                JavaString::from_rust_str("name"),
                Some(Arc::new(TemplateValue::string(self.to_java_string()))),
            ),
            (
                JavaString::from_rust_str("roles"),
                Some(Arc::new(TemplateValue::List(Arc::new(
                    self.principal
                        .roles()
                        .iter()
                        .map(|role| {
                            Arc::new(TemplateValue::string(JavaString::from_rust_str(role)))
                        })
                        .collect(),
                )))),
            ),
        ])
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<
        Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectPropertyError>,
    > {
        let value = match property_name.to_string_lossy().as_str() {
            "name" | "subject" => Some(Arc::new(TemplateValue::string(self.to_java_string()))),
            "roles" => Some(Arc::new(TemplateValue::List(Arc::new(
                self.principal
                    .roles()
                    .iter()
                    .map(|role| Arc::new(TemplateValue::string(JavaString::from_rust_str(role))))
                    .collect(),
            )))),
            _ => return None,
        };
        Some(Ok(value))
    }

    fn java_invoke_method(
        &self,
        method_name: &JavaString,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, thymeleaf::expression::TemplateObjectMethodError>>
    {
        let role = arguments.first()?.as_ref()?;
        let TemplateValue::String(role) = role.as_ref() else {
            return None;
        };
        let result = match method_name.to_string_lossy().as_str() {
            "hasRole" | "has_role" => self.principal.has_role(&role.to_string_lossy()),
            _ => return None,
        };
        Some(Ok(Some(Arc::new(TemplateValue::Boolean(result)))))
    }
}

/// Vernal Web 会话（当前无状态实现，保留属性作用域接口）。
pub struct VernalWebSession {
    attributes: RwLock<IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
}

impl Default for VernalWebSession {
    fn default() -> Self {
        Self {
            attributes: RwLock::new(IndexMap::new()),
        }
    }
}

impl IWebSession for VernalWebSession {
    fn exists(&self) -> bool {
        true
    }
    fn contains_attribute(&self, name: Option<&JavaString>) -> bool {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(&name.cloned())
    }
    fn get_attribute_count(&self) -> i32 {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len() as i32
    }
    fn get_all_attribute_names(&self) -> Vec<Option<JavaString>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .keys()
            .cloned()
            .collect()
    }
    fn get_attribute_map(&self) -> IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
    fn get_attribute_value(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&name.cloned())
            .cloned()
            .flatten()
    }
    fn set_attribute_value(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        let mut attributes = self
            .attributes
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if value.is_some() {
            attributes.insert(name, value);
        } else {
            attributes.shift_remove(&name);
        }
    }
    fn remove_attribute(&self, name: Option<&JavaString>) {
        self.attributes
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .shift_remove(&name.cloned());
    }
}

/// Vernal Web 应用（当前无状态实现）。
#[derive(Default)]
pub struct VernalWebApplication {
    attributes: RwLock<IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
}

impl IWebApplication for VernalWebApplication {
    fn contains_attribute(&self, name: Option<&JavaString>) -> bool {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(&name.cloned())
    }
    fn get_attribute_count(&self) -> i32 {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len() as i32
    }
    fn get_all_attribute_names(&self) -> Vec<Option<JavaString>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .keys()
            .cloned()
            .collect()
    }
    fn get_attribute_map(&self) -> IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
    fn get_attribute_value(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&name.cloned())
            .cloned()
            .flatten()
    }
    fn set_attribute_value(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        let mut attributes = self
            .attributes
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if value.is_some() {
            attributes.insert(name, value);
        } else {
            attributes.shift_remove(&name);
        }
    }
    fn remove_attribute(&self, name: Option<&JavaString>) {
        self.attributes
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .shift_remove(&name.cloned());
    }
    fn resource_exists(&self, _path: Option<&JavaString>) -> bool {
        false
    }
    fn get_resource_as_stream(
        &self,
        _path: Option<&JavaString>,
    ) -> Option<Box<dyn std::io::Read + Send>> {
        None
    }
}
