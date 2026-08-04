use std::any::Any;
use std::sync::Arc;

use thymeleaf::expression::{TemplateObject, TemplateObjectMethodError, TemplateValue};
use thymeleaf::util::Utf16String;

/// 返回匿名 LazyContextVariable 的测试宿主。
///
/// 对应 Java: `org.thymeleaf.templateengine.features.lazy.LazyExpressionReturner`。
pub struct LazyExpressionReturner;

impl TemplateObject for LazyExpressionReturner {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.templateengine.features.lazy.LazyExpressionReturner"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(self.java_class_name())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_invoke_method(
        &self,
        method_name: &Utf16String,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        (method_name.to_string_lossy() == "doSomething" && arguments.is_empty()).then(|| {
            Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
                LazyExpressionValue,
            )))))
        })
    }
}

/// Java 匿名内部 LazyContextVariable；按主对象的内部类规则同文件保存。
struct LazyExpressionValue;

impl TemplateObject for LazyExpressionValue {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.templateengine.features.lazy.LazyExpressionReturner$1"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(
            "org.thymeleaf.templateengine.features.lazy.LazyExpressionReturner$1@1",
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn resolve_lazy_context_variable(&self) -> Option<Option<Arc<TemplateValue>>> {
        Some(Some(Arc::new(TemplateValue::string(
            Utf16String::from_rust_str("The lazy value"),
        ))))
    }
}
