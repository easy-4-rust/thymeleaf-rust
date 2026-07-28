//! 模板解析结果与表达式的缓存契约。

mod always_valid_cache_entry_validity;
mod i_cache_entry_validity;
mod non_cacheable_cache_entry_validity;
mod ttl_cache_entry_validity;

pub use always_valid_cache_entry_validity::AlwaysValidCacheEntryValidity;
pub use i_cache_entry_validity::ICacheEntryValidity;
pub use non_cacheable_cache_entry_validity::NonCacheableCacheEntryValidity;
pub use ttl_cache_entry_validity::TTLCacheEntryValidity;
