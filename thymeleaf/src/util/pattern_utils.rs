use std::error::Error;
use std::fmt::{Display, Formatter};

use regex::Regex;

/// Thymeleaf 字符串模式编译失败。
///
/// 对应 Java: `org.thymeleaf.util.PatternUtils` 运行时可能产生的
/// `NullPointerException` 与 `java.util.regex.PatternSyntaxException`。
///
/// Java 正则诊断文本依赖 JDK 版本；Rust 保留稳定异常类别、转换后的 Java 正则文本
/// 和底层编译诊断，不伪造特定 JDK 的插入符位置排版。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternUtilsError {
    /// Java 输入 pattern 为 null 时的隐式空引用错误。
    NullPointer,
    /// 转换后的正则表达式无法编译。
    PatternSyntax {
        /// `PatternUtils` 按 Java 替换顺序生成的表达式。
        pattern: String,
        /// Rust 正则编译器的诊断文本。
        message: String,
    },
}

impl PatternUtilsError {
    /// 返回对应的 Java 异常类名。
    ///
    /// # 返回
    /// `java.lang.NullPointerException` 或
    /// `java.util.regex.PatternSyntaxException`。
    #[must_use]
    pub const fn class_name(&self) -> &'static str {
        match self {
            Self::NullPointer => "java.lang.NullPointerException",
            Self::PatternSyntax { .. } => "java.util.regex.PatternSyntaxException",
        }
    }

    /// 返回稳定的错误消息。
    ///
    /// # 返回
    /// null 输入没有稳定 JDK 消息，因此返回 `None`；语法错误返回底层编译诊断。
    /// 对应 Java 语义：Java 接口/超类方法 `getMessage()` 的 Rust 移植（`PatternUtils` 继承路径）。
    #[must_use]
    pub fn get_message(&self) -> Option<&str> {
        match self {
            Self::NullPointer => None,
            Self::PatternSyntax { message, .. } => Some(message),
        }
    }

    /// 返回发生语法错误时转换后的 Java 正则文本。
    ///
    /// # 返回
    /// 语法错误返回表达式；null 输入返回 `None`。
    /// 对应 Java 语义：`PatternUtils` 的 `get_pattern` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn get_pattern(&self) -> Option<&str> {
        match self {
            Self::NullPointer => None,
            Self::PatternSyntax { pattern, .. } => Some(pattern),
        }
    }
}

impl Display for PatternUtilsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(message) = self.get_message() {
            formatter.write_str(message)?;
        }
        Ok(())
    }
}

impl Error for PatternUtilsError {}

/// 由 Thymeleaf 字符串模式编译得到的只读正则。
///
/// 对应 Java 返回类型: `java.util.regex.Pattern`。
///
/// 对外保留 Java `Pattern#pattern()` 可观察的原始表达式；内部表达式额外包裹完整匹配
/// 边界，以复现 `Pattern.matcher(value).matches()` 而不是 `find()`。
#[derive(Clone, Debug)]
pub struct StringPattern {
    process_pattern: String,
    regex: Regex,
}

impl StringPattern {
    /// 返回 Java `Pattern#pattern()` 对应的表达式文本。
    ///
    /// # 返回
    /// 以 `^` 和 `$` 包围、且按 `PatternUtils` 替换规则生成的文本。
    /// 对应 Java 语义：`PatternUtils` 的 `as_str` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.process_pattern
    }

    /// 对整个输入执行 Java `Matcher#matches()` 语义。
    ///
    /// # 参数
    /// - `input`：待匹配文本；`None` 对应 Java null。
    ///
    /// # 返回
    /// 整个输入满足模式时返回 `true`。
    ///
    /// # 错误
    /// input 为 `None` 时返回 Java `NullPointerException` 对应错误。
    pub fn matches(&self, input: Option<&str>) -> Result<bool, PatternUtilsError> {
        let input = input.ok_or(PatternUtilsError::NullPointer)?;
        Ok(self.regex.is_match(input))
    }
}

/// Thymeleaf 字符串通配模式工具。
///
/// 对应 Java: `org.thymeleaf.util.PatternUtils`。
///
/// 本对象严格按上游链式 `String.replace` 顺序转义 `.()[]?$+`，并把每个 `*`
/// 转换成非贪婪任意字符片段。Java 有意未转义的 `\`、`^`、`{}` 和 `|` 继续具有
/// 正则意义。
pub struct PatternUtils;

impl PatternUtils {
    /// 将 Thymeleaf 字符串模式转换并编译为正则。
    ///
    /// 对应 Java: `PatternUtils#strPatternToPattern(String)`。
    ///
    /// # 参数
    /// - `pattern`：字符串模式；`None` 对应 Java null。
    ///
    /// # 返回
    /// 保留 Java 表达式文本和完整匹配行为的 [`StringPattern`]。
    ///
    /// # 错误
    /// null 输入返回空引用错误；无效正则返回语法错误并保留转换后表达式。
    pub fn str_pattern_to_pattern(
        pattern: Option<&str>,
    ) -> Result<StringPattern, PatternUtilsError> {
        let pattern = pattern.ok_or(PatternUtilsError::NullPointer)?;
        let process_pattern = pattern_source(pattern);
        let translated_pattern = translate_java_regex(&process_pattern);
        let full_match_pattern = format!(r"\A(?:{translated_pattern})\z");
        let regex =
            Regex::new(&full_match_pattern).map_err(|error| PatternUtilsError::PatternSyntax {
                pattern: process_pattern.clone(),
                message: error.to_string(),
            })?;
        Ok(StringPattern {
            process_pattern,
            regex,
        })
    }
}

fn pattern_source(pattern: &str) -> String {
    let pattern = pattern
        .replace('.', r"\.")
        .replace('(', r"\(")
        .replace(')', r"\)")
        .replace('[', r"\[")
        .replace(']', r"\]")
        .replace('?', r"\?")
        .replace('$', r"\$")
        .replace('+', r"\+")
        .replace('*', "(?:.*?)");
    format!("^{pattern}$")
}

fn translate_java_regex(pattern: &str) -> String {
    let mut translated = String::with_capacity(pattern.len() + 16);
    let mut remaining = pattern;
    while !remaining.is_empty() {
        // Java 的引用段必须先于星号片段识别：引用段内的替换产物只是普通文本。
        if let Some(quoted) = remaining.strip_prefix(r"\Q") {
            if let Some(end) = quoted.find(r"\E") {
                translated.push_str(&regex::escape(&quoted[..end]));
                remaining = &quoted[end + 2..];
            } else {
                translated.push_str(&regex::escape(quoted));
                remaining = "";
            }
            continue;
        }

        // PatternUtils 会转义输入中的圆括号，因此该片段只能由星号替换产生。
        if let Some(suffix) = remaining.strip_prefix("(?:.*?)") {
            translated.push_str(r"(?:[^\n\r\u{0085}\u{2028}\u{2029}]*?)");
            remaining = suffix;
            continue;
        }

        let character = remaining.chars().next().expect("non-empty pattern");
        remaining = &remaining[character.len_utf8()..];
        if character != '\\' {
            translated.push(character);
            continue;
        }

        let Some(escaped) = remaining.chars().next() else {
            translated.push('\\');
            continue;
        };
        remaining = &remaining[escaped.len_utf8()..];
        match escaped {
            'd' => translated.push_str("[0-9]"),
            'D' => translated.push_str("[^0-9]"),
            'w' => translated.push_str("[A-Za-z_0-9]"),
            'W' => translated.push_str("[^A-Za-z_0-9]"),
            's' => translated.push_str(r"[\x20\t\n\x0B\f\r]"),
            'S' => translated.push_str(r"[^\x20\t\n\x0B\f\r]"),
            'h' => translated.push_str(
                r"[\x20\t\u{00A0}\u{1680}\u{180E}\u{2000}-\u{200A}\u{202F}\u{205F}\u{3000}]",
            ),
            'H' => translated.push_str(
                r"[^\x20\t\u{00A0}\u{1680}\u{180E}\u{2000}-\u{200A}\u{202F}\u{205F}\u{3000}]",
            ),
            'v' => translated.push_str(r"[\n\x0B\f\r\u{0085}\u{2028}\u{2029}]"),
            'V' => translated.push_str(r"[^\n\x0B\f\r\u{0085}\u{2028}\u{2029}]"),
            'R' => translated.push_str(r"(?:\r\n|[\n\x0B\f\r\u{0085}\u{2028}\u{2029}])"),
            other => {
                translated.push('\\');
                translated.push(other);
            }
        }
    }
    translated
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::{PatternUtils, PatternUtilsError, translate_java_regex};

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write_str(&mut self, _value: &str) -> std::fmt::Result {
            Err(std::fmt::Error)
        }
    }

    #[test]
    fn converts_in_exact_java_replacement_order_and_matches_entire_input() {
        let pattern =
            PatternUtils::str_pattern_to_pattern(Some("a.(b)[c]?$+*")).expect("valid pattern");
        assert_eq!(pattern.as_str(), r"^a\.\(b\)\[c\]\?\$\+(?:.*?)$");
        assert_eq!(pattern.matches(Some("a.(b)[c]?$+tail")), Ok(true));
        assert_eq!(pattern.matches(Some("prefixa.(b)[c]?$+tail")), Ok(false));
    }

    #[test]
    fn preserves_java_ascii_classes_and_line_terminator_dot_rules() {
        let digit = PatternUtils::str_pattern_to_pattern(Some(r"\d*")).expect("digit");
        assert_eq!(digit.matches(Some("1tail")), Ok(true));
        assert_eq!(digit.matches(Some("١tail")), Ok(false));
        assert_eq!(digit.matches(Some("1line\nbreak")), Ok(false));

        let classes = [
            (r"\D", "x", true),
            (r"\w", "_", true),
            (r"\W", "-", true),
            (r"\s", "\t", true),
            (r"\S", "x", true),
            (r"\h", "\u{3000}", true),
            (r"\H", "x", true),
            (r"\v", "\u{2028}", true),
            (r"\V", "x", true),
            (r"\R", "\r\n", true),
        ];
        for (source, input, expected) in classes {
            let pattern = PatternUtils::str_pattern_to_pattern(Some(source)).expect("class");
            assert_eq!(pattern.matches(Some(input)), Ok(expected));
        }
    }

    #[test]
    fn preserves_regex_operators_that_java_source_does_not_escape() {
        for (source, matching, rejected) in [
            ("foo|bar", "foo", "foobar"),
            ("a{2}", "aa", "a"),
            ("^name", "name", "xname"),
        ] {
            let pattern = PatternUtils::str_pattern_to_pattern(Some(source)).expect("pattern");
            assert_eq!(pattern.matches(Some(matching)), Ok(true));
            assert_eq!(pattern.matches(Some(rejected)), Ok(false));
        }
    }

    #[test]
    fn preserves_java_quoted_literals_before_interpreting_generated_wildcards() {
        let quoted =
            PatternUtils::str_pattern_to_pattern(Some(r"\Qfoo|*\E")).expect("quoted pattern");
        assert_eq!(quoted.as_str(), r"^\Qfoo|(?:.*?)\E$");
        assert_eq!(quoted.matches(Some("foo|(?:.*?)")), Ok(true));
        assert_eq!(quoted.matches(Some("foo|anything")), Ok(false));

        let unterminated =
            PatternUtils::str_pattern_to_pattern(Some(r"\Qfoo")).expect("unterminated quote");
        assert_eq!(unterminated.matches(Some("foo$")), Ok(true));
        assert_eq!(unterminated.matches(Some("foo")), Ok(false));
    }

    #[test]
    fn maps_null_and_syntax_failures_with_stable_metadata() {
        let null = PatternUtils::str_pattern_to_pattern(None).expect_err("null");
        assert_eq!(null, PatternUtilsError::NullPointer);
        assert_eq!(null.class_name(), "java.lang.NullPointerException");
        assert_eq!(null.get_message(), None);
        assert_eq!(null.get_pattern(), None);
        assert_eq!(null.to_string(), "");

        let syntax = PatternUtils::str_pattern_to_pattern(Some("{")).expect_err("syntax");
        assert_eq!(
            syntax.class_name(),
            "java.util.regex.PatternSyntaxException"
        );
        assert_eq!(syntax.get_pattern(), Some("^{$"));
        assert!(syntax.get_message().is_some());
        assert!(!syntax.to_string().is_empty());
        assert!(write!(&mut FailingWriter, "{syntax}").is_err());
    }

    #[test]
    fn preserves_the_trailing_escape_effect_on_the_generated_end_anchor() {
        assert_eq!(translate_java_regex(r"^abc\"), r"^abc\");
        let pattern = PatternUtils::str_pattern_to_pattern(Some(r"abc\")).expect("escaped anchor");
        assert_eq!(pattern.as_str(), r"^abc\$");
        assert_eq!(pattern.matches(Some("abc$")), Ok(true));
        assert_eq!(pattern.matches(Some("abc")), Ok(false));
    }
}
