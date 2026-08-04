use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;

use super::{JavaLocale, Utf16String, ValidateError};

static RANDOM_STATE: AtomicU64 = AtomicU64::new(0);
const ALPHA_NUMERIC: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// 字符串工具操作错误。
/// 对应 Java 语义：`StringUtils` 的 Rust 侧类型 `StringUtilsError`。
#[derive(Debug, Error)]
pub enum StringUtilsError {
    /// Java 参数校验错误。
    #[error(transparent)]
    Validation(#[from] ValidateError),
    /// Java UTF-16 下标范围错误。
    #[error("String index out of range: {index}")]
    IndexOutOfBounds {
        /// 失败下标。
        index: i32,
    },
}

/// Thymeleaf 字符串操作基础工具。
///
/// 对应 Java: `org.thymeleaf.util.StringUtils`。
pub struct StringUtils;

impl StringUtils {
    /// 对可空对象文本执行 null-safe `toString()`。
    /// 对应 Java: `StringUtils#toString()`。
    #[must_use]
    pub fn to_string(target: Option<&Utf16String>) -> Option<Utf16String> {
        target.cloned()
    }

    /// 把超长文本截成 `max_size - 3` 个 UTF-16 单元并追加省略号。
    /// 对应 Java: `StringUtils#abbreviate()`。
    pub fn abbreviate(
        target: Option<&Utf16String>,
        max_size: i32,
    ) -> Result<Option<Utf16String>, StringUtilsError> {
        require(max_size >= 3, "Maximum size must be greater or equal to 3")?;
        let Some(target) = target else {
            return Ok(None);
        };
        let max_size = max_size as usize;
        if target.len() <= max_size {
            return Ok(Some(target.clone()));
        }
        let mut units = target.as_utf16()[..max_size - 3].to_vec();
        units.extend("...".encode_utf16());
        Ok(Some(Utf16String::from_utf16(units)))
    }

    /// 按 Java `toString()` 文本执行 null-safe相等比较。
    /// 对应 Java: `StringUtils#equals()`。
    #[must_use]
    pub fn equals(first: Option<&Utf16String>, second: Option<&Utf16String>) -> bool {
        first == second
    }

    /// 按 Java `String#equalsIgnoreCase` 执行 null-safe比较。
    /// 对应 Java: `StringUtils#equalsIgnoreCase()`。
    #[must_use]
    pub fn equals_ignore_case(first: Option<&Utf16String>, second: Option<&Utf16String>) -> bool {
        match (first, second) {
            (None, None) => true,
            (Some(first), Some(second)) => first
                .to_string_lossy()
                .eq_ignore_ascii_case(&second.to_string_lossy()),
            (None, Some(_)) | (Some(_), None) => false,
        }
    }

    /// 判断文本是否包含片段。
    /// 对应 Java: `StringUtils#contains()`。
    pub fn contains(
        target: Option<&Utf16String>,
        fragment: Option<&Utf16String>,
    ) -> Result<bool, StringUtilsError> {
        let target = required(target, "Cannot apply contains on null")?;
        let fragment = required(fragment, "Fragment cannot be null")?;
        Ok(find_utf16(target.as_utf16(), fragment.as_utf16(), 0).is_some())
    }

    /// 使用指定 Locale 的大小写规则判断是否包含片段。
    /// 对应 Java: `StringUtils#containsIgnoreCase()`。
    pub fn contains_ignore_case(
        target: Option<&Utf16String>,
        fragment: Option<&Utf16String>,
        locale: Option<&JavaLocale>,
    ) -> Result<bool, StringUtilsError> {
        let target = required(target, "Cannot apply containsIgnoreCase on null")?;
        let fragment = required(fragment, "Fragment cannot be null")?;
        let locale = required(locale, "Locale cannot be null")?;
        let target = to_upper_case_for_locale(target, locale);
        let fragment = to_upper_case_for_locale(fragment, locale);
        Ok(find_utf16(target.as_utf16(), fragment.as_utf16(), 0).is_some())
    }

    /// 判断文本是否以前缀开始。
    /// 对应 Java: `StringUtils#startsWith()`。
    pub fn starts_with(
        target: Option<&Utf16String>,
        prefix: Option<&Utf16String>,
    ) -> Result<bool, StringUtilsError> {
        Ok(required(target, "Cannot apply startsWith on null")?
            .as_utf16()
            .starts_with(required(prefix, "Prefix cannot be null")?.as_utf16()))
    }

    /// 判断文本是否以后缀结束。
    /// 对应 Java: `StringUtils#endsWith()`。
    pub fn ends_with(
        target: Option<&Utf16String>,
        suffix: Option<&Utf16String>,
    ) -> Result<bool, StringUtilsError> {
        Ok(required(target, "Cannot apply endsWith on null")?
            .as_utf16()
            .ends_with(required(suffix, "Suffix cannot be null")?.as_utf16()))
    }

    /// 返回 `[begin_index,end_index)` UTF-16 子串。
    /// 对应 Java: `StringUtils#substring()`。
    pub fn substring(
        target: Option<&Utf16String>,
        begin_index: i32,
        end_index: i32,
    ) -> Result<Option<Utf16String>, StringUtilsError> {
        let Some(target) = target else {
            return Ok(None);
        };
        require(begin_index >= 0, "Begin index must be >= 0")?;
        let begin = usize::try_from(begin_index).unwrap_or(usize::MAX);
        let end = usize::try_from(end_index).unwrap_or(usize::MAX);
        if begin > end || end > target.len() {
            return Err(StringUtilsError::IndexOutOfBounds { index: end_index });
        }
        Ok(Some(Utf16String::from_utf16(
            target.as_utf16()[begin..end].to_vec(),
        )))
    }

    /// 返回从指定 UTF-16 下标到末尾的子串。
    /// 对应 Java 语义：`StringUtils` 的 `substring_from` 行为（Rust 侧辅助/私有路径）。
    pub fn substring_from(
        target: Option<&Utf16String>,
        begin_index: i32,
    ) -> Result<Option<Utf16String>, StringUtilsError> {
        let Some(target) = target else {
            return Ok(None);
        };
        require(
            begin_index >= 0 && usize::try_from(begin_index).is_ok_and(|v| v < target.len()),
            &format!("beginIndex must be >= 0 and < {}", target.len()),
        )?;
        Ok(Some(Utf16String::from_utf16(
            target.as_utf16()[begin_index as usize..].to_vec(),
        )))
    }

    /// 返回第一次出现片段之后的文本；未出现时返回 null。
    /// 对应 Java: `StringUtils#substringAfter()`。
    pub fn substring_after(
        target: Option<&Utf16String>,
        substring: Option<&Utf16String>,
    ) -> Result<Option<Utf16String>, StringUtilsError> {
        let substring = required(substring, "Parameter substring cannot be null")?;
        let Some(target) = target else {
            return Ok(None);
        };
        Ok(
            find_utf16(target.as_utf16(), substring.as_utf16(), 0).map(|index| {
                Utf16String::from_utf16(target.as_utf16()[index + substring.len()..].to_vec())
            }),
        )
    }

    /// 返回第一次出现片段之前的文本；未出现时返回 null。
    /// 对应 Java: `StringUtils#substringBefore()`。
    pub fn substring_before(
        target: Option<&Utf16String>,
        substring: Option<&Utf16String>,
    ) -> Result<Option<Utf16String>, StringUtilsError> {
        let substring = required(substring, "Parameter substring cannot be null")?;
        let Some(target) = target else {
            return Ok(None);
        };
        Ok(find_utf16(target.as_utf16(), substring.as_utf16(), 0)
            .map(|index| Utf16String::from_utf16(target.as_utf16()[..index].to_vec())))
    }

    /// 在非空目标前追加前缀。
    /// 对应 Java: `StringUtils#prepend()`。
    pub fn prepend(
        target: Option<&Utf16String>,
        prefix: Option<&Utf16String>,
    ) -> Result<Option<Utf16String>, StringUtilsError> {
        let prefix = required(prefix, "Prefix cannot be null")?;
        Ok(target.map(|target| concatenate(prefix, target)))
    }

    /// 在非空目标后追加后缀。
    /// 对应 Java: `StringUtils#append()`。
    pub fn append(
        target: Option<&Utf16String>,
        suffix: Option<&Utf16String>,
    ) -> Result<Option<Utf16String>, StringUtilsError> {
        let suffix = required(suffix, "Suffix cannot be null")?;
        Ok(target.map(|target| concatenate(target, suffix)))
    }

    /// 重复目标指定次数。
    /// 对应 Java: `StringUtils#repeat()`。
    #[must_use]
    pub fn repeat(target: Option<&Utf16String>, times: i32) -> Option<Utf16String> {
        target.map(|target| {
            let mut units = Vec::new();
            for _ in 0..times.max(0) {
                units.extend_from_slice(target.as_utf16());
            }
            Utf16String::from_utf16(units)
        })
    }

    /// 拼接所有值，以空串替代 null。
    /// 对应 Java: `StringUtils#concat()`。
    #[must_use]
    pub fn concat(values: Option<&[Option<Utf16String>]>) -> Utf16String {
        Self::concat_replace_nulls(Some(&Utf16String::from_rust_str("")), values)
    }

    /// 拼接所有值，以指定文本替代 null。
    /// 对应 Java: `StringUtils#concatReplaceNulls()`。
    #[must_use]
    pub fn concat_replace_nulls(
        null_value: Option<&Utf16String>,
        values: Option<&[Option<Utf16String>]>,
    ) -> Utf16String {
        let mut units = Vec::new();
        for value in values.unwrap_or(&[]) {
            if let Some(value) = value.as_ref().or(null_value) {
                units.extend_from_slice(value.as_utf16());
            }
        }
        Utf16String::from_utf16(units)
    }

    /// 返回片段第一次出现的 UTF-16 下标，未出现时返回 -1。
    /// 对应 Java: `StringUtils#indexOf()`。
    pub fn index_of(
        target: Option<&Utf16String>,
        fragment: Option<&Utf16String>,
    ) -> Result<i32, StringUtilsError> {
        let target = required(target, "Cannot apply indexOf on null")?;
        let fragment = required(fragment, "Fragment cannot be null")?;
        Ok(find_utf16(target.as_utf16(), fragment.as_utf16(), 0)
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1))
    }

    /// 判断字符串为 null 或零长度。
    /// 对应 Java: `StringUtils#isEmpty()`。
    #[must_use]
    pub fn is_empty(target: Option<&Utf16String>) -> bool {
        target.is_none_or(Utf16String::is_empty)
    }

    /// 判断字符串为 null、空或全部为 Java whitespace。
    /// 对应 Java: `StringUtils#isEmptyOrWhitespace()`。
    #[must_use]
    pub fn is_empty_or_whitespace(target: Option<&Utf16String>) -> bool {
        target.is_none_or(|target| {
            target.is_empty()
                || target
                    .as_utf16()
                    .iter()
                    .all(|unit| is_java_whitespace(*unit))
        })
    }

    /// 使用分隔符连接值；null 元素按文本 `null`。
    /// 对应 Java: `StringUtils#join()`。
    pub fn join(
        target: Option<&[Option<Utf16String>]>,
        separator: Option<&Utf16String>,
    ) -> Result<Option<Utf16String>, StringUtilsError> {
        let separator = required(separator, "Separator cannot be null")?;
        let Some(target) = target else {
            return Ok(None);
        };
        let null = Utf16String::from_rust_str("null");
        let mut units = Vec::new();
        for (index, value) in target.iter().enumerate() {
            if index != 0 {
                units.extend_from_slice(separator.as_utf16());
            }
            units.extend_from_slice(value.as_ref().unwrap_or(&null).as_utf16());
        }
        Ok(Some(Utf16String::from_utf16(units)))
    }

    /// 按 Java `StringTokenizer` 的“分隔符字符集合”规则拆分文本。
    /// 对应 Java: `StringUtils#split()`。
    pub fn split(
        target: Option<&Utf16String>,
        separator: Option<&Utf16String>,
    ) -> Result<Option<Vec<Utf16String>>, StringUtilsError> {
        let separator = required(separator, "Separator cannot be null")?;
        let Some(target) = target else {
            return Ok(None);
        };
        let delimiters = separator.as_utf16();
        Ok(Some(
            target
                .as_utf16()
                .split(|unit| delimiters.contains(unit))
                .filter(|token| !token.is_empty())
                .map(|token| Utf16String::from_utf16(token.to_vec()))
                .collect(),
        ))
    }

    /// 返回 `toString()` 后的 UTF-16 长度。
    /// 对应 Java: `StringUtils#length()`。
    pub fn length(target: Option<&Utf16String>) -> Result<i32, StringUtilsError> {
        let target = required(target, "Cannot apply length on null")?;
        Ok(i32::try_from(target.len()).unwrap_or(i32::MAX))
    }

    /// 非正则地替换全部非重叠片段。
    /// 对应 Java: `StringUtils#replace()`。
    pub fn replace(
        target: Option<&Utf16String>,
        before: Option<&Utf16String>,
        after: Option<&Utf16String>,
    ) -> Result<Option<Utf16String>, StringUtilsError> {
        let before = required(before, "Parameter \"before\" cannot be null")?;
        let after = required(after, "Parameter \"after\" cannot be null")?;
        let Some(target) = target else {
            return Ok(None);
        };
        if target.is_empty() || before.is_empty() {
            return Ok(Some(target.clone()));
        }
        let mut result = Vec::new();
        let mut last = 0usize;
        while let Some(index) = find_utf16(target.as_utf16(), before.as_utf16(), last) {
            result.extend_from_slice(&target.as_utf16()[last..index]);
            result.extend_from_slice(after.as_utf16());
            last = index + before.len();
        }
        if last == 0 {
            return Ok(Some(target.clone()));
        }
        result.extend_from_slice(&target.as_utf16()[last..]);
        Ok(Some(Utf16String::from_utf16(result)))
    }

    /// 使用 Locale 转为大写。
    /// 对应 Java: `StringUtils#toUpperCase()`。
    pub fn to_upper_case(
        target: Option<&Utf16String>,
        locale: Option<&JavaLocale>,
    ) -> Result<Option<Utf16String>, StringUtilsError> {
        let locale = required(locale, "Locale cannot be null")?;
        Ok(target.map(|target| to_upper_case_for_locale(target, locale)))
    }

    /// 使用 Locale 转为小写。
    /// 对应 Java: `StringUtils#toLowerCase()`。
    pub fn to_lower_case(
        target: Option<&Utf16String>,
        locale: Option<&JavaLocale>,
    ) -> Result<Option<Utf16String>, StringUtilsError> {
        let locale = required(locale, "Locale cannot be null")?;
        Ok(target.map(|target| to_lower_case_for_locale(target, locale)))
    }

    /// 按 Java `String#trim` 删除首尾 `<= U+0020` 的单元。
    /// 对应 Java: `StringUtils#trim()`。
    #[must_use]
    pub fn trim(target: Option<&Utf16String>) -> Option<Utf16String> {
        target.map(|target| {
            let units = target.as_utf16();
            let start = units
                .iter()
                .position(|unit| *unit > 0x20)
                .unwrap_or(units.len());
            let end = units
                .iter()
                .rposition(|unit| *unit > 0x20)
                .map_or(start, |index| index + 1);
            Utf16String::from_utf16(units[start..end].to_vec())
        })
    }

    /// 删除全部 whitespace/控制字符并按默认 Locale 小写。
    /// 对应 Java: `StringUtils#pack()`。
    #[must_use]
    pub fn pack(target: Option<&Utf16String>) -> Option<Utf16String> {
        target.map(|target| {
            let compact = Utf16String::from_utf16(
                target
                    .as_utf16()
                    .iter()
                    .copied()
                    .filter(|unit| *unit > 0x20 && !is_java_whitespace(*unit))
                    .collect::<Vec<_>>(),
            );
            to_lower_case_for_locale(&compact, &JavaLocale::get_default())
        })
    }

    /// 把第一个 UTF-16 字符转为 title case。
    /// 对应 Java: `StringUtils#capitalize()`。
    #[must_use]
    pub fn capitalize(target: Option<&Utf16String>) -> Option<Utf16String> {
        change_first_case(target, true)
    }

    /// 把第一个 UTF-16 字符转为小写。
    /// 对应 Java: `StringUtils#unCapitalize()`。
    #[must_use]
    pub fn un_capitalize(target: Option<&Utf16String>) -> Option<Utf16String> {
        change_first_case(target, false)
    }

    /// 按 whitespace 界定单词并 title-case 每个首字符。
    /// 对应 Java: `StringUtils#capitalizeWords()`。
    #[must_use]
    pub fn capitalize_words(
        target: Option<&Utf16String>,
        delimiters: Option<&Utf16String>,
    ) -> Option<Utf16String> {
        target.map(|target| {
            let delimiters = delimiters.map(Utf16String::as_utf16);
            let mut units = target.as_utf16().to_vec();
            let mut at_word_start = true;
            for unit in &mut units {
                let delimiter = delimiters
                    .map_or_else(|| is_java_whitespace(*unit), |values| values.contains(unit));
                if delimiter {
                    at_word_start = true;
                } else if at_word_start {
                    *unit = title_case_unit(*unit);
                    at_word_start = false;
                }
            }
            Utf16String::from_utf16(units)
        })
    }

    /// 按 HTML4/XML 兼容规则转义文本。
    /// 对应 Java: `StringUtils#escapeXml()`。
    #[must_use]
    pub fn escape_xml(target: Option<&Utf16String>) -> Option<Utf16String> {
        target.map(|target| escape_common(target, EscapeKind::Xml))
    }

    /// 按 JavaScript 字符串规则转义文本。
    /// 对应 Java: `StringUtils#escapeJavaScript()`。
    #[must_use]
    pub fn escape_java_script(target: Option<&Utf16String>) -> Option<Utf16String> {
        target.map(|target| escape_common(target, EscapeKind::JavaScript))
    }

    /// 按 Java 字符串规则转义文本。
    /// 对应 Java: `StringUtils#escapeJava()`。
    #[must_use]
    pub fn escape_java(target: Option<&Utf16String>) -> Option<Utf16String> {
        target.map(|target| escape_common(target, EscapeKind::Java))
    }

    /// 反解 JavaScript 字符串转义。
    /// 对应 Java: `StringUtils#unescapeJavaScript()`。
    #[must_use]
    pub fn unescape_java_script(target: Option<&Utf16String>) -> Option<Utf16String> {
        target.map(unescape_backslashes)
    }

    /// 反解 Java 字符串转义。
    /// 对应 Java: `StringUtils#unescapeJava()`。
    #[must_use]
    pub fn unescape_java(target: Option<&Utf16String>) -> Option<Utf16String> {
        target.map(unescape_backslashes)
    }

    /// 返回只含数字和大写英文字母的随机文本。
    /// 对应 Java: `StringUtils#randomAlphanumeric()`。
    #[must_use]
    pub fn random_alphanumeric(count: i32) -> Utf16String {
        let count = usize::try_from(count).unwrap_or(0);
        let mut state = next_random_seed();
        let mut bytes = Vec::with_capacity(count);
        for _ in 0..count {
            state = state.wrapping_mul(0x5DEECE66D).wrapping_add(0xB);
            bytes.push(ALPHA_NUMERIC[((state >> 16) as usize) % ALPHA_NUMERIC.len()]);
        }
        RANDOM_STATE.store(state, Ordering::Relaxed);
        Utf16String::from_rust_str(&String::from_utf8(bytes).expect("ASCII"))
    }
}

fn required<'a, T>(value: Option<&'a T>, message: &str) -> Result<&'a T, StringUtilsError> {
    value.ok_or_else(|| {
        StringUtilsError::Validation(ValidateError::IllegalArgument {
            message: Some(message.to_owned()),
        })
    })
}

fn require(condition: bool, message: &str) -> Result<(), StringUtilsError> {
    if condition {
        Ok(())
    } else {
        Err(StringUtilsError::Validation(
            ValidateError::IllegalArgument {
                message: Some(message.to_owned()),
            },
        ))
    }
}

fn find_utf16(haystack: &[u16], needle: &[u16], from: usize) -> Option<usize> {
    if needle.is_empty() {
        return Some(from.min(haystack.len()));
    }
    haystack
        .get(from..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| position + from)
}

fn concatenate(first: &Utf16String, second: &Utf16String) -> Utf16String {
    let mut units = first.as_utf16().to_vec();
    units.extend_from_slice(second.as_utf16());
    Utf16String::from_utf16(units)
}

fn is_java_whitespace(unit: u16) -> bool {
    matches!(
        unit,
        0x0009..=0x000D
            | 0x001C..=0x0020
            | 0x1680
            | 0x2000..=0x2006
            | 0x2008..=0x200A
            | 0x2028
            | 0x2029
            | 0x205F
            | 0x3000
    )
}

fn locale_is_turkic(locale: &JavaLocale) -> bool {
    matches!(
        locale
            .get_language()
            .to_string_lossy()
            .to_ascii_lowercase()
            .as_str(),
        "tr" | "az"
    )
}

fn to_upper_case_for_locale(value: &Utf16String, locale: &JavaLocale) -> Utf16String {
    let turkic = locale_is_turkic(locale);
    let text = value.to_string_lossy();
    let mut result = String::new();
    for character in text.chars() {
        if turkic && character == 'i' {
            result.push('\u{130}');
        } else if turkic && character == '\u{131}' {
            result.push('I');
        } else {
            result.extend(character.to_uppercase());
        }
    }
    Utf16String::from_rust_str(&result)
}

fn to_lower_case_for_locale(value: &Utf16String, locale: &JavaLocale) -> Utf16String {
    let turkic = locale_is_turkic(locale);
    let text = value.to_string_lossy();
    let mut result = String::new();
    for character in text.chars() {
        if turkic && character == 'I' {
            result.push('\u{131}');
        } else if turkic && character == '\u{130}' {
            result.push('i');
        } else {
            result.extend(character.to_lowercase());
        }
    }
    Utf16String::from_rust_str(&result)
}

fn change_first_case(target: Option<&Utf16String>, title: bool) -> Option<Utf16String> {
    target.map(|target| {
        let mut units = target.as_utf16().to_vec();
        if let Some(first) = units.first_mut() {
            *first = if title {
                title_case_unit(*first)
            } else {
                char::from_u32(u32::from(*first))
                    .and_then(|character| character.to_lowercase().next())
                    .map_or(*first, |character| character as u16)
            };
        }
        Utf16String::from_utf16(units)
    })
}

fn title_case_unit(unit: u16) -> u16 {
    char::from_u32(u32::from(unit))
        .and_then(|character| character.to_uppercase().next())
        .map_or(unit, |character| character as u16)
}

enum EscapeKind {
    Xml,
    JavaScript,
    Java,
}

fn escape_common(target: &Utf16String, kind: EscapeKind) -> Utf16String {
    let mut result = String::new();
    for character in target.to_string_lossy().chars() {
        match (character, &kind) {
            ('&', EscapeKind::Xml) => result.push_str("&amp;"),
            ('<', EscapeKind::Xml) => result.push_str("&lt;"),
            ('>', EscapeKind::Xml) => result.push_str("&gt;"),
            ('"', EscapeKind::Xml) => result.push_str("&quot;"),
            ('\'', EscapeKind::Xml) => result.push_str("&#39;"),
            ('\u{8}', _) => result.push_str("\\b"),
            ('\t', _) => result.push_str("\\t"),
            ('\n', _) => result.push_str("\\n"),
            ('\u{c}', _) => result.push_str("\\f"),
            ('\r', _) => result.push_str("\\r"),
            ('"', _) => result.push_str("\\\""),
            ('\'', EscapeKind::JavaScript) => result.push_str("\\'"),
            ('\\', _) => result.push_str("\\\\"),
            ('/', EscapeKind::JavaScript) => result.push_str("\\/"),
            (character, _) if character.is_control() => {
                result.push_str(&format!("\\u{:04X}", character as u32));
            }
            (character, _) => result.push(character),
        }
    }
    Utf16String::from_rust_str(&result)
}

fn unescape_backslashes(target: &Utf16String) -> Utf16String {
    let units = target.as_utf16();
    let mut result = Vec::with_capacity(units.len());
    let mut index = 0usize;
    while index < units.len() {
        if units[index] != b'\\' as u16 || index + 1 >= units.len() {
            result.push(units[index]);
            index += 1;
            continue;
        }
        let next = units[index + 1];
        let simple = match next {
            0x62 => Some(0x08),
            0x74 => Some(0x09),
            0x6E => Some(0x0A),
            0x66 => Some(0x0C),
            0x72 => Some(0x0D),
            0x22 | 0x27 | 0x5C | 0x2F => Some(next),
            _ => None,
        };
        if let Some(value) = simple {
            result.push(value);
            index += 2;
        } else if next == b'u' as u16 && index + 6 <= units.len() {
            let digits = String::from_utf16_lossy(&units[index + 2..index + 6]);
            if let Ok(value) = u16::from_str_radix(&digits, 16) {
                result.push(value);
                index += 6;
            } else {
                result.push(next);
                index += 2;
            }
        } else {
            result.push(next);
            index += 2;
        }
    }
    Utf16String::from_utf16(result)
}

fn next_random_seed() -> u64 {
    let current = RANDOM_STATE.load(Ordering::Relaxed);
    if current != 0 {
        current
    } else {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(1, |duration| duration.as_nanos() as u64)
    }
}
