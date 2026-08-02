use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

use crate::util::{MapUtils, ValidateError};

/// Thymeleaf 标准表达式中的映射操作对象。
///
/// 对应 Java: `org.thymeleaf.expression.Maps`。
///
/// 该无状态对象通常以 `#maps` 暴露，全部方法严格委托给 [`MapUtils`]，因而
/// 共享 Java null、校验顺序、空请求集合以及重复请求项的语义。
#[derive(Debug, Default, Clone, Copy)]
pub struct Maps;

impl Maps {
    /// 创建无状态映射表达式对象。
    ///
    /// 对应 Java: `Maps#Maps()`。
    ///
    /// # 返回
    /// 新的 `#maps` 表达式对象。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 返回映射大小。
    ///
    /// 对应 Java: `Maps#size(Map)`。
    ///
    /// # 参数
    /// - `target`：目标映射；`None` 对应 Java null。
    ///
    /// # 返回
    /// Java `Map#size()` 等价值。
    ///
    /// # 错误
    /// 传播 [`MapUtils::size`] 的精确参数错误。
    pub fn size<K, V, S>(&self, target: Option<&HashMap<K, V, S>>) -> Result<i32, ValidateError> {
        MapUtils::size(target)
    }

    /// 判断映射是否为 null 或为空。
    ///
    /// 对应 Java: `Maps#isEmpty(Map)`。
    ///
    /// # 参数
    /// - `target`：目标映射。
    ///
    /// # 返回
    /// target 为 null 或没有条目时返回 `true`。
    #[must_use]
    pub fn is_empty<K, V, S>(&self, target: Option<&HashMap<K, V, S>>) -> bool {
        MapUtils::is_empty(target)
    }

    /// 判断映射是否包含指定键。
    ///
    /// 对应 Java: `Maps#containsKey(Map,Object)`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `key`：请求键。
    ///
    /// # 返回
    /// 键存在时返回 `true`。
    ///
    /// # 错误
    /// 传播 [`MapUtils::contains_key`] 的精确参数错误。
    pub fn contains_key<K, V, Q, S>(
        &self,
        target: Option<&HashMap<K, V, S>>,
        key: &Q,
    ) -> Result<bool, ValidateError>
    where
        K: Borrow<Q> + Eq + Hash,
        Q: Eq + Hash + ?Sized,
        S: BuildHasher,
    {
        MapUtils::contains_key(target, key)
    }

    /// 判断映射是否包含数组中的全部键。
    ///
    /// 对应 Java: `Maps#containsAllKeys(Map,Object[])`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `keys`：请求键数组。
    ///
    /// # 返回
    /// 全部请求键存在时返回 `true`。
    ///
    /// # 错误
    /// 按 Java 顺序传播 target、keys 校验错误。
    pub fn contains_all_keys_array<K, V, Q, S>(
        &self,
        target: Option<&HashMap<K, V, S>>,
        keys: Option<&[Q]>,
    ) -> Result<bool, ValidateError>
    where
        K: Borrow<Q> + Eq + Hash,
        Q: Eq + Hash,
        S: BuildHasher,
    {
        MapUtils::contains_all_keys_array(target, keys)
    }

    /// 判断映射是否包含 Collection 中的全部键。
    ///
    /// 对应 Java: `Maps#containsAllKeys(Map,Collection)`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `keys`：请求键 Collection 的迭代器。
    ///
    /// # 返回
    /// 全部请求键存在时返回 `true`。
    ///
    /// # 错误
    /// 按 Java 顺序传播 target、keys 校验错误。
    pub fn contains_all_keys_collection<'a, K, V, Q, S, I>(
        &self,
        target: Option<&HashMap<K, V, S>>,
        keys: Option<I>,
    ) -> Result<bool, ValidateError>
    where
        K: Borrow<Q> + Eq + Hash,
        Q: Eq + Hash + ?Sized + 'a,
        S: BuildHasher,
        I: IntoIterator<Item = &'a Q>,
    {
        MapUtils::contains_all_keys_collection(target, keys)
    }

    /// 判断映射是否包含指定值。
    ///
    /// 对应 Java: `Maps#containsValue(Map,Object)`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `value`：请求值。
    ///
    /// # 返回
    /// 任一映射值相等时返回 `true`。
    ///
    /// # 错误
    /// 传播 [`MapUtils::contains_value`] 的精确参数错误。
    pub fn contains_value<K, V, Q, S>(
        &self,
        target: Option<&HashMap<K, V, S>>,
        value: &Q,
    ) -> Result<bool, ValidateError>
    where
        Q: PartialEq<V> + ?Sized,
    {
        MapUtils::contains_value(target, value)
    }

    /// 判断映射是否包含数组中的全部值。
    ///
    /// 对应 Java: `Maps#containsAllValues(Map,Object[])`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `values`：请求值数组。
    ///
    /// # 返回
    /// 每个请求值至少出现一次时返回 `true`。
    ///
    /// # 错误
    /// 按 Java 顺序传播 target、values 校验错误。
    pub fn contains_all_values_array<K, V, Q, S>(
        &self,
        target: Option<&HashMap<K, V, S>>,
        values: Option<&[Q]>,
    ) -> Result<bool, ValidateError>
    where
        Q: PartialEq<V>,
    {
        MapUtils::contains_all_values_array(target, values)
    }

    /// 判断映射是否包含 Collection 中的全部值。
    ///
    /// 对应 Java: `Maps#containsAllValues(Map,Collection)`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `values`：请求值 Collection 的迭代器。
    ///
    /// # 返回
    /// 每个请求值至少出现一次时返回 `true`。
    ///
    /// # 错误
    /// 按 Java 顺序传播 target、values 校验错误。
    pub fn contains_all_values_collection<'a, K, V, Q, S, I>(
        &self,
        target: Option<&HashMap<K, V, S>>,
        values: Option<I>,
    ) -> Result<bool, ValidateError>
    where
        Q: PartialEq<V> + ?Sized + 'a,
        I: IntoIterator<Item = &'a Q>,
    {
        MapUtils::contains_all_values_collection(target, values)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Maps;

    #[test]
    fn delegates_all_java_operations_to_map_utils() {
        let maps = Maps::new();
        let target = HashMap::from([
            (Some("one".to_owned()), Some("value".to_owned())),
            (Some("two".to_owned()), Some("other".to_owned())),
            (None, None),
        ]);
        let keys = [Some("one".to_owned()), None];
        let values = [Some("value".to_owned()), None];

        assert_eq!(maps.size(Some(&target)), Ok(3));
        assert!(!maps.is_empty(Some(&target)));
        assert_eq!(maps.contains_key(Some(&target), &None), Ok(true));
        assert_eq!(
            maps.contains_all_keys_array(Some(&target), Some(&keys)),
            Ok(true)
        );
        assert_eq!(
            maps.contains_all_keys_collection(Some(&target), Some(keys.iter())),
            Ok(true)
        );
        assert_eq!(maps.contains_value(Some(&target), &None), Ok(true));
        assert_eq!(
            maps.contains_all_values_array(Some(&target), Some(&values)),
            Ok(true)
        );
        assert_eq!(
            maps.contains_all_values_collection(Some(&target), Some(values.iter())),
            Ok(true)
        );
        assert!(Maps.is_empty(None::<&HashMap<String, String>>));
    }
}
