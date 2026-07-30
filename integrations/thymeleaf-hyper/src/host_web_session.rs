use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::JavaString;
use thymeleaf::web::IWebSession;

/// Hyper 应用可选会话后端的 Thymeleaf 适配器。
///
/// 对应 Java Servlet 的 `IServletWebSession`、`JakartaServletWebSession`
/// 与 `JavaxServletWebSession`；首次写入时把延迟会话标记为已创建。
pub struct HostWebSession {
    exists: AtomicBool,
    attributes: RwLock<IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
}

impl HostWebSession {
    /// 创建尚未建立的延迟会话。
    #[must_use]
    pub fn new() -> Self {
        Self {
            exists: AtomicBool::new(false),
            attributes: RwLock::new(IndexMap::new()),
        }
    }

    /// 创建已经由宿主建立的会话。
    #[must_use]
    pub fn existing() -> Self {
        Self {
            exists: AtomicBool::new(true),
            attributes: RwLock::new(IndexMap::new()),
        }
    }
}

impl Default for HostWebSession {
    fn default() -> Self {
        Self::new()
    }
}

impl IWebSession for HostWebSession {
    fn exists(&self) -> bool {
        self.exists.load(Ordering::Acquire)
    }

    fn contains_attribute(&self, name: Option<&JavaString>) -> bool {
        read_attributes(&self.attributes).contains_key(&name.cloned())
    }

    fn get_attribute_count(&self) -> i32 {
        i32::try_from(read_attributes(&self.attributes).len()).unwrap_or(i32::MAX)
    }

    fn get_all_attribute_names(&self) -> Vec<Option<JavaString>> {
        read_attributes(&self.attributes).keys().cloned().collect()
    }

    fn get_attribute_map(&self) -> IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>> {
        read_attributes(&self.attributes).clone()
    }

    fn get_attribute_value(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>> {
        read_attributes(&self.attributes)
            .get(&name.cloned())
            .cloned()
            .flatten()
    }

    fn set_attribute_value(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        if value.is_some() {
            self.exists.store(true, Ordering::Release);
            write_attributes(&self.attributes).insert(name, value);
        } else {
            write_attributes(&self.attributes).shift_remove(&name);
        }
    }

    fn remove_attribute(&self, name: Option<&JavaString>) {
        write_attributes(&self.attributes).shift_remove(&name.cloned());
    }
}

fn read_attributes<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_attributes<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
