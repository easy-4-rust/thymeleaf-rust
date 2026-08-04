use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::engine::{AttributeNameError, AttributeNameKind, AttributeNameValue};
use crate::templatemode::TemplateMode;
use crate::util::{Utf16String, java_case_fold_unit};

/// `MatchingAttributeName` 构造、匹配和显示错误。
#[derive(Clone, Debug, Eq, PartialEq)]
/// 对应 Java 语义：`MatchingAttributeName` 的 Rust 侧类型 `MatchingAttributeNameError`。
pub enum MatchingAttributeNameError {
    /// Java `Validate.notNull` 或类型/模式不匹配。
    IllegalArgument(&'static str),
    /// 属性名比较或字符串化失败。
    AttributeName(AttributeNameError),
}

impl MatchingAttributeNameError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn class_name(&self) -> &'static str {
        match self {
            Self::IllegalArgument(_) => "java.lang.IllegalArgumentException",
            Self::AttributeName(error) => error.class_name(),
        }
    }
}

impl Display for MatchingAttributeNameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalArgument(message) => formatter.write_str(message),
            Self::AttributeName(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for MatchingAttributeNameError {}

/// 元素 Processor 的属性名匹配规则。
///
/// 对应 Java: `org.thymeleaf.processor.element.MatchingAttributeName`。
pub struct MatchingAttributeName {
    template_mode: TemplateMode,
    matching_attribute_name: Option<AttributeNameValue>,
    matching_all_attributes_with_prefix: Option<Utf16String>,
    matching_all_attributes: bool,
}

impl MatchingAttributeName {
    /// 创建精确属性名称规则。
    ///
    /// # 错误
    ///
    /// mode/name 为 null 或名称具体子类与模板模式不一致时返回参数错误。
    /// 对应 Java: `MatchingAttributeName#forAttributeName()`。
    pub fn for_attribute_name(
        template_mode: Option<TemplateMode>,
        matching_attribute_name: Option<AttributeNameValue>,
    ) -> Result<Self, MatchingAttributeNameError> {
        let template_mode = require_mode(template_mode)?;
        let matching_attribute_name = matching_attribute_name.ok_or(
            MatchingAttributeNameError::IllegalArgument("Matching attribute name cannot be null"),
        )?;
        validate_kind(
            template_mode,
            matching_attribute_name.as_attribute_name().kind(),
        )?;
        Ok(Self {
            template_mode,
            matching_attribute_name: Some(matching_attribute_name),
            matching_all_attributes_with_prefix: None,
            matching_all_attributes: false,
        })
    }

    /// 创建匹配指定可空 prefix 下所有属性的规则。
    /// 对应 Java: `MatchingAttributeName#forAllAttributesWithPrefix()`。
    pub fn for_all_attributes_with_prefix(
        template_mode: Option<TemplateMode>,
        prefix: Option<Utf16String>,
    ) -> Result<Self, MatchingAttributeNameError> {
        Ok(Self {
            template_mode: require_mode(template_mode)?,
            matching_attribute_name: None,
            matching_all_attributes_with_prefix: prefix,
            matching_all_attributes: false,
        })
    }

    /// 创建匹配指定模式全部属性的规则。
    /// 对应 Java: `MatchingAttributeName#forAllAttributes()`。
    pub fn for_all_attributes(
        template_mode: Option<TemplateMode>,
    ) -> Result<Self, MatchingAttributeNameError> {
        Ok(Self {
            template_mode: require_mode(template_mode)?,
            matching_attribute_name: None,
            matching_all_attributes_with_prefix: None,
            matching_all_attributes: true,
        })
    }

    /// 返回规则所属模板模式。
    #[must_use]
    pub const fn get_template_mode(&self) -> TemplateMode {
        self.template_mode
    }

    /// 返回可空精确匹配属性名。
    #[must_use]
    pub const fn get_matching_attribute_name(&self) -> Option<&AttributeNameValue> {
        self.matching_attribute_name.as_ref()
    }

    /// 返回“prefix 下全部属性”规则的可空 prefix。
    #[must_use]
    pub const fn get_matching_all_attributes_with_prefix(&self) -> Option<&Utf16String> {
        self.matching_all_attributes_with_prefix.as_ref()
    }

    /// 判断规则是否匹配模式中的全部属性。
    #[must_use]
    pub const fn is_matching_all_attributes(&self) -> bool {
        self.matching_all_attributes
    }

    /// 判断给定属性名是否命中规则。
    ///
    /// # 错误
    ///
    /// 输入为 null 或精确名称数组被外部破坏时返回 Java 对应错误。
    /// 对应 Java: `MatchingAttributeName#matches()`。
    pub fn matches(
        &self,
        attribute_name: Option<&AttributeNameValue>,
    ) -> Result<bool, MatchingAttributeNameError> {
        let attribute_name = attribute_name.ok_or(MatchingAttributeNameError::IllegalArgument(
            "Attributes name cannot be null",
        ))?;
        if let Some(expected) = self.matching_attribute_name.as_ref() {
            return expected
                .as_attribute_name()
                .equals_java(attribute_name.as_attribute_name())
                .map_err(MatchingAttributeNameError::AttributeName);
        }
        if !kind_matches_mode(
            self.template_mode,
            attribute_name.as_attribute_name().kind(),
        ) {
            return Ok(false);
        }
        if self.matching_all_attributes {
            return Ok(true);
        }
        let actual_prefix = attribute_name.as_attribute_name().get_prefix();
        let Some(expected_prefix) = self.matching_all_attributes_with_prefix.as_ref() else {
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
    /// 精确属性名数组被外部破坏时传播对应错误。
    /// 对应 Java 语义：`MatchingAttributeName` 的 `to_utf16_string` 行为（Rust 侧辅助/私有路径）。
    pub fn to_utf16_string(&self) -> Result<Utf16String, MatchingAttributeNameError> {
        if let Some(name) = self.matching_attribute_name.as_ref() {
            return name
                .as_attribute_name()
                .to_utf16_string()
                .map_err(MatchingAttributeNameError::AttributeName);
        }
        if self.matching_all_attributes {
            return Ok(Utf16String::from_rust_str("*"));
        }
        let Some(prefix) = self.matching_all_attributes_with_prefix.as_ref() else {
            return Ok(Utf16String::from_rust_str("[^:]*"));
        };
        let mut result = prefix.as_utf16().to_vec();
        result.extend(":*".encode_utf16());
        Ok(Utf16String::from_utf16(result))
    }
}

fn require_mode(mode: Option<TemplateMode>) -> Result<TemplateMode, MatchingAttributeNameError> {
    mode.ok_or(MatchingAttributeNameError::IllegalArgument(
        "Template mode cannot be null",
    ))
}

fn validate_kind(
    mode: TemplateMode,
    kind: AttributeNameKind,
) -> Result<(), MatchingAttributeNameError> {
    if kind_matches_mode(mode, kind) {
        return Ok(());
    }
    let message = match mode {
        TemplateMode::HTML => {
            "Attribute names for HTML template mode must be of class org.thymeleaf.engine.HTMLAttributeName"
        }
        TemplateMode::XML => {
            "Attribute names for XML template mode must be of class org.thymeleaf.engine.XMLAttributeName"
        }
        mode if mode.is_text() => {
            "Attribute names for any text template modes must be of class org.thymeleaf.engine.TextAttributeName"
        }
        _ => return Ok(()),
    };
    Err(MatchingAttributeNameError::IllegalArgument(message))
}

fn kind_matches_mode(mode: TemplateMode, kind: AttributeNameKind) -> bool {
    match mode {
        TemplateMode::HTML => kind == AttributeNameKind::Html,
        TemplateMode::XML => kind == AttributeNameKind::Xml,
        mode if mode.is_text() => kind == AttributeNameKind::Text,
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
                    || (!case_sensitive
                        && java_case_fold_unit(*left) == java_case_fold_unit(*right))
            })
}
