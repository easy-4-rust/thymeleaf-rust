import org.thymeleaf.cache.ExpressionCacheKey;
import org.thymeleaf.cache.ICache;
import org.thymeleaf.cache.StandardCacheManager;
import org.thymeleaf.cache.TemplateCacheKey;
import org.thymeleaf.engine.TemplateModel;

/**
 * 从固定 Thymeleaf Java 源码导出 StandardCacheManager Golden。
 */
public final class StandardCacheManagerGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private StandardCacheManagerGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        final StandardCacheManager defaults = new StandardCacheManager();
        emit("default.template.name", defaults.getTemplateCacheName());
        emit("default.template.initial", defaults.getTemplateCacheInitialSize());
        emit("default.template.max", defaults.getTemplateCacheMaxSize());
        emit("default.template.soft", defaults.getTemplateCacheUseSoftReferences());
        emit("default.template.logger", defaults.getTemplateCacheLoggerName());
        emit(
                "default.template.checker",
                defaults.getTemplateCacheValidityChecker().getClass().getName());
        emit("default.expression.name", defaults.getExpressionCacheName());
        emit("default.expression.initial", defaults.getExpressionCacheInitialSize());
        emit("default.expression.max", defaults.getExpressionCacheMaxSize());
        emit("default.expression.soft", defaults.getExpressionCacheUseSoftReferences());
        emit("default.expression.logger", defaults.getExpressionCacheLoggerName());
        emit("default.expression.checker", defaults.getExpressionCacheValidityChecker());
        emit("default.specific.names", defaults.getAllSpecificCacheNames());
        emit("default.specific.cache", defaults.getSpecificCache("missing"));

        final ICache<TemplateCacheKey,TemplateModel> template0 = defaults.getTemplateCache();
        final ICache<TemplateCacheKey,TemplateModel> template1 = defaults.getTemplateCache();
        emit("lazy.template.same", template0 == template1);
        final ICache<ExpressionCacheKey,Object> expression0 = defaults.getExpressionCache();
        final ICache<ExpressionCacheKey,Object> expression1 = defaults.getExpressionCache();
        emit("lazy.expression.same", expression0 == expression1);

        final StandardCacheManager configured = new StandardCacheManager();
        configured.setTemplateCacheName("T");
        configured.setTemplateCacheInitialSize(7);
        configured.setTemplateCacheMaxSize(-2);
        configured.setTemplateCacheUseSoftReferences(false);
        configured.setTemplateCacheLoggerName("template.logger");
        configured.setTemplateCacheValidityChecker(null);
        configured.setTemplateCacheEnableCounters(true);
        configured.setExpressionCacheName("E");
        configured.setExpressionCacheInitialSize(9);
        configured.setExpressionCacheMaxSize(11);
        configured.setExpressionCacheUseSoftReferences(false);
        configured.setExpressionCacheLoggerName("expression.logger");
        configured.setExpressionCacheValidityChecker(null);
        configured.setExpressionCacheEnableCounters(true);
        emit("configured.template.name", configured.getTemplateCacheName());
        emit("configured.template.initial", configured.getTemplateCacheInitialSize());
        emit("configured.template.max", configured.getTemplateCacheMaxSize());
        emit("configured.template.soft", configured.getTemplateCacheUseSoftReferences());
        emit("configured.template.logger", configured.getTemplateCacheLoggerName());
        emit("configured.template.checker", configured.getTemplateCacheValidityChecker());
        emit("configured.expression.name", configured.getExpressionCacheName());
        emit("configured.expression.initial", configured.getExpressionCacheInitialSize());
        emit("configured.expression.max", configured.getExpressionCacheMaxSize());
        emit("configured.expression.soft", configured.getExpressionCacheUseSoftReferences());
        emit("configured.expression.logger", configured.getExpressionCacheLoggerName());
        emit("configured.expression.checker", configured.getExpressionCacheValidityChecker());

        final StandardCacheManager disabled = new StandardCacheManager();
        disabled.setTemplateCacheMaxSize(0);
        disabled.setExpressionCacheMaxSize(0);
        emit("disabled.template.first", disabled.getTemplateCache());
        emit("disabled.expression.first", disabled.getExpressionCache());
        disabled.setTemplateCacheMaxSize(1);
        disabled.setExpressionCacheMaxSize(1);
        emit("disabled.template.sticky", disabled.getTemplateCache());
        emit("disabled.expression.sticky", disabled.getExpressionCache());

        final StandardCacheManager mutation = new StandardCacheManager();
        mutation.setExpressionCacheName("before");
        final ICache<ExpressionCacheKey,Object> mutation0 = mutation.getExpressionCache();
        mutation.setExpressionCacheName("after");
        final ICache<ExpressionCacheKey,Object> mutation1 = mutation.getExpressionCache();
        emit("mutation.getter", mutation.getExpressionCacheName());
        emit("mutation.cache.same", mutation0 == mutation1);

        final ExpressionCacheKey key = new ExpressionCacheKey("type", "expression");
        mutation0.put(key, "value");
        emit("clear.before", mutation0.keySet().size());
        mutation.clearAllCaches();
        emit("clear.after", mutation0.keySet().size());
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }
}
