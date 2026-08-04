use std::any::Any;

use thymeleaf::expression::TemplateObject;
use thymeleaf::util::{HashCodeValue, Utf16String};

/// 上游布尔求值语料使用的 `SimpleDateFormat` 夹具。
///
/// 对应 Java: `java.text.SimpleDateFormat`。这些用例只验证普通非 null Java
/// 对象在条件表达式中为真，因此夹具保存构造 pattern，不扩大核心反射权限。
pub struct CorpusSimpleDateFormat {
    pattern: Utf16String,
}

impl CorpusSimpleDateFormat {
    /// 使用 Java 构造器的 pattern 参数创建夹具。
    #[must_use]
    pub const fn new(pattern: Utf16String) -> Self {
        Self { pattern }
    }
}

impl TemplateObject for CorpusSimpleDateFormat {
    fn class_name(&self) -> &str {
        "java.text.SimpleDateFormat"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(&format!(
            "java.text.SimpleDateFormat@{:x}",
            self.pattern.hash_code()
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
