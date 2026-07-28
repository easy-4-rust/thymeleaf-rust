use std::error::Error;
use std::fmt::{Display, Formatter};

use super::{TemplateEngineException, TemplateProcessingException};

/// 表示写出模板处理结果时发生的异常。
///
/// 对应 Java: `org.thymeleaf.exceptions.TemplateOutputException`。
///
/// 上游自 Thymeleaf 3.0 起使用该形态，并保留模板名、行、列和底层输出错误。
#[derive(Debug)]
pub struct TemplateOutputException {
    processing: TemplateProcessingException,
}

impl TemplateOutputException {
    /// 使用消息、模板位置和原因创建输出异常。
    ///
    /// 对应 Java:
    /// `TemplateOutputException#TemplateOutputException(String, String, int, int, Throwable)`。
    ///
    /// # 参数
    /// - `message`：异常消息。
    /// - `template_name`：发生写出错误的模板名。
    /// - `line`：模板行号，负数表示未知。
    /// - `col`：模板列号，负数表示未知。
    /// - `cause`：底层 Writer 或输出设备错误。
    pub fn new<E>(
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

    /// 返回完整异常消息。
    #[must_use]
    pub fn get_message(&self) -> String {
        self.processing.get_message()
    }

    /// 返回模板名称。
    #[must_use]
    pub fn get_template_name(&self) -> Option<&str> {
        self.processing.get_template_name()
    }

    /// 判断是否存在模板名称。
    #[must_use]
    pub fn has_template_name(&self) -> bool {
        self.processing.has_template_name()
    }

    /// 返回行号。
    #[must_use]
    pub fn get_line(&self) -> Option<i32> {
        self.processing.get_line()
    }

    /// 返回列号。
    #[must_use]
    pub fn get_col(&self) -> Option<i32> {
        self.processing.get_col()
    }

    /// 判断行列是否同时存在。
    #[must_use]
    pub fn has_line_and_col(&self) -> bool {
        self.processing.has_line_and_col()
    }

    /// 修改模板名称。
    pub fn set_template_name(&mut self, template_name: Option<String>) {
        self.processing.set_template_name(template_name);
    }

    /// 修改行列；负数位置转换为缺失值。
    pub fn set_line_and_col(&mut self, line: i32, col: i32) {
        self.processing.set_line_and_col(line, col);
    }
}

impl Display for TemplateOutputException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.processing, formatter)
    }
}

impl Error for TemplateOutputException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.processing.source()
    }
}

impl TemplateEngineException for TemplateOutputException {}

impl AsRef<TemplateProcessingException> for TemplateOutputException {
    fn as_ref(&self) -> &TemplateProcessingException {
        &self.processing
    }
}

impl AsMut<TemplateProcessingException> for TemplateOutputException {
    fn as_mut(&mut self) -> &mut TemplateProcessingException {
        &mut self.processing
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use super::TemplateOutputException;

    #[test]
    fn preserves_location_cause_and_inherited_mutation() {
        let mut error = TemplateOutputException::new(
            Some("output".to_owned()),
            Some("index.html".to_owned()),
            9,
            10,
            io::Error::other("writer"),
        );
        assert_eq!(
            error.get_message(),
            "output (template: \"index.html\" - line 9, col 10)"
        );
        assert_eq!(error.get_template_name(), Some("index.html"));
        assert!(error.has_template_name());
        assert_eq!(error.get_line(), Some(9));
        assert_eq!(error.get_col(), Some(10));
        assert!(error.has_line_and_col());
        assert_eq!(error.as_ref().get_message(), error.get_message());
        assert_eq!(
            error.source().map(ToString::to_string),
            Some("writer".to_owned())
        );

        error.set_template_name(None);
        error.set_line_and_col(-1, -1);
        error.as_mut().set_line_and_col(-1, -1);
        assert!(!error.has_template_name());
        assert!(!error.has_line_and_col());
        assert_eq!(error.to_string(), "output");
    }
}
