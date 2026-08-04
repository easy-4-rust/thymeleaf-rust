use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::{any::Any, sync::Arc};

use thiserror::Error;

use crate::expression::{TemplateObject, TemplateObjectMethodError, TemplateValue};
use crate::util::{NumberValue, Utf16String};

/// Thymeleaf 支持的模板解析与输出模式。
///
/// 对应 Java: `org.thymeleaf.templatemode.TemplateMode`。
///
/// HTML 与 XML 属于标记模式；TEXT、JAVASCRIPT 与 CSS 属于文本模式；
/// RAW 不执行结构化解析。除 HTML 外，所有模式均区分大小写。
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TemplateMode {
    /// 宽容、大小写不敏感的 HTML 标记模式。
    HTML,
    /// 严格、大小写敏感的 XML 标记模式。
    XML,
    /// 通用文本模板模式。
    TEXT,
    /// JavaScript 文本模板模式。
    JAVASCRIPT,
    /// CSS 文本模板模式。
    CSS,
    /// 不进行模板结构解析的原始模式。
    RAW,
}

impl TemplateMode {
    /// 判断当前模式是否为 HTML 或 XML 标记模式。
    ///
    /// 对应 Java: `TemplateMode#isMarkup()`。
    #[must_use]
    pub const fn is_markup(self) -> bool {
        matches!(self, Self::HTML | Self::XML)
    }

    /// 判断当前模式是否使用文本模板解析器。
    ///
    /// 对应 Java: `TemplateMode#isText()`。
    #[must_use]
    pub const fn is_text(self) -> bool {
        matches!(self, Self::TEXT | Self::JAVASCRIPT | Self::CSS)
    }

    /// 判断名称和属性比较是否区分大小写。
    ///
    /// 对应 Java: `TemplateMode#isCaseSensitive()`。
    #[must_use]
    pub const fn is_case_sensitive(self) -> bool {
        !matches!(self, Self::HTML)
    }

    /// 按 Thymeleaf 语义解析模板模式。
    ///
    /// 对应 Java: `TemplateMode#parse(String)`。
    ///
    /// # 参数
    /// - `mode`：Java 参数 `mode`；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// 已知名称按 ASCII 大小写不敏感方式解析；未知非空名称记录警告并回退
    /// 到 HTML。上游仅使用 trim 判断是否为空，比较时不 trim，本实现保持该细节。
    ///
    /// # 错误
    /// `mode` 为 `None`、空字符串或纯空白时返回 `TemplateModeParseError`。
    pub fn parse(mode: Option<&str>) -> Result<Self, TemplateModeParseError> {
        let Some(mode) = mode else {
            return Err(TemplateModeParseError);
        };
        if trim(mode).is_empty() {
            return Err(TemplateModeParseError);
        }

        let parsed = if mode.eq_ignore_ascii_case("HTML") {
            Some(Self::HTML)
        } else if mode.eq_ignore_ascii_case("XML") {
            Some(Self::XML)
        } else if mode.eq_ignore_ascii_case("TEXT") {
            Some(Self::TEXT)
        } else if mode.eq_ignore_ascii_case("JAVASCRIPT") {
            Some(Self::JAVASCRIPT)
        } else if mode.eq_ignore_ascii_case("CSS") {
            Some(Self::CSS)
        } else if mode.eq_ignore_ascii_case("RAW") {
            Some(Self::RAW)
        } else {
            None
        };

        if let Some(parsed) = parsed {
            return Ok(parsed);
        }

        // tracing 在未注册订阅器时可能跳过字段求值，因此先取得线程名；
        // 这也与 Java 在调用 logger.warn 前执行 TemplateEngine.threadIndex() 一致。
        let thread_index = current_thread_index();
        tracing::warn!(
            thread = %thread_index,
            mode,
            default_mode = %Self::HTML,
            "Unknown Template Mode. Must be one of: HTML, XML, TEXT, JAVASCRIPT, CSS, RAW. Using default Template Mode."
        );
        Ok(Self::HTML)
    }
}

impl Display for TemplateMode {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::HTML => "HTML",
            Self::XML => "XML",
            Self::TEXT => "TEXT",
            Self::JAVASCRIPT => "JAVASCRIPT",
            Self::CSS => "CSS",
            Self::RAW => "RAW",
        })
    }
}

impl FromStr for TemplateMode {
    type Err = TemplateModeParseError;

    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        Self::parse(Some(mode))
    }
}

impl TemplateObject for TemplateMode {
    fn class_name(&self) -> &str {
        "org.thymeleaf.templatemode.TemplateMode"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(&self.to_string())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn invoke_method(
        &self,
        method_name: &Utf16String,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        if !arguments.is_empty() {
            return None;
        }
        let value = match method_name.to_string_lossy().as_str() {
            "isMarkup" => TemplateValue::Boolean(self.is_markup()),
            "isText" => TemplateValue::Boolean(self.is_text()),
            "isCaseSensitive" => TemplateValue::Boolean(self.is_case_sensitive()),
            "name" | "toString" => TemplateValue::string(self.to_utf16_string()),
            "ordinal" => TemplateValue::Number(NumberValue::Integer(match self {
                Self::HTML => 0,
                Self::XML => 1,
                Self::TEXT => 2,
                Self::JAVASCRIPT => 3,
                Self::CSS => 4,
                Self::RAW => 5,
            })),
            _ => return None,
        };
        Some(Ok(Some(Arc::new(value))))
    }
}

fn current_thread_index() -> String {
    std::thread::current()
        .name()
        .unwrap_or("unnamed")
        .to_owned()
}

fn trim(value: &str) -> &str {
    value.trim_matches(|character| character <= '\u{0020}')
}

/// 模板模式为 `null`、空字符串或纯空白时产生的解析错误。
///
/// 对应 Java: `TemplateMode#parse(String)` 抛出的 `IllegalArgumentException`。
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("Template mode cannot be null or empty")]
pub struct TemplateModeParseError;

#[cfg(test)]
mod tests {
    use super::{TemplateMode, TemplateModeParseError, current_thread_index};

    #[test]
    fn exposes_exact_mode_flags() {
        assert!(TemplateMode::HTML.is_markup());
        assert!(!TemplateMode::HTML.is_text());
        assert!(!TemplateMode::HTML.is_case_sensitive());

        assert!(TemplateMode::XML.is_markup());
        assert!(!TemplateMode::XML.is_text());
        assert!(TemplateMode::XML.is_case_sensitive());

        for mode in [
            TemplateMode::TEXT,
            TemplateMode::JAVASCRIPT,
            TemplateMode::CSS,
        ] {
            assert!(!mode.is_markup());
            assert!(mode.is_text());
            assert!(mode.is_case_sensitive());
        }

        assert!(!TemplateMode::RAW.is_markup());
        assert!(!TemplateMode::RAW.is_text());
        assert!(TemplateMode::RAW.is_case_sensitive());
    }

    #[test]
    fn parses_all_names_case_insensitively() {
        let cases = [
            ("html", TemplateMode::HTML),
            ("XML", TemplateMode::XML),
            ("Text", TemplateMode::TEXT),
            ("javascript", TemplateMode::JAVASCRIPT),
            ("Css", TemplateMode::CSS),
            ("raw", TemplateMode::RAW),
        ];
        for (input, expected) in cases {
            assert_eq!(TemplateMode::parse(Some(input)), Ok(expected));
            assert_eq!(input.parse::<TemplateMode>(), Ok(expected));
            assert_eq!(expected.to_string(), input.to_ascii_uppercase());
        }
    }

    #[test]
    fn rejects_null_empty_and_whitespace() {
        for input in [None, Some(""), Some(" "), Some("\n\t"), Some("\0")] {
            assert_eq!(
                TemplateMode::parse(input),
                Err(TemplateModeParseError),
                "input: {input:?}"
            );
        }
        assert_eq!(
            TemplateModeParseError.to_string(),
            "Template mode cannot be null or empty"
        );
    }

    #[test]
    fn unknown_and_untrimmed_modes_fall_back_to_html() {
        assert_eq!(
            TemplateMode::parse(Some("MARKDOWN")),
            Ok(TemplateMode::HTML)
        );
        assert_eq!(TemplateMode::parse(Some(" XML ")), Ok(TemplateMode::HTML));
        assert_eq!(
            TemplateMode::parse(Some("\u{00A0}")),
            Ok(TemplateMode::HTML)
        );
        assert_eq!(
            current_thread_index(),
            "templatemode::template_mode::tests::unknown_and_untrimmed_modes_fall_back_to_html"
        );

        let unnamed = std::thread::spawn(current_thread_index).join().unwrap();
        assert_eq!(unnamed, "unnamed");
    }
}
