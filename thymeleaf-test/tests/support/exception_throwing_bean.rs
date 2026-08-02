use std::any::Any;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use thymeleaf::expression::{TemplateObject, TemplateObjectMethodError, TemplateValue};
use thymeleaf::util::JavaString;

/// 调用测试方法时稳定抛出 `Kapow!!` 的宿主对象。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.attrprocessors.model.ExceptionThrowingBean`。
pub struct ExceptionThrowingBean;

impl TemplateObject for ExceptionThrowingBean {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.templateengine.attrprocessors.model.ExceptionThrowingBean"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str(self.java_class_name())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_invoke_method(
        &self,
        method_name: &JavaString,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        (arguments.is_empty()
            && matches!(
                method_name.to_string_lossy().as_str(),
                "throwRuntimeException" | "throwException"
            ))
        .then(|| Err(Box::new(ExceptionThrowingBeanError) as TemplateObjectMethodError))
    }
}

#[derive(Debug)]
struct ExceptionThrowingBeanError;

impl Display for ExceptionThrowingBeanError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Kapow!!")
    }
}

impl std::error::Error for ExceptionThrowingBeanError {}
