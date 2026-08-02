use std::fmt::{Debug, Formatter};

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
        matches!(self, Self::Borrowed(value) if std::ptr::eq(*value, source))
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

#[cfg(test)]
mod tests {
    use super::{JavaString, JavaStringResult};

    #[test]
    fn preserves_null_empty_and_short_reference_identity() {
        let empty = JavaString::from_rust_str("");
        assert!(empty.is_empty());
        let result = JavaStringResult::Borrowed(&empty);
        assert!(result.is_borrowed_from(&empty));
        assert_eq!(result.as_java_string().as_utf16(), &[] as &[u16]);
        assert_eq!(result.as_java_string().to_string_lossy(), "");
        assert_eq!(format!("{empty:?}"), "JavaString { utf16: [] }");
        assert!(
            JavaStringResult::Owned(empty.clone())
                .into_owned()
                .as_utf16()
                .is_empty()
        );
    }

    #[test]
    fn preserves_isolated_surrogates_without_replacement() {
        let mut utf16 = vec![0xD83D];
        utf16.extend("abc".encode_utf16());
        let value = JavaString::from_utf16(utf16);
        assert_eq!(value.len(), 4);
        assert_eq!(value.as_utf16()[0], 0xD83D);
    }
}
