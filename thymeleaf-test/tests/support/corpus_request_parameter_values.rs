use std::any::Any;
use std::sync::Arc;

use thymeleaf::expression::{TemplateObject, TemplateObjectPropertyError, TemplateValue};
use thymeleaf::util::{NumberValue, Utf16String};

/// 上游 Web 测试上下文中的同名请求参数值。
///
/// 对应 Java: `WebProcessingContextBuilder` 写入并由测试 Web exchange 暴露的
/// `String[]` 参数。属性读取支持 `length` 和索引，直接输出采用第一个值。
pub struct CorpusRequestParameterValues {
    values: Vec<Arc<TemplateValue>>,
}

impl CorpusRequestParameterValues {
    /// 创建只含首个请求参数值的集合。
    #[must_use]
    pub fn new(value: Arc<TemplateValue>) -> Self {
        Self {
            values: vec![value],
        }
    }

    /// 返回追加一个同名参数值后的独立集合。
    #[must_use]
    pub fn with_appended(&self, value: Arc<TemplateValue>) -> Self {
        let mut values = self.values.clone();
        values.push(value);
        Self { values }
    }
}

impl TemplateObject for CorpusRequestParameterValues {
    fn java_class_name(&self) -> &str {
        "[Ljava.lang.String;"
    }

    fn to_utf16_string(&self) -> Utf16String {
        self.values
            .first()
            .and_then(|value| value.to_utf16_string())
            .unwrap_or_else(|| Utf16String::from_rust_str(""))
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
