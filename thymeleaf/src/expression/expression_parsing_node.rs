#![expect(
    dead_code,
    reason = "解析节点将在同一批次后续 StandardExpressionParser 主链中消费"
)]

use std::sync::Arc;

use crate::util::JavaString;

use super::{IStandardExpression, StandardExpressionResult};

/// Standard Expression 解析状态中的输入或已解析表达式节点。
///
/// 对应 Java: `org.thymeleaf.standard.expression.ExpressionParsingNode`。
pub(crate) struct ExpressionParsingNode {
    input: Option<JavaString>,
    expression: Option<Arc<dyn IStandardExpression>>,
}

impl ExpressionParsingNode {
    /// 从半解析文本创建节点，并执行 Java `String#trim()`。
    /// 对应 Java 语义：`ExpressionParsingNode` 的 `from_input` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn from_input(input: JavaString) -> Self {
        Self {
            input: Some(java_trim(&input)),
            expression: None,
        }
    }

    /// 从已解析表达式创建节点。
    /// 对应 Java 语义：`ExpressionParsingNode` 的 `from_expression` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn from_expression(expression: Arc<dyn IStandardExpression>) -> Self {
        Self {
            input: None,
            expression: Some(expression),
        }
    }

    /// 判断节点是否保存输入文本。
    /// 对应 Java: `ExpressionParsingNode#isInput()`。
    pub(crate) fn is_input(&self) -> bool {
        self.input.is_some()
    }

    /// 判断节点是否保存已解析表达式。
    /// 对应 Java: `ExpressionParsingNode#isExpression()`。
    pub(crate) fn is_expression(&self) -> bool {
        self.expression.is_some()
    }

    /// 判断节点是否保存 SimpleExpression。
    /// 对应 Java: `ExpressionParsingNode#isSimpleExpression()`。
    pub(crate) fn is_simple_expression(&self) -> bool {
        self.expression
            .as_ref()
            .is_some_and(|expression| !expression.is_complex())
    }

    /// 判断节点是否保存 ComplexExpression。
    /// 对应 Java 语义：`ExpressionParsingNode` 的 `complex_expression` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn complex_expression(&self) -> bool {
        self.expression
            .as_ref()
            .is_some_and(|expression| expression.is_complex())
    }

    /// 返回可空输入文本。
    /// 对应 Java: `ExpressionParsingNode#getInput()`。
    pub(crate) fn get_input(&self) -> Option<&JavaString> {
        self.input.as_ref()
    }

    /// 返回可空表达式。
    /// 对应 Java: `ExpressionParsingNode#getExpression()`。
    pub(crate) fn get_expression(&self) -> Option<&Arc<dyn IStandardExpression>> {
        self.expression.as_ref()
    }

    /// 返回 Java `toString()` 的节点诊断文本。
    /// 对应 Java 语义：`ExpressionParsingNode` 的 `to_java_string` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn to_java_string(&self) -> StandardExpressionResult<JavaString> {
        if let Some(expression) = &self.expression {
            let mut units = vec![b'[' as u16];
            units.extend_from_slice(expression.get_string_representation()?.as_utf16());
            units.push(b']' as u16);
            return Ok(JavaString::from_utf16(units));
        }
        Ok(self
            .input
            .clone()
            .unwrap_or_else(|| JavaString::from_rust_str("null")))
    }
}

fn java_trim(input: &JavaString) -> JavaString {
    let units = input.as_utf16();
    let mut start = 0;
    while start < units.len() && units[start] <= 0x20 {
        start += 1;
    }
    let mut end = units.len();
    while end > start && units[end - 1] <= 0x20 {
        end -= 1;
    }
    JavaString::from_utf16(units[start..end].to_vec())
}
