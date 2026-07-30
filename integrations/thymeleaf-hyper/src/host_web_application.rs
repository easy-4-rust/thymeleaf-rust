use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::JavaString;
use thymeleaf::web::IWebApplication;

/// Hyper 宿主的应用属性与资源根目录适配器。
///
/// 对应 Java Servlet 的 `IServletWebApplication`、`JakartaServletWebApplication`
/// 与 `JavaxServletWebApplication`，以中立 `IWebApplication` SPI 替代 ServletContext。
pub struct HostWebApplication {
    attributes: RwLock<IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    resource_roots: Vec<PathBuf>,
}

impl HostWebApplication {
    /// 使用有序资源根目录创建应用作用域。
    #[must_use]
    pub fn new(resource_roots: Vec<PathBuf>) -> Self {
        Self {
            attributes: RwLock::new(IndexMap::new()),
            resource_roots,
        }
    }

    fn resolve_resource(&self, path: Option<&JavaString>) -> Option<PathBuf> {
        let path = path?.to_string_lossy();
        let path = path.strip_prefix('/').unwrap_or(&path);
        self.resource_roots
            .iter()
            .map(|root| root.join(path))
            .find(|candidate| candidate.is_file())
    }
}

impl IWebApplication for HostWebApplication {
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
        let mut attributes = write_attributes(&self.attributes);
        if value.is_some() {
            attributes.insert(name, value);
        } else {
            attributes.shift_remove(&name);
        }
    }

    fn remove_attribute(&self, name: Option<&JavaString>) {
        write_attributes(&self.attributes).shift_remove(&name.cloned());
    }

    fn resource_exists(&self, path: Option<&JavaString>) -> bool {
        self.resolve_resource(path).is_some()
    }

    fn get_resource_as_stream(&self, path: Option<&JavaString>) -> Option<Box<dyn Read + Send>> {
        let file = File::open(self.resolve_resource(path)?).ok()?;
        Some(Box::new(BufReader::new(file)))
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
