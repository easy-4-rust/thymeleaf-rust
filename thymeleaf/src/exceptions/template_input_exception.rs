use std::error::Error;
use std::fmt::{Display, Formatter};

use super::{TemplateEngineException, TemplateProcessingException};

/// 表示读取、解析或解析定位模板输入时发生的异常。
///
/// 对应 Java: `org.thymeleaf.exceptions.TemplateInputException`。
/// Java 类型继承 `TemplateProcessingException`，Rust 使用组合并完整转发
/// 模板名称、行列、消息修改和原因链语义。
#[derive(Debug)]
pub struct TemplateInputException {
    processing: TemplateProcessingException,
}

impl TemplateInputException {
    /// 仅使用消息创建输入异常。
    ///
    /// 对应 Java: `TemplateInputException#TemplateInputException(String)`。
    #[must_use]
    pub fn new(message: Option<String>) -> Self {
        Self {
            processing: TemplateProcessingException::new(message),
        }
    }

    /// 使用消息和原因创建输入异常。
    ///
    /// 对应 Java:
    /// `TemplateInputException#TemplateInputException(String, Throwable)`。
    pub fn with_cause<E>(message: Option<String>, cause: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            processing: TemplateProcessingException::with_cause(message, cause),
        }
    }

    /// 使用消息、模板名和原因创建输入异常。
    ///
    /// 对应 Java:
    /// `TemplateInputException#TemplateInputException(String, String, Throwable)`。
    pub fn with_template_and_cause<E>(
        message: Option<String>,
        template_name: Option<String>,
        cause: E,
    ) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            processing: TemplateProcessingException::with_template_and_cause(
                message,
                template_name,
                cause,
            ),
        }
    }

    /// 使用消息、模板名和位置创建输入异常。
    ///
    /// 对应 Java:
    /// `TemplateInputException#TemplateInputException(String, String, int, int)`。
    #[must_use]
    pub fn with_location(
        message: Option<String>,
        template_name: Option<String>,
        line: i32,
        col: i32,
    ) -> Self {
        Self {
            processing: TemplateProcessingException::with_location(
                message,
                template_name,
                line,
                col,
            ),
        }
    }

    /// 使用消息、模板名、位置和原因创建输入异常。
    ///
    /// 对应 Java:
    /// `TemplateInputException#TemplateInputException(String, String, int, int, Throwable)`。
    pub fn with_location_and_cause<E>(
        message: Option<String>,
        template_name: Option<String>,
        line: i32,
        col: i32,
        cause: E,
    ) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            processing: TemplateProcessingException::with_location_and_cause(
                message,
                template_name,
                line,
                col,
                cause,
            ),
        }
    }

    /// 返回包含模板名和位置后缀的完整消息。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `getMessage()` 的 Rust 移植（`TemplateInputException` 继承路径）。
    pub fn get_message(&self) -> String {
        self.processing.get_message()
    }

    /// 返回模板名称。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `getTemplateName()` 的 Rust 移植（`TemplateInputException` 继承路径）。
    pub fn get_template_name(&self) -> Option<&str> {
        self.processing.get_template_name()
    }

    /// 判断是否存在模板名称。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `hasTemplateName()` 的 Rust 移植（`TemplateInputException` 继承路径）。
    pub fn has_template_name(&self) -> bool {
        self.processing.has_template_name()
    }

    /// 返回行号。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `getLine()` 的 Rust 移植（`TemplateInputException` 继承路径）。
    pub fn get_line(&self) -> Option<i32> {
        self.processing.get_line()
    }

    /// 返回列号。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `getCol()` 的 Rust 移植（`TemplateInputException` 继承路径）。
    pub fn get_col(&self) -> Option<i32> {
        self.processing.get_col()
    }

    /// 判断行和列是否同时存在。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `hasLineAndCol()` 的 Rust 移植（`TemplateInputException` 继承路径）。
    pub fn has_line_and_col(&self) -> bool {
        self.processing.has_line_and_col()
    }

    /// 修改模板名称。
    /// 对应 Java 语义：Java 接口/超类方法 `setTemplateName()` 的 Rust 移植（`TemplateInputException` 继承路径）。
    pub fn set_template_name(&mut self, template_name: Option<String>) {
        self.processing.set_template_name(template_name);
    }

    /// 修改行列；负数位置转换为缺失值。
    /// 对应 Java 语义：Java 接口/超类方法 `setLineAndCol()` 的 Rust 移植（`TemplateInputException` 继承路径）。
    pub fn set_line_and_col(&mut self, line: i32, col: i32) {
        self.processing.set_line_and_col(line, col);
    }
}

impl Display for TemplateInputException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.processing, formatter)
    }
}

impl Error for TemplateInputException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.processing.source()
    }
}

impl TemplateEngineException for TemplateInputException {
    fn as_processing_exception_mut(&mut self) -> Option<&mut TemplateProcessingException> {
        Some(&mut self.processing)
    }
}

impl AsRef<TemplateProcessingException> for TemplateInputException {
    fn as_ref(&self) -> &TemplateProcessingException {
        &self.processing
    }
}

impl AsMut<TemplateProcessingException> for TemplateInputException {
    fn as_mut(&mut self) -> &mut TemplateProcessingException {
        &mut self.processing
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use super::TemplateInputException;

    #[test]
    fn maps_every_java_constructor_and_inherited_operation() {
        let plain = TemplateInputException::new(Some("input".to_owned()));
        assert_eq!(plain.get_message(), "input");
        assert!(!plain.has_template_name());
        assert!(plain.source().is_none());

        let caused =
            TemplateInputException::with_cause(Some("input".to_owned()), io::Error::other("cause"));
        assert_eq!(
            caused.source().map(ToString::to_string),
            Some("cause".to_owned())
        );

        let template_caused = TemplateInputException::with_template_and_cause(
            Some("input".to_owned()),
            Some("index.html".to_owned()),
            io::Error::other("cause"),
        );
        assert_eq!(template_caused.get_template_name(), Some("index.html"));
        assert_eq!(template_caused.get_line(), None);

        let located = TemplateInputException::with_location(
            Some("input".to_owned()),
            Some("index.html".to_owned()),
            3,
            4,
        );
        assert_eq!(located.get_line(), Some(3));
        assert_eq!(located.get_col(), Some(4));
        assert!(located.has_line_and_col());

        let mut located_caused = TemplateInputException::with_location_and_cause(
            Some("input".to_owned()),
            Some("old.html".to_owned()),
            5,
            6,
            io::Error::other("cause"),
        );
        assert_eq!(located_caused.as_ref().get_line(), Some(5));
        located_caused.set_template_name(Some("new.html".to_owned()));
        located_caused.set_line_and_col(-1, 8);
        located_caused
            .as_mut()
            .set_template_name(Some("new.html".to_owned()));
        assert_eq!(located_caused.get_template_name(), Some("new.html"));
        assert_eq!(located_caused.get_line(), None);
        assert_eq!(located_caused.get_col(), Some(8));
        assert!(!located_caused.has_line_and_col());
        assert_eq!(
            located_caused.to_string(),
            "input (template: \"new.html\" - , col 8)"
        );
    }
}
