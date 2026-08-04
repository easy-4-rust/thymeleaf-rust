use std::any::Any;
use std::sync::{Arc, Mutex};

use crate::util::Utf16String;

use super::{TemplateObject, TemplateObjectMethodError, TemplateValue};

/// OGNL 可见的 Java Iterator 顺序快照。
///
/// 对应 Java: `java.util.Iterator`。
pub(crate) struct IteratorValue {
    values: Arc<Vec<Arc<TemplateValue>>>,
    position: Mutex<usize>,
}

impl IteratorValue {
    /// 从集合当前顺序创建迭代器。
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub(crate) fn new(values: Arc<Vec<Arc<TemplateValue>>>) -> Self {
        Self {
            values,
            position: Mutex::new(0),
        }
    }
}

impl TemplateObject for IteratorValue {
    fn class_name(&self) -> &str {
        "java.util.Iterator"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str("java.util.Iterator")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn invoke_method(
        &self,
        method_name: &Utf16String,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        let method_name = method_name.to_string_lossy();
        let mut position = self
            .position
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match (method_name.as_str(), arguments) {
            ("hasNext", []) => Some(Ok(Some(Arc::new(TemplateValue::Boolean(
                *position < self.values.len(),
            ))))),
            ("next", []) => {
                let value = self.values.get(*position).cloned();
                *position = position.saturating_add(1);
                Some(
                    value
                        .map(Some)
                        .ok_or_else(|| "java.util.NoSuchElementException".to_owned().into()),
                )
            }
            _ => None,
        }
    }
}
