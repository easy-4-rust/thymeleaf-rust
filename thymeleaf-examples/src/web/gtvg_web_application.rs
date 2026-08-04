//! 应用对象 —— 对应 Java `ServletContext`（GTVG 不使用应用属性）。

use std::io::Read;
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::Utf16String;
use thymeleaf::web::IWebApplication;

/// 空应用作用域。
#[derive(Default)]
pub struct GtvgWebApplication {
    attributes: RwLock<IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>,
}

impl IWebApplication for GtvgWebApplication {
    fn contains_attribute(&self, name: Option<&Utf16String>) -> bool {
        self.attributes
            .read()
            .expect("application lock")
            .contains_key(&name.cloned())
    }
    fn get_attribute_count(&self) -> i32 {
        self.attributes.read().expect("application lock").len() as i32
    }
    fn get_all_attribute_names(&self) -> Vec<Option<Utf16String>> {
        self.attributes
            .read()
            .expect("application lock")
            .keys()
            .cloned()
            .collect()
    }
    fn get_attribute_map(&self) -> IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>> {
        self.attributes.read().expect("application lock").clone()
    }
    fn get_attribute_value(&self, name: Option<&Utf16String>) -> Option<Arc<TemplateValue>> {
        self.attributes
            .read()
            .expect("application lock")
            .get(&name.cloned())
            .cloned()
            .flatten()
    }
    fn set_attribute_value(&self, name: Option<Utf16String>, value: Option<Arc<TemplateValue>>) {
        self.attributes
            .write()
            .expect("application lock")
            .insert(name, value);
    }
    fn remove_attribute(&self, name: Option<&Utf16String>) {
        self.attributes
            .write()
            .expect("application lock")
            .shift_remove(&name.cloned());
    }
    fn resource_exists(&self, _path: Option<&Utf16String>) -> bool {
        false
    }
    fn get_resource_as_stream(&self, _path: Option<&Utf16String>) -> Option<Box<dyn Read + Send>> {
        None
    }
}
