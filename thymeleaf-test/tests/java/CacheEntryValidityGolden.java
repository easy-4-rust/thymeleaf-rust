import org.thymeleaf.cache.AlwaysValidCacheEntryValidity;
import org.thymeleaf.cache.ICacheEntryValidity;
import org.thymeleaf.cache.NonCacheableCacheEntryValidity;
import org.thymeleaf.cache.TTLCacheEntryValidity;

/**
 * 从固定 Thymeleaf Java 源码导出缓存条目有效性 Golden。
 */
public final class CacheEntryValidityGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private CacheEntryValidityGolden() {
    }

    public static void main(final String[] args) throws InterruptedException {
        emit("baseline", BASELINE);
        exportAlwaysValid();
        exportNonCacheable();
        exportTTL();
    }

    private static void exportAlwaysValid() {
        final ICacheEntryValidity singleton = AlwaysValidCacheEntryValidity.INSTANCE;
        final AlwaysValidCacheEntryValidity first = new AlwaysValidCacheEntryValidity();
        final AlwaysValidCacheEntryValidity second = new AlwaysValidCacheEntryValidity();

        emit("always.instance.cacheable", singleton.isCacheable());
        emit("always.instance.valid", singleton.isCacheStillValid());
        emit(
                "always.instance.identity",
                AlwaysValidCacheEntryValidity.INSTANCE == AlwaysValidCacheEntryValidity.INSTANCE);
        emit("always.new.cacheable", first.isCacheable());
        emit("always.new.valid", first.isCacheStillValid());
        emit("always.new_not_instance", first != AlwaysValidCacheEntryValidity.INSTANCE);
        emit("always.new_identity", first != second);
    }

    private static void exportNonCacheable() {
        final ICacheEntryValidity singleton = NonCacheableCacheEntryValidity.INSTANCE;
        final NonCacheableCacheEntryValidity first = new NonCacheableCacheEntryValidity();
        final NonCacheableCacheEntryValidity second = new NonCacheableCacheEntryValidity();

        emit("non_cacheable.instance.cacheable", singleton.isCacheable());
        emit("non_cacheable.instance.valid", singleton.isCacheStillValid());
        emit(
                "non_cacheable.instance.identity",
                NonCacheableCacheEntryValidity.INSTANCE == NonCacheableCacheEntryValidity.INSTANCE);
        emit("non_cacheable.new.cacheable", first.isCacheable());
        emit("non_cacheable.new.valid", first.isCacheStillValid());
        emit("non_cacheable.new_not_instance", first != NonCacheableCacheEntryValidity.INSTANCE);
        emit("non_cacheable.new_identity", first != second);
    }

    private static void exportTTL() throws InterruptedException {
        exportTTLCase("positive", 60_000L);
        exportTTLCase("zero", 0L);
        exportTTLCase("negative", -1L);
        exportTTLCase("max", Long.MAX_VALUE);
        exportTTLCase("min", Long.MIN_VALUE);

        final TTLCacheEntryValidity expiring = new TTLCacheEntryValidity(1L);
        Thread.sleep(10L);
        emit("ttl.expired.valid", expiring.isCacheStillValid());
    }

    private static void exportTTLCase(final String name, final long ttl) {
        final TTLCacheEntryValidity validity = new TTLCacheEntryValidity(ttl);
        emit("ttl." + name + ".value", validity.getCacheTTLMs());
        emit("ttl." + name + ".cacheable", validity.isCacheable());
        emit("ttl." + name + ".valid", validity.isCacheStillValid());
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }
}
