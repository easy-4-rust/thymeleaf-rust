import java.util.TreeSet;

import org.thymeleaf.cache.ICacheEntryValidityChecker;
import org.thymeleaf.cache.StandardCache;

/**
 * 从固定 Thymeleaf Java 源码导出 StandardCache Golden。
 */
public final class StandardCacheGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private StandardCacheGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitFailure("ctor.null_name", () -> new StandardCache<String,String>(null, false, 0, 0, null));
        emitFailure("ctor.empty_name", () -> new StandardCache<String,String>("", false, 1, null));
        emitFailure("ctor.whitespace_name", () -> new StandardCache<String,String>("\u2003", false, 1, null));
        emitFailure("ctor.capacity", () -> new StandardCache<String,String>("cache", false, 0, null));
        emitFailure("ctor.max_size", () -> new StandardCache<String,String>("cache", false, 1, 0, null));

        final StandardCache<String,String> unlimited =
                new StandardCache<String,String>("\u00A0", true, 2, -2, null);
        emit("config.name", unlimited.getName());
        emit("config.soft", unlimited.getUseSoftReferences());
        emit("config.has_max", unlimited.hasMaxSize());
        emit("config.max", unlimited.getMaxSize());
        emit("config.size", unlimited.size());
        emit("config.hit_ratio", unlimited.getHitRatio());
        emit("config.miss_ratio", unlimited.getMissRatio());

        final StandardCache<String,String> fifo =
                new StandardCache<String,String>("fifo", false, 2, 2, null, null, true);
        final String original = new String("one");
        fifo.put("a", original);
        fifo.put("a", new String("replacement"));
        fifo.put("b", new String("two"));
        emit("fifo.put_if_absent.identity", fifo.get("a") == original);
        fifo.put("c", new String("three"));
        emit("fifo.keys", new TreeSet<String>(fifo.keySet()).toString());
        emit("fifo.a_miss", fifo.get("a") == null);
        emit("fifo.b_hit", fifo.get("b") != null);
        emit("fifo.c_hit", fifo.get("c") != null);
        emitCounters("fifo", fifo);

        final RecordingChecker invalid = new RecordingChecker(false);
        final StandardCache<String,String> checked =
                new StandardCache<String,String>("checked", false, 2, invalid, null);
        checked.put("key", "value");
        final RecordingChecker valid = new RecordingChecker(true);
        emit("checker.explicit_hit", checked.get("key", valid) != null);
        emit("checker.key", valid.key);
        emit("checker.value", valid.value);
        emit("checker.timestamp_positive", valid.timestamp > 0);
        emit("checker.default_miss", checked.get("key") == null);
        emit("checker.removed", !checked.keySet().contains("key"));

        checked.clearKey("missing");
        checked.put("first", "one");
        checked.clear();
        emit("clear.empty", checked.size());
        emitCounters("checked.disabled", checked);
    }

    private static void emitCounters(final String prefix, final StandardCache<String,String> cache) {
        emit(prefix + ".put_count", cache.getPutCount());
        emit(prefix + ".get_count", cache.getGetCount());
        emit(prefix + ".hit_count", cache.getHitCount());
        emit(prefix + ".miss_count", cache.getMissCount());
        emit(prefix + ".hit_ratio", cache.getHitRatio());
        emit(prefix + ".miss_ratio", cache.getMissRatio());
    }

    private static void emitFailure(final String key, final ThrowingRunnable action) {
        try {
            action.run();
            emit(key, "NO_ERROR");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName() + ":" + exception.getMessage());
        }
    }

    private static void emit(final String key, final boolean value) {
        emit(key, Boolean.toString(value));
    }

    private static void emit(final String key, final int value) {
        emit(key, Integer.toString(value));
    }

    private static void emit(final String key, final long value) {
        emit(key, Long.toString(value));
    }

    private static void emit(final String key, final double value) {
        emit(key, Double.toString(value));
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private interface ThrowingRunnable {
        void run();
    }

    private static final class RecordingChecker
            implements ICacheEntryValidityChecker<String,String> {
        private static final long serialVersionUID = 1L;

        private final boolean valid;
        private String key;
        private String value;
        private long timestamp;

        private RecordingChecker(final boolean valid) {
            this.valid = valid;
        }

        public boolean checkIsValueStillValid(
                final String key, final String value, final long entryCreationTimestamp) {
            this.key = key;
            this.value = value;
            this.timestamp = entryCreationTimestamp;
            return this.valid;
        }
    }
}
