use std::any::Any;

use thymeleaf::expression::{TemplateObject, TemplateValue};
use thymeleaf::util::Utf16String;

/// 上游类型元数据语料使用的 ByteArrayInputStream 宿主对象。
///
/// 对应 Java: `java.io.ByteArrayInputStream`。
pub struct CorpusByteArrayInputStream {
    #[allow(dead_code)]
    bytes: TemplateValue,
}

impl CorpusByteArrayInputStream {
    /// 使用 Java byte[] 值创建输入流夹具。
    pub fn new(bytes: TemplateValue) -> Self {
        Self { bytes }
    }
}

impl TemplateObject for CorpusByteArrayInputStream {
    fn java_class_name(&self) -> &str {
        "java.io.ByteArrayInputStream"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str("java.io.ByteArrayInputStream")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
