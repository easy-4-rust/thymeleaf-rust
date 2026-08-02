use std::any::Any;
use std::sync::Arc;

use crate::engine::TemplateModel;
use crate::util::JavaString;

use super::{
    AbstractCacheManager, ExpressionCacheKey, ICache, ICacheEntryValidityChecker, ICacheManager,
    StandardCache, StandardParsedTemplateEntryValidator, TemplateCacheKey,
};

/// Thymeleaf 两个默认缓存的标准可配置管理器。
///
/// 配置项与 Java 保持一致：名称、初始容量、最大容量、计数器、软引用声明、logger
/// 名称以及条目有效性检查器。最大容量为 `0` 时对应缓存被禁用；配置在缓存首次读取时
/// 才被消费，之后修改配置不会替换已经创建的缓存。
///
/// 对应 Java: `org.thymeleaf.cache.StandardCacheManager`。
pub struct StandardCacheManager {
    cache_manager: AbstractCacheManager,
    template_cache_name: JavaString,
    template_cache_initial_size: i32,
    template_cache_max_size: i32,
    template_cache_enable_counters: bool,
    template_cache_use_soft_references: bool,
    template_cache_logger_name: Option<JavaString>,
    template_cache_validity_checker:
        Option<Arc<dyn ICacheEntryValidityChecker<TemplateCacheKey, TemplateModel>>>,
    expression_cache_name: JavaString,
    expression_cache_initial_size: i32,
    expression_cache_max_size: i32,
    expression_cache_enable_counters: bool,
    expression_cache_use_soft_references: bool,
    expression_cache_logger_name: Option<JavaString>,
    expression_cache_validity_checker:
        Option<Arc<dyn ICacheEntryValidityChecker<ExpressionCacheKey, dyn Any + Send + Sync>>>,
}

impl StandardCacheManager {
    /// 默认模板缓存名称。
    pub const DEFAULT_TEMPLATE_CACHE_NAME: &'static str = "TEMPLATE_CACHE";
    /// 默认模板缓存初始容量。
    pub const DEFAULT_TEMPLATE_CACHE_INITIAL_SIZE: i32 = 20;
    /// 默认模板缓存最大容量。
    pub const DEFAULT_TEMPLATE_CACHE_MAX_SIZE: i32 = 200;
    /// 默认模板缓存是否启用计数器。
    pub const DEFAULT_TEMPLATE_CACHE_ENABLE_COUNTERS: bool = false;
    /// 默认模板缓存是否声明使用软引用。
    pub const DEFAULT_TEMPLATE_CACHE_USE_SOFT_REFERENCES: bool = true;

    /// 默认表达式缓存名称。
    pub const DEFAULT_EXPRESSION_CACHE_NAME: &'static str = "EXPRESSION_CACHE";
    /// 默认表达式缓存初始容量。
    pub const DEFAULT_EXPRESSION_CACHE_INITIAL_SIZE: i32 = 100;
    /// 默认表达式缓存最大容量。
    pub const DEFAULT_EXPRESSION_CACHE_MAX_SIZE: i32 = 500;
    /// 默认表达式缓存是否启用计数器。
    pub const DEFAULT_EXPRESSION_CACHE_ENABLE_COUNTERS: bool = false;
    /// 默认表达式缓存是否声明使用软引用。
    pub const DEFAULT_EXPRESSION_CACHE_USE_SOFT_REFERENCES: bool = true;

    /// 使用与 Java 完全相同的默认配置创建管理器。
    ///
    /// 对应 Java: `StandardCacheManager#StandardCacheManager()`。
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache_manager: AbstractCacheManager::new(),
            template_cache_name: JavaString::from_rust_str(Self::DEFAULT_TEMPLATE_CACHE_NAME),
            template_cache_initial_size: Self::DEFAULT_TEMPLATE_CACHE_INITIAL_SIZE,
            template_cache_max_size: Self::DEFAULT_TEMPLATE_CACHE_MAX_SIZE,
            template_cache_enable_counters: Self::DEFAULT_TEMPLATE_CACHE_ENABLE_COUNTERS,
            template_cache_use_soft_references: Self::DEFAULT_TEMPLATE_CACHE_USE_SOFT_REFERENCES,
            template_cache_logger_name: None,
            template_cache_validity_checker: Some(Arc::new(
                StandardParsedTemplateEntryValidator::new(),
            )),
            expression_cache_name: JavaString::from_rust_str(Self::DEFAULT_EXPRESSION_CACHE_NAME),
            expression_cache_initial_size: Self::DEFAULT_EXPRESSION_CACHE_INITIAL_SIZE,
            expression_cache_max_size: Self::DEFAULT_EXPRESSION_CACHE_MAX_SIZE,
            expression_cache_enable_counters: Self::DEFAULT_EXPRESSION_CACHE_ENABLE_COUNTERS,
            expression_cache_use_soft_references:
                Self::DEFAULT_EXPRESSION_CACHE_USE_SOFT_REFERENCES,
            expression_cache_logger_name: None,
            expression_cache_validity_checker: None,
        }
    }

    /// 返回模板缓存名称。
    #[must_use]
    /// 对应 Java: `StandardCacheManager#getTemplateCacheName()`。
    pub fn get_template_cache_name(&self) -> &JavaString {
        &self.template_cache_name
    }

    /// 返回模板缓存是否声明使用软引用。
    #[must_use]
    pub const fn get_template_cache_use_soft_references(&self) -> bool {
        self.template_cache_use_soft_references
    }

    /// 返回模板缓存初始容量。
    #[must_use]
    pub const fn get_template_cache_initial_size(&self) -> i32 {
        self.template_cache_initial_size
    }

    /// 返回模板缓存最大容量；`0` 表示禁用，负数表示不限制。
    #[must_use]
    pub const fn get_template_cache_max_size(&self) -> i32 {
        self.template_cache_max_size
    }

    /// 返回模板缓存 logger 名；`None` 使用引擎默认 logger 路径。
    #[must_use]
    /// 对应 Java: `StandardCacheManager#getTemplateCacheLoggerName()`。
    pub fn get_template_cache_logger_name(&self) -> Option<&JavaString> {
        self.template_cache_logger_name.as_ref()
    }

    /// 返回模板缓存有效性检查器。
    #[must_use]
    /// 对应 Java: `StandardCacheManager#getTemplateCacheValidityChecker()`。
    pub fn get_template_cache_validity_checker(
        &self,
    ) -> Option<&dyn ICacheEntryValidityChecker<TemplateCacheKey, TemplateModel>> {
        self.template_cache_validity_checker.as_deref()
    }

    /// 返回表达式缓存名称。
    #[must_use]
    /// 对应 Java: `StandardCacheManager#getExpressionCacheName()`。
    pub fn get_expression_cache_name(&self) -> &JavaString {
        &self.expression_cache_name
    }

    /// 返回表达式缓存是否声明使用软引用。
    #[must_use]
    pub const fn get_expression_cache_use_soft_references(&self) -> bool {
        self.expression_cache_use_soft_references
    }

    /// 返回表达式缓存初始容量。
    #[must_use]
    pub const fn get_expression_cache_initial_size(&self) -> i32 {
        self.expression_cache_initial_size
    }

    /// 返回表达式缓存最大容量；`0` 表示禁用，负数表示不限制。
    #[must_use]
    pub const fn get_expression_cache_max_size(&self) -> i32 {
        self.expression_cache_max_size
    }

    /// 返回表达式缓存 logger 名；`None` 使用引擎默认 logger 路径。
    #[must_use]
    /// 对应 Java: `StandardCacheManager#getExpressionCacheLoggerName()`。
    pub fn get_expression_cache_logger_name(&self) -> Option<&JavaString> {
        self.expression_cache_logger_name.as_ref()
    }

    /// 返回表达式缓存有效性检查器。
    #[must_use]
    /// 对应 Java: `StandardCacheManager#getExpressionCacheValidityChecker()`。
    pub fn get_expression_cache_validity_checker(
        &self,
    ) -> Option<&dyn ICacheEntryValidityChecker<ExpressionCacheKey, dyn Any + Send + Sync>> {
        self.expression_cache_validity_checker.as_deref()
    }

    /// 设置模板缓存名称。对应 Java `setTemplateCacheName`。
    pub fn set_template_cache_name(&mut self, template_cache_name: JavaString) {
        self.template_cache_name = template_cache_name;
    }

    /// 设置模板缓存初始容量。对应 Java `setTemplateCacheInitialSize`。
    pub const fn set_template_cache_initial_size(&mut self, template_cache_initial_size: i32) {
        self.template_cache_initial_size = template_cache_initial_size;
    }

    /// 设置模板缓存最大容量。对应 Java `setTemplateCacheMaxSize`。
    pub const fn set_template_cache_max_size(&mut self, template_cache_max_size: i32) {
        self.template_cache_max_size = template_cache_max_size;
    }

    /// 设置模板缓存软引用策略。对应 Java `setTemplateCacheUseSoftReferences`。
    pub const fn set_template_cache_use_soft_references(
        &mut self,
        template_cache_use_soft_references: bool,
    ) {
        self.template_cache_use_soft_references = template_cache_use_soft_references;
    }

    /// 设置模板缓存 logger 名。对应 Java `setTemplateCacheLoggerName`。
    pub fn set_template_cache_logger_name(
        &mut self,
        template_cache_logger_name: Option<JavaString>,
    ) {
        self.template_cache_logger_name = template_cache_logger_name;
    }

    /// 设置模板缓存有效性检查器。`None` 表示每次读取不做额外检查。
    /// 对应 Java: `StandardCacheManager#setTemplateCacheValidityChecker()`。
    pub fn set_template_cache_validity_checker(
        &mut self,
        template_cache_validity_checker: Option<
            Arc<dyn ICacheEntryValidityChecker<TemplateCacheKey, TemplateModel>>,
        >,
    ) {
        self.template_cache_validity_checker = template_cache_validity_checker;
    }

    /// 设置模板缓存计数器开关。对应 Java `setTemplateCacheEnableCounters`。
    pub const fn set_template_cache_enable_counters(
        &mut self,
        template_cache_enable_counters: bool,
    ) {
        self.template_cache_enable_counters = template_cache_enable_counters;
    }

    /// 设置表达式缓存名称。对应 Java `setExpressionCacheName`。
    pub fn set_expression_cache_name(&mut self, expression_cache_name: JavaString) {
        self.expression_cache_name = expression_cache_name;
    }

    /// 设置表达式缓存初始容量。对应 Java `setExpressionCacheInitialSize`。
    pub const fn set_expression_cache_initial_size(&mut self, expression_cache_initial_size: i32) {
        self.expression_cache_initial_size = expression_cache_initial_size;
    }

    /// 设置表达式缓存最大容量。对应 Java `setExpressionCacheMaxSize`。
    pub const fn set_expression_cache_max_size(&mut self, expression_cache_max_size: i32) {
        self.expression_cache_max_size = expression_cache_max_size;
    }

    /// 设置表达式缓存软引用策略。对应 Java `setExpressionCacheUseSoftReferences`。
    pub const fn set_expression_cache_use_soft_references(
        &mut self,
        expression_cache_use_soft_references: bool,
    ) {
        self.expression_cache_use_soft_references = expression_cache_use_soft_references;
    }

    /// 设置表达式缓存 logger 名。对应 Java `setExpressionCacheLoggerName`。
    pub fn set_expression_cache_logger_name(
        &mut self,
        expression_cache_logger_name: Option<JavaString>,
    ) {
        self.expression_cache_logger_name = expression_cache_logger_name;
    }

    /// 设置表达式缓存有效性检查器。
    /// 对应 Java: `StandardCacheManager#setExpressionCacheValidityChecker()`。
    pub fn set_expression_cache_validity_checker(
        &mut self,
        expression_cache_validity_checker: Option<
            Arc<dyn ICacheEntryValidityChecker<ExpressionCacheKey, dyn Any + Send + Sync>>,
        >,
    ) {
        self.expression_cache_validity_checker = expression_cache_validity_checker;
    }

    /// 设置表达式缓存计数器开关。对应 Java `setExpressionCacheEnableCounters`。
    pub const fn set_expression_cache_enable_counters(
        &mut self,
        expression_cache_enable_counters: bool,
    ) {
        self.expression_cache_enable_counters = expression_cache_enable_counters;
    }

    fn initialize_template_cache(
        &self,
    ) -> Option<Arc<dyn ICache<TemplateCacheKey, TemplateModel>>> {
        if self.template_cache_max_size == 0 {
            return None;
        }
        let cache = StandardCache::with_options(
            Some(&self.template_cache_name.to_string_lossy()),
            self.template_cache_use_soft_references,
            self.template_cache_initial_size,
            self.template_cache_max_size,
            self.template_cache_validity_checker.clone(),
            self.template_cache_enable_counters,
            false,
        )
        .expect("Invalid template cache configuration");
        Some(Arc::new(cache))
    }

    fn initialize_expression_cache(
        &self,
    ) -> Option<Arc<dyn ICache<ExpressionCacheKey, dyn Any + Send + Sync>>> {
        if self.expression_cache_max_size == 0 {
            return None;
        }
        let cache: StandardCache<ExpressionCacheKey, dyn Any + Send + Sync> =
            StandardCache::with_options(
                Some(&self.expression_cache_name.to_string_lossy()),
                self.expression_cache_use_soft_references,
                self.expression_cache_initial_size,
                self.expression_cache_max_size,
                self.expression_cache_validity_checker.clone(),
                self.expression_cache_enable_counters,
                false,
            )
            .expect("Invalid expression cache configuration");
        Some(Arc::new(cache))
    }
}

impl Default for StandardCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ICacheManager for StandardCacheManager {
    fn java_class_name(&self) -> &'static str {
        "org.thymeleaf.cache.StandardCacheManager"
    }

    fn get_template_cache(&self) -> Option<&dyn ICache<TemplateCacheKey, TemplateModel>> {
        self.cache_manager
            .get_template_cache(|| self.initialize_template_cache())
    }

    fn get_expression_cache(
        &self,
    ) -> Option<&dyn ICache<ExpressionCacheKey, dyn Any + Send + Sync>> {
        self.cache_manager
            .get_expression_cache(|| self.initialize_expression_cache())
    }

    fn get_specific_cache<K, V>(&self, _name: &JavaString) -> Option<&dyn ICache<K, V>>
    where
        Self: Sized,
        K: Clone + Eq + std::hash::Hash + Send + Sync,
        V: Send + Sync,
    {
        None
    }

    fn get_all_specific_cache_names(&self) -> Option<Vec<JavaString>> {
        Some(Vec::new())
    }

    fn clear_all_caches(&self) {
        // Java 实现调用 getter，因此清理从未访问过的缓存也会触发一次惰性初始化。
        if let Some(cache) = self.get_template_cache() {
            cache.clear();
        }
        if let Some(cache) = self.get_expression_cache() {
            cache.clear();
        }
    }
}
