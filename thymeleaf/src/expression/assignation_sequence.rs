#![expect(
    clippy::type_complexity,
    reason = "类型完整表达 Java 可空 List 与共享实时只读视图语义"
)]

use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::util::{JavaString, ValidateError};

use super::{Assignation, StandardExpressionResult};

/// 由原始列表支撑的不可修改赋值序列视图。
///
/// 对应 Java: `org.thymeleaf.standard.expression.AssignationSequence`。
pub struct AssignationSequence {
    assignations: Arc<RwLock<Vec<Option<Arc<Assignation>>>>>,
}

impl AssignationSequence {
    /// 保存原列表身份，并在构造瞬间拒绝 null 列表或 null 元素。
    pub(crate) fn new(
        assignations: Option<Arc<RwLock<Vec<Option<Arc<Assignation>>>>>>,
    ) -> Result<Self, ValidateError> {
        let assignations = assignations.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Assignation list cannot be null".to_owned()),
        })?;
        if read_recovering_poison(&assignations)
            .iter()
            .any(Option::is_none)
        {
            return Err(ValidateError::IllegalArgument {
                message: Some("Assignation list cannot contain any nulls".to_owned()),
            });
        }
        Ok(Self { assignations })
    }

    /// 返回 Java unmodifiableList 背后的实时只读视图。
    pub fn get_assignations(&self) -> RwLockReadGuard<'_, Vec<Option<Arc<Assignation>>>> {
        read_recovering_poison(&self.assignations)
    }

    /// 返回当前 backing list 大小。
    pub fn size(&self) -> i32 {
        i32::try_from(read_recovering_poison(&self.assignations).len()).unwrap_or(i32::MAX)
    }

    /// 返回逗号连接且不插入空格的当前字符串表示。
    pub fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        let assignations = read_recovering_poison(&self.assignations);
        let mut units = Vec::new();
        for (index, assignation) in assignations.iter().enumerate() {
            if index != 0 {
                units.push(b',' as u16);
            }
            match assignation {
                Some(assignation) => {
                    units.extend_from_slice(assignation.get_string_representation()?.as_utf16());
                }
                None => units.extend("null".encode_utf16()),
            }
        }
        Ok(JavaString::from_utf16(units))
    }
}

fn read_recovering_poison<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
