use std::sync::Arc;

use crate::context::ITemplateContext;
use crate::expression::{IStandardExpression, StandardExpressionResult, StandardExpressions};
use crate::model::{ICDATASection, IComment, IProcessableElementTag, IText};
use crate::util::{JavaCharSequence, JavaString, TextUtilsError};

use super::AttributeName;

/// 查询模板事件内容特征并复用属性级表达式缓存的工具。
///
/// 对应 Java: `org.thymeleaf.engine.EngineEventUtils`。
pub struct EngineEventUtils;

impl EngineEventUtils {
    /// 判断 Text 是否为非空全 Java-whitespace 内容。
    /// 对应 Java 语义：`EngineEventUtils` 的 `is_whitespace_text` 行为（Rust 侧辅助/私有路径）。
    pub fn is_whitespace_text(text: Option<&dyn IText>) -> Result<bool, TextUtilsError> {
        text.map_or(Ok(false), |text| compute_whitespace(text))
    }

    /// 判断 CDATA 内容是否为非空全 Java-whitespace。
    /// 对应 Java 语义：`EngineEventUtils` 的 `is_whitespace_cdata` 行为（Rust 侧辅助/私有路径）。
    pub fn is_whitespace_cdata(
        cdata_section: Option<&dyn ICDATASection>,
    ) -> Result<bool, TextUtilsError> {
        let Some(cdata_section) = cdata_section else {
            return Ok(false);
        };
        let content = cdata_section
            .get_content()?
            .ok_or(TextUtilsError::NullPointer)?;
        compute_whitespace(&content)
    }

    /// 判断 Comment 内容是否为非空全 Java-whitespace。
    /// 对应 Java 语义：`EngineEventUtils` 的 `is_whitespace_comment` 行为（Rust 侧辅助/私有路径）。
    pub fn is_whitespace_comment(comment: Option<&dyn IComment>) -> Result<bool, TextUtilsError> {
        let Some(comment) = comment else {
            return Ok(false);
        };
        let content = comment.get_content()?.ok_or(TextUtilsError::NullPointer)?;
        compute_whitespace(&content)
    }

    /// 判断 Text 是否包含内联表达式边界。
    /// 对应 Java 语义：`EngineEventUtils` 的 `is_inlineable_text` 行为（Rust 侧辅助/私有路径）。
    pub fn is_inlineable_text(text: Option<&dyn IText>) -> Result<bool, TextUtilsError> {
        text.map_or(Ok(false), |text| compute_inlineable(text))
    }

    /// 判断 CDATA 内容是否包含内联表达式边界。
    /// 对应 Java 语义：`EngineEventUtils` 的 `is_inlineable_cdata` 行为（Rust 侧辅助/私有路径）。
    pub fn is_inlineable_cdata(
        cdata_section: Option<&dyn ICDATASection>,
    ) -> Result<bool, TextUtilsError> {
        let Some(cdata_section) = cdata_section else {
            return Ok(false);
        };
        let content = cdata_section
            .get_content()?
            .ok_or(TextUtilsError::NullPointer)?;
        compute_inlineable(&content)
    }

    /// 判断 Comment 内容是否包含内联表达式边界。
    /// 对应 Java 语义：`EngineEventUtils` 的 `is_inlineable_comment` 行为（Rust 侧辅助/私有路径）。
    pub fn is_inlineable_comment(comment: Option<&dyn IComment>) -> Result<bool, TextUtilsError> {
        let Some(comment) = comment else {
            return Ok(false);
        };
        let content = comment.get_content()?.ok_or(TextUtilsError::NullPointer)?;
        compute_inlineable(&content)
    }

    /// 解析属性表达式，并在内建 Attribute 上缓存安全结果。
    ///
    /// 含预处理标记 `_` 或 FragmentExpression 的结果不得缓存。
    /// 对应 Java: `EngineEventUtils#computeAttributeExpression()`。
    pub fn compute_attribute_expression(
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        attribute_name: &AttributeName,
        attribute_value: &JavaString,
    ) -> StandardExpressionResult<Arc<dyn IStandardExpression>> {
        let Some(processable_tag) = tag.as_engine_processable_element_tag() else {
            return parse_attribute_expression(context, attribute_value);
        };
        let attribute = processable_tag
            .get_attribute_name(attribute_name)
            .ok_or_else(|| {
                Box::new(crate::exceptions::TemplateProcessingException::new(Some(
                    "Cannot compute expression for an attribute not present in the tag".to_owned(),
                ))) as crate::expression::StandardExpressionError
            })?;
        if let Some(cached) = attribute.get_cached_standard_expression()
            && let Some(expression) = cached.downcast_ref::<Arc<dyn IStandardExpression>>()
        {
            return Ok(Arc::clone(expression));
        }
        let expression = parse_attribute_expression(context, attribute_value)?;
        if !expression.is_fragment_expression()
            && !attribute_value.as_utf16().contains(&u16::from(b'_'))
        {
            attribute.set_cached_standard_expression(Some(Arc::new(Arc::clone(&expression))));
        }
        Ok(expression)
    }
}

fn parse_attribute_expression(
    context: &dyn ITemplateContext,
    attribute_value: &JavaString,
) -> StandardExpressionResult<Arc<dyn IStandardExpression>> {
    StandardExpressions::get_expression_parser(context.get_configuration())?
        .parse_expression(context, Some(attribute_value))
}

fn compute_whitespace(text: &dyn JavaCharSequence) -> Result<bool, TextUtilsError> {
    let mut remaining = text.java_length()?;
    if remaining == 0 {
        return Ok(false);
    }
    while remaining != 0 {
        remaining -= 1;
        if !java_is_whitespace(text.java_char_at(remaining)?) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// 判断内容是否包含 `[[...]]` 或 `[(...)]` 内联标记对。
///
/// 逐行对应 Java `AbstractTextualTemplateEvent#computeInlineable()`
/// （事件版本：右向左扫描，后遇到的闭包覆盖前者，仅在 `n > 0` 时
/// 识别闭包并跳过其前一个字符）。
pub(crate) fn compute_inlineable(text: &dyn JavaCharSequence) -> Result<bool, TextUtilsError> {
    let mut remaining = text.java_length()?;
    if remaining == 0 {
        return Ok(false);
    }
    let mut previous = 0_u16;
    let mut inline = 0_u8;
    while remaining != 0 {
        remaining -= 1;
        let current = text.java_char_at(remaining)?;
        if remaining > 0 && current == u16::from(b']') && previous == u16::from(b']') {
            inline = 1;
            remaining -= 1;
            previous = text.java_char_at(remaining)?;
        } else if remaining > 0 && current == u16::from(b')') && previous == u16::from(b']') {
            inline = 2;
            remaining -= 1;
            previous = text.java_char_at(remaining)?;
        } else if (inline == 1 && current == u16::from(b'[') && previous == u16::from(b'['))
            || (inline == 2 && current == u16::from(b'[') && previous == u16::from(b'('))
        {
            return Ok(true);
        } else {
            previous = current;
        }
    }
    Ok(false)
}

fn java_is_whitespace(character: u16) -> bool {
    matches!(
        character,
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
