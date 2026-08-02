use std::fmt::{Display, Formatter};

/// HTML 语法定义的元素类别。
///
/// 对应 Java: `org.thymeleaf.engine.HTMLElementType`。
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HTMLElementType {
    /// HTML void 元素。
    VOID,
    /// 原始文本元素。
    RAW_TEXT,
    /// 可转义原始文本元素。
    ESCAPABLE_RAW_TEXT,
    /// 外部命名空间元素。
    FOREIGN,
    /// 普通 HTML 元素。
    NORMAL,
}

impl HTMLElementType {
    /// 按 Java enum 声明顺序列出全部值。
    pub const VALUES: [Self; 5] = [
        Self::VOID,
        Self::RAW_TEXT,
        Self::ESCAPABLE_RAW_TEXT,
        Self::FOREIGN,
        Self::NORMAL,
    ];

    /// 判断当前类别是否为 HTML void 元素。
    ///
    /// 对应 Java: `HTMLElementType#isVoid()`。
    ///
    /// # 返回
    /// 仅 `VOID` 返回 `true`，其余四类返回 `false`。
    #[must_use]
    pub const fn is_void(self) -> bool {
        matches!(self, Self::VOID)
    }

    /// 返回 Java enum 的声明序号。
    ///
    /// # 返回
    /// 从 `VOID` 的 0 到 `NORMAL` 的 4。
    #[must_use]
    pub const fn ordinal(self) -> usize {
        match self {
            Self::VOID => 0,
            Self::RAW_TEXT => 1,
            Self::ESCAPABLE_RAW_TEXT => 2,
            Self::FOREIGN => 3,
            Self::NORMAL => 4,
        }
    }
}

impl Display for HTMLElementType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::VOID => "VOID",
            Self::RAW_TEXT => "RAW_TEXT",
            Self::ESCAPABLE_RAW_TEXT => "ESCAPABLE_RAW_TEXT",
            Self::FOREIGN => "FOREIGN",
            Self::NORMAL => "NORMAL",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::HTMLElementType;

    #[test]
    fn preserves_java_declaration_order_names_and_void_flag() {
        for (ordinal, value) in HTMLElementType::VALUES.into_iter().enumerate() {
            assert_eq!(value.ordinal(), ordinal);
            assert_eq!(value.is_void(), value == HTMLElementType::VOID);
        }
        assert_eq!(HTMLElementType::VOID.to_string(), "VOID");
        assert_eq!(HTMLElementType::RAW_TEXT.to_string(), "RAW_TEXT");
        assert_eq!(
            HTMLElementType::ESCAPABLE_RAW_TEXT.to_string(),
            "ESCAPABLE_RAW_TEXT"
        );
        assert_eq!(HTMLElementType::FOREIGN.to_string(), "FOREIGN");
        assert_eq!(HTMLElementType::NORMAL.to_string(), "NORMAL");
    }
}
