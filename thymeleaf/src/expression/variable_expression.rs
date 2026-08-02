use std::any::Any;
use std::sync::{Arc, RwLock};

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::util::{JavaString, ValidateError};

use super::{
    IStandardExpression, IStandardVariableExpression, SimpleExpression,
    StandardExpressionExecutionContext, StandardExpressionResult, StandardExpressions,
    TemplateValue,
};

/// `${...}` Standard Variable Expression。
///
/// 对应 Java: `org.thymeleaf.standard.expression.VariableExpression`。
pub struct VariableExpression {
    expression: JavaString,
    convert_to_string: bool,
    cached_expression: RwLock<Option<Arc<dyn Any + Send + Sync>>>,
}

impl VariableExpression {
    /// 创建不启用字符串转换的变量表达式。
    ///
    /// # 参数
    /// - `expression`：`${}` 内部表达式；Java null 返回参数错误。
    /// 对应 Java 语义：`VariableExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(expression: Option<JavaString>) -> Result<Self, ValidateError> {
        Self::with_convert_to_string(expression, false)
    }

    /// 创建变量表达式。
    ///
    /// # 参数
    /// - `expression`：`${}` 内部表达式；
    /// - `convert_to_string`：是否对应双括号 `${{...}}`。
    /// 对应 Java 语义：`VariableExpression` 的 `with_convert_to_string` 行为（Rust 侧辅助/私有路径）。
    pub fn with_convert_to_string(
        expression: Option<JavaString>,
        convert_to_string: bool,
    ) -> Result<Self, ValidateError> {
        let expression = expression.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Expression cannot be null".to_owned()),
        })?;
        Ok(Self {
            expression,
            convert_to_string,
            cached_expression: RwLock::new(None),
        })
    }

    /// 返回定界符内部表达式。
    /// 对应 Java 语义：`VariableExpression` 的 `get_expression_value` 行为（Rust 侧辅助/私有路径）。
    pub fn get_expression_value(&self) -> &JavaString {
        &self.expression
    }

    /// 解析完整 `${...}` 文本；不匹配时返回 `None`。
    /// 对应 Java: `VariableExpression#parseVariableExpression()`。
    pub(crate) fn parse_variable_expression(input: &JavaString) -> Option<Self> {
        parse_delimited(input, b'$' as u16).and_then(|(expression, convert)| {
            Self::with_convert_to_string(Some(expression), convert).ok()
        })
    }
}

impl IStandardVariableExpression for VariableExpression {
    fn get_expression(&self) -> Option<&JavaString> {
        Some(&self.expression)
    }

    fn get_use_selection_as_root(&self) -> bool {
        false
    }

    fn get_convert_to_string(&self) -> bool {
        self.convert_to_string
    }

    fn get_cached_expression(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.cached_expression
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn set_cached_expression(&self, cached_expression: Option<Arc<dyn Any + Send + Sync>>) {
        *self
            .cached_expression
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = cached_expression;
    }
}

impl IStandardExpression for VariableExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        Ok(build_representation(
            b'$' as u16,
            &self.expression,
            self.convert_to_string,
        ))
    }

    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        execute_variable(context, self, execution_context)
    }
}

impl SimpleExpression for VariableExpression {}
/// 对应 Java 语义：`VariableExpression` 的 `parse_delimited` 行为（Rust 侧辅助/私有路径）。

pub(crate) fn parse_delimited(input: &JavaString, selector: u16) -> Option<(JavaString, bool)> {
    let units = input.as_utf16();
    let mut start = 0;
    while start < units.len() && regex_whitespace(units[start]) {
        start += 1;
    }
    let mut end = units.len();
    while end > start && regex_whitespace(units[end - 1]) {
        end -= 1;
    }
    if end.saturating_sub(start) < 4
        || units[start] != selector
        || units[start + 1] != b'{' as u16
        || units[end - 1] != b'}' as u16
    {
        return None;
    }
    let content = &units[start + 2..end - 1];
    if content.is_empty() {
        return None;
    }
    if content.len() > 2 && content[0] == b'{' as u16 && content[content.len() - 1] == b'}' as u16 {
        return Some((
            JavaString::from_utf16(content[1..content.len() - 1].to_vec()),
            true,
        ));
    }
    Some((JavaString::from_utf16(content.to_vec()), false))
}
/// 对应 Java 语义：`VariableExpression` 的 `build_representation` 行为（Rust 侧辅助/私有路径）。

pub(crate) fn build_representation(
    selector: u16,
    expression: &JavaString,
    convert_to_string: bool,
) -> JavaString {
    let mut units = vec![selector, b'{' as u16];
    if convert_to_string {
        units.push(b'{' as u16);
    }
    units.extend_from_slice(expression.as_utf16());
    if convert_to_string {
        units.push(b'}' as u16);
    }
    units.push(b'}' as u16);
    JavaString::from_utf16(units)
}
/// 对应 Java 语义：`VariableExpression` 的 `execute_variable` 行为（Rust 侧辅助/私有路径）。

pub(crate) fn execute_variable(
    context: &dyn IExpressionContext,
    expression: &dyn IStandardVariableExpression,
    execution_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let evaluator =
        StandardExpressions::get_variable_expression_evaluator(context.get_configuration())?;
    let evaluator_context = if expression.get_convert_to_string() {
        execution_context.with_type_conversion()
    } else {
        execution_context.without_type_conversion()
    };
    let result = evaluator.evaluate(context, expression, evaluator_context)?;
    if !execution_context.get_forbid_unsafe_expression_results()
        || result.as_deref().is_none_or(|value| {
            matches!(
                value,
                TemplateValue::Null | TemplateValue::Number(_) | TemplateValue::Boolean(_)
            )
        })
    {
        return Ok(result);
    }
    Err(Box::new(TemplateProcessingException::new(Some(
        "Only variable expressions returning numbers or booleans are allowed in this context, \
         any other data types are not trusted in the context of this expression, including \
         Strings or any other object that could be rendered as a text literal. A typical case \
         is HTML attributes for event handlers (e.g. \"onload\"), in which textual data from \
         variables should better be output to \"data-*\" attributes and then read from the event \
         handler."
            .to_owned(),
    ))))
}

fn regex_whitespace(unit: u16) -> bool {
    matches!(unit, 0x09..=0x0d | 0x20)
}
