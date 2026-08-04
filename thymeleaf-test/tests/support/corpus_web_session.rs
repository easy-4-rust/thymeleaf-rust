use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::Utf16String;
use thymeleaf::web::IWebSession;

/// 为上游语料提供已建立会话的内存实现。
///
/// 对应 Java: `org.thymeleaf.testing.templateengine.context.web.WebProcessingContextBuilder`
/// 所创建的 `IWebSession` 测试对象。
#[derive(Default)]
pub struct CorpusWebSession {
    attributes: RwLock<IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>,
}

impl IWebSession for CorpusWebSession {
    fn exists(&self) -> bool {
        true
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
}
