use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

use super::{Validate, ValidateError};

/// Thymeleaf 的映射集合工具。
///
/// 对应 Java: `org.thymeleaf.util.MapUtils`。
///
/// Java 对象只提供无状态静态操作。Rust 以 `HashMap` 表达 Java `Map` 的键值查找
/// 合同，并保留 null target、null keys/values 集合、空集合、null 键值以及
/// `containsAll` 忽略请求项重复次数的语义。调用失败直接返回 [`ValidateError`]，
/// 其异常类别和消息与上游 `Validate.notNull` 一致。
pub struct MapUtils;

impl MapUtils {
    /// 返回映射大小。
    ///
    /// 对应 Java: `MapUtils#size(Map)`。
    ///
    /// # 参数
    /// - `target`：待计算大小的映射；`None` 对应 Java null。
    ///
    /// # 返回
    /// 映射大小；若 Rust 集合理论上超过 `Integer.MAX_VALUE`，按 Java `Map#size`
    /// 合同返回 `i32::MAX`。
    ///
    /// # 错误
    /// target 为 `None` 时返回消息为 `Cannot get map size of null` 的参数错误。
    pub fn size<K, V, S>(target: Option<&HashMap<K, V, S>>) -> Result<i32, ValidateError> {
        Validate::not_null(target, Some("Cannot get map size of null"))?;
        let size = target.expect("validated target").len();
        Ok(java_map_size(size))
    }

    /// 判断映射是否为 null 或没有条目。
    ///
    /// 对应 Java: `MapUtils#isEmpty(Map)`。
    ///
    /// # 参数
    /// - `target`：待判断映射；`None` 对应 Java null。
    ///
    /// # 返回
    /// target 为 `None` 或空映射时返回 `true`。
    #[must_use]
    pub fn is_empty<K, V, S>(target: Option<&HashMap<K, V, S>>) -> bool {
        target.is_none_or(HashMap::is_empty)
    }

    /// 判断映射是否包含指定键。
    ///
    /// 对应 Java: `MapUtils#containsKey(Map, Object)`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `key`：待查找键，可通过 `K = Option<T>` 保留 Java null 键。
    ///
    /// # 返回
    /// 键存在时返回 `true`。
    ///
    /// # 错误
    /// target 为 `None` 时返回上游精确参数错误。
    pub fn contains_key<K, V, Q, S>(
        target: Option<&HashMap<K, V, S>>,
        key: &Q,
    ) -> Result<bool, ValidateError>
    where
        K: Borrow<Q> + Eq + Hash,
        Q: Eq + Hash + ?Sized,
        S: BuildHasher,
    {
        Validate::not_null(
            target,
            Some("Cannot execute map containsKey: target is null"),
        )?;
        Ok(target.expect("validated target").contains_key(key))
    }

    /// 判断映射是否包含数组中的全部键。
    ///
    /// 对应 Java: `MapUtils#containsAllKeys(Map, Object[])`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `keys`：待查找键数组；`None` 对应 Java null。
    ///
    /// # 返回
    /// 每个请求键都存在时返回 `true`；空数组恒为 `true`，重复键不要求重复条目。
    ///
    /// # 错误
    /// 按 Java 顺序先校验 target，再校验 keys，并保留各自精确消息。
    pub fn contains_all_keys_array<K, V, Q, S>(
        target: Option<&HashMap<K, V, S>>,
        keys: Option<&[Q]>,
    ) -> Result<bool, ValidateError>
    where
        K: Borrow<Q> + Eq + Hash,
        Q: Eq + Hash,
        S: BuildHasher,
    {
        Validate::not_null(
            target,
            Some("Cannot execute map containsAllKeys: target is null"),
        )?;
        Validate::not_null(
            keys,
            Some("Cannot execute map containsAllKeys: keys is null"),
        )?;
        let target = target.expect("validated target");
        Ok(keys
            .expect("validated keys")
            .iter()
            .all(|key| target.contains_key(key)))
    }

    /// 判断映射是否包含集合迭代器中的全部键。
    ///
    /// 对应 Java: `MapUtils#containsAllKeys(Map, Collection)`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `keys`：借用键的可选迭代器；`None` 对应 Java null Collection。
    ///
    /// # 返回
    /// 所有请求键都存在时返回 `true`。
    ///
    /// # 错误
    /// 按 Java 顺序先校验 target，再校验 keys。
    pub fn contains_all_keys_collection<'a, K, V, Q, S, I>(
        target: Option<&HashMap<K, V, S>>,
        keys: Option<I>,
    ) -> Result<bool, ValidateError>
    where
        K: Borrow<Q> + Eq + Hash,
        Q: Eq + Hash + ?Sized + 'a,
        S: BuildHasher,
        I: IntoIterator<Item = &'a Q>,
    {
        Validate::not_null(
            target,
            Some("Cannot execute map containsAllKeys: target is null"),
        )?;
        Validate::not_null(
            keys.as_ref(),
            Some("Cannot execute map containsAllKeys: keys is null"),
        )?;
        let target = target.expect("validated target");
        Ok(keys
            .expect("validated keys")
            .into_iter()
            .all(|key| target.contains_key(key)))
    }

    /// 判断映射是否包含指定值。
    ///
    /// 对应 Java: `MapUtils#containsValue(Map, Object)`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `value`：待查找值，可通过 `V = Option<T>` 保留 Java null 值。
    ///
    /// # 返回
    /// 任一映射值与请求值相等时返回 `true`。
    ///
    /// # 错误
    /// target 为 `None` 时返回上游精确参数错误。
    pub fn contains_value<K, V, Q, S>(
        target: Option<&HashMap<K, V, S>>,
        value: &Q,
    ) -> Result<bool, ValidateError>
    where
        Q: PartialEq<V> + ?Sized,
    {
        Validate::not_null(
            target,
            Some("Cannot execute map containsValue: target is null"),
        )?;
        Ok(target
            .expect("validated target")
            .values()
            .any(|candidate| value == candidate))
    }

    /// 判断映射是否包含数组中的全部值。
    ///
    /// 对应 Java: `MapUtils#containsAllValues(Map, Object[])`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `values`：待查找值数组；`None` 对应 Java null。
    ///
    /// # 返回
    /// 每个请求值至少在映射中出现一次时返回 `true`；重复请求值不要求多个映射项。
    ///
    /// # 错误
    /// 按 Java 顺序先校验 target，再校验 values。
    pub fn contains_all_values_array<K, V, Q, S>(
        target: Option<&HashMap<K, V, S>>,
        values: Option<&[Q]>,
    ) -> Result<bool, ValidateError>
    where
        Q: PartialEq<V>,
    {
        Validate::not_null(
            target,
            Some("Cannot execute map containsAllValues: target is null"),
        )?;
        Validate::not_null(
            values,
            Some("Cannot execute map containsAllValues: values is null"),
        )?;
        let target = target.expect("validated target");
        Ok(values
            .expect("validated values")
            .iter()
            .all(|value| target.values().any(|candidate| value == candidate)))
    }

    /// 判断映射是否包含集合迭代器中的全部值。
    ///
    /// 对应 Java: `MapUtils#containsAllValues(Map, Collection)`。
    ///
    /// # 参数
    /// - `target`：目标映射；
    /// - `values`：借用值的可选迭代器；`None` 对应 Java null Collection。
    ///
    /// # 返回
    /// 所有请求值至少出现一次时返回 `true`。
    ///
    /// # 错误
    /// 按 Java 顺序先校验 target，再校验 values。
    pub fn contains_all_values_collection<'a, K, V, Q, S, I>(
        target: Option<&HashMap<K, V, S>>,
        values: Option<I>,
    ) -> Result<bool, ValidateError>
    where
        Q: PartialEq<V> + ?Sized + 'a,
        I: IntoIterator<Item = &'a Q>,
    {
        Validate::not_null(
            target,
            Some("Cannot execute map containsAllValues: target is null"),
        )?;
        Validate::not_null(
            values.as_ref(),
            Some("Cannot execute map containsAllValues: values is null"),
        )?;
        let target = target.expect("validated target");
        Ok(values
            .expect("validated values")
            .into_iter()
            .all(|value| target.values().any(|candidate| value == candidate)))
    }
}

fn java_map_size(size: usize) -> i32 {
    i32::try_from(size).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::MapUtils;
    use crate::util::ValidateError;

    fn map() -> HashMap<Option<String>, Option<String>> {
        HashMap::from([
            (Some("one".to_owned()), Some("value".to_owned())),
            (Some("two".to_owned()), Some("other".to_owned())),
            (None, None),
        ])
    }

    #[test]
    fn preserves_size_and_empty_null_semantics() {
        let map = map();
        let empty: HashMap<String, String> = HashMap::new();
        assert_eq!(MapUtils::size(Some(&map)), Ok(3));
        assert_eq!(
            MapUtils::size(None::<&HashMap<Option<String>, Option<String>>>),
            Err(ValidateError::IllegalArgument {
                message: Some("Cannot get map size of null".to_owned())
            })
        );
        assert!(!MapUtils::is_empty(Some(&map)));
        assert!(MapUtils::is_empty(Some(&empty)));
        assert!(MapUtils::is_empty(None::<&HashMap<String, String>>));
        assert_eq!(super::java_map_size(usize::MAX), i32::MAX);
    }

    #[test]
    fn finds_present_missing_and_null_keys() {
        let map = map();
        assert_eq!(
            MapUtils::contains_key(Some(&map), &Some("one".to_owned())),
            Ok(true)
        );
        assert_eq!(
            MapUtils::contains_key(Some(&map), &Some("missing".to_owned())),
            Ok(false)
        );
        assert_eq!(MapUtils::contains_key(Some(&map), &None), Ok(true));
        let error = MapUtils::contains_key(
            None::<&HashMap<Option<String>, Option<String>>>,
            &Some("key".to_owned()),
        )
        .expect_err("error");
        assert_eq!(
            error.get_message(),
            Some("Cannot execute map containsKey: target is null")
        );
    }

    #[test]
    fn validates_and_checks_all_key_overloads() {
        let map = map();
        let present = [Some("one".to_owned()), None];
        let missing = [Some("one".to_owned()), Some("missing".to_owned())];
        assert_eq!(
            MapUtils::contains_all_keys_array(Some(&map), Some(&present)),
            Ok(true)
        );
        assert_eq!(
            MapUtils::contains_all_keys_array(Some(&map), Some(&missing)),
            Ok(false)
        );
        assert_eq!(
            MapUtils::contains_all_keys_array(Some(&map), Some(&[])),
            Ok(true)
        );
        let duplicate = [Some("one".to_owned()), Some("one".to_owned())];
        assert_eq!(
            MapUtils::contains_all_keys_collection(Some(&map), Some(duplicate.iter())),
            Ok(true)
        );
        let null_target = MapUtils::contains_all_keys_array(
            None::<&HashMap<Option<String>, Option<String>>>,
            None::<&[Option<String>]>,
        )
        .expect_err("target error");
        assert_eq!(
            null_target.get_message(),
            Some("Cannot execute map containsAllKeys: target is null")
        );
        let null_keys = MapUtils::contains_all_keys_array(Some(&map), None::<&[Option<String>]>)
            .expect_err("keys error");
        assert_eq!(
            null_keys.get_message(),
            Some("Cannot execute map containsAllKeys: keys is null")
        );
        assert!(
            MapUtils::contains_all_keys_collection(
                Some(&map),
                None::<std::slice::Iter<'_, Option<String>>>
            )
            .is_err()
        );
        assert!(
            MapUtils::contains_all_keys_collection(
                None::<&HashMap<Option<String>, Option<String>>>,
                None::<std::slice::Iter<'_, Option<String>>>
            )
            .is_err()
        );
    }

    #[test]
    fn finds_present_missing_and_null_values() {
        let map = map();
        assert_eq!(
            MapUtils::contains_value(Some(&map), &Some("value".to_owned())),
            Ok(true)
        );
        assert_eq!(
            MapUtils::contains_value(Some(&map), &Some("missing".to_owned())),
            Ok(false)
        );
        assert_eq!(MapUtils::contains_value(Some(&map), &None), Ok(true));
        let error = MapUtils::contains_value(
            None::<&HashMap<Option<String>, Option<String>>>,
            &Some("value".to_owned()),
        )
        .expect_err("error");
        assert_eq!(
            error.get_message(),
            Some("Cannot execute map containsValue: target is null")
        );
    }

    #[test]
    fn validates_and_checks_all_value_overloads_without_counting_duplicates() {
        let map = map();
        let present = [Some("value".to_owned()), None];
        let missing = [Some("value".to_owned()), Some("missing".to_owned())];
        assert_eq!(
            MapUtils::contains_all_values_array(Some(&map), Some(&present)),
            Ok(true)
        );
        assert_eq!(
            MapUtils::contains_all_values_array(Some(&map), Some(&missing)),
            Ok(false)
        );
        assert_eq!(
            MapUtils::contains_all_values_array(Some(&map), Some(&[] as &[Option<String>]),),
            Ok(true)
        );
        let duplicate = [Some("value".to_owned()), Some("value".to_owned())];
        assert_eq!(
            MapUtils::contains_all_values_collection(Some(&map), Some(duplicate.iter())),
            Ok(true)
        );
        let null_target = MapUtils::contains_all_values_array(
            None::<&HashMap<Option<String>, Option<String>>>,
            None::<&[Option<String>]>,
        )
        .expect_err("target error");
        assert_eq!(
            null_target.get_message(),
            Some("Cannot execute map containsAllValues: target is null")
        );
        let null_values =
            MapUtils::contains_all_values_array(Some(&map), None::<&[Option<String>]>)
                .expect_err("values error");
        assert_eq!(
            null_values.get_message(),
            Some("Cannot execute map containsAllValues: values is null")
        );
        assert!(
            MapUtils::contains_all_values_collection(
                Some(&map),
                None::<std::slice::Iter<'_, Option<String>>>
            )
            .is_err()
        );
        assert!(
            MapUtils::contains_all_values_collection(
                None::<&HashMap<Option<String>, Option<String>>>,
                None::<std::slice::Iter<'_, Option<String>>>
            )
            .is_err()
        );
    }
}
