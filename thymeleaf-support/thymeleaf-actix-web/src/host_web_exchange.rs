use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::{Locale, Utf16String};
use thymeleaf::web::{IWebApplication, IWebExchange, IWebRequest, IWebSession};

use crate::{HostWebRequest, HostWebSession};

/// Actix 请求、可选会话与应用作用域组成的 Thymeleaf Web exchange。
///
/// 对应 Java Servlet 的 `IServletWebExchange`、`JakartaServletWebExchange`
/// 与 `JavaxServletWebExchange`，但不依赖 Servlet/JVM。
pub struct HostWebExchange {
    request: Arc<HostWebRequest>,
    session: Option<Arc<HostWebSession>>,
    application: Arc<dyn IWebApplication>,
    attributes: RwLock<IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>,
    principal: Option<Arc<TemplateValue>>,
    locale: Option<Locale>,
    content_type: RwLock<Option<Utf16String>>,
    character_encoding: RwLock<Option<Utf16String>>,
}

impl HostWebExchange {
    /// 组合一次 Actix 请求的全部 Thymeleaf Web 作用域。对应 Java:
    /// `JakartaServletWebExchange#JakartaServletWebExchange`。
    ///
    /// # 参数
    /// - `request`：稳定的请求快照。
    /// - `session`：可选且可惰性建立的会话作用域。
    /// - `application`：应用全局作用域。
    /// - `principal`：宿主身份对象。
    /// - `locale`：响应 Locale。
    ///
    /// # 返回
    /// 聚合上述作用域的新 exchange。
    #[must_use]
    pub fn new(
        request: Arc<HostWebRequest>,
        session: Option<Arc<HostWebSession>>,
        application: Arc<dyn IWebApplication>,
        principal: Option<Arc<TemplateValue>>,
        locale: Option<Locale>,
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

    /// 设置渲染响应的内容类型。对应 Java:
    /// `JakartaServletWebExchange#getContentType` 所读取的响应状态。
    ///
    /// # 参数
    /// - `content_type`：MIME 文本；`None` 对应尚未设置。
    pub fn set_content_type(&self, content_type: Option<Utf16String>) {
        *write_lock(&self.content_type) = content_type;
    }

    /// 设置渲染响应的字符编码。对应 Java:
    /// `JakartaServletWebExchange#getCharacterEncoding` 所读取的响应状态。
    ///
    /// # 参数
    /// - `character_encoding`：字符集名称；`None` 对应尚未设置。
    pub fn set_character_encoding(&self, character_encoding: Option<Utf16String>) {
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

    fn get_locale(&self) -> Option<Locale> {
        self.locale.clone()
    }

    fn get_content_type(&self) -> Option<Utf16String> {
        read_lock(&self.content_type).clone()
    }

    fn get_character_encoding(&self) -> Option<Utf16String> {
        read_lock(&self.character_encoding).clone()
    }

    fn contains_attribute(&self, name: Option<&Utf16String>) -> bool {
        require_name(name);
        read_lock(&self.attributes).contains_key(&name.cloned())
    }

    fn get_attribute_count(&self) -> i32 {
        i32::try_from(read_lock(&self.attributes).len()).unwrap_or(i32::MAX)
    }

    fn get_all_attribute_names(&self) -> Vec<Option<Utf16String>> {
        read_lock(&self.attributes).keys().cloned().collect()
    }

    fn get_attribute_map(&self) -> IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>> {
        read_lock(&self.attributes).clone()
    }

    fn get_attribute_value(&self, name: Option<&Utf16String>) -> Option<Arc<TemplateValue>> {
        require_name(name);
        read_lock(&self.attributes)
            .get(&name.cloned())
            .cloned()
            .flatten()
    }

    fn set_attribute_value(&self, name: Option<Utf16String>, value: Option<Arc<TemplateValue>>) {
        require_owned_name(&name);
        if value.is_some() {
            write_lock(&self.attributes).insert(name, value);
        } else {
            write_lock(&self.attributes).shift_remove(&name);
        }
    }

    fn remove_attribute(&self, name: Option<&Utf16String>) {
        require_name(name);
        write_lock(&self.attributes).shift_remove(&name.cloned());
    }

    fn transform_url(&self, url: Option<&Utf16String>) -> Option<Utf16String> {
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

fn require_name(name: Option<&Utf16String>) {
    if name.is_none() {
        panic!("Name cannot be null");
    }
}

fn require_owned_name(name: &Option<Utf16String>) {
    if name.is_none() {
        panic!("Name cannot be null");
    }
}
