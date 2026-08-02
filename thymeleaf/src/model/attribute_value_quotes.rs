use std::fmt::{Display, Formatter};

/// 标签属性值两侧的引号形态。
///
/// 变体和声明顺序严格对应 Java enum；`NONE` 表示属性值没有引号。
///
/// 对应 Java: `org.thymeleaf.model.AttributeValueQuotes`。
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AttributeValueQuotes {
    /// 双引号。
    DOUBLE,
    /// 单引号。
    SINGLE,
    /// 无引号。
    NONE,
}

impl AttributeValueQuotes {
    /// 按 Java enum 声明顺序列出全部值。
    pub const VALUES: [Self; 3] = [Self::DOUBLE, Self::SINGLE, Self::NONE];

    /// 返回 Java enum 的声明序号。
    ///
    /// # 返回
    /// `DOUBLE`、`SINGLE`、`NONE` 分别返回 0、1、2。
    #[must_use]
    pub const fn ordinal(self) -> usize {
        match self {
            Self::DOUBLE => 0,
            Self::SINGLE => 1,
            Self::NONE => 2,
        }
    }
}

impl Display for AttributeValueQuotes {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::DOUBLE => "DOUBLE",
            Self::SINGLE => "SINGLE",
            Self::NONE => "NONE",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::AttributeValueQuotes;

    #[test]
    fn preserves_java_declaration_order_ordinals_and_names() {
        assert_eq!(
            AttributeValueQuotes::VALUES,
            [
                AttributeValueQuotes::DOUBLE,
                AttributeValueQuotes::SINGLE,
                AttributeValueQuotes::NONE,
            ]
        );
        for (ordinal, value) in AttributeValueQuotes::VALUES.into_iter().enumerate() {
            assert_eq!(value.ordinal(), ordinal);
        }
        assert_eq!(AttributeValueQuotes::DOUBLE.to_string(), "DOUBLE");
        assert_eq!(AttributeValueQuotes::SINGLE.to_string(), "SINGLE");
        assert_eq!(AttributeValueQuotes::NONE.to_string(), "NONE");
    }
}
