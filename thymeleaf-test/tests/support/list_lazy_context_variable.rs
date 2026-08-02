use std::any::Any;
use std::sync::Arc;

use thymeleaf::expression::{TemplateObject, TemplateValue};
use thymeleaf::util::JavaString;

/// 构造五元素列表的上游 lazy `.thtest` 宿主对象。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.features.lazy.ListLazyContextVariable`。
pub struct ListLazyContextVariable;

impl ListLazyContextVariable {
    /// 创建无状态的列表 lazy 对象。
    pub const fn new() -> Self {
        Self
    }
}

impl TemplateObject for ListLazyContextVariable {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.templateengine.features.lazy.ListLazyContextVariable"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str(self.java_class_name())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn resolve_lazy_context_variable(&self) -> Option<Option<Arc<TemplateValue>>> {
        let values = ["one", "two", "three", "four", "five"]
            .into_iter()
            .map(|value| Arc::new(TemplateValue::string(JavaString::from_rust_str(value))))
            .collect();
        Some(Some(Arc::new(TemplateValue::List(Arc::new(values)))))
    }
}
