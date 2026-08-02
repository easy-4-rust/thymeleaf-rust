use std::any::Any;
use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::cache::ExpressionCacheKey;
use crate::util::JavaString;

use super::IStandardExpression;

struct CachedStandardExpression(Arc<dyn IStandardExpression>);

/// Standard Expression 各类解析制品的统一缓存适配。
///
/// 对应 Java: `org.thymeleaf.standard.expression.ExpressionCache`。
pub(crate) struct ExpressionCache;

impl ExpressionCache {
    const EXPRESSION: &'static str = "expr";
    const ASSIGNATION_SEQUENCE: &'static str = "aseq";
    const EXPRESSION_SEQUENCE: &'static str = "eseq";
    const EACH: &'static str = "each";
    const FRAGMENT_SIGNATURE: &'static str = "fsig";

    pub(crate) fn get_expression_from_cache(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
    ) -> Option<Arc<dyn IStandardExpression>> {
        Self::get_from_cache::<CachedStandardExpression>(configuration, input, Self::EXPRESSION)
            .map(|cached| Arc::clone(&cached.0))
    }

    pub(crate) fn put_expression_into_cache(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
        value: Arc<dyn IStandardExpression>,
    ) {
        Self::put_into_cache(
            configuration,
            input,
            Arc::new(CachedStandardExpression(value)),
            Self::EXPRESSION,
        );
    }

    pub(crate) fn get_assignation_sequence_from_cache<T>(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
    ) -> Option<Arc<T>>
    where
        T: Any + Send + Sync,
    {
        Self::get_from_cache(configuration, input, Self::ASSIGNATION_SEQUENCE)
    }

    pub(crate) fn put_assignation_sequence_into_cache<T>(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
        value: Arc<T>,
    ) where
        T: Any + Send + Sync,
    {
        Self::put_into_cache(configuration, input, value, Self::ASSIGNATION_SEQUENCE);
    }

    pub(crate) fn get_expression_sequence_from_cache<T>(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
    ) -> Option<Arc<T>>
    where
        T: Any + Send + Sync,
    {
        Self::get_from_cache(configuration, input, Self::EXPRESSION_SEQUENCE)
    }

    pub(crate) fn put_expression_sequence_into_cache<T>(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
        value: Arc<T>,
    ) where
        T: Any + Send + Sync,
    {
        Self::put_into_cache(configuration, input, value, Self::EXPRESSION_SEQUENCE);
    }

    pub(crate) fn get_each_from_cache<T>(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
    ) -> Option<Arc<T>>
    where
        T: Any + Send + Sync,
    {
        Self::get_from_cache(configuration, input, Self::EACH)
    }

    pub(crate) fn put_each_into_cache<T>(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
        value: Arc<T>,
    ) where
        T: Any + Send + Sync,
    {
        Self::put_into_cache(configuration, input, value, Self::EACH);
    }

    pub(crate) fn get_fragment_signature_from_cache<T>(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
    ) -> Option<Arc<T>>
    where
        T: Any + Send + Sync,
    {
        Self::get_from_cache(configuration, input, Self::FRAGMENT_SIGNATURE)
    }

    pub(crate) fn put_fragment_signature_into_cache<T>(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
        value: Arc<T>,
    ) where
        T: Any + Send + Sync,
    {
        Self::put_into_cache(configuration, input, value, Self::FRAGMENT_SIGNATURE);
    }

    fn get_from_cache<T>(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
        cache_type: &str,
    ) -> Option<Arc<T>>
    where
        T: Any + Send + Sync,
    {
        let key = ExpressionCacheKey::new(Some(cache_type), Some(&input.to_string_lossy())).ok()?;
        configuration
            .get_cache_manager()?
            .get_expression_cache()?
            .get(&key)?
            .downcast::<T>()
            .ok()
    }

    fn put_into_cache<T>(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
        value: Arc<T>,
        cache_type: &str,
    ) where
        T: Any + Send + Sync,
    {
        let Ok(key) = ExpressionCacheKey::new(Some(cache_type), Some(&input.to_string_lossy()))
        else {
            return;
        };
        if let Some(cache) = configuration
            .get_cache_manager()
            .and_then(crate::cache::ICacheManager::get_expression_cache)
        {
            let value: Arc<dyn Any + Send + Sync> = value;
            cache.put(key, value);
        }
    }

    /// 从表达式缓存移除指定类型的解析制品。
    ///
    /// 对应 Java: `ExpressionCache#removeFromCache`。缓存管理器或表达式缓存被禁用时
    /// 保持无副作用；键不存在时由 `ICache#clearKey` 保证幂等。
    #[expect(
        dead_code,
        reason = "保留 Java ExpressionCache.removeFromCache 包级合同"
    )]
    pub(crate) fn remove_from_cache(
        configuration: &dyn IEngineConfiguration,
        input: &JavaString,
        cache_type: &str,
    ) {
        let Ok(key) = ExpressionCacheKey::new(Some(cache_type), Some(&input.to_string_lossy()))
        else {
            return;
        };
        if let Some(cache) = configuration
            .get_cache_manager()
            .and_then(crate::cache::ICacheManager::get_expression_cache)
        {
            cache.clear_key(&key);
        }
    }
}
