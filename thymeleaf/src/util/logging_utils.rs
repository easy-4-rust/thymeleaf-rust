use super::utf16_string::{Utf16String, Utf16StringResult};

/// Thymeleaf 日志格式化工具。
///
/// 对应 Java: `org.thymeleaf.util.LoggingUtils`。
///
/// 本对象无状态，保留模板名称的 null、UTF-16 长度、换行替换、头尾截断以及短名称
/// 无需改写时复用原字符串对象的可观察语义。
pub struct LoggingUtils;

impl LoggingUtils {
    /// 将模板名称压缩为适合日志输出的形式。
    ///
    /// 对应 Java: `LoggingUtils#loggifyTemplateName(String)`。
    ///
    /// # 参数
    /// - `template`：模板名称；`None` 对应 Java null。
    ///
    /// # 返回
    /// null 输入返回 `None`。长度不超过 120 个 UTF-16 单元时仅将 LF 替换为空格；
    /// 更长时保留前 35 单元和后 80 单元，中间插入 `[...]`。无需替换的短字符串
    /// 返回借用分支，从而保留 Java 对象身份。
    #[must_use]
    pub fn loggify_template_name(template: Option<&Utf16String>) -> Option<Utf16StringResult<'_>> {
        let template = template?;
        if template.len() <= 120 {
            if !template.as_utf16().contains(&u16::from(b'\n')) {
                return Some(Utf16StringResult::Borrowed(template));
            }
            return Some(Utf16StringResult::Owned(Utf16String::from_utf16(
                replace_line_feeds(template.as_utf16()),
            )));
        }

        let mut result = Vec::with_capacity(120);
        let units = template.as_utf16();
        result.extend(replace_line_feeds(&units[..35]));
        result.extend("[...]".encode_utf16());
        result.extend(replace_line_feeds(&units[units.len() - 80..]));
        Some(Utf16StringResult::Owned(Utf16String::from_utf16(result)))
    }

    /// 格式化有效 Unicode Rust 字符串供内部 `Display` 调用。
    ///
    /// # 参数
    /// - `template`：可空 Rust 字符串。
    ///
    /// # 返回
    /// 调用精确 UTF-16 实现后生成的可显示文本；若截断恰好切开代理对，孤立代理项
    /// 因 Rust `String` 限制显示为替换字符。需要无损比较时使用
    /// [`Self::loggify_template_name`]。
    /// 对应 Java 语义：`LoggingUtils` 的 `loggify_str` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub(crate) fn loggify_str(template: Option<&str>) -> Option<String> {
        let template = template.map(Utf16String::from_rust_str)?;
        Self::loggify_template_name(Some(&template))
            .map(Utf16StringResult::into_owned)
            .map(|result| result.to_string_lossy())
    }
}

fn replace_line_feeds(source: &[u16]) -> Vec<u16> {
    source
        .iter()
        .map(|unit| {
            if *unit == u16::from(b'\n') {
                u16::from(b' ')
            } else {
                *unit
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::LoggingUtils;
    use crate::util::Utf16String;

    #[test]
    fn preserves_null_empty_and_short_reference_identity() {
        assert!(LoggingUtils::loggify_template_name(None).is_none());

        let empty = Utf16String::from_rust_str("");
        assert!(empty.is_empty());
        let result = LoggingUtils::loggify_template_name(Some(&empty)).expect("result");
        assert!(result.is_borrowed_from(&empty));
        assert_eq!(result.as_utf16_string().as_utf16(), &[] as &[u16]);
        assert_eq!(result.as_utf16_string().to_string_lossy(), "");
        assert_eq!(format!("{empty:?}"), "Utf16String { utf16: [] }");
    }

    #[test]
    fn replaces_short_line_feeds_with_an_independent_value() {
        let source = Utf16String::from_rust_str("home\npage");
        let result = LoggingUtils::loggify_template_name(Some(&source)).expect("result");
        assert!(!result.is_borrowed_from(&source));
        assert_eq!(result.as_utf16_string().to_string_lossy(), "home page");
        assert_eq!(result.into_owned().to_string_lossy(), "home page");
        assert_eq!(source.to_string_lossy(), "home\npage");
    }

    #[test]
    fn truncates_by_java_utf16_units_and_can_preserve_isolated_surrogates() {
        let mut utf16 = vec![u16::from(b'a'); 34];
        utf16.extend("😀".encode_utf16());
        utf16.extend(vec![u16::from(b'b'); 90]);
        let source = Utf16String::from_utf16(utf16);
        let result = LoggingUtils::loggify_template_name(Some(&source))
            .expect("result")
            .into_owned();

        assert_eq!(result.len(), 120);
        assert_eq!(result.as_utf16()[34], 0xD83D);
        assert_eq!(
            &result.as_utf16()[35..40],
            "[...]".encode_utf16().collect::<Vec<_>>().as_slice()
        );
        assert_eq!(result.as_utf16().last(), Some(&u16::from(b'b')));
    }

    #[test]
    fn internal_string_adapter_matches_display_contract() {
        assert_eq!(LoggingUtils::loggify_str(None), None);
        assert_eq!(
            LoggingUtils::loggify_str(Some("home\npage")),
            Some("home page".to_owned())
        );
        let long = format!("{}\n{}尾", "a".repeat(34), "b".repeat(90));
        let result = LoggingUtils::loggify_str(Some(&long)).expect("result");
        assert!(result.starts_with(&format!("{} ", "a".repeat(34))));
        assert!(result.contains("[...]"));
        assert!(result.ends_with('尾'));
    }
}
