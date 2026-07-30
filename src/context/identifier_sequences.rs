use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Mutex, MutexGuard};

use crate::util::JavaString;

/// 标识符序列参数或状态错误。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentifierSequencesError {
    /// `Validate.notNull` 对应的参数错误。
    NullId,
    /// 尚未生成过该 ID，无法取得 previous 值。
    MissingPrevious(JavaString),
}

impl IdentifierSequencesError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::NullId => "java.lang.IllegalArgumentException",
            Self::MissingPrevious(_) => "org.thymeleaf.exceptions.TemplateProcessingException",
        }
    }
}

impl Display for IdentifierSequencesError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullId => formatter.write_str("ID cannot be null"),
            Self::MissingPrevious(id) => write!(
                formatter,
                "Cannot obtain previous ID count for ID \"{}\"",
                id.to_string_lossy()
            ),
        }
    }
}

impl Error for IdentifierSequencesError {}

/// 模板执行期间为每个 HTML `id` 维护独立的 Java `Integer` 序列。
///
/// 对应 Java: `org.thymeleaf.context.IdentifierSequences`。
pub struct IdentifierSequences {
    id_counts: Mutex<HashMap<JavaString, i32>>,
}

impl IdentifierSequences {
    /// 创建空序列集合。
    #[must_use]
    pub fn new() -> Self {
        Self {
            id_counts: Mutex::new(HashMap::with_capacity(1)),
        }
    }

    /// 返回当前计数，并以 Java `int` 回绕语义递增保存值。
    pub fn get_and_increment_id_seq(
        &self,
        id: Option<&JavaString>,
    ) -> Result<i32, IdentifierSequencesError> {
        let id = id.ok_or(IdentifierSequencesError::NullId)?;
        let mut id_counts = lock_recovering_poison(&self.id_counts);
        let count = id_counts.get(id).copied().unwrap_or(1);
        id_counts.insert(id.clone(), count.wrapping_add(1));
        Ok(count)
    }

    /// 返回下一次会使用的计数，但不改变状态。
    pub fn get_next_id_seq(
        &self,
        id: Option<&JavaString>,
    ) -> Result<i32, IdentifierSequencesError> {
        let id = id.ok_or(IdentifierSequencesError::NullId)?;
        Ok(lock_recovering_poison(&self.id_counts)
            .get(id)
            .copied()
            .unwrap_or(1))
    }

    /// 返回最近一次分配的计数。
    pub fn get_previous_id_seq(
        &self,
        id: Option<&JavaString>,
    ) -> Result<i32, IdentifierSequencesError> {
        let id = id.ok_or(IdentifierSequencesError::NullId)?;
        lock_recovering_poison(&self.id_counts)
            .get(id)
            .copied()
            .map(|count| count.wrapping_sub(1))
            .ok_or_else(|| IdentifierSequencesError::MissingPrevious(id.clone()))
    }
}

impl Default for IdentifierSequences {
    fn default() -> Self {
        Self::new()
    }
}

fn lock_recovering_poison<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
