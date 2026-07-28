use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{ICache, ICacheEntryValidityChecker};

const REPORT_INTERVAL_MILLIS: i64 = 300_000;

/// 标准缓存构造失败。
///
/// 对应 Java 构造阶段由 `org.thymeleaf.util.Validate` 抛出的
/// `IllegalArgumentException`。Rust 使用类型化错误保留原始消息。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardCacheError {
    message: &'static str,
}

impl StandardCacheError {
    fn new(message: &'static str) -> Self {
        Self { message }
    }
}

impl Display for StandardCacheError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}

impl Error for StandardCacheError {}

/// Thymeleaf 默认的并发内存缓存。
///
/// 对应 Java: `org.thymeleaf.cache.StandardCache`。
///
/// 缓存保留 Java 实现的 put-if-absent、插入顺序 FIFO 容量限制、惰性有效性检查、
/// 可选计数器和共享值身份。Java `SoftReference` 依赖 JVM 垃圾收集器的内存压力
/// 策略；Rust 没有等价 GC 原语，因此软引用条目默认保持强引用，并可通过
/// [`Self::sacrifice_soft_references`] 显式模拟 JVM 回收。
pub struct StandardCache<K, V: ?Sized>
where
    K: Clone + Eq + Hash + Send + Sync,
    V: Send + Sync,
{
    name: String,
    use_soft_references: bool,
    max_size: i32,
    entry_validity_checker: Option<Arc<dyn ICacheEntryValidityChecker<K, V>>>,
    trace_execution: bool,
    enable_counters: bool,
    data_container: Mutex<CacheDataContainer<K, V>>,
    last_execution: AtomicI64,
    get_count: AtomicI64,
    put_count: AtomicI64,
    hit_count: AtomicI64,
    miss_count: AtomicI64,
}

impl<K, V: ?Sized> StandardCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync,
    V: Send + Sync,
{
    /// 创建无容量上限、无默认有效性检查器且不启用计数器的缓存。
    ///
    /// 对应 Java:
    /// `StandardCache(String, boolean, int, Logger)`；Rust 日志由全局 `tracing`
    /// subscriber 管理，本便捷构造器默认关闭逐操作 trace。
    ///
    /// # 参数
    /// - `name`：非空缓存名称；
    /// - `use_soft_references`：是否声明使用软引用策略；
    /// - `initial_capacity`：必须大于零的初始容量。
    pub fn new(
        name: Option<&str>,
        use_soft_references: bool,
        initial_capacity: i32,
    ) -> Result<Self, StandardCacheError> {
        Self::with_options(
            name,
            use_soft_references,
            initial_capacity,
            -1,
            None,
            false,
            false,
        )
    }

    /// 创建无容量上限并带默认条目有效性检查器的缓存。
    ///
    /// 对应 Java:
    /// `StandardCache(String, boolean, int, ICacheEntryValidityChecker, Logger)`。
    pub fn with_validity_checker(
        name: Option<&str>,
        use_soft_references: bool,
        initial_capacity: i32,
        entry_validity_checker: Option<Arc<dyn ICacheEntryValidityChecker<K, V>>>,
    ) -> Result<Self, StandardCacheError> {
        Self::with_options(
            name,
            use_soft_references,
            initial_capacity,
            -1,
            entry_validity_checker,
            false,
            false,
        )
    }

    /// 创建带 FIFO 最大容量、无默认有效性检查器的缓存。
    ///
    /// 对应 Java: `StandardCache(String, boolean, int, int, Logger)`。
    pub fn with_max_size(
        name: Option<&str>,
        use_soft_references: bool,
        initial_capacity: i32,
        max_size: i32,
    ) -> Result<Self, StandardCacheError> {
        Self::with_options(
            name,
            use_soft_references,
            initial_capacity,
            max_size,
            None,
            false,
            false,
        )
    }

    /// 创建带 FIFO 最大容量和默认条目有效性检查器的缓存。
    ///
    /// 对应 Java:
    /// `StandardCache(String, boolean, int, int, ICacheEntryValidityChecker, Logger)`。
    pub fn with_max_size_and_validity_checker(
        name: Option<&str>,
        use_soft_references: bool,
        initial_capacity: i32,
        max_size: i32,
        entry_validity_checker: Option<Arc<dyn ICacheEntryValidityChecker<K, V>>>,
    ) -> Result<Self, StandardCacheError> {
        Self::with_options(
            name,
            use_soft_references,
            initial_capacity,
            max_size,
            entry_validity_checker,
            false,
            false,
        )
    }

    /// 使用完整选项创建缓存。
    ///
    /// 对应 Java:
    /// `StandardCache(String, boolean, int, int, ICacheEntryValidityChecker, Logger, boolean)`。
    ///
    /// `trace_execution` 对应非空且启用 trace 的 SLF4J logger。与 Java 一致，开启
    /// trace 会强制开启计数器。
    #[allow(clippy::too_many_arguments)]
    pub fn with_options(
        name: Option<&str>,
        use_soft_references: bool,
        initial_capacity: i32,
        max_size: i32,
        entry_validity_checker: Option<Arc<dyn ICacheEntryValidityChecker<K, V>>>,
        enable_counters: bool,
        trace_execution: bool,
    ) -> Result<Self, StandardCacheError> {
        let name = name
            .filter(|value| !is_java_empty_or_whitespace(value))
            .ok_or_else(|| StandardCacheError::new("Name cannot be null or empty"))?;
        if initial_capacity <= 0 {
            return Err(StandardCacheError::new("Initial capacity must be > 0"));
        }
        if max_size == 0 {
            return Err(StandardCacheError::new(
                "Cache max size must be either -1 (no limit) or > 0",
            ));
        }

        if trace_execution {
            tracing::trace!(
                cache_name = name,
                max_size,
                use_soft_references,
                "Initializing Thymeleaf cache"
            );
        }

        Ok(Self {
            name: name.to_owned(),
            use_soft_references,
            max_size,
            entry_validity_checker,
            trace_execution,
            enable_counters: trace_execution || enable_counters,
            data_container: Mutex::new(CacheDataContainer::new(
                initial_capacity as usize,
                max_size,
            )),
            last_execution: AtomicI64::new(current_time_millis()),
            get_count: AtomicI64::new(0),
            put_count: AtomicI64::new(0),
            hit_count: AtomicI64::new(0),
            miss_count: AtomicI64::new(0),
        })
    }

    /// 返回缓存名称。
    ///
    /// 对应 Java: `StandardCache#getName()`。
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// 返回缓存是否具有正数最大容量。
    ///
    /// 对应 Java: `StandardCache#hasMaxSize()`。
    pub fn has_max_size(&self) -> bool {
        self.max_size > 0
    }

    /// 返回构造时指定的最大容量原值。
    ///
    /// 对应 Java: `StandardCache#getMaxSize()`。包括 Java 实现接受的任意负数。
    pub fn get_max_size(&self) -> i32 {
        self.max_size
    }

    /// 返回是否声明使用软引用。
    ///
    /// 对应 Java: `StandardCache#getUseSoftReferences()`。
    pub fn get_use_soft_references(&self) -> bool {
        self.use_soft_references
    }

    /// 返回当前条目数。
    ///
    /// 对应 Java: `StandardCache#size()`。
    pub fn size(&self) -> usize {
        self.lock_data().size()
    }

    /// 返回 put 调用计数。
    ///
    /// 对应 Java: `StandardCache#getPutCount()`。
    pub fn get_put_count(&self) -> i64 {
        self.put_count.load(Ordering::Relaxed)
    }

    /// 返回 get 调用计数。
    ///
    /// 对应 Java: `StandardCache#getGetCount()`。
    pub fn get_get_count(&self) -> i64 {
        self.get_count.load(Ordering::Relaxed)
    }

    /// 返回命中计数。
    ///
    /// 对应 Java: `StandardCache#getHitCount()`。
    pub fn get_hit_count(&self) -> i64 {
        self.hit_count.load(Ordering::Relaxed)
    }

    /// 返回未命中计数。
    ///
    /// 对应 Java: `StandardCache#getMissCount()`。
    pub fn get_miss_count(&self) -> i64 {
        self.miss_count.load(Ordering::Relaxed)
    }

    /// 返回命中次数占 get 次数的比例。
    ///
    /// 对应 Java: `StandardCache#getHitRatio()`。命中数或 get 数为零时返回 `0.0`。
    pub fn get_hit_ratio(&self) -> f64 {
        let hit_count = self.get_hit_count();
        let get_count = self.get_get_count();
        if hit_count == 0 || get_count == 0 {
            return 0.0;
        }
        hit_count as f64 / get_count as f64
    }

    /// 返回 `1 - hit_ratio`。
    ///
    /// 对应 Java: `StandardCache#getMissRatio()`。因此尚未读取的缓存返回 `1.0`。
    pub fn get_miss_ratio(&self) -> f64 {
        1.0 - self.get_hit_ratio()
    }

    /// 显式牺牲全部软引用条目的强锚点。
    ///
    /// Java 的 `SoftReference` 可在 JVM 内存压力下被 GC 清除，Rust 没有等价的自动
    /// 回收通知。本扩展方法提供确定性触发点：仅软引用模式受影响；仍被外部 `Arc`
    /// 持有的值保持可用，其余条目会在下次 get 时惰性删除。
    pub fn sacrifice_soft_references(&self) {
        if !self.use_soft_references {
            return;
        }
        self.lock_data().sacrifice_soft_references();
    }

    fn get_with_checker(
        &self,
        key: &K,
        validity_checker: Option<&dyn ICacheEntryValidityChecker<K, V>>,
    ) -> Option<Arc<V>> {
        self.increment_report_entity(&self.get_count);
        let mut data = self.lock_data();
        let result = data
            .get(key)
            .and_then(|entry| entry.get_value_if_still_valid(key, validity_checker));

        if let Some(value) = result {
            self.increment_report_entity(&self.hit_count);
            drop(data);
            self.output_report_if_needed();
            return Some(value);
        }

        data.remove(key);
        self.increment_report_entity(&self.miss_count);
        drop(data);
        self.output_report_if_needed();
        None
    }

    fn increment_report_entity(&self, entity: &AtomicI64) {
        if self.enable_counters {
            entity.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn output_report_if_needed(&self) {
        if !self.trace_execution {
            return;
        }
        let current_time = current_time_millis();
        let last_execution = self.last_execution.load(Ordering::Relaxed);
        if current_time.wrapping_sub(last_execution) < REPORT_INTERVAL_MILLIS {
            return;
        }
        self.try_output_report(current_time, last_execution);
    }

    fn try_output_report(&self, current_time: i64, last_execution: i64) {
        if self
            .last_execution
            .compare_exchange(
                last_execution,
                current_time,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            tracing::trace!(
                cache_name = self.name,
                size = self.size(),
                put_count = self.get_put_count(),
                get_count = self.get_get_count(),
                hit_count = self.get_hit_count(),
                miss_count = self.get_miss_count(),
                hit_ratio = self.get_hit_ratio(),
                miss_ratio = self.get_miss_ratio(),
                "Thymeleaf cache report"
            );
        }
    }

    fn lock_data(&self) -> std::sync::MutexGuard<'_, CacheDataContainer<K, V>> {
        self.data_container
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl<K, V: ?Sized> ICache<K, V> for StandardCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync,
    V: Send + Sync,
{
    fn put(&self, key: K, value: Arc<V>) {
        self.increment_report_entity(&self.put_count);
        let entry = CacheEntry::new(value, self.use_soft_references);
        self.lock_data().put(key, entry);
        self.output_report_if_needed();
    }

    fn get(&self, key: &K) -> Option<Arc<V>> {
        self.get_with_checker(key, self.entry_validity_checker.as_deref())
    }

    fn get_with_validity_checker(
        &self,
        key: &K,
        validity_checker: &dyn ICacheEntryValidityChecker<K, V>,
    ) -> Option<Arc<V>> {
        self.get_with_checker(key, Some(validity_checker))
    }

    fn clear(&self) {
        self.lock_data().clear();
        if self.trace_execution {
            tracing::trace!(
                cache_name = self.name,
                "Cleared all Thymeleaf cache entries"
            );
        }
    }

    fn clear_key(&self, key: &K) {
        let removed = self.lock_data().remove(key);
        if self.trace_execution && removed {
            tracing::trace!(
                cache_name = self.name,
                size = self.size(),
                "Cleared Thymeleaf cache key"
            );
        }
    }

    fn key_set(&self) -> HashSet<K> {
        self.lock_data().key_set()
    }
}

/// 标准缓存内部的数据容器。
///
/// 对应 Java: `org.thymeleaf.cache.StandardCache.CacheDataContainer`。
struct CacheDataContainer<K, V: ?Sized> {
    container: HashMap<K, CacheEntry<V>>,
    fifo: Option<Vec<Option<K>>>,
    fifo_pointer: usize,
}

impl<K, V: ?Sized> CacheDataContainer<K, V>
where
    K: Clone + Eq + Hash,
{
    fn new(initial_capacity: usize, max_size: i32) -> Self {
        let fifo = (max_size >= 0).then(|| vec![None; max_size as usize]);
        Self {
            container: HashMap::with_capacity(initial_capacity),
            fifo,
            fifo_pointer: 0,
        }
    }

    fn get(&self, key: &K) -> Option<&CacheEntry<V>> {
        // 与 Java 一致，读取不调整 FIFO，容量策略不是 LRU。
        self.container.get(key)
    }

    fn key_set(&self) -> HashSet<K> {
        self.container.keys().cloned().collect()
    }

    fn put(&mut self, key: K, value: CacheEntry<V>) {
        if let Entry::Occupied(_) = self.container.entry(key.clone()) {
            return;
        }
        self.container.insert(key.clone(), value);

        if let Some(fifo) = self.fifo.as_mut() {
            if let Some(removed_key) = fifo[self.fifo_pointer].take() {
                self.container.remove(&removed_key);
            }
            fifo[self.fifo_pointer] = Some(key);
            self.fifo_pointer = (self.fifo_pointer + 1) % fifo.len();
        }
    }

    fn remove(&mut self, key: &K) -> bool {
        if self.container.remove(key).is_none() {
            return false;
        }
        if let Some(fifo) = self.fifo.as_mut() {
            if let Some(position) = fifo
                .iter()
                .position(|candidate| candidate.as_ref() == Some(key))
            {
                fifo[position] = None;
            }
        }
        true
    }

    fn clear(&mut self) {
        // Java 只清理 map，不重置 FIFO 数组或指针。
        self.container.clear();
    }

    fn size(&self) -> usize {
        self.container.len()
    }

    fn sacrifice_soft_references(&mut self) {
        for entry in self.container.values_mut() {
            entry.sacrifice_soft_reference();
        }
    }
}

/// 标准缓存内部的单个缓存条目。
///
/// 对应 Java: `org.thymeleaf.cache.StandardCache.CacheEntry`。
struct CacheEntry<V: ?Sized> {
    cached_value_reference: Weak<V>,
    cached_value_anchor: Option<Arc<V>>,
    creation_time_in_millis: i64,
    soft_reference: bool,
}

impl<V: ?Sized> CacheEntry<V> {
    fn new(cached_value: Arc<V>, use_soft_references: bool) -> Self {
        Self {
            cached_value_reference: Arc::downgrade(&cached_value),
            cached_value_anchor: Some(cached_value),
            creation_time_in_millis: current_time_millis(),
            soft_reference: use_soft_references,
        }
    }

    fn get_value_if_still_valid<K>(
        &self,
        key: &K,
        checker: Option<&dyn ICacheEntryValidityChecker<K, V>>,
    ) -> Option<Arc<V>> {
        let cached_value = self.cached_value_reference.upgrade()?;
        if checker.is_none_or(|checker| {
            checker.check_is_value_still_valid(
                key,
                cached_value.as_ref(),
                self.creation_time_in_millis,
            )
        }) {
            return Some(cached_value);
        }
        None
    }

    fn sacrifice_soft_reference(&mut self) {
        if self.soft_reference {
            self.cached_value_anchor = None;
        }
    }

    #[cfg(test)]
    fn get_creation_time_in_millis(&self) -> i64 {
        self.creation_time_in_millis
    }
}

fn current_time_millis() -> i64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    i64::try_from(millis).unwrap_or(i64::MAX)
}

fn is_java_empty_or_whitespace(value: &str) -> bool {
    value.is_empty()
        || value.chars().all(|character| {
            matches!(
                character,
                '\u{0009}'..='\u{000D}'
                    | '\u{001C}'..='\u{0020}'
                    | '\u{1680}'
                    | '\u{2000}'..='\u{2006}'
                    | '\u{2008}'..='\u{200A}'
                    | '\u{2028}'
                    | '\u{2029}'
                    | '\u{205F}'
                    | '\u{3000}'
            )
        })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use tracing::Level;

    use super::{CacheEntry, StandardCache, StandardCacheError, current_time_millis};
    use crate::cache::{ICache, ICacheEntryValidityChecker};

    struct TimestampChecker {
        valid: bool,
    }

    impl ICacheEntryValidityChecker<String, String> for TimestampChecker {
        fn check_is_value_still_valid(
            &self,
            key: &String,
            value: &String,
            entry_creation_timestamp: i64,
        ) -> bool {
            self.valid && key == "key" && value == "value" && entry_creation_timestamp > 0
        }
    }

    #[test]
    fn validates_constructor_arguments_in_java_order() {
        assert_error(
            StandardCache::<String, String>::with_options(None, false, 0, 0, None, false, false),
            "Name cannot be null or empty",
        );
        assert_error(
            StandardCache::<String, String>::new(Some(""), false, 1),
            "Name cannot be null or empty",
        );
        assert_error(
            StandardCache::<String, String>::new(Some("cache"), false, 0),
            "Initial capacity must be > 0",
        );
        assert_error(
            StandardCache::<String, String>::with_max_size(Some("cache"), false, 1, 0),
            "Cache max size must be either -1 (no limit) or > 0",
        );
        assert_error(
            StandardCache::<String, String>::new(Some("\u{2003}"), false, 1),
            "Name cannot be null or empty",
        );
        assert!(StandardCache::<String, String>::new(Some("\u{00A0}"), false, 1).is_ok());
    }

    #[test]
    fn exposes_configuration_and_accepts_any_negative_max_size() {
        let cache = StandardCache::<String, String>::with_max_size(Some("cache"), true, 2, -2)
            .expect("cache");
        assert_eq!(cache.get_name(), "cache");
        assert!(cache.get_use_soft_references());
        assert!(!cache.has_max_size());
        assert_eq!(cache.get_max_size(), -2);
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.get_hit_ratio(), 0.0);
        assert_eq!(cache.get_miss_ratio(), 1.0);
    }

    #[test]
    fn put_is_put_if_absent_and_fifo_is_not_lru() {
        let cache = StandardCache::<String, String>::with_max_size(Some("cache"), false, 2, 2)
            .expect("cache");
        cache.put("a".to_owned(), Arc::new("one".to_owned()));
        cache.put("a".to_owned(), Arc::new("replacement".to_owned()));
        cache.put("b".to_owned(), Arc::new("two".to_owned()));
        assert_eq!(
            cache.get(&"a".to_owned()).as_deref(),
            Some(&"one".to_owned())
        );

        // 读取 a 不改变 FIFO 顺序，因此插入 c 仍淘汰最早插入的 a。
        cache.put("c".to_owned(), Arc::new("three".to_owned()));
        assert!(cache.get(&"a".to_owned()).is_none());
        assert!(cache.get(&"b".to_owned()).is_some());
        assert!(cache.get(&"c".to_owned()).is_some());
    }

    #[test]
    fn default_and_explicit_checkers_remove_invalid_entries() {
        let default_checker: Arc<dyn ICacheEntryValidityChecker<String, String>> =
            Arc::new(TimestampChecker { valid: false });
        let cache =
            StandardCache::with_validity_checker(Some("cache"), false, 2, Some(default_checker))
                .expect("cache");
        let key = "key".to_owned();
        cache.put(key.clone(), Arc::new("value".to_owned()));

        let valid = TimestampChecker { valid: true };
        assert!(cache.get_with_validity_checker(&key, &valid).is_some());
        assert!(cache.get(&key).is_none());
        assert!(!cache.key_set().contains(&key));
    }

    #[test]
    fn counts_every_call_only_when_enabled_and_clear_does_not_reset_counts() {
        let cache = StandardCache::<String, String>::with_options(
            Some("cache"),
            false,
            2,
            -1,
            None,
            true,
            false,
        )
        .expect("cache");
        let key = "key".to_owned();
        cache.put(key.clone(), Arc::new("value".to_owned()));
        cache.put(key.clone(), Arc::new("ignored".to_owned()));
        assert!(cache.get(&key).is_some());
        assert!(cache.get(&"missing".to_owned()).is_none());
        assert_eq!(cache.get_put_count(), 2);
        assert_eq!(cache.get_get_count(), 2);
        assert_eq!(cache.get_hit_count(), 1);
        assert_eq!(cache.get_miss_count(), 1);
        assert_eq!(cache.get_hit_ratio(), 0.5);
        assert_eq!(cache.get_miss_ratio(), 0.5);

        cache.clear();
        assert_eq!(cache.get_put_count(), 2);
        assert_eq!(cache.get_get_count(), 2);
        assert!(cache.key_set().is_empty());
    }

    #[test]
    fn disabled_counters_remain_zero_and_clear_key_is_idempotent() {
        let cache = StandardCache::<String, String>::new(Some("cache"), false, 1).expect("cache");
        let key = "key".to_owned();
        cache.put(key.clone(), Arc::new("value".to_owned()));
        assert!(cache.get(&key).is_some());
        cache.clear_key(&key);
        cache.clear_key(&key);
        assert_eq!(cache.get_put_count(), 0);
        assert_eq!(cache.get_get_count(), 0);
        assert_eq!(cache.get_hit_count(), 0);
        assert_eq!(cache.get_miss_count(), 0);
    }

    #[test]
    fn clear_preserves_java_fifo_pointer_behavior() {
        let cache = StandardCache::<String, String>::with_max_size(Some("cache"), false, 2, 2)
            .expect("cache");
        cache.put("a".to_owned(), Arc::new("one".to_owned()));
        cache.clear();
        cache.put("b".to_owned(), Arc::new("two".to_owned()));
        cache.put("c".to_owned(), Arc::new("three".to_owned()));
        assert_eq!(cache.size(), 2);
        assert!(cache.get(&"b".to_owned()).is_some());
        assert!(cache.get(&"c".to_owned()).is_some());
    }

    #[test]
    fn soft_reference_sacrifice_is_lazy_and_strong_mode_ignores_it() {
        let soft = StandardCache::<String, String>::new(Some("soft"), true, 1).expect("soft");
        let key = "key".to_owned();
        let external = Arc::new("value".to_owned());
        soft.put(key.clone(), Arc::clone(&external));
        soft.sacrifice_soft_references();
        assert!(soft.get(&key).is_some());
        drop(external);
        soft.sacrifice_soft_references();
        assert!(soft.get(&key).is_none());

        let strong =
            StandardCache::<String, String>::new(Some("strong"), false, 1).expect("strong");
        strong.put(key.clone(), Arc::new("value".to_owned()));
        strong.sacrifice_soft_references();
        assert!(strong.get(&key).is_some());
    }

    #[test]
    fn cache_entry_exposes_creation_timestamp_internally() {
        let mut entry = CacheEntry::new(Arc::new("value".to_owned()), false);
        entry.sacrifice_soft_reference();
        assert!(entry.get_creation_time_in_millis() > 0);
    }

    #[test]
    fn traces_initialization_operations_and_periodic_report() {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_writer(std::io::sink)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            let checker: Arc<dyn ICacheEntryValidityChecker<String, String>> =
                Arc::new(TimestampChecker { valid: true });
            let cache = StandardCache::with_max_size_and_validity_checker(
                Some("trace"),
                false,
                2,
                2,
                Some(checker),
            )
            .expect("cache");
            let cache =
                StandardCache::with_options(Some(cache.get_name()), false, 2, 2, None, false, true)
                    .expect("traced cache");
            cache.last_execution.store(0, Ordering::Relaxed);
            cache.put("key".to_owned(), Arc::new("value".to_owned()));
            cache.try_output_report(current_time_millis(), 0);
            cache.clear_key(&"key".to_owned());
            cache.clear_key(&"missing".to_owned());
            cache.put("other".to_owned(), Arc::new("value".to_owned()));
            cache.clear();
            assert_eq!(cache.get_put_count(), 2);
        });
    }

    #[test]
    fn removes_bounded_fifo_slot_and_covers_all_java_whitespace_ranges() {
        let cache = StandardCache::<String, String>::with_max_size(Some("cache"), false, 2, 2)
            .expect("cache");
        cache.put("a".to_owned(), Arc::new("one".to_owned()));
        cache.clear_key(&"a".to_owned());
        assert!(cache.key_set().is_empty());

        cache.put("b".to_owned(), Arc::new("two".to_owned()));
        cache
            .lock_data()
            .fifo
            .as_mut()
            .expect("bounded FIFO")
            .fill(None);
        cache.clear_key(&"b".to_owned());
        assert!(cache.key_set().is_empty());

        assert_error(
            StandardCache::<String, String>::new(Some("\u{2008}"), false, 1),
            "Name cannot be null or empty",
        );
    }

    #[test]
    #[should_panic(expected = "expected error")]
    fn assertion_helper_rejects_an_unexpected_success() {
        assert_error(
            StandardCache::<String, String>::new(Some("cache"), false, 1),
            "unused",
        );
    }

    fn assert_error(
        result: Result<StandardCache<String, String>, StandardCacheError>,
        expected: &str,
    ) {
        match result {
            Ok(_) => panic!("expected error"),
            Err(error) => assert_eq!(error.to_string(), expected),
        }
    }
}
