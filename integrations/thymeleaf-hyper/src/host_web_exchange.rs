use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::{JavaLocale, JavaString};
use thymeleaf::web::{IWebApplication, IWebExchange, IWebRequest, IWebSession};

use crate::{HostWebRequest, HostWebSession};

/// Hyper 请求、可选会话与应用作用域组成的 Thymeleaf Web exchange。
///
/// 对应 Java Servlet 的 `IServletWebExchange`、`JakartaServletWebExchange`
/// 与 `JavaxServletWebExchange`，但不依赖 Servlet/JVM。
pub struct HostWebExchange {
    request: Arc<HostWebRequest>,
    session: Option<Arc<HostWebSession>>,
    application: Arc<dyn IWebApplication>,
    attributes: RwLock<IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    principal: Option<Arc<TemplateValue>>,
    locale: Option<JavaLocale>,
    content_type: RwLock<Option<JavaString>>,
    character_encoding: RwLock<Option<JavaString>>,
}

impl HostWebExchange {
    /// 组合一次 Hyper 请求的全部 Thymeleaf Web 作用域。
    #[must_use]
    pub fn new(
        request: Arc<HostWebRequest>,
        session: Option<Arc<HostWebSession>>,
        application: Arc<dyn IWebApplication>,
        principal: Option<Arc<TemplateValue>>,
        locale: Option<JavaLocale>,
    ) -> Self {
        Self {
            request,
            session,
            application,
            attributes: RwLock::new(IndexMap::new()),
            principal,
            locale,
            content_type: RwLock::new(None),
            character_encoding: RwLock::new(None),
        }
    }

    /// 设置渲染响应的内容类型。
    pub fn set_content_type(&self, content_type: Option<JavaString>) {
        *write_lock(&self.content_type) = content_type;
    }

    /// 设置渲染响应的字符编码。
    pub fn set_character_encoding(&self, character_encoding: Option<JavaString>) {
        *write_lock(&self.character_encoding) = character_encoding;
    }
}

impl IWebExchange for HostWebExchange {
    fn get_request(&self) -> &dyn IWebRequest {
        self.request.as_ref()
    }

    fn get_session(&self) -> Option<&dyn IWebSession> {
        self.session
            .as_deref()
            .map(|session| session as &dyn IWebSession)
    }

    fn get_application(&self) -> &dyn IWebApplication {
        self.application.as_ref()
    }

    fn get_principal(&self) -> Option<Arc<TemplateValue>> {
        self.principal.clone()
    }

    fn get_locale(&self) -> Option<JavaLocale> {
        self.locale.clone()
    }

    fn get_content_type(&self) -> Option<JavaString> {
        read_lock(&self.content_type).clone()
    }

    fn get_character_encoding(&self) -> Option<JavaString> {
        read_lock(&self.character_encoding).clone()
    }

    fn contains_attribute(&self, name: Option<&JavaString>) -> bool {
        read_lock(&self.attributes).contains_key(&name.cloned())
    }

    fn get_attribute_count(&self) -> i32 {
        i32::try_from(read_lock(&self.attributes).len()).unwrap_or(i32::MAX)
    }

    fn get_all_attribute_names(&self) -> Vec<Option<JavaString>> {
        read_lock(&self.attributes).keys().cloned().collect()
    }

    fn get_attribute_map(&self) -> IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>> {
        read_lock(&self.attributes).clone()
    }

    fn get_attribute_value(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>> {
        read_lock(&self.attributes)
            .get(&name.cloned())
            .cloned()
            .flatten()
    }

    fn set_attribute_value(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        if value.is_some() {
            write_lock(&self.attributes).insert(name, value);
        } else {
            write_lock(&self.attributes).shift_remove(&name);
        }
    }

    fn remove_attribute(&self, name: Option<&JavaString>) {
        write_lock(&self.attributes).shift_remove(&name.cloned());
    }

    fn transform_url(&self, url: Option<&JavaString>) -> Option<JavaString> {
        url.cloned()
    }
}

fn read_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
