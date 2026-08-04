use crate::exceptions::TemplateProcessingException;
use crate::expression::TemplateValue;
use crate::util::TemplateWriter;

/// Standard Dialect 的 JavaScript 值序列化合同。
///
/// 对应 Java:
/// `org.thymeleaf.standard.serializer.IStandardJavaScriptSerializer`。
///
/// 序列化器同时用于 JAVASCRIPT 模板与 `th:inline="javascript"`，具体实现必须
/// 负责字符串转义、集合/Map、数字、日期及普通对象的 JavaScript 表示。
pub trait IStandardJavaScriptSerializer: Send + Sync {
    /// 将任意可空对象序列化到 JavaScript 输出。
    ///
    /// # 参数
    ///
    /// - `object`：待序列化对象；`None` 对应 Java null。
    /// - `writer`：JavaScript 输出目标。
    ///
    /// # 错误
    ///
    /// writer 失败或对象转换失败时返回 Java
    /// `TemplateProcessingException` 对应错误。
    fn serialize_value(
        &self,
        object: Option<&TemplateValue>,
        writer: &mut dyn TemplateWriter,
    ) -> Result<(), TemplateProcessingException>;
}
