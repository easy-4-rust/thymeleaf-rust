use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::engine::TemplateModel;

use super::{ExpressionCacheKey, ICache, TemplateCacheKey};

type TemplateCache = dyn ICache<TemplateCacheKey, TemplateModel>;
type ExpressionCache = dyn ICache<ExpressionCacheKey, dyn Any + Send + Sync>;

/// 缓存管理器的惰性初始化公共实现。
///
/// 该对象保存 Java `volatile` 字段及双重检查锁对应的状态；具体管理器通过初始化闭包
/// 提供两类默认缓存。`OnceLock` 保证初始化至多执行一次，也会永久记住“禁用缓存”的
/// `None` 结果。
///
/// 对应 Java: `org.thymeleaf.cache.AbstractCacheManager`。
#[derive(Default)]
pub struct AbstractCacheManager {
    template_cache: OnceLock<Option<Arc<TemplateCache>>>,
    expression_cache: OnceLock<Option<Arc<ExpressionCache>>>,
}

impl AbstractCacheManager {
    /// 创建两个默认缓存均未初始化的管理器状态。
    ///
    /// 对应 Java: `AbstractCacheManager#AbstractCacheManager()`。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            template_cache: OnceLock::new(),
            expression_cache: OnceLock::new(),
        }
    }

    /// 惰性取得模板缓存，首次调用时执行具体管理器的初始化逻辑。
    ///
    /// # 参数
    /// - `initialize_template_cache`：对应 Java `initializeTemplateCache()`。
    ///
    /// # 返回
    /// 已初始化的唯一缓存；闭包返回 `None` 时表示永久禁用。
    pub fn get_template_cache<F>(&self, initialize_template_cache: F) -> Option<&TemplateCache>
    where
        F: FnOnce() -> Option<Arc<TemplateCache>>,
    {
        self.template_cache
            .get_or_init(initialize_template_cache)
            .as_deref()
    }

    /// 惰性取得表达式缓存，首次调用时执行具体管理器的初始化逻辑。
    ///
    /// # 参数
    /// - `initialize_expression_cache`：对应 Java `initializeExpressionCache()`。
    ///
    /// # 返回
    /// 已初始化的唯一缓存；闭包返回 `None` 时表示永久禁用。
    pub fn get_expression_cache<F>(
        &self,
        initialize_expression_cache: F,
    ) -> Option<&ExpressionCache>
    where
        F: FnOnce() -> Option<Arc<ExpressionCache>>,
    {
        self.expression_cache
            .get_or_init(initialize_expression_cache)
            .as_deref()
    }

    /// 清理已经初始化的两个默认缓存。
    ///
    /// 尚未访问的缓存不会仅因清理操作之外的内部探测而初始化；调用方若要严格复现
    /// Java `clearAllCaches()` 的初始化副作用，应先通过两个 `get` 方法取得缓存。
    pub fn clear_initialized_caches(&self) {
        if let Some(cache) = self.template_cache.get().and_then(Option::as_deref) {
            cache.clear();
        }
        if let Some(cache) = self.expression_cache.get().and_then(Option::as_deref) {
            cache.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};

    use crate::cache::{ICache, ICacheEntryValidityChecker, TemplateCacheKey};
    use crate::engine::TemplateModel;

    use super::AbstractCacheManager;

    #[derive(Default)]
    struct RecordingTemplateCache {
        clear_count: AtomicUsize,
    }

    impl ICache<TemplateCacheKey, TemplateModel> for RecordingTemplateCache {
        fn put(&self, _key: TemplateCacheKey, _value: Arc<TemplateModel>) {}

        fn get(&self, _key: &TemplateCacheKey) -> Option<Arc<TemplateModel>> {
            None
        }

        fn get_with_validity_checker(
            &self,
            _key: &TemplateCacheKey,
            _validity_checker: &dyn ICacheEntryValidityChecker<TemplateCacheKey, TemplateModel>,
        ) -> Option<Arc<TemplateModel>> {
            None
        }

        fn clear(&self) {
            self.clear_count.fetch_add(1, Ordering::SeqCst);
        }

        fn clear_key(&self, _key: &TemplateCacheKey) {}

        fn key_set(&self) -> HashSet<TemplateCacheKey> {
            HashSet::new()
        }
    }

    #[test]
    fn initializes_once_under_concurrency_and_clears_once_per_request() {
        let manager = Arc::new(AbstractCacheManager::new());
        let cache = Arc::new(RecordingTemplateCache::default());
        let initialize_count = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(9));

        let mut handles = Vec::new();
        for _ in 0..8 {
            let manager = Arc::clone(&manager);
            let cache = Arc::clone(&cache);
            let initialize_count = Arc::clone(&initialize_count);
            let barrier = Arc::clone(&barrier);
            handles.push(std::thread::spawn(move || {
                barrier.wait();
                let initialized = manager.get_template_cache(|| {
                    initialize_count.fetch_add(1, Ordering::SeqCst);
                    Some(cache)
                });
                assert!(initialized.is_some());
            }));
        }
        barrier.wait();
        for handle in handles {
            handle.join().expect("cache initialization thread");
        }

        assert_eq!(initialize_count.load(Ordering::SeqCst), 1);
        manager.clear_initialized_caches();
        assert_eq!(cache.clear_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn remembers_disabled_cache_without_retrying_initializer() {
        let manager = AbstractCacheManager::new();
        let initialize_count = AtomicUsize::new(0);

        for _ in 0..2 {
            assert!(
                manager
                    .get_template_cache(|| {
                        initialize_count.fetch_add(1, Ordering::SeqCst);
                        None
                    })
                    .is_none()
            );
        }

        assert_eq!(initialize_count.load(Ordering::SeqCst), 1);
    }
}
