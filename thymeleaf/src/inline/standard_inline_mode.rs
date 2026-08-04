use std::fmt::{Display, Formatter};

use thiserror::Error;

use crate::util::Utf16String;

/// 标准方言支持的文本内联模式。
///
/// 上游没有 RAW 内联模式，因为元素体中的 RAW 与 NONE 没有可观察差异。
///
/// 对应 Java: `org.thymeleaf.standard.inline.StandardInlineMode`。
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StandardInlineMode {
    /// 禁用内联。
    NONE,
    /// HTML 内联。
    HTML,
    /// XML 内联。
    XML,
    /// TEXT 内联。
    TEXT,
    /// JavaScript 内联。
    JAVASCRIPT,
    /// CSS 内联。
    CSS,
}

impl StandardInlineMode {
    /// 按 Java enum 声明顺序列出全部值。
    pub const VALUES: [Self; 6] = [
        Self::NONE,
        Self::HTML,
        Self::XML,
        Self::TEXT,
        Self::JAVASCRIPT,
        Self::CSS,
    ];

    /// 按 Thymeleaf 标准方言规则解析内联模式。
    ///
    /// 对应 Java: `StandardInlineMode#parse(String)`。
    ///
    /// # 参数
    /// - `mode`：待解析 Java UTF-16 字符串；`None` 对应 Java null。
    ///
    /// # 返回
    /// 名称以 Java `String#equalsIgnoreCase` 规则匹配时返回对应 enum。
    /// 上游只使用 `trim()` 判断空白，不会在匹配前去除首尾字符。
    ///
    /// # 错误
    /// null、经 Java `trim()` 后为空及未识别名称分别保留精确异常消息。
    pub fn parse(mode: Option<&Utf16String>) -> Result<Self, StandardInlineModeParseError> {
        let Some(mode) = mode else {
            return Err(StandardInlineModeParseError::NullOrEmpty);
        };
        if trim(mode.as_utf16()).is_empty() {
            return Err(StandardInlineModeParseError::NullOrEmpty);
        }

        for candidate in Self::VALUES {
            if equals_ignore_case_ascii(mode.as_utf16(), candidate.name().as_bytes()) {
                return Ok(candidate);
            }
        }
        Err(StandardInlineModeParseError::Unrecognized(mode.clone()))
    }

    /// 返回 Java enum 的声明序号。
    ///
    /// # 返回
    /// 从 `NONE` 的 0 到 `CSS` 的 5。
    #[must_use]
    pub const fn ordinal(self) -> usize {
        match self {
            Self::NONE => 0,
            Self::HTML => 1,
            Self::XML => 2,
            Self::TEXT => 3,
            Self::JAVASCRIPT => 4,
            Self::CSS => 5,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::NONE => "NONE",
            Self::HTML => "HTML",
            Self::XML => "XML",
            Self::TEXT => "TEXT",
            Self::JAVASCRIPT => "JAVASCRIPT",
            Self::CSS => "CSS",
        }
    }
}

impl Display for StandardInlineMode {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

/// 标准内联模式解析对应的 Java `IllegalArgumentException`。
///
/// 对应 Java: `StandardInlineMode#parse(String)` 的异常边界。
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum StandardInlineModeParseError {
    /// 输入为 null 或经 Java `trim()` 后为空。
    #[error("Inline mode cannot be null or empty")]
    NullOrEmpty,
    /// 输入非空但不是受支持的内联模式。
    #[error("Unrecognized inline mode: {}", .0.to_string_lossy())]
    Unrecognized(Utf16String),
}

impl StandardInlineModeParseError {
    /// 返回对应 Java 异常类名。
    ///
    /// # 返回
    /// 上游两个失败分支均返回 `java.lang.IllegalArgumentException`。
    #[must_use]
    pub const fn class_name(&self) -> &'static str {
        "java.lang.IllegalArgumentException"
    }

    /// 返回无损 UTF-16 Java detail message。
    ///
    /// # 返回
    /// 精确保留固定前缀和未识别输入，包括孤立代理 code unit。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `message()` 的 Rust 移植（`StandardInlineMode` 继承路径）。
    pub fn message(&self) -> Utf16String {
        match self {
            Self::NullOrEmpty => Utf16String::from_rust_str("Inline mode cannot be null or empty"),
            Self::Unrecognized(mode) => {
                let mut message = Utf16String::from_rust_str("Unrecognized inline mode: ")
                    .as_utf16()
                    .to_vec();
                message.extend_from_slice(mode.as_utf16());
                Utf16String::from_utf16(message)
            }
        }
    }
}

fn trim(units: &[u16]) -> &[u16] {
    let start = units
        .iter()
        .position(|unit| *unit > 0x0020)
        .unwrap_or(units.len());
    let end = units
        .iter()
        .rposition(|unit| *unit > 0x0020)
        .map_or(start, |position| position + 1);
    &units[start..end]
}

fn equals_ignore_case_ascii(actual: &[u16], expected: &[u8]) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| code_unit_equals_ascii_ignore_case(*actual, *expected))
}

fn code_unit_equals_ascii_ignore_case(actual: u16, expected: u8) -> bool {
    let expected = u16::from(expected);
    actual == expected
        || (expected >= u16::from(b'A') && expected <= u16::from(b'Z') && actual == expected + 0x20)
        || (expected == u16::from(b'I') && matches!(actual, 0x0130 | 0x0131))
        || (expected == u16::from(b'S') && actual == 0x017F)
}

#[cfg(test)]
mod tests {
    use super::{StandardInlineMode, StandardInlineModeParseError};
    use crate::util::Utf16String;

    #[test]
    fn preserves_values_ordinals_display_and_ascii_case_parsing() {
        for (ordinal, value) in StandardInlineMode::VALUES.into_iter().enumerate() {
            assert_eq!(value.ordinal(), ordinal);
            assert_eq!(
                StandardInlineMode::parse(Some(&Utf16String::from_rust_str(
                    &value.to_string().to_ascii_lowercase()
                ))),
                Ok(value)
            );
        }
        assert_eq!(StandardInlineMode::NONE.to_string(), "NONE");
        assert_eq!(StandardInlineMode::HTML.to_string(), "HTML");
        assert_eq!(StandardInlineMode::XML.to_string(), "XML");
        assert_eq!(StandardInlineMode::TEXT.to_string(), "TEXT");
        assert_eq!(StandardInlineMode::JAVASCRIPT.to_string(), "JAVASCRIPT");
        assert_eq!(StandardInlineMode::CSS.to_string(), "CSS");
    }

    #[test]
    fn preserves_java_trim_unicode_case_and_error_messages() {
        for input in [
            None,
            Some(Utf16String::from_rust_str("")),
            Some(Utf16String::from_utf16([0x0000, 0x0020])),
        ] {
            assert_eq!(
                StandardInlineMode::parse(input.as_ref()),
                Err(StandardInlineModeParseError::NullOrEmpty)
            );
        }
        assert_eq!(
            StandardInlineMode::parse(Some(&Utf16String::from_utf16([
                b'C' as u16,
                0x017F,
                0x017F,
            ]))),
            Ok(StandardInlineMode::CSS)
        );
        assert_eq!(
            StandardInlineMode::parse(Some(&Utf16String::from_rust_str(" HTML "))),
            Err(StandardInlineModeParseError::Unrecognized(
                Utf16String::from_rust_str(" HTML ")
            ))
        );
        assert_eq!(
            StandardInlineModeParseError::NullOrEmpty.to_string(),
            "Inline mode cannot be null or empty"
        );
        assert_eq!(
            StandardInlineModeParseError::Unrecognized(Utf16String::from_rust_str("RAW"))
                .to_string(),
            "Unrecognized inline mode: RAW"
        );
        assert_eq!(
            StandardInlineModeParseError::NullOrEmpty.class_name(),
            "java.lang.IllegalArgumentException"
        );
        assert_eq!(
            StandardInlineModeParseError::NullOrEmpty.message(),
            Utf16String::from_rust_str("Inline mode cannot be null or empty")
        );
        assert_eq!(
            StandardInlineModeParseError::Unrecognized(Utf16String::from_utf16([0xD800]))
                .message()
                .as_utf16()
                .last(),
            Some(&0xD800)
        );
    }
}
