use super::ICacheEntryValidity;

static ALWAYS_VALID_INSTANCE: AlwaysValidCacheEntryValidity =
    AlwaysValidCacheEntryValidity { _identity: 0 };

/// 始终允许缓存且永不主动过期的有效性策略。
///
/// 对应 Java: `org.thymeleaf.cache.AlwaysValidCacheEntryValidity`。
///
/// 使用本策略的模板解析结果只会因缓存的 LRU 淘汰策略被移除，不会因有效性检查
/// 失效。`INSTANCE` 用于避免创建大量等价对象，同时仍保留 Java 公开构造器。
pub struct AlwaysValidCacheEntryValidity {
    // 非零大小确保独立构造值具有可观察的不同地址，保留 Java Object 身份语义。
    _identity: u8,
}

#[allow(clippy::new_without_default)]
impl AlwaysValidCacheEntryValidity {
    /// 可共享的单例实例。
    ///
    /// 对应 Java: `AlwaysValidCacheEntryValidity.INSTANCE`。
    pub const INSTANCE: &'static Self = &ALWAYS_VALID_INSTANCE;

    /// 创建一个新的始终有效策略对象。
    ///
    /// 对应 Java:
    /// `AlwaysValidCacheEntryValidity#AlwaysValidCacheEntryValidity()`。
    ///
    /// # 返回
    /// 与单例行为相同、但具有独立对象身份的新策略。
    #[must_use]
    pub const fn new() -> Self {
        Self { _identity: 0 }
    }

    /// 始终允许模板解析结果进入缓存。
    ///
    /// 对应 Java: `AlwaysValidCacheEntryValidity#isCacheable()`。
    ///
    /// # 返回
    /// 始终返回 `true`。
    #[must_use]
    pub const fn is_cacheable(&self) -> bool {
        true
    }

    /// 始终认为已有缓存条目仍然有效。
    ///
    /// 对应 Java: `AlwaysValidCacheEntryValidity#isCacheStillValid()`。
    ///
    /// # 返回
    /// 始终返回 `true`；条目只能由 LRU 等外部策略淘汰。
    #[must_use]
    pub const fn is_cache_still_valid(&self) -> bool {
        true
    }
}

impl ICacheEntryValidity for AlwaysValidCacheEntryValidity {
    fn is_cacheable(&self) -> bool {
        Self::is_cacheable(self)
    }

    fn is_cache_still_valid(&self) -> bool {
        Self::is_cache_still_valid(self)
    }
}

#[cfg(test)]
mod tests {
    use super::AlwaysValidCacheEntryValidity;
    use crate::cache::ICacheEntryValidity;

    #[test]
    fn singleton_and_public_constructor_have_java_behavior() {
        let first = AlwaysValidCacheEntryValidity::new();
        let second = AlwaysValidCacheEntryValidity::new();

        assert!(AlwaysValidCacheEntryValidity::INSTANCE.is_cacheable());
        assert!(AlwaysValidCacheEntryValidity::INSTANCE.is_cache_still_valid());
        assert!(!std::ptr::eq(&first, &second));
        assert!(!std::ptr::eq(
            &first,
            AlwaysValidCacheEntryValidity::INSTANCE
        ));
        assert!(std::ptr::eq(
            AlwaysValidCacheEntryValidity::INSTANCE,
            AlwaysValidCacheEntryValidity::INSTANCE
        ));
    }

    #[test]
    fn dynamic_contract_also_returns_true() {
        let validity: &dyn ICacheEntryValidity = AlwaysValidCacheEntryValidity::INSTANCE;

        assert!(validity.is_cacheable());
        assert!(validity.is_cache_still_valid());
    }
}
