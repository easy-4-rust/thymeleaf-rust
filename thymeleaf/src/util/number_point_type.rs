use std::fmt::{Display, Formatter};

/// 数字格式化时小数点或分组点的选择方式。
///
/// 对应 Java: `org.thymeleaf.util.NumberPointType`。
///
/// `Point`、`Comma` 和 `Whitespace` 强制使用指定字符；`None` 禁用对应分隔；
/// `Default` 保留 Locale 默认字符。枚举名称、声明顺序和显示文本与 Java 完全一致。
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NumberPointType {
    /// 使用英文句点 `.`。
    Point,
    /// 使用逗号 `,`。
    Comma,
    /// 使用普通空格。
    Whitespace,
    /// 不使用分隔点。
    None,
    /// 使用 Locale 默认分隔点。
    Default,
}

impl NumberPointType {
    /// 按区分大小写的 Java 枚举名称匹配点类型。
    ///
    /// 对应 Java: `NumberPointType#match(String)`。
    ///
    /// # 参数
    /// - `name`：Java 参数 `name`；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// 精确匹配五个大写名称时返回对应枚举，否则返回 `None`。
    #[must_use]
    pub fn match_name(name: Option<&str>) -> Option<Self> {
        match name {
            Some("NONE") => Some(Self::None),
            Some("DEFAULT") => Some(Self::Default),
            Some("POINT") => Some(Self::Point),
            Some("COMMA") => Some(Self::Comma),
            Some("WHITESPACE") => Some(Self::Whitespace),
            Some(_) | None => None,
        }
    }

    /// 返回 Java 枚举保存的名称。
    ///
    /// 对应 Java: `NumberPointType#getName()`。
    #[must_use]
    pub const fn get_name(self) -> &'static str {
        match self {
            Self::Point => "POINT",
            Self::Comma => "COMMA",
            Self::Whitespace => "WHITESPACE",
            Self::None => "NONE",
            Self::Default => "DEFAULT",
        }
    }
}

impl Display for NumberPointType {
    /// 输出 Java `toString()` 返回的枚举名称。
    ///
    /// 对应 Java: `NumberPointType#toString()`。
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.get_name())
    }
}

#[cfg(test)]
mod tests {
    use super::NumberPointType;

    #[test]
    fn preserves_declaration_order_names_and_display() {
        let values = [
            NumberPointType::Point,
            NumberPointType::Comma,
            NumberPointType::Whitespace,
            NumberPointType::None,
            NumberPointType::Default,
        ];
        let names = ["POINT", "COMMA", "WHITESPACE", "NONE", "DEFAULT"];

        for (value, name) in values.into_iter().zip(names) {
            assert_eq!(value.get_name(), name);
            assert_eq!(value.to_string(), name);
        }
    }

    #[test]
    fn matches_only_exact_java_names() {
        assert_eq!(
            NumberPointType::match_name(Some("NONE")),
            Some(NumberPointType::None)
        );
        assert_eq!(
            NumberPointType::match_name(Some("DEFAULT")),
            Some(NumberPointType::Default)
        );
        assert_eq!(
            NumberPointType::match_name(Some("POINT")),
            Some(NumberPointType::Point)
        );
        assert_eq!(
            NumberPointType::match_name(Some("COMMA")),
            Some(NumberPointType::Comma)
        );
        assert_eq!(
            NumberPointType::match_name(Some("WHITESPACE")),
            Some(NumberPointType::Whitespace)
        );
        assert_eq!(NumberPointType::match_name(Some("point")), None);
        assert_eq!(NumberPointType::match_name(Some(" POINT")), None);
        assert_eq!(NumberPointType::match_name(Some("")), None);
        assert_eq!(NumberPointType::match_name(None), None);
    }
}
