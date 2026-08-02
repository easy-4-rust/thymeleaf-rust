use std::any::Any;
use std::sync::Arc;

use thymeleaf::expression::{TemplateObject, TemplateObjectMethodError, TemplateValue};
use thymeleaf::util::JavaString;

/// 上游语料所需的 Java Optional 族只读宿主值。
///
/// 对应 Java: `java.util.Optional`、`OptionalInt`、`OptionalLong`、`OptionalDouble`。
pub struct CorpusOptional {
    class_name: &'static str,
    value: Option<Arc<TemplateValue>>,
}

impl CorpusOptional {
    /// 创建指定 Optional 类型的非空值。
    pub fn new(class_name: &'static str, value: Arc<TemplateValue>) -> Self {
        Self {
            class_name,
            value: Some(value),
        }
    }

    /// 创建指定 Optional 类型的空值。
    pub fn empty(class_name: &'static str) -> Self {
        Self {
            class_name,
            value: None,
        }
    }
}

impl TemplateObject for CorpusOptional {
    fn java_class_name(&self) -> &str {
        self.class_name
    }

    fn to_java_string(&self) -> JavaString {
        self.value.as_ref().map_or_else(
            || JavaString::from_rust_str("Optional.empty"),
            |value| {
                JavaString::from_rust_str(&format!(
                    "Optional[{}]",
                    value
                        .to_java_string()
                        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy())
                ))
            },
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_serializable_value(&self) -> Option<Option<Arc<TemplateValue>>> {
        Some(self.value.clone())
    }

    fn java_invoke_method(
        &self,
        method_name: &JavaString,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        match (method_name.to_string_lossy().as_str(), arguments) {
            ("orElse", [fallback]) => Some(Ok(self.value.clone().or_else(|| fallback.clone()))),
            ("get", []) | ("getAsInt", []) | ("getAsLong", []) | ("getAsDouble", []) => {
                Some(Ok(self.value.clone()))
            }
            ("isPresent", []) => Some(Ok(Some(Arc::new(TemplateValue::Boolean(
                self.value.is_some(),
            ))))),
            _ => None,
        }
    }
}
