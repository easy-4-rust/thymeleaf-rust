//! Web exchange —— 对应 Java `JakartaServletWebExchange`。
//!
//! `GTVGFilter` 在 `buildTemplateEngine` 之后通过 `application.buildExchange(...)`
//! 构造 exchange；Rust 侧直接组合请求、会话与应用并携带请求 Locale。
//!
//! 属性作用域承载 `WebContext` 的变量：`StandardEngineContextFactory` 把调用方
//! Context 变量写入 exchange 属性（对应 Java WebEngineContext 的共享身份语义），
//! 模板变量读取也以 exchange 属性为准 —— 对应 Java 交换对象的属性 Map。

use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::{JavaLocale, JavaString};
use thymeleaf::web::{IWebApplication, IWebExchange, IWebRequest, IWebSession};

use super::gtvg_web_application::GtvgWebApplication;
use super::gtvg_web_request::GtvgWebRequest;
use super::gtvg_web_session::GtvgWebSession;

/// 组合 Web 宿主对象。
pub struct GtvgWebExchange {
    request: GtvgWebRequest,
    session: Arc<GtvgWebSession>,
    application: Arc<GtvgWebApplication>,
    attributes: RwLock<IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    locale: JavaLocale,
}

impl GtvgWebExchange {
    /// 创建 exchange：请求参数、会话（含固定 user）与 Locale。
    #[must_use]
    pub fn new(request: GtvgWebRequest, locale: JavaLocale) -> Self {
        Self {
            request,
            session: Arc::new(GtvgWebSession::with_user()),
            application: Arc::new(GtvgWebApplication::default()),
            attributes: RwLock::new(IndexMap::new()),
            locale,
        }
    }
}

impl IWebExchange for GtvgWebExchange {
    fn get_request(&self) -> &dyn IWebRequest {
        &self.request
    }
    fn get_session(&self) -> Option<&dyn IWebSession> {
        Some(self.session.as_ref())
    }
    fn get_application(&self) -> &dyn IWebApplication {
        self.application.as_ref()
    }
    fn get_principal(&self) -> Option<Arc<TemplateValue>> {
        None
    }
    fn get_locale(&self) -> Option<JavaLocale> {
        Some(self.locale.clone())
    }
    fn get_content_type(&self) -> Option<JavaString> {
        None
    }
    fn get_character_encoding(&self) -> Option<JavaString> {
        None
    }
    fn contains_attribute(&self, name: Option<&JavaString>) -> bool {
        self.attributes
            .read()
            .expect("exchange lock")
            .contains_key(&name.cloned())
    }
    fn get_attribute_count(&self) -> i32 {
        self.attributes.read().expect("exchange lock").len() as i32
    }
    fn get_all_attribute_names(&self) -> Vec<Option<JavaString>> {
        self.attributes
            .read()
            .expect("exchange lock")
            .keys()
            .cloned()
            .collect()
    }
    fn get_attribute_map(&self) -> IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>> {
        self.attributes.read().expect("exchange lock").clone()
    }
    fn get_attribute_value(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>> {
        self.attributes
            .read()
            .expect("exchange lock")
            .get(&name.cloned())
            .cloned()
            .flatten()
    }
    fn set_attribute_value(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        self.attributes
            .write()
            .expect("exchange lock")
            .insert(name, value);
    }
    fn remove_attribute(&self, name: Option<&JavaString>) {
        self.attributes
            .write()
            .expect("exchange lock")
            .shift_remove(&name.cloned());
    }
    fn transform_url(&self, url: Option<&JavaString>) -> Option<JavaString> {
        url.cloned()
    }
}
