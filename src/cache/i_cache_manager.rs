use std::any::Any;

use crate::engine::TemplateModel;
use crate::util::JavaString;

use super::{ExpressionCacheKey, ICache, TemplateCacheKey};

/// 模板缓存和表达式制品缓存的管理合同。
///
/// 对应 Java: `org.thymeleaf.cache.ICacheManager`。
pub trait ICacheManager: Send + Sync {
    /// 返回唯一模板缓存；禁用时返回 `None`。
    fn get_template_cache(&self) -> Option<&dyn ICache<TemplateCacheKey, TemplateModel>>;
    /// 返回唯一异构表达式制品缓存；禁用时返回 `None`。
    fn get_expression_cache(
        &self,
    ) -> Option<&dyn ICache<ExpressionCacheKey, dyn Any + Send + Sync>>;
    /// 返回指定名称的自定义强类型缓存。
    ///
    /// `Self: Sized` 是 Java 泛型方法在 Rust 中保持强类型的必要映射；引擎通过
    /// `clear_all_caches` 操作擦除后的管理器，不会丢失自定义实现内部的清理能力。
    fn get_specific_cache<K, V>(&self, name: &JavaString) -> Option<&dyn ICache<K, V>>
    where
        Self: Sized,
        K: Clone + Eq + std::hash::Hash + Send + Sync,
        V: Send + Sync;
    /// 返回全部自定义缓存名称；Java null 映射为 `None`。
    fn get_all_specific_cache_names(&self) -> Option<Vec<JavaString>>;
    /// 清理默认缓存及所有自定义缓存。
    fn clear_all_caches(&self);
}
