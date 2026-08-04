#![expect(
    clippy::type_complexity,
    reason = "类型完整表达 Java 可空 List 与共享实时只读视图语义"
)]

use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::util::{Utf16String, ValidateError};

use super::{IStandardExpression, StandardExpressionResult};

/// 由原始列表支撑的不可修改 Standard Expression 序列视图。
///
/// 对应 Java: `org.thymeleaf.standard.expression.ExpressionSequence`。
pub struct ExpressionSequence {
    expressions: Arc<RwLock<Vec<Option<Arc<dyn IStandardExpression>>>>>,
}

impl ExpressionSequence {
    /// 保存原列表身份，并在构造瞬间拒绝 null 列表或 null 元素。
    /// 对应 Java 语义：`ExpressionSequence` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        expressions: Option<Arc<RwLock<Vec<Option<Arc<dyn IStandardExpression>>>>>>,
    ) -> Result<Self, ValidateError> {
        let expressions = expressions.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Expression list cannot be null".to_owned()),
        })?;
        if read_recovering_poison(&expressions)
            .iter()
            .any(Option::is_none)
        {
            return Err(ValidateError::IllegalArgument {
                message: Some("Expression list cannot contain any nulls".to_owned()),
            });
        }
        Ok(Self { expressions })
    }

    /// 返回 Java unmodifiableList 背后的实时只读视图。
    /// 对应 Java: `ExpressionSequence#getExpressions()`。
    pub fn get_expressions(
        &self,
    ) -> RwLockReadGuard<'_, Vec<Option<Arc<dyn IStandardExpression>>>> {
        read_recovering_poison(&self.expressions)
    }

    /// 返回当前 backing list 大小。
    /// 对应 Java: `ExpressionSequence#size()`。
    pub fn size(&self) -> i32 {
        i32::try_from(read_recovering_poison(&self.expressions).len()).unwrap_or(i32::MAX)
    }

    /// 返回逗号连接且不插入空格的当前字符串表示。
    /// 对应 Java: `ExpressionSequence#getStringRepresentation()`。
    pub fn get_string_representation(&self) -> StandardExpressionResult<Utf16String> {
        let expressions = read_recovering_poison(&self.expressions);
        let mut units = Vec::new();
        for (index, expression) in expressions.iter().enumerate() {
            if index != 0 {
                units.push(b',' as u16);
            }
            match expression {
                Some(expression) => {
                    units.extend_from_slice(expression.get_string_representation()?.as_utf16());
                }
                None => units.extend("null".encode_utf16()),
            }
        }
        Ok(Utf16String::from_utf16(units))
    }
}

fn read_recovering_poison<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
