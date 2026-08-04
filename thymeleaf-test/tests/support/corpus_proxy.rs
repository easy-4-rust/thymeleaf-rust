use std::any::Any;
use std::sync::Arc;

use thymeleaf::expression::{TemplateObject, TemplateObjectMethodError, TemplateValue};
use thymeleaf::util::{NumberValue, Utf16String};

/// 上游动态代理访问限制语料的宿主代理。
///
/// 对应 Java: `org.thymeleaf.templateengine.features.TestProxyFactory` 创建的代理对象。
pub struct CorpusProxy;

impl TemplateObject for CorpusProxy {
    fn class_name(&self) -> &str {
        "org.thymeleaf.templateengine.features.ITestInterface"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(self.class_name())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn invoke_method(
        &self,
        method_name: &Utf16String,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        (method_name.to_string_lossy() == "getValue" && arguments.is_empty()).then(|| {
            Ok(Some(Arc::new(TemplateValue::Number(NumberValue::Integer(
                10,
            )))))
        })
    }
}
