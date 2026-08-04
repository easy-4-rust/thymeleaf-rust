use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::engine::{ElementNameError, ElementNameKind, ElementNameValue};
use crate::templatemode::TemplateMode;
use crate::util::{Utf16String, case_fold_unit};

/// `MatchingElementName` 构造、匹配和显示错误。
#[derive(Clone, Debug, Eq, PartialEq)]
/// 对应 Java 语义：`MatchingElementName` 的 Rust 侧类型 `MatchingElementNameError`。
pub enum MatchingElementNameError {
    /// Java `Validate.notNull` 或类型/模式不匹配。
    IllegalArgument(&'static str),
    /// 被匹配名称的 `toString()` 失败。
    ElementName(ElementNameError),
}

impl MatchingElementNameError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn class_name(&self) -> &'static str {
        match self {
            Self::IllegalArgument(_) => "java.lang.IllegalArgumentException",
            Self::ElementName(error) => error.class_name(),
        }
    }
}

impl Display for MatchingElementNameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalArgument(message) => formatter.write_str(message),
            Self::ElementName(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for MatchingElementNameError {}

/// 元素 Processor 的元素名匹配规则。
///
/// 对应 Java: `org.thymeleaf.processor.element.MatchingElementName`。
///
/// 支持精确名称、指定 prefix 下的所有元素、无 prefix 元素以及所有元素四种状态。
pub struct MatchingElementName {
    template_mode: TemplateMode,
    matching_element_name: Option<ElementNameValue>,
    matching_all_elements_with_prefix: Option<Utf16String>,
    matching_all_elements: bool,
}

impl MatchingElementName {
    /// 创建精确元素名称规则。
    ///
    /// # 错误
    ///
    /// mode/name 为 null 或名称具体子类与 HTML/XML/文本模式不一致时返回参数错误。
    /// 对应 Java: `MatchingElementName#forElementName()`。
    pub fn for_element_name(
        template_mode: Option<TemplateMode>,
        matching_element_name: Option<ElementNameValue>,
    ) -> Result<Self, MatchingElementNameError> {
        let template_mode = require_mode(template_mode)?;
        let matching_element_name = matching_element_name.ok_or(
            MatchingElementNameError::IllegalArgument("Matching element name cannot be null"),
        )?;
        validate_kind(
            template_mode,
            matching_element_name.as_element_name().kind(),
        )?;
        Ok(Self {
            template_mode,
            matching_element_name: Some(matching_element_name),
            matching_all_elements_with_prefix: None,
            matching_all_elements: false,
        })
    }

    /// 创建匹配指定可空 prefix 下所有元素的规则。
    /// 对应 Java: `MatchingElementName#forAllElementsWithPrefix()`。
    pub fn for_all_elements_with_prefix(
        template_mode: Option<TemplateMode>,
        prefix: Option<Utf16String>,
    ) -> Result<Self, MatchingElementNameError> {
        Ok(Self {
            template_mode: require_mode(template_mode)?,
            matching_element_name: None,
            matching_all_elements_with_prefix: prefix,
            matching_all_elements: false,
        })
    }

    /// 创建匹配指定模式全部元素的规则。
    /// 对应 Java: `MatchingElementName#forAllElements()`。
    pub fn for_all_elements(
        template_mode: Option<TemplateMode>,
    ) -> Result<Self, MatchingElementNameError> {
        Ok(Self {
            template_mode: require_mode(template_mode)?,
            matching_element_name: None,
            matching_all_elements_with_prefix: None,
            matching_all_elements: true,
        })
    }

    /// 返回规则所属模板模式。
    #[must_use]
    pub const fn get_template_mode(&self) -> TemplateMode {
        self.template_mode
    }

    /// 返回可空精确匹配名称。
    #[must_use]
    pub const fn get_matching_element_name(&self) -> Option<&ElementNameValue> {
        self.matching_element_name.as_ref()
    }

    /// 返回“prefix 下全部元素”规则的可空 prefix。
    #[must_use]
    pub const fn get_matching_all_elements_with_prefix(&self) -> Option<&Utf16String> {
        self.matching_all_elements_with_prefix.as_ref()
    }

    /// 判断规则是否匹配模式中的所有元素。
    #[must_use]
    pub const fn is_matching_all_elements(&self) -> bool {
        self.matching_all_elements
    }

    /// 判断给定元素名是否命中规则。
    ///
    /// # 错误
    ///
    /// 输入名称为 null 时返回 Java `IllegalArgumentException` 对应错误。
    /// 对应 Java: `MatchingElementName#matches()`。
    pub fn matches(
        &self,
        element_name: Option<&ElementNameValue>,
    ) -> Result<bool, MatchingElementNameError> {
        let element_name = element_name.ok_or(MatchingElementNameError::IllegalArgument(
            "Element name cannot be null",
        ))?;
        if let Some(expected) = self.matching_element_name.as_ref() {
            return Ok(expected.as_element_name() == element_name.as_element_name());
        }
        if !kind_matches_mode(self.template_mode, element_name.as_element_name().kind()) {
            return Ok(false);
        }
        if self.matching_all_elements {
            return Ok(true);
        }
        let actual_prefix = element_name.as_element_name().get_prefix();
        let Some(expected_prefix) = self.matching_all_elements_with_prefix.as_ref() else {
            return Ok(actual_prefix.is_none());
        };
        let Some(actual_prefix) = actual_prefix else {
            return Ok(false);
        };
        Ok(text_equals(
            self.template_mode.is_case_sensitive(),
            expected_prefix,
            actual_prefix,
        ))
    }

    /// 返回 `*`、`[^:]*`、`prefix:*` 或精确名称文本。
    ///
    /// # 错误
    ///
    /// 精确名称的 complete names 数组被外部破坏时传播对应错误。
    /// 对应 Java 语义：`MatchingElementName` 的 `to_utf16_string` 行为（Rust 侧辅助/私有路径）。
    pub fn to_utf16_string(&self) -> Result<Utf16String, MatchingElementNameError> {
        if let Some(name) = self.matching_element_name.as_ref() {
            return name
                .as_element_name()
                .to_utf16_string()
                .map_err(MatchingElementNameError::ElementName);
        }
        if self.matching_all_elements {
            return Ok(Utf16String::from_rust_str("*"));
        }
        let Some(prefix) = self.matching_all_elements_with_prefix.as_ref() else {
            return Ok(Utf16String::from_rust_str("[^:]*"));
        };
        let mut result = prefix.as_utf16().to_vec();
        result.extend(":*".encode_utf16());
        Ok(Utf16String::from_utf16(result))
    }
}

fn require_mode(mode: Option<TemplateMode>) -> Result<TemplateMode, MatchingElementNameError> {
    mode.ok_or(MatchingElementNameError::IllegalArgument(
        "Template mode cannot be null",
    ))
}

fn validate_kind(
    mode: TemplateMode,
    kind: ElementNameKind,
) -> Result<(), MatchingElementNameError> {
    if kind_matches_mode(mode, kind) {
        return Ok(());
    }
    let message = match mode {
        TemplateMode::HTML => {
            "Element names for HTML template mode must be of class org.thymeleaf.engine.HTMLElementName"
        }
        TemplateMode::XML => {
            "Element names for XML template mode must be of class org.thymeleaf.engine.XMLElementName"
        }
        mode if mode.is_text() => {
            "Element names for any text template modes must be of class org.thymeleaf.engine.TextElementName"
        }
        _ => return Ok(()),
    };
    Err(MatchingElementNameError::IllegalArgument(message))
}

fn kind_matches_mode(mode: TemplateMode, kind: ElementNameKind) -> bool {
    match mode {
        TemplateMode::HTML => kind == ElementNameKind::Html,
        TemplateMode::XML => kind == ElementNameKind::Xml,
        mode if mode.is_text() => kind == ElementNameKind::Text,
        _ => true,
    }
}

fn text_equals(case_sensitive: bool, left: &Utf16String, right: &Utf16String) -> bool {
    left.len() == right.len()
        && left
            .as_utf16()
            .iter()
            .zip(right.as_utf16())
            .all(|(left, right)| {
                left == right
                    || (!case_sensitive && case_fold_unit(*left) == case_fold_unit(*right))
            })
}
