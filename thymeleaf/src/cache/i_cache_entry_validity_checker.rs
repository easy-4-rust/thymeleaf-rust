/// 缓存条目读取前的可选有效性检查契约。
///
/// 对应 Java: `org.thymeleaf.cache.ICacheEntryValidityChecker`。
///
/// 缓存的 `get_with_validity_checker` 操作在返回命中值之前调用本接口。返回 `false`
/// 表示条目已经失效，缓存必须删除该条目并按未命中返回。传入的创建时间戳使用 Unix
/// 纪元毫秒，并保留 Java `long` 的 `i64` 数值范围。
///
/// Java `Serializable` 是 JVM 对象序列化标记；Rust 不把任意 trait object 隐式变成
/// 不安全的线格式，而以 `Send + Sync` 保证检查器可在线程间共享。需要持久化的具体实现
/// 应自行定义显式 serde 格式。
pub trait ICacheEntryValidityChecker<K, V: ?Sized>: Send + Sync {
    /// 检查指定缓存条目是否仍然有效。
    ///
    /// 对应 Java:
    /// `ICacheEntryValidityChecker#checkIsValueStillValid(Object, Object, long)`。
    ///
    /// # 参数
    /// - `key`：当前缓存条目的键；
    /// - `value`：当前缓存条目的共享值；
    /// - `entry_creation_timestamp`：条目创建时的 Unix 纪元毫秒。
    ///
    /// # 返回
    /// 条目可以继续返回时为 `true`；返回 `false` 时缓存必须删除该条目。
    fn check_is_value_still_valid(&self, key: &K, value: &V, entry_creation_timestamp: i64)
    -> bool;
}

#[cfg(test)]
mod tests {
    use super::ICacheEntryValidityChecker;

    struct BoundaryChecker;

    impl ICacheEntryValidityChecker<String, String> for BoundaryChecker {
        fn check_is_value_still_valid(
            &self,
            key: &String,
            value: &String,
            entry_creation_timestamp: i64,
        ) -> bool {
            key == "key" && value == "value" && entry_creation_timestamp == i64::MIN
        }
    }

    #[test]
    fn supports_dynamic_checkers_and_preserves_all_arguments() {
        let checker: &dyn ICacheEntryValidityChecker<String, String> = &BoundaryChecker;

        assert!(checker.check_is_value_still_valid(
            &"key".to_owned(),
            &"value".to_owned(),
            i64::MIN
        ));
        assert!(!checker.check_is_value_still_valid(
            &"other".to_owned(),
            &"value".to_owned(),
            i64::MAX
        ));
    }
}
