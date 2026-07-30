use std::any::Any;

use thymeleaf::expression::TemplateObject;
use thymeleaf::util::{JavaHashCode, JavaString};

/// 上游布尔求值语料使用的 `SimpleDateFormat` 夹具。
///
/// 对应 Java: `java.text.SimpleDateFormat`。这些用例只验证普通非 null Java
/// 对象在条件表达式中为真，因此夹具保存构造 pattern，不扩大核心反射权限。
pub struct CorpusSimpleDateFormat {
    pattern: JavaString,
}

impl CorpusSimpleDateFormat {
    /// 使用 Java 构造器的 pattern 参数创建夹具。
    #[must_use]
    pub const fn new(pattern: JavaString) -> Self {
        Self { pattern }
    }
}

impl TemplateObject for CorpusSimpleDateFormat {
    fn java_class_name(&self) -> &str {
        "java.text.SimpleDateFormat"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str(&format!(
            "java.text.SimpleDateFormat@{:x}",
            self.pattern.java_hash_code()
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
