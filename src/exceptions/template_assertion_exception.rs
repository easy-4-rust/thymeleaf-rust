use std::error::Error;
use std::fmt::{Display, Formatter};

/// 表示模板断言不成立的异常。
///
/// 对应 Java: `org.thymeleaf.exceptions.TemplateAssertionException`。
/// Standard Dialect 的 `th:assert` 处理器会抛出该错误。
#[derive(Debug, Eq, PartialEq)]
pub struct TemplateAssertionException {
    message: String,
}

impl TemplateAssertionException {
    /// 使用断言表达式和模板名创建异常。
    ///
    /// 对应 Java:
    /// `TemplateAssertionException#TemplateAssertionException(String, String)`。
    ///
    /// `None` 按 Java `String.format("%s")` 语义格式化为字符串 `null`。
    #[must_use]
    pub fn new(assertion_expression: Option<&str>, template_name: Option<&str>) -> Self {
        Self {
            message: create_message(assertion_expression, template_name, None, None),
        }
    }

    /// 使用断言表达式、模板名和位置创建异常。
    ///
    /// 对应 Java:
    /// `TemplateAssertionException#TemplateAssertionException(String, String, int, int)`。
    #[must_use]
    pub fn with_location(
        assertion_expression: Option<&str>,
        template_name: Option<&str>,
        line: i32,
        col: i32,
    ) -> Self {
        Self {
            message: create_message(assertion_expression, template_name, Some(line), Some(col)),
        }
    }

    /// 返回与上游完全相同格式的断言错误消息。
    #[must_use]
    pub fn get_message(&self) -> &str {
        &self.message
    }
}

fn create_message(
    assertion_expression: Option<&str>,
    template_name: Option<&str>,
    line: Option<i32>,
    col: Option<i32>,
) -> String {
    let assertion_expression = assertion_expression.unwrap_or("null");
    let template_name = template_name.unwrap_or("null");
    match (line, col) {
        (Some(line), Some(col)) => format!(
            "Assertion '{assertion_expression}' not valid in template '{template_name}', line {line} col {col}"
        ),
        _ => format!("Assertion '{assertion_expression}' not valid in template '{template_name}'"),
    }
}

impl Display for TemplateAssertionException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for TemplateAssertionException {}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::{TemplateAssertionException, create_message};

    #[test]
    fn formats_both_java_constructors_and_null_strings() {
        let plain = TemplateAssertionException::new(Some("${user != null}"), Some("index.html"));
        assert_eq!(
            plain.get_message(),
            "Assertion '${user != null}' not valid in template 'index.html'"
        );
        assert_eq!(plain.to_string(), plain.get_message());
        assert!(plain.source().is_none());

        let located = TemplateAssertionException::with_location(
            Some("${user != null}"),
            Some("index.html"),
            7,
            3,
        );
        assert_eq!(
            located.get_message(),
            "Assertion '${user != null}' not valid in template 'index.html', line 7 col 3"
        );

        let null_values = TemplateAssertionException::new(None, None);
        assert_eq!(
            null_values.get_message(),
            "Assertion 'null' not valid in template 'null'"
        );

        assert_eq!(
            create_message(Some("x"), Some("t"), Some(1), None),
            "Assertion 'x' not valid in template 't'"
        );
    }
}
