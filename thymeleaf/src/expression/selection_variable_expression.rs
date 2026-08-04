use std::any::Any;
use std::sync::{Arc, RwLock};

use crate::context::IExpressionContext;
use crate::util::{Utf16String, ValidateError};

use super::{
    IStandardExpression, IStandardVariableExpression, SimpleExpression,
    StandardExpressionExecutionContext, StandardExpressionResult, TemplateValue,
    variable_expression::{build_representation, execute_variable, parse_delimited},
};

/// `*{...}` Standard Selection Variable Expression。
///
/// 对应 Java: `org.thymeleaf.standard.expression.SelectionVariableExpression`。
pub struct SelectionVariableExpression {
    expression: Utf16String,
    convert_to_string: bool,
    cached_expression: RwLock<Option<Arc<dyn Any + Send + Sync>>>,
}

impl SelectionVariableExpression {
    /// 创建不启用字符串转换的 selection 表达式。
    /// 对应 Java 语义：`SelectionVariableExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(expression: Option<Utf16String>) -> Result<Self, ValidateError> {
        Self::with_convert_to_string(expression, false)
    }

    /// 创建 selection 表达式。
    /// 对应 Java 语义：`SelectionVariableExpression` 的 `with_convert_to_string` 行为（Rust 侧辅助/私有路径）。
    pub fn with_convert_to_string(
        expression: Option<Utf16String>,
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
    /// 对应 Java 语义：`SelectionVariableExpression` 的 `get_expression_value` 行为（Rust 侧辅助/私有路径）。
    pub fn get_expression_value(&self) -> &Utf16String {
        &self.expression
    }

    /// 解析完整 `*{...}` 文本；不匹配时返回 `None`。
    /// 对应 Java: `SelectionVariableExpression#parseSelectionVariableExpression()`。
    pub(crate) fn parse_selection_variable_expression(input: &Utf16String) -> Option<Self> {
        parse_delimited(input, b'*' as u16).and_then(|(expression, convert)| {
            Self::with_convert_to_string(Some(expression), convert).ok()
        })
    }
}

impl IStandardVariableExpression for SelectionVariableExpression {
    fn get_expression(&self) -> Option<&Utf16String> {
        Some(&self.expression)
    }

    fn get_use_selection_as_root(&self) -> bool {
        true
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

impl IStandardExpression for SelectionVariableExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<Utf16String> {
        Ok(build_representation(
            b'*' as u16,
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

impl SimpleExpression for SelectionVariableExpression {}
