use std::time::{SystemTime, UNIX_EPOCH};

use super::ICacheEntryValidity;

/// 按毫秒 TTL 判断模板缓存条目是否仍然有效的策略。
///
/// 对应 Java: `org.thymeleaf.cache.TTLCacheEntryValidity`。
///
/// 对象在构造时记录 `System.currentTimeMillis()` 等价的 Unix 墙上时钟毫秒值。
/// 每次检查都判断当前值是否严格小于“创建时间 + TTL”。为保留 Java `long`
/// 运算，本实现使用有符号 64 位环绕加法；因此负数、零和溢出 TTL 的边界行为与
/// 上游一致，而不是改用饱和运算或单调时钟。
pub struct TTLCacheEntryValidity {
    cache_ttl_ms: i64,
    creation_time_in_millis: i64,
}

impl TTLCacheEntryValidity {
    /// 创建使用指定毫秒 TTL 的有效性策略。
    ///
    /// 对应 Java: `TTLCacheEntryValidity#TTLCacheEntryValidity(long)`。
    ///
    /// # 参数
    /// - `cache_ttl_ms`：Java 参数 `cacheTTLMs`；不进行正数校验，完整接受
    ///   Java `long` 范围。
    ///
    /// # 返回
    /// 创建时间已记录为当前墙上时钟毫秒值的策略。
    #[must_use]
    pub fn new(cache_ttl_ms: i64) -> Self {
        Self::new_at(cache_ttl_ms, current_time_millis())
    }

    fn new_at(cache_ttl_ms: i64, creation_time_in_millis: i64) -> Self {
        Self {
            cache_ttl_ms,
            creation_time_in_millis,
        }
    }

    /// 返回构造时指定的毫秒 TTL。
    ///
    /// 对应 Java: `TTLCacheEntryValidity#getCacheTTLMs()`。
    ///
    /// # 返回
    /// 未经归一化或校验的原始 Java `long` 等价值。
    #[must_use]
    pub const fn get_cache_ttl_ms(&self) -> i64 {
        self.cache_ttl_ms
    }

    /// TTL 策略始终允许模板解析结果进入缓存。
    ///
    /// 对应 Java: `TTLCacheEntryValidity#isCacheable()`。
    ///
    /// # 返回
    /// 始终返回 `true`；即使 TTL 为零或负数，条目也可以先进入缓存，然后在检查时
    /// 立即失效。
    #[must_use]
    pub const fn is_cacheable(&self) -> bool {
        true
    }

    /// 按当前墙上时钟判断缓存条目是否仍在 TTL 内。
    ///
    /// 对应 Java: `TTLCacheEntryValidity#isCacheStillValid()`。
    ///
    /// # 返回
    /// 当前毫秒值严格小于创建时间与 TTL 的 Java 环绕和时返回 `true`。
    #[must_use]
    pub fn is_cache_still_valid(&self) -> bool {
        self.is_cache_still_valid_at(current_time_millis())
    }

    fn is_cache_still_valid_at(&self, current_time_in_millis: i64) -> bool {
        current_time_in_millis < self.creation_time_in_millis.wrapping_add(self.cache_ttl_ms)
    }
}

impl ICacheEntryValidity for TTLCacheEntryValidity {
    fn is_cacheable(&self) -> bool {
        Self::is_cacheable(self)
    }

    fn is_cache_still_valid(&self) -> bool {
        Self::is_cache_still_valid(self)
    }
}

fn current_time_millis() -> i64 {
    system_time_millis(SystemTime::now())
}

fn system_time_millis(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as i64,
        Err(error) => {
            let duration = error.duration();
            let whole_millis = duration.as_millis() as i64;
            let sub_millisecond = duration.subsec_nanos() % 1_000_000 != 0;
            whole_millis
                .wrapping_neg()
                .wrapping_sub(i64::from(sub_millisecond))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{TTLCacheEntryValidity, UNIX_EPOCH, system_time_millis};
    use crate::cache::ICacheEntryValidity;

    #[test]
    fn preserves_ttl_and_dynamic_cacheable_contract() {
        let validity = TTLCacheEntryValidity::new(60_000);
        let dynamic: &dyn ICacheEntryValidity = &validity;

        assert_eq!(validity.get_cache_ttl_ms(), 60_000);
        assert!(validity.is_cacheable());
        assert!(validity.is_cache_still_valid());
        assert!(dynamic.is_cacheable());
        assert!(dynamic.is_cache_still_valid());
    }

    #[test]
    fn uses_strict_wall_clock_boundary_and_preserves_clock_rollback() {
        let validity = TTLCacheEntryValidity::new_at(10, 1_000);

        assert!(validity.is_cache_still_valid_at(999));
        assert!(validity.is_cache_still_valid_at(1_009));
        assert!(!validity.is_cache_still_valid_at(1_010));
        assert!(!validity.is_cache_still_valid_at(1_011));
    }

    #[test]
    fn preserves_java_long_overflow_and_non_positive_ttl_behavior() {
        let zero = TTLCacheEntryValidity::new_at(0, 100);
        let negative = TTLCacheEntryValidity::new_at(-1, 100);
        let overflowing = TTLCacheEntryValidity::new_at(i64::MAX, 100);

        assert!(!zero.is_cache_still_valid_at(100));
        assert!(!negative.is_cache_still_valid_at(100));
        assert!(!overflowing.is_cache_still_valid_at(100));
    }

    #[test]
    fn converts_unix_wall_clock_milliseconds_on_both_sides_of_epoch() {
        assert_eq!(system_time_millis(UNIX_EPOCH), 0);
        assert_eq!(
            system_time_millis(UNIX_EPOCH + Duration::from_millis(1_234)),
            1_234
        );
        assert_eq!(
            system_time_millis(UNIX_EPOCH - Duration::from_millis(1_234)),
            -1_234
        );
        assert_eq!(
            system_time_millis(UNIX_EPOCH - Duration::from_micros(500)),
            -1
        );
    }
}
