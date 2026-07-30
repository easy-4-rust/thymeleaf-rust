use std::io::Read;
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::JavaString;
use thymeleaf::web::IWebApplication;

/// 为上游语料提供应用作用域的内存实现。
///
/// 对应 Java: `org.thymeleaf.testing.templateengine.context.web.WebProcessingContextBuilder`
/// 所创建的 `IWebApplication` 测试对象。
#[derive(Default)]
pub struct CorpusWebApplication {
    attributes: RwLock<IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
}

impl IWebApplication for CorpusWebApplication {
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

    fn get_resource_as_stream(&self, _path: Option<&JavaString>) -> Option<Box<dyn Read + Send>> {
        None
    }
}
