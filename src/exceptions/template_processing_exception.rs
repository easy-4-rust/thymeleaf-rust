use std::error::Error;
use std::fmt::{Display, Formatter};

use super::TemplateEngineException;

/// 表示模板处理阶段发生的一般错误。
///
/// 对应 Java: `org.thymeleaf.exceptions.TemplateProcessingException`。
///
/// 除原始消息和原因链外，本对象保留模板名称、行和列。负数行列会像
/// Java 实现一样被规范化为缺失值；显示消息严格遵循上游拼接顺序。
#[derive(Debug)]
pub struct TemplateProcessingException {
    message: Option<String>,
    template_name: Option<String>,
    line: Option<i32>,
    col: Option<i32>,
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl TemplateProcessingException {
    /// 仅使用消息创建异常。
    ///
    /// 对应 Java: `TemplateProcessingException#TemplateProcessingException(String)`。
    #[must_use]
    pub fn new(message: Option<String>) -> Self {
        Self::with_template_and_optional_cause(message, None, None)
    }

    /// 使用消息和原因创建异常。
    ///
    /// 对应 Java:
    /// `TemplateProcessingException#TemplateProcessingException(String, Throwable)`。
    pub fn with_cause<E>(message: Option<String>, cause: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::with_template_and_optional_cause(message, None, Some(Box::new(cause)))
    }

    /// 使用消息、模板名和原因创建异常。
    ///
    /// 对应 Java:
    /// `TemplateProcessingException#TemplateProcessingException(String, String, Throwable)`。
    pub fn with_template_and_cause<E>(
        message: Option<String>,
        template_name: Option<String>,
        cause: E,
    ) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::with_template_and_optional_cause(message, template_name, Some(Box::new(cause)))
    }

    /// 使用消息、模板名和位置创建异常。
    ///
    /// 对应 Java:
    /// `TemplateProcessingException#TemplateProcessingException(String, String, int, int)`。
    #[must_use]
    pub fn with_location(
        message: Option<String>,
        template_name: Option<String>,
        line: i32,
        col: i32,
    ) -> Self {
        Self {
            message,
            template_name,
            line: normalize_position(line),
            col: normalize_position(col),
            cause: None,
        }
    }

    /// 使用消息、模板名、位置和原因创建异常。
    ///
    /// 对应 Java:
    /// `TemplateProcessingException#TemplateProcessingException(String, String, int, int, Throwable)`。
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
            message,
            template_name,
            line: normalize_position(line),
            col: normalize_position(col),
            cause: Some(Box::new(cause)),
        }
    }

    fn with_template_and_optional_cause(
        message: Option<String>,
        template_name: Option<String>,
        cause: Option<Box<dyn Error + Send + Sync>>,
    ) -> Self {
        Self {
            message,
            template_name,
            line: None,
            col: None,
            cause,
        }
    }

    /// 返回包含模板名和位置后缀的完整消息。
    ///
    /// 对应 Java: `TemplateProcessingException#getMessage()`。
    #[must_use]
    pub fn get_message(&self) -> String {
        let mut result = self.message.as_deref().unwrap_or("null").to_owned();
        if let Some(template_name) = &self.template_name {
            result.push_str(" (template: \"");
            result.push_str(template_name);
            result.push('"');
            if self.line.is_some() || self.col.is_some() {
                result.push_str(" - ");
                if let Some(line) = self.line {
                    result.push_str("line ");
                    result.push_str(&line.to_string());
                }
                if let Some(col) = self.col {
                    result.push_str(", col ");
                    result.push_str(&col.to_string());
                }
            }
            result.push(')');
        }
        result
    }

    /// 返回模板名称。
    ///
    /// 对应 Java: `TemplateProcessingException#getTemplateName()`。
    #[must_use]
    pub fn get_template_name(&self) -> Option<&str> {
        self.template_name.as_deref()
    }

    /// 判断是否存在模板名称。
    ///
    /// 对应 Java: `TemplateProcessingException#hasTemplateName()`。
    #[must_use]
    pub fn has_template_name(&self) -> bool {
        self.template_name.is_some()
    }

    /// 返回从 1 开始的行号；缺失时返回 `None`。
    ///
    /// 对应 Java: `TemplateProcessingException#getLine()`。
    #[must_use]
    pub fn get_line(&self) -> Option<i32> {
        self.line
    }

    /// 返回从 1 开始的列号；缺失时返回 `None`。
    ///
    /// 对应 Java: `TemplateProcessingException#getCol()`。
    #[must_use]
    pub fn get_col(&self) -> Option<i32> {
        self.col
    }

    /// 仅当行和列同时存在时返回 `true`。
    ///
    /// 对应 Java: `TemplateProcessingException#hasLineAndCol()`。
    #[must_use]
    pub fn has_line_and_col(&self) -> bool {
        self.line.is_some() && self.col.is_some()
    }

    /// 修改模板名称。
    ///
    /// 对应 Java: `TemplateProcessingException#setTemplateName(String)`。
    pub fn set_template_name(&mut self, template_name: Option<String>) {
        self.template_name = template_name;
    }

    /// 修改行列；任何负数位置都会被规范化为缺失值。
    ///
    /// 对应 Java: `TemplateProcessingException#setLineAndCol(int, int)`。
    pub fn set_line_and_col(&mut self, line: i32, col: i32) {
        self.line = normalize_position(line);
        self.col = normalize_position(col);
    }
}

fn normalize_position(position: i32) -> Option<i32> {
    (position >= 0).then_some(position)
}

impl Display for TemplateProcessingException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.get_message())
    }
}

impl Error for TemplateProcessingException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|cause| cause as &(dyn Error + 'static))
    }
}

impl TemplateEngineException for TemplateProcessingException {}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use super::TemplateProcessingException;

    #[test]
    fn formats_every_java_location_branch() {
        let plain = TemplateProcessingException::new(Some("problem".to_owned()));
        assert_eq!(plain.get_message(), "problem");
        assert_eq!(plain.to_string(), "problem");
        assert_eq!(plain.get_template_name(), None);
        assert!(!plain.has_template_name());
        assert_eq!(plain.get_line(), None);
        assert_eq!(plain.get_col(), None);
        assert!(!plain.has_line_and_col());
        assert!(plain.source().is_none());

        let null_message = TemplateProcessingException::new(None);
        assert_eq!(null_message.get_message(), "null");

        let template_only = TemplateProcessingException::with_location(
            Some("problem".to_owned()),
            Some("index.html".to_owned()),
            -1,
            -1,
        );
        assert_eq!(
            template_only.get_message(),
            "problem (template: \"index.html\")"
        );

        let complete = TemplateProcessingException::with_location(
            Some("problem".to_owned()),
            Some("index.html".to_owned()),
            7,
            11,
        );
        assert_eq!(
            complete.get_message(),
            "problem (template: \"index.html\" - line 7, col 11)"
        );
        assert!(complete.has_line_and_col());

        let line_only = TemplateProcessingException::with_location(
            Some("problem".to_owned()),
            Some("index.html".to_owned()),
            7,
            -1,
        );
        assert_eq!(
            line_only.get_message(),
            "problem (template: \"index.html\" - line 7)"
        );

        let col_only = TemplateProcessingException::with_location(
            Some("problem".to_owned()),
            Some("index.html".to_owned()),
            -1,
            11,
        );
        assert_eq!(
            col_only.get_message(),
            "problem (template: \"index.html\" - , col 11)"
        );

        let hidden_location =
            TemplateProcessingException::with_location(Some("problem".to_owned()), None, 1, 2);
        assert_eq!(hidden_location.get_message(), "problem");
    }

    #[test]
    fn preserves_causes_and_mutation() {
        let caused = TemplateProcessingException::with_cause(
            Some("problem".to_owned()),
            io::Error::other("cause"),
        );
        assert_eq!(
            caused.source().map(ToString::to_string),
            Some("cause".to_owned())
        );

        let template_caused = TemplateProcessingException::with_template_and_cause(
            Some("problem".to_owned()),
            Some("index.html".to_owned()),
            io::Error::other("cause"),
        );
        assert_eq!(template_caused.get_line(), None);
        assert_eq!(template_caused.get_col(), None);

        let mut located_caused = TemplateProcessingException::with_location_and_cause(
            Some("problem".to_owned()),
            Some("old.html".to_owned()),
            1,
            2,
            io::Error::other("cause"),
        );
        located_caused.set_template_name(Some("new.html".to_owned()));
        located_caused.set_line_and_col(-1, 9);
        assert_eq!(located_caused.get_template_name(), Some("new.html"));
        assert_eq!(located_caused.get_line(), None);
        assert_eq!(located_caused.get_col(), Some(9));
        assert!(!located_caused.has_line_and_col());
        assert_eq!(
            located_caused.source().map(ToString::to_string),
            Some("cause".to_owned())
        );
    }
}
