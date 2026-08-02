/// 模板解析结果的缓存有效性契约。
///
/// 对应 Java: `org.thymeleaf.cache.ICacheEntryValidity`。
///
/// 模板缓存先调用 `is_cacheable` 判断解析结果能否进入缓存；只有返回 `true` 时，
/// 才会在读取已有条目前调用 `is_cache_still_valid`。后者返回 `false` 时，缓存应当
/// 移除旧条目并重新执行模板解析。
///
/// `Send + Sync` 是 Rust 多线程模板引擎共享有效性对象所需的安全约束。
pub trait ICacheEntryValidity: Send + Sync {
    /// 判断模板解析结果是否允许进入缓存。
    ///
    /// 对应 Java: `ICacheEntryValidity#isCacheable()`。
    ///
    /// # 返回
    /// 可以缓存时返回 `true`；否则返回 `false`。
    fn is_cacheable(&self) -> bool;

    /// 判断已经缓存的模板解析结果是否仍然有效。
    ///
    /// 对应 Java: `ICacheEntryValidity#isCacheStillValid()`。
    ///
    /// 本方法只应在 `is_cacheable` 返回 `true` 后调用。缓存会在返回条目前检查该值，
    /// 失效时移除条目并触发新的模板解析。
    ///
    /// # 返回
    /// 缓存条目仍可复用时返回 `true`。
    fn is_cache_still_valid(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::ICacheEntryValidity;

    struct CustomValidity {
        cacheable: bool,
        valid: bool,
    }

    impl ICacheEntryValidity for CustomValidity {
        fn is_cacheable(&self) -> bool {
            self.cacheable
        }

        fn is_cache_still_valid(&self) -> bool {
            self.valid
        }
    }

    #[test]
    fn supports_custom_dynamic_validity_implementations() {
        let validity: &dyn ICacheEntryValidity = &CustomValidity {
            cacheable: true,
            valid: false,
        };

        assert!(validity.is_cacheable());
        assert!(!validity.is_cache_still_valid());
    }
}
