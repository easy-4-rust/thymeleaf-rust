use std::any::Any;
use std::sync::Arc;

use thymeleaf::expression::{TemplateObject, TemplateValue};
use thymeleaf::util::Utf16String;

/// 保存单个值的上游 lazy `.thtest` 宿主对象。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.features.lazy.ValueLazyContextVariable`。
pub struct ValueLazyContextVariable {
    value: Option<Arc<TemplateValue>>,
}

impl ValueLazyContextVariable {
    /// 创建保存指定 Java 测试夹具返回值的 lazy 对象。
    pub fn new(value: Option<Arc<TemplateValue>>) -> Self {
        Self { value }
    }
}

impl TemplateObject for ValueLazyContextVariable {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.templateengine.features.lazy.ValueLazyContextVariable"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(self.java_class_name())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn resolve_lazy_context_variable(&self) -> Option<Option<Arc<TemplateValue>>> {
        Some(self.value.clone())
    }
}
