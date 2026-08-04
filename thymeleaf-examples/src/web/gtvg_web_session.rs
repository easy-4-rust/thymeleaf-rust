//! 会话对象 —— 对应 Java `HttpSession`（GTVG 仅存放 `user` 属性）。

use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::Utf16String;
use thymeleaf::web::IWebSession;

use crate::business::entities::User;

/// 模拟真实用户会话：`GTVGFilter#addUserToSession` 在每次请求前写入
/// `User("John", "Apricot", "Antarctica", null)`。
#[derive(Default)]
pub struct GtvgWebSession {
    attributes: RwLock<IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>,
}

impl GtvgWebSession {
    /// 创建会话并写入 Java 过滤器的固定用户。
    #[must_use]
    pub fn with_user() -> Self {
        let session = Self::default();
        session.set_attribute_value(
            Some(Utf16String::from_rust_str("user")),
            Some(Arc::new(TemplateValue::Object(Arc::new(User {
                first_name: "John".to_owned(),
                last_name: "Apricot".to_owned(),
                nationality: "Antarctica".to_owned(),
                age: None,
            })))),
        );
        session
    }
}

impl IWebSession for GtvgWebSession {
    fn exists(&self) -> bool {
        true
    }
    fn contains_attribute(&self, name: Option<&Utf16String>) -> bool {
        self.attributes
            .read()
            .expect("session lock")
            .contains_key(&name.cloned())
    }
    fn get_attribute_count(&self) -> i32 {
        self.attributes.read().expect("session lock").len() as i32
    }
    fn get_all_attribute_names(&self) -> Vec<Option<Utf16String>> {
        self.attributes
            .read()
            .expect("session lock")
            .keys()
            .cloned()
            .collect()
    }
    fn get_attribute_map(&self) -> IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>> {
        self.attributes.read().expect("session lock").clone()
    }
    fn get_attribute_value(&self, name: Option<&Utf16String>) -> Option<Arc<TemplateValue>> {
        self.attributes
            .read()
            .expect("session lock")
            .get(&name.cloned())
            .cloned()
            .flatten()
    }
    fn set_attribute_value(&self, name: Option<Utf16String>, value: Option<Arc<TemplateValue>>) {
        self.attributes
            .write()
            .expect("session lock")
            .insert(name, value);
    }
    fn remove_attribute(&self, name: Option<&Utf16String>) {
        self.attributes
            .write()
            .expect("session lock")
            .shift_remove(&name.cloned());
    }
}
