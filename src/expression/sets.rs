use std::hash::Hash;

use crate::util::{JavaSet, SetTarget, SetUtils, SetUtilsError, SetView, ValidateError};

/// Thymeleaf 标准表达式中的集合操作对象。
///
/// 对应 Java: `org.thymeleaf.expression.Sets`。
///
/// 该对象通常以 `#sets` 名称出现在变量求值表达式中。所有操作严格委托给
/// [`SetUtils`]，因此与 Java 一样共享动态转换、校验、集合身份和顺序语义。
#[derive(Debug, Default, Clone, Copy)]
pub struct Sets;

impl Sets {
    /// 创建无状态集合表达式对象。
    ///
    /// 对应 Java: `Sets#Sets()`。
    ///
    /// # 返回
    /// 新的无状态 `#sets` 表达式对象。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 将动态目标转换为只读集合。
    ///
    /// 对应 Java: `Sets#toSet(Object)`。
    ///
    /// # 参数
    /// - `target`：Java 参数 `target`；`None` 对应 Java null。
    ///
    /// # 错误
    /// 传播 [`SetUtils::to_set`] 的原始参数或类型错误。
    ///
    /// # 返回
    /// 已有集合的借用视图，或新建的有序只读集合。
    pub fn to_set<'a, T>(
        &self,
        target: Option<SetTarget<'a, T>>,
    ) -> Result<JavaSet<'a, T>, SetUtilsError>
    where
        T: Clone + Eq + Hash,
    {
        SetUtils::to_set(target)
    }

    /// 返回集合大小。
    ///
    /// 对应 Java: `Sets#size(Set)`。
    ///
    /// # 参数
    /// - `target`：待计算大小的集合；`None` 对应 Java null。
    ///
    /// # 返回
    /// 集合大小。
    ///
    /// # 错误
    /// target 为 null 时返回上游精确参数错误。
    pub fn size<T>(&self, target: Option<&dyn SetView<T>>) -> Result<i32, ValidateError> {
        SetUtils::size(target)
    }

    /// 判断集合是否为 null 或没有元素。
    ///
    /// 对应 Java: `Sets#isEmpty(Set)`。
    ///
    /// # 参数
    /// - `target`：待判断集合。
    ///
    /// # 返回
    /// target 为 null 或空集合时返回 `true`。
    #[must_use]
    pub fn is_empty<T>(&self, target: Option<&dyn SetView<T>>) -> bool {
        SetUtils::is_empty(target)
    }

    /// 判断集合是否包含指定元素。
    ///
    /// 对应 Java: `Sets#contains(Set,Object)`。
    ///
    /// # 参数
    /// - `target`：目标集合；
    /// - `element`：待查找元素。
    ///
    /// # 返回
    /// 集合包含元素时返回 `true`。
    ///
    /// # 错误
    /// target 为 null 时返回上游精确参数错误。
    pub fn contains<T>(
        &self,
        target: Option<&dyn SetView<T>>,
        element: &T,
    ) -> Result<bool, ValidateError> {
        SetUtils::contains(target, element)
    }

    /// 判断集合是否包含数组中的全部元素。
    ///
    /// 对应 Java: `Sets#containsAll(Set,Object[])`。
    ///
    /// # 参数
    /// - `target`：目标集合；
    /// - `elements`：待检查元素数组。
    ///
    /// # 返回
    /// 所有元素均存在时返回 `true`。
    ///
    /// # 错误
    /// target 或 elements 为 null 时返回上游精确参数错误。
    pub fn contains_all_array<T>(
        &self,
        target: Option<&dyn SetView<T>>,
        elements: Option<&[T]>,
    ) -> Result<bool, ValidateError> {
        SetUtils::contains_all_array(target, elements)
    }

    /// 判断集合是否包含 Collection 中的全部元素。
    ///
    /// 对应 Java: `Sets#containsAll(Set,Collection)`。
    ///
    /// # 参数
    /// - `target`：目标集合；
    /// - `elements`：待检查 Collection 迭代器。
    ///
    /// # 返回
    /// 所有元素均存在时返回 `true`。
    ///
    /// # 错误
    /// target 或 elements 为 null 时返回上游精确参数错误。
    pub fn contains_all_collection<'a, T, I>(
        &self,
        target: Option<&dyn SetView<T>>,
        elements: Option<I>,
    ) -> Result<bool, ValidateError>
    where
        T: 'a,
        I: IntoIterator<Item = &'a T>,
    {
        SetUtils::contains_all_collection(target, elements)
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexSet;

    use super::Sets;
    use crate::util::{SetTarget, SetView};

    #[test]
    fn delegates_every_operation_to_set_utils() {
        let sets = Sets::new();
        let source = IndexSet::from([Some("one".to_owned()), Some("two".to_owned()), None]);
        let view: &dyn SetView<Option<String>> = &source;

        let converted = sets
            .to_set(Some(SetTarget::Set(view)))
            .expect("set conversion");
        assert!(converted.is_borrowed_from(view));
        assert_eq!(sets.size(Some(view)), Ok(3));
        assert!(!sets.is_empty(Some(view)));
        assert_eq!(sets.contains(Some(view), &None), Ok(true));
        assert_eq!(
            sets.contains_all_array(Some(view), Some(&[Some("one".to_owned()), None])),
            Ok(true)
        );
        assert_eq!(
            sets.contains_all_collection(Some(view), Some([Some("two".to_owned()), None].iter())),
            Ok(true)
        );

        let default_sets = Sets;
        assert!(default_sets.is_empty(None::<&dyn SetView<Option<String>>>));
    }
}
