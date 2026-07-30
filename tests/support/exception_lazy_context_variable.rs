use std::any::Any;
use std::fmt::{Display, Formatter};
use std::panic::panic_any;
use std::sync::Arc;

use thymeleaf::expression::{TemplateObject, TemplateValue};
use thymeleaf::util::JavaString;

/// 首次读取时抛出固定运行时异常的惰性测试变量。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.features.lazy.ExceptionLazyContextVariable`。
pub struct ExceptionLazyContextVariable;

impl TemplateObject for ExceptionLazyContextVariable {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.templateengine.features.lazy.ExceptionLazyContextVariable"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str(self.java_class_name())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn resolve_lazy_context_variable(&self) -> Option<Option<Arc<TemplateValue>>> {
        panic_any(ExceptionLazyContextVariableError)
    }
}

/// `ExceptionLazyContextVariable#loadValue` 抛出的 Java RuntimeException 等价对象。
#[derive(Debug)]
pub struct ExceptionLazyContextVariableError;

impl Display for ExceptionLazyContextVariableError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("This should have never been called!!")
    }
}

impl std::error::Error for ExceptionLazyContextVariableError {}
