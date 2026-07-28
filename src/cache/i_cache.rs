use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;

use super::ICacheEntryValidityChecker;

/// Thymeleaf 模板引擎使用的通用缓存契约。
///
/// 对应 Java: `org.thymeleaf.cache.ICache`。
///
/// Java 泛型缓存保存对象引用并以 `null` 表示未命中。Rust 使用 `Arc<V>` 保存和返回
/// 同一共享值身份，使用 `Option` 区分命中与未命中。实现必须支持整库清理、单键清理和
/// key 快照；key 快照可以包含尚未被惰性清除的失效条目。`V: ?Sized` 允许表达式缓存
/// 使用 `dyn Any + Send + Sync` 保存 Java `Object` 对应的异构制品。
pub trait ICache<K, V: ?Sized>: Send + Sync
where
    K: Clone + Eq + Hash + Send + Sync,
    V: Send + Sync,
{
    /// 向缓存写入或替换一个条目。
    ///
    /// 对应 Java: `ICache#put(Object, Object)`。
    ///
    /// # 参数
    /// - `key`：新条目的键；
    /// - `value`：要缓存的共享值，后续命中返回同一 `Arc` 身份。
    fn put(&self, key: K, value: Arc<V>);

    /// 按键读取缓存值。
    ///
    /// 对应 Java: `ICache#get(Object)`。
    ///
    /// # 参数
    /// - `key`：要读取的条目键。
    ///
    /// # 返回
    /// 命中时返回共享值；不存在或默认有效性检查判定失效时返回 `None`。
    fn get(&self, key: &K) -> Option<Arc<V>>;

    /// 使用本次调用指定的有效性检查器读取缓存值。
    ///
    /// 对应 Java:
    /// `ICache#get(Object, ICacheEntryValidityChecker)`。
    ///
    /// 本检查器必须覆盖缓存实现的默认检查器。条目失效时，实现必须删除该条目并返回
    /// `None`。
    ///
    /// # 参数
    /// - `key`：要读取的条目键；
    /// - `validity_checker`：本次读取专用的有效性检查器。
    ///
    /// # 返回
    /// 条目存在且检查通过时返回共享值，否则返回 `None`。
    fn get_with_validity_checker(
        &self,
        key: &K,
        validity_checker: &dyn ICacheEntryValidityChecker<K, V>,
    ) -> Option<Arc<V>>;

    /// 清除当前缓存的全部条目。
    ///
    /// 对应 Java: `ICache#clear()`。
    fn clear(&self);

    /// 清除指定键对应的单个条目。
    ///
    /// 对应 Java: `ICache#clearKey(Object)`。
    ///
    /// # 参数
    /// - `key`：要删除的条目键；不存在时保持幂等。
    fn clear_key(&self, key: &K);

    /// 返回当前缓存键的快照集合。
    ///
    /// 对应 Java: `ICache#keySet()`。
    ///
    /// # 返回
    /// 完整键集合；根据上游合同，其中可能包含已失效但尚未惰性清理的条目。
    fn key_set(&self) -> HashSet<K>;
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    use super::ICache;
    use crate::cache::ICacheEntryValidityChecker;

    struct Entry<V> {
        value: Arc<V>,
        creation_timestamp: i64,
    }

    #[derive(Default)]
    struct ContractCache {
        entries: Mutex<HashMap<String, Entry<String>>>,
    }

    impl ICache<String, String> for ContractCache {
        fn put(&self, key: String, value: Arc<String>) {
            self.entries.lock().expect("cache lock").insert(
                key,
                Entry {
                    value,
                    creation_timestamp: 7,
                },
            );
        }

        fn get(&self, key: &String) -> Option<Arc<String>> {
            self.entries
                .lock()
                .expect("cache lock")
                .get(key)
                .map(|entry| Arc::clone(&entry.value))
        }

        fn get_with_validity_checker(
            &self,
            key: &String,
            validity_checker: &dyn ICacheEntryValidityChecker<String, String>,
        ) -> Option<Arc<String>> {
            let mut entries = self.entries.lock().expect("cache lock");
            let is_valid = entries.get(key).is_some_and(|entry| {
                validity_checker.check_is_value_still_valid(
                    key,
                    &entry.value,
                    entry.creation_timestamp,
                )
            });
            if !is_valid {
                entries.remove(key);
                return None;
            }
            entries.get(key).map(|entry| Arc::clone(&entry.value))
        }

        fn clear(&self) {
            self.entries.lock().expect("cache lock").clear();
        }

        fn clear_key(&self, key: &String) {
            self.entries.lock().expect("cache lock").remove(key);
        }

        fn key_set(&self) -> HashSet<String> {
            self.entries
                .lock()
                .expect("cache lock")
                .keys()
                .cloned()
                .collect()
        }
    }

    struct Checker {
        expected_timestamp: i64,
        valid: bool,
    }

    impl ICacheEntryValidityChecker<String, String> for Checker {
        fn check_is_value_still_valid(
            &self,
            key: &String,
            value: &String,
            entry_creation_timestamp: i64,
        ) -> bool {
            key == "key"
                && value == "value"
                && entry_creation_timestamp == self.expected_timestamp
                && self.valid
        }
    }

    #[test]
    fn preserves_shared_value_identity_and_all_cache_operations() {
        let cache: &dyn ICache<String, String> = &ContractCache::default();
        let key = "key".to_owned();
        let value = Arc::new("value".to_owned());

        assert!(cache.get(&key).is_none());
        cache.put(key.clone(), Arc::clone(&value));
        let found = cache.get(&key).expect("cache hit");
        assert!(Arc::ptr_eq(&found, &value));
        assert_eq!(cache.key_set(), HashSet::from([key.clone()]));

        cache.clear_key(&"missing".to_owned());
        assert!(cache.get(&key).is_some());
        cache.clear_key(&key);
        assert!(cache.get(&key).is_none());

        cache.put(key.clone(), value);
        cache.put("second".to_owned(), Arc::new("two".to_owned()));
        cache.clear();
        assert!(cache.key_set().is_empty());
    }

    #[test]
    fn explicit_checker_overrides_read_and_removes_invalid_entries() {
        let cache = ContractCache::default();
        let key = "key".to_owned();
        let value = Arc::new("value".to_owned());
        cache.put(key.clone(), Arc::clone(&value));

        let valid = Checker {
            expected_timestamp: 7,
            valid: true,
        };
        let found = cache
            .get_with_validity_checker(&key, &valid)
            .expect("valid entry");
        assert!(Arc::ptr_eq(&found, &value));

        let invalid = Checker {
            expected_timestamp: 7,
            valid: false,
        };
        assert!(cache.get_with_validity_checker(&key, &invalid).is_none());
        assert!(!cache.key_set().contains(&key));
        assert!(
            cache
                .get_with_validity_checker(&"missing".to_owned(), &valid)
                .is_none()
        );
    }
}
