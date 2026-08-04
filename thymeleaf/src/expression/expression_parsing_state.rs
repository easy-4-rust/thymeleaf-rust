#![expect(
    dead_code,
    reason = "解析状态将在同一批次后续 StandardExpressionParser 主链中消费"
)]

use std::sync::Arc;

use thiserror::Error;

use crate::util::Utf16String;

use super::{IStandardExpression, expression_parsing_node::ExpressionParsingNode};

/// Standard Expression 逐阶段解析使用的可变节点列表。
///
/// 对应 Java: `org.thymeleaf.standard.expression.ExpressionParsingState`。
pub(crate) struct ExpressionParsingState {
    nodes: Vec<ExpressionParsingNode>,
}

impl ExpressionParsingState {
    /// 创建空解析状态。
    /// 对应 Java 语义：`ExpressionParsingState` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// 返回节点数量。
    /// 对应 Java 语义：Java 接口/超类方法 `size()` 的 Rust 移植（`ExpressionParsingState` 继承路径）。
    pub(crate) fn size(&self) -> usize {
        self.nodes.len()
    }

    /// 返回指定节点。
    /// 对应 Java 语义：Java 接口/超类方法 `get()` 的 Rust 移植（`ExpressionParsingState` 继承路径）。
    pub(crate) fn get(
        &self,
        position: i32,
    ) -> Result<&ExpressionParsingNode, ExpressionParsingStateError> {
        self.nodes
            .get(to_index(position)?)
            .ok_or(ExpressionParsingStateError::IndexOutOfBounds { position })
    }

    /// 追加非空半解析文本节点。
    /// 对应 Java 语义：`ExpressionParsingState` 的 `add_node_input` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn add_node_input(
        &mut self,
        semi_parsed_string: Option<Utf16String>,
    ) -> Result<(), ExpressionParsingStateError> {
        let input = semi_parsed_string.ok_or(ExpressionParsingStateError::IllegalArgument {
            message: "String cannot be null",
        })?;
        self.nodes.push(ExpressionParsingNode::from_input(input));
        Ok(())
    }

    /// 追加非空表达式节点。
    /// 对应 Java 语义：`ExpressionParsingState` 的 `add_node_expression` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn add_node_expression(
        &mut self,
        parsed_expression: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<(), ExpressionParsingStateError> {
        let expression = parsed_expression.ok_or(ExpressionParsingStateError::IllegalArgument {
            message: "Expression cannot be null",
        })?;
        self.nodes
            .push(ExpressionParsingNode::from_expression(expression));
        Ok(())
    }

    /// 在指定位置插入半解析文本节点。
    /// 对应 Java 语义：`ExpressionParsingState` 的 `insert_node_input` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn insert_node_input(
        &mut self,
        position: i32,
        semi_parsed_string: Option<Utf16String>,
    ) -> Result<(), ExpressionParsingStateError> {
        let input = semi_parsed_string.ok_or(ExpressionParsingStateError::IllegalArgument {
            message: "String cannot be null",
        })?;
        let position = insertion_index(position, self.nodes.len())?;
        self.nodes
            .insert(position, ExpressionParsingNode::from_input(input));
        Ok(())
    }

    /// 在指定位置插入表达式节点。
    /// 对应 Java 语义：`ExpressionParsingState` 的 `insert_node_expression` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn insert_node_expression(
        &mut self,
        position: i32,
        parsed_expression: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<(), ExpressionParsingStateError> {
        let expression = parsed_expression.ok_or(ExpressionParsingStateError::IllegalArgument {
            message: "Expression cannot be null",
        })?;
        let position = insertion_index(position, self.nodes.len())?;
        self.nodes
            .insert(position, ExpressionParsingNode::from_expression(expression));
        Ok(())
    }

    /// 替换指定位置的半解析文本节点。
    /// 对应 Java 语义：`ExpressionParsingState` 的 `set_node_input` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn set_node_input(
        &mut self,
        position: i32,
        semi_parsed_string: Option<Utf16String>,
    ) -> Result<(), ExpressionParsingStateError> {
        let input = semi_parsed_string.ok_or(ExpressionParsingStateError::IllegalArgument {
            message: "String cannot be null",
        })?;
        *self.node_mut(position)? = ExpressionParsingNode::from_input(input);
        Ok(())
    }

    /// 替换指定位置的表达式节点。
    /// 对应 Java 语义：`ExpressionParsingState` 的 `set_node_expression` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn set_node_expression(
        &mut self,
        position: i32,
        parsed_expression: Option<Arc<dyn IStandardExpression>>,
    ) -> Result<(), ExpressionParsingStateError> {
        let expression = parsed_expression.ok_or(ExpressionParsingStateError::IllegalArgument {
            message: "Expression cannot be null",
        })?;
        *self.node_mut(position)? = ExpressionParsingNode::from_expression(expression);
        Ok(())
    }

    /// 判断根节点是否为输入字符串。
    /// 对应 Java: `ExpressionParsingState#hasStringRoot()`。
    pub(crate) fn has_string_root(&self) -> bool {
        self.has_string_at(0)
    }

    /// 判断根节点是否为表达式。
    /// 对应 Java: `ExpressionParsingState#hasExpressionRoot()`。
    pub(crate) fn has_expression_root(&self) -> bool {
        self.has_expression_at(0)
    }

    /// 判断给定位置存在且为输入字符串。
    /// 对应 Java: `ExpressionParsingState#hasStringAt()`。
    pub(crate) fn has_string_at(&self, position: i32) -> bool {
        usize::try_from(position)
            .ok()
            .and_then(|position| self.nodes.get(position))
            .is_some_and(ExpressionParsingNode::is_input)
    }

    /// 判断给定位置存在且为表达式。
    /// 对应 Java: `ExpressionParsingState#hasExpressionAt()`。
    pub(crate) fn has_expression_at(&self, position: i32) -> bool {
        usize::try_from(position)
            .ok()
            .and_then(|position| self.nodes.get(position))
            .is_some_and(ExpressionParsingNode::is_expression)
    }

    fn node_mut(
        &mut self,
        position: i32,
    ) -> Result<&mut ExpressionParsingNode, ExpressionParsingStateError> {
        self.nodes
            .get_mut(to_index(position)?)
            .ok_or(ExpressionParsingStateError::IndexOutOfBounds { position })
    }
}

/// 解析状态的 Java 参数与索引错误。
#[derive(Debug, Error, Eq, PartialEq)]
/// 对应 Java 语义：`ExpressionParsingState` 的 Rust 侧类型 `ExpressionParsingStateError`。
pub(crate) enum ExpressionParsingStateError {
    /// Java `IllegalArgumentException`。
    #[error("{message}")]
    IllegalArgument { message: &'static str },
    /// Java `IndexOutOfBoundsException`。
    #[error("Index: {position}")]
    IndexOutOfBounds { position: i32 },
}

fn to_index(position: i32) -> Result<usize, ExpressionParsingStateError> {
    usize::try_from(position)
        .map_err(|_| ExpressionParsingStateError::IndexOutOfBounds { position })
}

fn insertion_index(position: i32, len: usize) -> Result<usize, ExpressionParsingStateError> {
    let position = to_index(position)?;
    if position > len {
        return Err(ExpressionParsingStateError::IndexOutOfBounds {
            position: i32::try_from(position).unwrap_or(i32::MAX),
        });
    }
    Ok(position)
}
