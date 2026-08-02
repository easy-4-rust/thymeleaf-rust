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
    ///
    /// 对应 Java: `IdentifierSequences#IdentifierSequences()`。
    ///
    /// # 返回值
    ///
    /// 返回每个 ID 首次读取均为 `1` 的独立序列容器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            id_counts: Mutex::new(HashMap::with_capacity(1)),
        }
    }

    /// 返回当前计数，并以 Java `int` 回绕语义递增保存值。
    ///
    /// 对应 Java: `IdentifierSequences#getAndIncrementIDSeq(String)`。
    ///
    /// # 参数
    ///
    /// - `id`：非空 ID；`None` 对应 Java `null`。
    ///
    /// # 返回值
    ///
    /// 返回本次可使用的计数，首次为 `1`；保存的下一值按 Java `i32` 回绕。
    ///
    /// # 错误
    ///
    /// `id` 为空时返回 Java `IllegalArgumentException` 等价错误。
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
    ///
    /// 对应 Java: `IdentifierSequences#getNextIDSeq(String)`。
    ///
    /// # 参数
    ///
    /// - `id`：非空 ID；`None` 对应 Java `null`。
    ///
    /// # 返回值
    ///
    /// 返回当前保存的下一计数，未见 ID 返回 `1`。
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
    ///
    /// 对应 Java: `IdentifierSequences#getPreviousIDSeq(String)`。
    ///
    /// # 参数
    ///
    /// - `id`：非空 ID；`None` 对应 Java `null`。
    ///
    /// # 返回值
    ///
    /// 返回最近一次分配值，不改变状态。
    ///
    /// # 错误
    ///
    /// 未分配过该 ID 时返回 Java `TemplateProcessingException` 等价错误。
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

#[cfg(test)]
mod tests {
    use super::{IdentifierSequences, IdentifierSequencesError, lock_recovering_poison};
    use crate::util::JavaString;

    #[test]
    fn preserves_java_int_wrap_and_exact_error_categories() {
        let sequences = IdentifierSequences::new();
        let maximum = JavaString::from_rust_str("max");
        let missing = JavaString::from_rust_str("missing");
        lock_recovering_poison(&sequences.id_counts).insert(maximum.clone(), i32::MAX);

        assert_eq!(
            sequences.get_and_increment_id_seq(Some(&maximum)),
            Ok(i32::MAX)
        );
        assert_eq!(sequences.get_next_id_seq(Some(&maximum)), Ok(i32::MIN));
        assert_eq!(sequences.get_previous_id_seq(Some(&maximum)), Ok(i32::MAX));
        assert_eq!(
            sequences.get_previous_id_seq(Some(&missing)),
            Err(IdentifierSequencesError::MissingPrevious(missing))
        );
        assert_eq!(
            sequences.get_next_id_seq(None),
            Err(IdentifierSequencesError::NullId)
        );
    }
}
