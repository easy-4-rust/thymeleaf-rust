use std::fmt::{Debug, Formatter};
use std::ptr;

/// Java `String` 的 UTF-16 代码单元适配值。
///
/// 对应 Java: `java.lang.String`，由
/// `org.thymeleaf.util.LoggingUtils#loggifyTemplateName(String)` 使用。
///
/// Java `String#substring` 可以在代理对中间切分，产生 Rust `String` 无法表示的孤立
/// 代理项。本类型保存原始 UTF-16 代码单元，确保模板名日志截断不会用替换字符改变
/// 上游结果。
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct JavaString {
    utf16: Vec<u16>,
}

impl JavaString {
    /// 从有效 Rust 字符串创建 Java 字符串。
    ///
    /// # 参数
    /// - `value`：待编码为 UTF-16 的字符串。
    ///
    /// # 返回
    /// 保存与 Java `String` 相同代码单元的值。
    #[must_use]
    pub fn from_rust_str(value: &str) -> Self {
        Self {
            utf16: value.encode_utf16().collect(),
        }
    }

    /// 从任意 Java UTF-16 代码单元创建字符串。
    ///
    /// # 参数
    /// - `utf16`：包括可能孤立代理项在内的原始代码单元。
    ///
    /// # 返回
    /// 不执行 Unicode 修复或替换的 Java 字符串适配值。
    #[must_use]
    pub fn from_utf16(utf16: impl Into<Vec<u16>>) -> Self {
        Self {
            utf16: utf16.into(),
        }
    }

    /// 返回 Java `String#length()`。
    ///
    /// # 返回
    /// UTF-16 代码单元数量，而不是 Unicode 标量或 UTF-8 字节数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.utf16.len()
    }

    /// 判断 Java 字符串是否为空。
    ///
    /// # 返回
    /// 不含 UTF-16 代码单元时返回 `true`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.utf16.is_empty()
    }

    /// 返回原始 UTF-16 代码单元。
    ///
    /// # 返回
    /// 与 Java `charAt`/`substring` 使用的代码单元序列相同的只读切片。
    #[must_use]
    pub fn as_utf16(&self) -> &[u16] {
        &self.utf16
    }

    /// 转换为可显示的 Rust 字符串。
    ///
    /// # 返回
    /// 有效代理对按原字符解码；孤立代理项按 Rust 标准规则显示为替换字符。
    ///
    /// 精确协议比较应使用 [`Self::as_utf16`]，不能使用本有损显示入口。
    #[must_use]
    pub fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(&self.utf16)
    }
}

impl Debug for JavaString {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JavaString")
            .field("utf16", &self.utf16)
            .finish()
    }
}

/// `LoggingUtils` 返回的 Java 字符串引用或新值。
///
/// 对应 Java `String.replace(char,char)` 在找不到换行符时返回原对象，以及需要替换
/// 或截断时创建新对象的身份语义。
#[derive(Debug)]
pub enum JavaStringResult<'a> {
    /// 返回输入的同一个 Java 字符串对象。
    Borrowed(&'a JavaString),
    /// 返回新创建的 Java 字符串对象。
    Owned(JavaString),
}

impl<'a> JavaStringResult<'a> {
    /// 返回结果 Java 字符串。
    ///
    /// # 返回
    /// 借用或拥有分支中的统一只读引用。
    #[must_use]
    pub fn as_java_string(&self) -> &JavaString {
        match self {
            Self::Borrowed(value) => value,
            Self::Owned(value) => value,
        }
    }

    /// 判断结果是否借用了指定输入对象。
    ///
    /// # 参数
    /// - `source`：待比较引用身份的 Java 字符串。
    ///
    /// # 返回
    /// 结果为同一借用对象时返回 `true`。
    #[must_use]
    pub fn is_borrowed_from(&self, source: &JavaString) -> bool {
        matches!(self, Self::Borrowed(value) if ptr::eq(*value, source))
    }

    /// 将结果转换为独立拥有的 Java 字符串。
    ///
    /// # 返回
    /// 已拥有结果直接移动；借用结果克隆相同 UTF-16 代码单元。
    #[must_use]
    pub fn into_owned(self) -> JavaString {
        match self {
            Self::Borrowed(value) => value.clone(),
            Self::Owned(value) => value,
        }
    }
}

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
    pub fn loggify_template_name(template: Option<&JavaString>) -> Option<JavaStringResult<'_>> {
        let template = template?;
        if template.len() <= 120 {
            if !template.utf16.contains(&u16::from(b'\n')) {
                return Some(JavaStringResult::Borrowed(template));
            }
            return Some(JavaStringResult::Owned(JavaString::from_utf16(
                replace_line_feeds(&template.utf16),
            )));
        }

        let mut result = Vec::with_capacity(120);
        result.extend(replace_line_feeds(&template.utf16[..35]));
        result.extend("[...]".encode_utf16());
        result.extend(replace_line_feeds(
            &template.utf16[template.utf16.len() - 80..],
        ));
        Some(JavaStringResult::Owned(JavaString::from_utf16(result)))
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
    #[must_use]
    pub(crate) fn loggify_str(template: Option<&str>) -> Option<String> {
        let template = template.map(JavaString::from_rust_str)?;
        Self::loggify_template_name(Some(&template))
            .map(JavaStringResult::into_owned)
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
    use super::{JavaString, LoggingUtils};

    #[test]
    fn preserves_null_empty_and_short_reference_identity() {
        assert!(LoggingUtils::loggify_template_name(None).is_none());

        let empty = JavaString::from_rust_str("");
        assert!(empty.is_empty());
        let result = LoggingUtils::loggify_template_name(Some(&empty)).expect("result");
        assert!(result.is_borrowed_from(&empty));
        assert_eq!(result.as_java_string().as_utf16(), &[] as &[u16]);
        assert_eq!(result.as_java_string().to_string_lossy(), "");
        assert_eq!(format!("{empty:?}"), "JavaString { utf16: [] }");
    }

    #[test]
    fn replaces_short_line_feeds_with_an_independent_value() {
        let source = JavaString::from_rust_str("home\npage");
        let result = LoggingUtils::loggify_template_name(Some(&source)).expect("result");
        assert!(!result.is_borrowed_from(&source));
        assert_eq!(result.as_java_string().to_string_lossy(), "home page");
        assert_eq!(result.into_owned().to_string_lossy(), "home page");
        assert_eq!(source.to_string_lossy(), "home\npage");
    }

    #[test]
    fn truncates_by_java_utf16_units_and_can_preserve_isolated_surrogates() {
        let mut utf16 = vec![u16::from(b'a'); 34];
        utf16.extend("😀".encode_utf16());
        utf16.extend(vec![u16::from(b'b'); 90]);
        let source = JavaString::from_utf16(utf16);
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
