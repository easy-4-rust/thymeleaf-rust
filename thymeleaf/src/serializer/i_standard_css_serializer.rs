use crate::exceptions::TemplateProcessingException;
use crate::expression::TemplateValue;
use crate::util::TemplateWriter;

/// Standard Dialect 的 CSS 值序列化合同。
///
/// 对应 Java:
/// `org.thymeleaf.standard.serializer.IStandardCSSSerializer`。
///
/// 序列化器同时用于 CSS 模板和 `th:inline="css"`；输出直接进入 UTF-16 writer，
/// 不要求先创建完整中间字符串。
pub trait IStandardCSSSerializer: Send + Sync {
    /// 将任意可空对象序列化到 CSS 输出。
    ///
    /// # 参数
    ///
    /// - `object`：待序列化对象；`None` 对应 Java null。
    /// - `writer`：CSS 输出目标。
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
