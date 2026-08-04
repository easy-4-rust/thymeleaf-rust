use std::any::Any;
use std::sync::Arc;

use thymeleaf::expression::{TemplateObject, TemplateObjectPropertyError, TemplateValue};
use thymeleaf::util::{NumberValue, Utf16String};

/// 上游测试语料使用的 Java `String[]` 对象。
///
/// 对应 Java: `java.lang.String[]`。保留数组类名、默认 `toString()`、长度和
/// 迭代顺序，仅用于受白名单约束的测试 OGNL 运行时。
pub struct CorpusStringArray {
    values: Vec<Arc<TemplateValue>>,
}

impl CorpusStringArray {
    /// 使用已经求值的 Java 字符串元素创建数组。
    #[must_use]
    pub fn new(values: Vec<Arc<TemplateValue>>) -> Self {
        Self { values }
    }
}

impl TemplateObject for CorpusStringArray {
    fn java_class_name(&self) -> &str {
        "[Ljava.lang.String;"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(&format!(
            "[Ljava.lang.String;@{:x}",
            self as *const Self as usize
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_iterable_values(&self) -> Option<Vec<Arc<TemplateValue>>> {
        Some(self.values.clone())
    }

    fn java_get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        (property_name == &Utf16String::from_rust_str("length")).then(|| {
            Ok(Some(Arc::new(TemplateValue::Number(NumberValue::Integer(
                i32::try_from(self.values.len()).unwrap_or(i32::MAX),
            )))))
        })
    }
}
