use std::any::Any;
use std::sync::Arc;

use crate::util::JavaString;

use super::{TemplateObject, TemplateObjectPropertyError, TemplateValue};

/// OGNL 可见的 Java Map.Entry 快照。
///
/// 对应 Java: `java.util.Map.Entry`。
pub(crate) struct JavaMapEntry {
    key: Arc<TemplateValue>,
    value: Arc<TemplateValue>,
}

impl JavaMapEntry {
    /// 创建键值条目快照。
    pub(crate) fn new(key: Arc<TemplateValue>, value: Arc<TemplateValue>) -> Self {
        Self { key, value }
    }
}

impl TemplateObject for JavaMapEntry {
    fn java_class_name(&self) -> &str {
        "java.util.Map$Entry"
    }

    fn to_java_string(&self) -> JavaString {
        let key = self
            .key
            .to_java_string()
            .unwrap_or_else(|| JavaString::from_rust_str("null"));
        let value = self
            .value
            .to_java_string()
            .unwrap_or_else(|| JavaString::from_rust_str("null"));
        JavaString::from_rust_str(&format!(
            "{}={}",
            key.to_string_lossy(),
            value.to_string_lossy()
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        match property_name.to_string_lossy().as_str() {
            "key" => Some(Ok(Some(Arc::clone(&self.key)))),
            "value" => Some(Ok(Some(Arc::clone(&self.value)))),
            _ => None,
        }
    }
}
