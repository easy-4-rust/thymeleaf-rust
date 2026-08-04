use std::any::Any;
use std::sync::Arc;

use crate::util::Utf16String;

use super::{TemplateObject, TemplateObjectPropertyError, TemplateValue};

/// OGNL 可见的 Java Map.Entry 快照。
///
/// 对应 Java: `java.util.Map.Entry`。
pub(crate) struct MapEntryValue {
    key: Arc<TemplateValue>,
    value: Arc<TemplateValue>,
}

impl MapEntryValue {
    /// 创建键值条目快照。
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub(crate) fn new(key: Arc<TemplateValue>, value: Arc<TemplateValue>) -> Self {
        Self { key, value }
    }
}

impl TemplateObject for MapEntryValue {
    fn class_name(&self) -> &str {
        "java.util.Map$Entry"
    }

    fn to_utf16_string(&self) -> Utf16String {
        let key = self
            .key
            .to_utf16_string()
            .unwrap_or_else(|| Utf16String::from_rust_str("null"));
        let value = self
            .value
            .to_utf16_string()
            .unwrap_or_else(|| Utf16String::from_rust_str("null"));
        Utf16String::from_rust_str(&format!(
            "{}={}",
            key.to_string_lossy(),
            value.to_string_lossy()
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        match property_name.to_string_lossy().as_str() {
            "key" => Some(Ok(Some(Arc::clone(&self.key)))),
            "value" => Some(Ok(Some(Arc::clone(&self.value)))),
            _ => None,
        }
    }
}
