use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::{JavaLocale, Utf16String};
use thymeleaf::web::{IWebApplication, IWebExchange, IWebRequest, IWebSession};

use super::{CorpusWebApplication, CorpusWebRequest, CorpusWebSession};

/// 为上游语料提供请求、会话、应用三层属性作用域。
///
/// 对应 Java: `org.thymeleaf.testing.templateengine.context.web.WebProcessingContextBuilder`
/// 所创建的 `IWebExchange` 测试对象。
pub struct CorpusWebExchange {
    request: CorpusWebRequest,
    session: Arc<CorpusWebSession>,
    application: Arc<CorpusWebApplication>,
    attributes: RwLock<IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>,
}

impl CorpusWebExchange {
    /// 创建空 exchange；会话与应用对象保持稳定共享身份。
    #[must_use]
    pub fn new() -> Self {
        Self {
            request: CorpusWebRequest,
            session: Arc::new(CorpusWebSession::default()),
            application: Arc::new(CorpusWebApplication::default()),
            attributes: RwLock::new(IndexMap::new()),
        }
    }
}

impl Default for CorpusWebExchange {
    fn default() -> Self {
        Self::new()
    }
}

impl IWebExchange for CorpusWebExchange {
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
        Some(JavaLocale::new(
            Utf16String::from_rust_str("en"),
            Utf16String::from_rust_str(""),
        ))
    }
    fn get_content_type(&self) -> Option<Utf16String> {
        None
    }
    fn get_character_encoding(&self) -> Option<Utf16String> {
        Some(Utf16String::from_rust_str("UTF-8"))
    }
    fn contains_attribute(&self, name: Option<&Utf16String>) -> bool {
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
    fn get_all_attribute_names(&self) -> Vec<Option<Utf16String>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .keys()
            .cloned()
            .collect()
    }
    fn get_attribute_map(&self) -> IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
    fn get_attribute_value(&self, name: Option<&Utf16String>) -> Option<Arc<TemplateValue>> {
        self.attributes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&name.cloned())
            .cloned()
            .flatten()
    }
    fn set_attribute_value(&self, name: Option<Utf16String>, value: Option<Arc<TemplateValue>>) {
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
    fn remove_attribute(&self, name: Option<&Utf16String>) {
        self.attributes
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .shift_remove(&name.cloned());
    }
    fn transform_url(&self, url: Option<&Utf16String>) -> Option<Utf16String> {
        url.cloned()
    }
}
