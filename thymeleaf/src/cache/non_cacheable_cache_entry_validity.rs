use super::ICacheEntryValidity;

static NON_CACHEABLE_INSTANCE: NonCacheableCacheEntryValidity =
    NonCacheableCacheEntryValidity { _identity: 0 };

/// 禁止模板解析结果进入缓存的有效性策略。
///
/// 对应 Java: `org.thymeleaf.cache.NonCacheableCacheEntryValidity`。
///
/// `is_cacheable` 始终返回 `false`，因此符合调用合同的模板缓存不会调用
/// `is_cache_still_valid`。后者仍保留 Java 的公开行为并返回 `false`。
pub struct NonCacheableCacheEntryValidity {
    // 非零大小确保独立构造值具有可观察的不同地址，保留 Java Object 身份语义。
    _identity: u8,
}

#[allow(clippy::new_without_default)]
impl NonCacheableCacheEntryValidity {
    /// 可共享的单例实例。
    ///
    /// 对应 Java: `NonCacheableCacheEntryValidity.INSTANCE`。
    pub const INSTANCE: &'static Self = &NON_CACHEABLE_INSTANCE;

    /// 创建一个新的不可缓存策略对象。
    ///
    /// 对应 Java:
    /// `NonCacheableCacheEntryValidity#NonCacheableCacheEntryValidity()`。
    ///
    /// # 返回
    /// 与单例行为相同、但具有独立对象身份的新策略。
    #[must_use]
    pub const fn new() -> Self {
        Self { _identity: 0 }
    }

    /// 始终拒绝模板解析结果进入缓存。
    ///
    /// 对应 Java: `NonCacheableCacheEntryValidity#isCacheable()`。
    ///
    /// # 返回
    /// 始终返回 `false`。
    #[must_use]
    pub const fn is_cacheable(&self) -> bool {
        false
    }

    /// 返回不可缓存策略的已有条目有效性。
    ///
    /// 对应 Java: `NonCacheableCacheEntryValidity#isCacheStillValid()`。
    ///
    /// 符合合同的缓存不会调用本方法，因为 `is_cacheable` 已返回 `false`。
    ///
    /// # 返回
    /// 始终返回 `false`。
    #[must_use]
    pub const fn is_cache_still_valid(&self) -> bool {
        false
    }
}

impl ICacheEntryValidity for NonCacheableCacheEntryValidity {
    fn is_cacheable(&self) -> bool {
        Self::is_cacheable(self)
    }

    fn is_cache_still_valid(&self) -> bool {
        Self::is_cache_still_valid(self)
    }
}

#[cfg(test)]
mod tests {
    use super::NonCacheableCacheEntryValidity;
    use crate::cache::ICacheEntryValidity;

    #[test]
    fn singleton_and_public_constructor_have_java_behavior() {
        let first = NonCacheableCacheEntryValidity::new();
        let second = NonCacheableCacheEntryValidity::new();

        assert!(!NonCacheableCacheEntryValidity::INSTANCE.is_cacheable());
        assert!(!NonCacheableCacheEntryValidity::INSTANCE.is_cache_still_valid());
        assert!(!std::ptr::eq(&first, &second));
        assert!(!std::ptr::eq(
            &first,
            NonCacheableCacheEntryValidity::INSTANCE
        ));
        assert!(std::ptr::eq(
            NonCacheableCacheEntryValidity::INSTANCE,
            NonCacheableCacheEntryValidity::INSTANCE
        ));
    }

    #[test]
    fn dynamic_contract_also_returns_false() {
        let validity: &dyn ICacheEntryValidity = NonCacheableCacheEntryValidity::INSTANCE;

        assert!(!validity.is_cacheable());
        assert!(!validity.is_cache_still_valid());
    }
}
