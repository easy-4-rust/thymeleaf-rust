use std::sync::Arc;

use thiserror::Error;
use unicode_general_category::{GeneralCategory, get_general_category};

use crate::context::IExpressionContext;
use crate::util::{JavaNumber, JavaString};

use super::TemplateValue;

/// 仅包含 Java 标识符和点号的 OGNL 快速属性访问表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.OGNLShortcutExpression`。
pub struct NativeShortcutExpression {
    expression_levels: Vec<JavaString>,
}

impl NativeShortcutExpression {
    /// 保存已经解析的属性层级。
    /// 对应 Java 语义：`OGNLShortcutExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(expression_levels: Vec<JavaString>) -> Self {
        Self { expression_levels }
    }

    /// 尝试把表达式解析为点分 Java 标识符序列。
    ///
    /// 任一层为空或包含非 Java 标识符字符时返回 `None`，由完整求值器处理。
    /// 对应 Java: `OGNLShortcutExpression#parse()`。
    pub fn parse(expression: Option<&JavaString>) -> Option<Vec<JavaString>> {
        let expression = expression?;
        let mut levels = Vec::new();
        for level in expression.as_utf16().split(|unit| *unit == b'.' as u16) {
            if level.is_empty() || !is_java_identifier(level) {
                return None;
            }
            levels.push(JavaString::from_utf16(level.to_vec()));
        }
        (!levels.is_empty()).then_some(levels)
    }

    /// 依次读取 Context、Map、List、数组或宿主对象属性。
    /// 对应 Java: `OGNLShortcutExpression#evaluate()`。
    pub fn evaluate(
        &self,
        context: &dyn IExpressionContext,
        use_selection_as_root: bool,
        restrict_variable_access: bool,
    ) -> Result<Option<Arc<TemplateValue>>, NativeShortcutError> {
        let mut target = if use_selection_as_root {
            context.as_template_context().and_then(|template_context| {
                template_context.has_selection_target().then(|| {
                    template_context
                        .get_selection_target()
                        .unwrap_or_else(|| Arc::new(TemplateValue::Null))
                })
            })
        } else {
            None
        };
        let mut context_root = target.is_none();

        for property_name in &self.expression_levels {
            if context_root {
                if restrict_variable_access && property_name == &JavaString::from_rust_str("param")
                {
                    return Err(NativeShortcutError::RestrictedVariable {
                        name: property_name.to_string_lossy(),
                    });
                }
                target = context.get_variable(Some(property_name));
                context_root = false;
                continue;
            }
            let current = target
                .as_deref()
                .ok_or_else(|| NativeShortcutError::NullSource {
                    property_name: property_name.to_string_lossy(),
                })?;
            target = read_property(current, property_name)?;
        }
        Ok(target)
    }
}

fn read_property(
    target: &TemplateValue,
    property_name: &JavaString,
) -> Result<Option<Arc<TemplateValue>>, NativeShortcutError> {
    let name = property_name.to_string_lossy();
    match target {
        TemplateValue::Map(entries) => {
            if name == "size" {
                return Ok(Some(integer_value(entries.len())));
            }
            if name == "isEmpty" {
                return Ok(Some(Arc::new(TemplateValue::Boolean(entries.is_empty()))));
            }
            if name == "keys" || name == "keySet" {
                return Ok(Some(Arc::new(TemplateValue::List(Arc::new(
                    entries.iter().map(|(key, _)| Arc::clone(key)).collect(),
                )))));
            }
            if name == "values" {
                return Ok(Some(Arc::new(TemplateValue::List(Arc::new(
                    entries.iter().map(|(_, value)| Arc::clone(value)).collect(),
                )))));
            }
            Ok(entries.iter().find_map(|(key, value)| {
                matches!(
                    key.as_ref(),
                    TemplateValue::String(key_value) | TemplateValue::SafeHtml(key_value)
                        if key_value.as_ref() == property_name
                )
                .then(|| Arc::clone(value))
            }))
        }
        TemplateValue::List(values) => match name.as_str() {
            "size" | "length" => Ok(Some(integer_value(values.len()))),
            "isEmpty" | "empty" => Ok(Some(Arc::new(TemplateValue::Boolean(values.is_empty())))),
            _ => Err(not_applicable(name, target)),
        },
        TemplateValue::Bytes(values) => match name.as_str() {
            "length" => Ok(Some(integer_value(values.len()))),
            _ => Err(not_applicable(name, target)),
        },
        TemplateValue::String(value) | TemplateValue::SafeHtml(value) => match name.as_str() {
            "length" => Ok(Some(integer_value(value.len()))),
            "isEmpty" | "empty" => Ok(Some(Arc::new(TemplateValue::Boolean(value.is_empty())))),
            _ => Err(not_applicable(name, target)),
        },
        TemplateValue::Object(value) => match value.java_get_property(property_name) {
            Some(result) => result.map_err(|error| NativeShortcutError::PropertyGetter {
                property_name: name,
                class_name: value.java_class_name().to_owned(),
                message: error.to_string(),
            }),
            None => Err(NativeShortcutError::NotApplicable(
                NativeShortcutExpressionNotApplicableError::new(
                    name,
                    value.java_class_name().to_owned(),
                ),
            )),
        },
        TemplateValue::Null => Err(NativeShortcutError::NullSource {
            property_name: name,
        }),
        _ => Err(not_applicable(name, target)),
    }
}

fn not_applicable(property_name: String, target: &TemplateValue) -> NativeShortcutError {
    NativeShortcutError::NotApplicable(NativeShortcutExpressionNotApplicableError::new(
        property_name,
        target.java_class_name().to_owned(),
    ))
}

fn integer_value(value: usize) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Number(JavaNumber::Integer(
        i32::try_from(value).unwrap_or(i32::MAX),
    )))
}

fn is_java_identifier(input: &[u16]) -> bool {
    let mut position = 0;
    let mut first = true;
    while position < input.len() {
        let (code_point, width) = decode_code_point(input, position);
        if if first {
            !is_java_identifier_start(code_point)
        } else {
            !is_java_identifier_part(code_point)
        } {
            return false;
        }
        first = false;
        position += width;
    }
    !first
}

fn decode_code_point(input: &[u16], position: usize) -> (u32, usize) {
    let first = input[position];
    if (0xD800..=0xDBFF).contains(&first)
        && input
            .get(position + 1)
            .is_some_and(|second| (0xDC00..=0xDFFF).contains(second))
    {
        let second = input[position + 1];
        (
            0x10000 + ((u32::from(first) - 0xD800) << 10) + (u32::from(second) - 0xDC00),
            2,
        )
    } else {
        (u32::from(first), 1)
    }
}

fn is_java_identifier_start(code_point: u32) -> bool {
    let Some(character) = char::from_u32(code_point) else {
        return false;
    };
    matches!(
        get_general_category(character),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
            | GeneralCategory::LetterNumber
            | GeneralCategory::CurrencySymbol
            | GeneralCategory::ConnectorPunctuation
    )
}

fn is_java_identifier_part(code_point: u32) -> bool {
    if is_java_identifier_start(code_point) {
        return true;
    }
    let Some(character) = char::from_u32(code_point) else {
        return false;
    };
    matches!(
        get_general_category(character),
        GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::DecimalNumber
            | GeneralCategory::Format
    ) || matches!(code_point, 0x0000..=0x0008 | 0x000E..=0x001B | 0x007F..=0x009F)
}

/// OGNL 快速路径的可观察失败类别。
#[derive(Debug, Error)]
/// 对应 Java 语义：`OGNLShortcutExpression` 的 Rust 侧类型 `NativeShortcutError`。
pub enum NativeShortcutError {
    /// 中间属性结果为 Java null。
    #[error("source is null for getProperty(null, \"{property_name}\")")]
    NullSource {
        /// 正在读取的属性名。
        property_name: String,
    },
    /// 当前 Rust 动态对象无法应用 JavaBean getter 快速路径。
    #[error(transparent)]
    NotApplicable(NativeShortcutExpressionNotApplicableError),
    /// 受限上下文禁止读取请求参数。
    #[error("Access to variable \"{name}\" is forbidden in this context")]
    RestrictedVariable {
        /// 变量名。
        name: String,
    },
    /// 宿主 getter 执行失败。
    #[error("Exception reading property \"{property_name}\" on {class_name}: {message}")]
    PropertyGetter {
        /// 属性名。
        property_name: String,
        /// 目标类名。
        class_name: String,
        /// 原错误消息。
        message: String,
    },
}

/// 表示原生快捷路径不适用于当前对象，需要回退到完整 evaluator。
///
/// 对应 Java:
/// `org.thymeleaf.standard.expression.OGNLShortcutExpression.OGNLShortcutExpressionNotApplicableException`。
#[derive(Debug, Error)]
#[error("property \"{property_name}\" is not readable on {class_name}")]
pub struct NativeShortcutExpressionNotApplicableError {
    property_name: String,
    class_name: String,
}

impl NativeShortcutExpressionNotApplicableError {
    fn new(property_name: String, class_name: String) -> Self {
        Self {
            property_name,
            class_name,
        }
    }
}
